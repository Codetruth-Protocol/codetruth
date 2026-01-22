//! CodeTruth Protocol CLI
//!
//! Command-line interface for analyzing code intent and detecting drift.

mod config;
mod engine_config;
mod lsp;
mod validation;
mod llm_utils;
mod spec_commands;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tracing_subscriber::fmt;

use ctp_core::{CodeTruthEngine, DriftSeverity};
use ctp_core::context_bridge::CodebaseContextBuilder;

use crate::config::{CliConfig, ConfigPaths, FileFilter, language_from_path};
use crate::engine_config::build_engine;

#[derive(Parser)]
#[command(name = "ctp")]
#[command(author, version, about = "CodeTruth Protocol - AI Code Intelligence & Drift Monitoring")]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Path to config file
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// LLM API key (auto-detects Groq/OpenAI from prefix)
    #[arg(long, global = true)]
    llm_key: Option<String>,

    /// LLM provider (groq, openai, anthropic, ollama)
    #[arg(long, global = true)]
    llm_provider: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

fn parse_drift_severity(value: &str, fallback: DriftSeverity) -> DriftSeverity {
    match value.to_lowercase().as_str() {
        "none" => DriftSeverity::None,
        "low" => DriftSeverity::Low,
        "medium" => DriftSeverity::Medium,
        "high" => DriftSeverity::High,
        "critical" => DriftSeverity::Critical,
        _ => fallback,
    }
}

fn collect_files(paths: &[PathBuf], filter: &FileFilter, max_file_size: usize) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = vec![];

    for path in paths {
        if path.is_file() {
            if filter.is_allowed_file(path) && file_within_size(path, max_file_size) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let entry_path = entry.path();
                if filter.is_allowed_file(entry_path)
                    && file_within_size(entry_path, max_file_size)
                {
                    files.push(entry_path.to_path_buf());
                }
            }
        }
    }

    files
}

fn file_within_size(path: &Path, max_file_size: usize) -> bool {
    match std::fs::metadata(path) {
        Ok(metadata) => metadata.len() <= max_file_size as u64,
        Err(_) => false,
    }
}

fn collect_changed_files(filter: &FileFilter, max_file_size: usize) -> Result<Vec<PathBuf>> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~1", "HEAD"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Git diff failed");
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .filter(|path| filter.is_allowed_file(path))
        .filter(|path| file_within_size(path, max_file_size))
        .collect();

    Ok(files)
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize CTP in the current repository
    Init {
        /// Path to initialize (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Analyze files or directories
    Analyze {
        /// Paths to analyze
        paths: Vec<PathBuf>,

        /// Output format (json, yaml, simple)
        #[arg(short, long, default_value = "simple")]
        format: String,

        /// Use LLM enhancement for complex analysis
        #[arg(long)]
        enhance: bool,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Minimum drift level to report (none, low, medium, high, critical)
        #[arg(long)]
        min_drift_level: Option<String>,
    },

    /// Explain a specific file in detail
    Explain {
        /// File to explain
        file: PathBuf,

        /// Output format
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Check policy compliance
    Check {
        /// Policy file or directory
        #[arg(short, long)]
        policies: Option<PathBuf>,

        /// Paths to check
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,

        /// Fail on any violation
        #[arg(long)]
        fail_on_violation: bool,
    },

    /// Compare drift between versions
    Diff {
        /// Base reference (commit, branch, or tag)
        base: String,

        /// Head reference (defaults to HEAD)
        #[arg(default_value = "HEAD")]
        head: String,
    },

    /// Generate audit report
    Audit {
        /// Output format (json, pdf, html)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Start Language Server Protocol server
    Lsp {
        /// Port to listen on
        #[arg(short, long, default_value_t = 9999)]
        port: u16,
    },

    /// CI/CD integration check
    CiCheck {
        /// Minimum drift level to fail (none, low, medium, high, critical)
        #[arg(long)]
        min_drift_level: Option<String>,

        /// Fail on policy violations
        #[arg(long)]
        fail_on_violation: bool,

        /// Paths to check (defaults to changed files)
        paths: Vec<PathBuf>,
    },

    /// Analyze codebase context and build hierarchical map
    Context {
        /// Root path to analyze
        #[arg(default_value = ".")]
        path: PathBuf,

        /// System name for the codebase
        #[arg(short, long)]
        name: Option<String>,

        /// System purpose description
        #[arg(short = 'd', long)]
        description: Option<String>,

        /// Output format (json, summary)
        #[arg(short, long, default_value = "summary")]
        format: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Check for redundancies
        #[arg(long)]
        check_redundancy: bool,
    },

    /// Product specification management
    Spec {
        #[command(subcommand)]
        action: SpecAction,
    },
}

#[derive(Subcommand)]
enum SpecAction {
    /// Generate product spec from codebase
    Generate {
        /// Root path to analyze
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Use LLM for enrichment
        #[arg(long)]
        use_llm: bool,

        /// Output file
        #[arg(short, long, default_value = "product-metadata.json")]
        output: PathBuf,
    },

    /// Validate spec against codebase
    Validate {
        /// Spec file to validate
        #[arg(default_value = "product-metadata.json")]
        spec_file: PathBuf,

        /// Root path to validate against
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Enrich spec with LLM
    Enrich {
        /// Spec file to enrich
        #[arg(default_value = "product-metadata.json")]
        spec_file: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show current spec
    Show {
        /// Spec file to show
        #[arg(default_value = "product-metadata.json")]
        spec_file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        fmt::init();
    }

    let current_dir = std::env::current_dir()?;
    let config_paths = ConfigPaths::resolve(cli.config.clone(), &current_dir)?;
    let config = CliConfig::load(&config_paths)?;
    let filter = FileFilter::new(&config)?;

    match cli.command {
        Commands::Init { path } => cmd_init(path).await,
        Commands::Analyze { paths, format, enhance, output, min_drift_level } => {
            cmd_analyze(&config, &filter, paths, format, enhance, output, min_drift_level, cli.llm_key.as_deref(), cli.llm_provider.as_deref()).await
        }
        Commands::Explain { file, format } => cmd_explain(&config, &filter, file, format).await,
        Commands::Check { policies, paths, fail_on_violation } => {
            cmd_check(&config, &filter, policies, paths, fail_on_violation).await
        }
        Commands::Diff { base, head } => cmd_diff(&config, &filter, base, head).await,
        Commands::Audit { format, output } => cmd_audit(&config, &filter, format, output).await,
        Commands::Lsp { port } => cmd_lsp(&config, port).await,
        Commands::CiCheck { min_drift_level, fail_on_violation, paths } => {
            cmd_ci_check(&config, &filter, min_drift_level, fail_on_violation, paths).await
        }
        Commands::Context { path, name, description, format, output, check_redundancy } => {
            cmd_context(&config, &filter, path, name, description, format, output, check_redundancy).await
        }
        Commands::Spec { action } => {
            cmd_spec(action, cli.llm_key.as_deref(), cli.llm_provider.as_deref()).await
        }
    }
}

async fn cmd_init(path: PathBuf) -> Result<()> {
    println!("{} Initializing CodeTruth Protocol...", style("→").cyan());

    let ctp_dir = path.join(".ctp");
    tokio::fs::create_dir_all(&ctp_dir).await?;
    tokio::fs::create_dir_all(ctp_dir.join("policies")).await?;
    tokio::fs::create_dir_all(ctp_dir.join("analyses")).await?;

    // Create default config
    let config = r#"# CodeTruth Protocol Configuration
version: "1.0"

# Analysis settings
analysis:
  # Languages to analyze
  languages:
    - python
    - javascript
    - typescript
    - rust
    - go
    - java

  # Files to exclude
  exclude:
    - "**/node_modules/**"
    - "**/target/**"
    - "**/.git/**"
    - "**/dist/**"
    - "**/build/**"
    - "**/__pycache__/**"
    - "**/venv/**"

  # Maximum file size to analyze (bytes)
  max_file_size: 10485760

# LLM settings (optional)
llm:
  enabled: false
  # provider: anthropic
  # model: claude-sonnet-4-20250514

# Drift detection
drift:
  # Minimum severity to report
  min_severity: low

  # Similarity threshold for "no drift"
  similarity_threshold: 0.7

# Policy settings
policies:
  # Path to policy files
  path: .ctp/policies/

  # Fail CI on policy violations
  fail_on_violation: false

# Output settings
output:
  # Default format (simple, json, yaml)
  format: simple

  # Store analysis results
  store_results: true
  results_path: .ctp/analyses/
"#;

    tokio::fs::write(ctp_dir.join("config.yaml"), config).await?;

    // Create example policy
    let example_policy = r#"# Example CTP Policy
ctp_version: "1.0.0"
policy_schema_version: "1.0.0"

policy:
  id: "documentation-required"
  name: "Documentation Required"
  description: |
    All public functions must have documentation.

  scope:
    include:
      - "**/*.py"
      - "**/*.js"
      - "**/*.ts"
    exclude:
      - "**/*_test.*"
      - "**/*.spec.*"

  severity: "WARNING"

  rules:
    - rule_id: "docstring-required"
      type: "documentation"
      requires:
        - pattern: "def |function |fn "
          must_have_doc: true

      violation_message: |
        Function detected without documentation.
        All public functions should have docstrings or comments
        explaining their purpose.
"#;

    tokio::fs::write(
        ctp_dir.join("policies/documentation-required.yaml"),
        example_policy,
    )
    .await?;

    println!(
        "{} Created .ctp/ directory with default configuration",
        style("✓").green()
    );
    println!(
        "{} Edit .ctp/config.yaml to customize settings",
        style("→").cyan()
    );
    println!(
        "{} Add policies to .ctp/policies/",
        style("→").cyan()
    );

    Ok(())
}

async fn cmd_analyze(
    config: &CliConfig,
    filter: &FileFilter,
    paths: Vec<PathBuf>,
    format: String,
    enhance: bool,
    output: Option<PathBuf>,
    min_drift_level: Option<String>,
    llm_key: Option<&str>,
    llm_provider: Option<&str>,
) -> Result<()> {
    let engine = build_engine(config, enhance);

    let files = collect_files(&paths, filter, config.analysis.max_file_size);

    if files.is_empty() {
        println!("{} No source files found to analyze", style("!").yellow());
        return Ok(());
    }

    println!(
        "{} Analyzing {} file(s)...",
        style("→").cyan(),
        files.len()
    );

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Process files sequentially with async/await (parallel processing requires more complex setup)
    let mut results = vec![];
    let mut errors = vec![];

    for file in &files {
        pb.set_message(file.file_name().unwrap_or_default().to_string_lossy().to_string());

        match engine.analyze_file(file).await {
            Ok(analysis) => results.push(analysis),
            Err(e) => errors.push((file.clone(), e.to_string())),
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    let min_severity = parse_drift_severity(
        min_drift_level.as_deref().unwrap_or(&config.drift.min_severity),
        DriftSeverity::Low,
    );

    let reported_results: Vec<_> = results
        .iter()
        .filter(|r| r.drift.drift_severity >= min_severity)
        .cloned()
        .collect();

    // Output results
    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&reported_results)?;
            if let Some(out) = output {
                tokio::fs::write(out, &json).await?;
            } else {
                println!("{}", json);
            }
        }
        "yaml" => {
            let yaml = serde_yaml::to_string(&reported_results)?;
            if let Some(out) = output {
                tokio::fs::write(out, &yaml).await?;
            } else {
                println!("{}", yaml);
            }
        }
        "simple" | _ => {
            print_simple_results(&reported_results);
        }
    }

    // Report errors
    if !errors.is_empty() {
        println!("\n{} {} file(s) failed to analyze:", style("!").yellow(), errors.len());
        for (file, err) in errors {
            println!("  {} {}: {}", style("✗").red(), file.display(), err);
        }
    }

    // Summary
    let drift_count = results
        .iter()
        .filter(|r| r.drift.drift_severity >= min_severity)
        .count();

    println!("\n{}", style("Summary").bold());
    println!("  Files analyzed: {}", results.len());
    println!("  Files with drift: {}", drift_count);

    Ok(())
}

async fn cmd_explain(
    config: &CliConfig,
    filter: &FileFilter,
    file: PathBuf,
    format: String,
) -> Result<()> {
    if !filter.is_allowed_file(&file) {
        println!("{} File excluded by config", style("!").yellow());
        return Ok(());
    }

    let engine = build_engine(config, config.llm.enabled);

    println!("{} Analyzing {}...", style("→").cyan(), file.display());

    let analysis = engine.analyze_file(&file).await?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(&analysis)?);
        }
        _ => {
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        }
    }

    Ok(())
}

async fn cmd_check(
    config: &CliConfig,
    filter: &FileFilter,
    policies_path: Option<PathBuf>,
    paths: Vec<PathBuf>,
    fail_on_violation: bool,
) -> Result<()> {
    println!("{} Checking policy compliance...", style("→").cyan());

    let engine = build_engine(config, config.llm.enabled);

    let policies_path = policies_path.unwrap_or_else(|| PathBuf::from(&config.policies.path));

    // Load policies
    if policies_path.exists() {
        let count = engine.load_policies(&policies_path)?;
        println!("  Loaded {} policy file(s)", count);
    } else {
        println!("{} No policies found at {:?}", style("!").yellow(), policies_path);
        return Ok(());
    }

    let files = collect_files(&paths, filter, config.analysis.max_file_size);

    if files.is_empty() {
        println!("{} No source files found to check", style("!").yellow());
        return Ok(());
    }

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut total_violations = 0;
    let mut total_warnings = 0;
    let mut files_with_issues = 0;

    for file in &files {
        pb.set_message(file.file_name().unwrap_or_default().to_string_lossy().to_string());

        match engine.analyze_file(file).await {
            Ok(analysis) => {
                let file_violations: usize = analysis
                    .policies
                    .policy_results
                    .iter()
                    .map(|p| p.violations.len())
                    .sum();

                let file_warnings: usize = analysis
                    .policies
                    .policy_results
                    .iter()
                    .filter(|p| matches!(p.status, ctp_core::PolicyStatus::Warning))
                    .count();

                if file_violations > 0 || file_warnings > 0 {
                    files_with_issues += 1;
                    pb.suspend(|| {
                        println!(
                            "\n{} {} - {} violation(s), {} warning(s)",
                            style("✗").red(),
                            file.display(),
                            file_violations,
                            file_warnings
                        );

                        for result in &analysis.policies.policy_results {
                            if !result.violations.is_empty() {
                                println!(
                                    "  {} [{}] {}",
                                    style("→").dim(),
                                    result.policy_id,
                                    result.policy_name
                                );
                                for violation in &result.violations {
                                    println!(
                                        "    {} Line {}: {}",
                                        match violation.severity {
                                            ctp_core::ViolationSeverity::Critical => style("✗✗").red().bold(),
                                            ctp_core::ViolationSeverity::Error => style("✗").red(),
                                            ctp_core::ViolationSeverity::Warning => style("!").yellow(),
                                            ctp_core::ViolationSeverity::Info => style("i").blue(),
                                        },
                                        violation.location.line_start,
                                        violation.message
                                    );
                                }
                            }
                        }
                    });
                }

                total_violations += file_violations;
                total_warnings += file_warnings;
            }
            Err(e) => {
                pb.suspend(|| {
                    println!("{} {}: {}", style("!").yellow(), file.display(), e);
                });
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    // Summary
    println!("\n{}", style("Policy Check Results").bold());
    println!("  Files checked: {}", files.len());
    println!("  Files with issues: {}", files_with_issues);
    println!("  Total violations: {}", total_violations);
    println!("  Total warnings: {}", total_warnings);

    let fail_on_violation = fail_on_violation || config.policies.fail_on_violation;

    if total_violations > 0 {
        println!(
            "\n{} {} policy violation(s) found",
            style("✗").red(),
            total_violations
        );
        if fail_on_violation {
            std::process::exit(1);
        }
    } else {
        println!("\n{} All policy checks passed", style("✓").green());
    }

    Ok(())
}

async fn cmd_diff(
    _config: &CliConfig,
    filter: &FileFilter,
    base: String,
    head: String,
) -> Result<()> {
    println!("{} Comparing {} → {}", style("→").cyan(), base, head);

    // Get changed files between base and head
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", &base, &head])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git diff failed: {}", stderr);
    }

    let changed_files: Vec<PathBuf> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .filter(|p| filter.is_allowed_file(p))
        .collect();

    if changed_files.is_empty() {
        println!("{} No source files changed between {} and {}", style("✓").green(), base, head);
        return Ok(());
    }

    println!("  Found {} changed source file(s)\n", changed_files.len());

    let engine = CodeTruthEngine::default();
    let mut drift_findings = vec![];

    for file in &changed_files {
        // Get file content at base
        let base_content = get_git_file_content(&base, file);
        // Get file content at head
        let head_content = get_git_file_content(&head, file);

        match (base_content, head_content) {
            (Ok(base_code), Ok(head_code)) => {
                // Analyze both versions
                let Some(language) = language_from_path(file) else {
                    continue;
                };

                let base_analysis = engine
                    .analyze_string(&base_code, &language, &file.display().to_string())
                    .await;
                let head_analysis = engine
                    .analyze_string(&head_code, &language, &file.display().to_string())
                    .await;

                if let (Ok(base_a), Ok(head_a)) = (base_analysis, head_analysis) {
                    // Compare intents
                    let intent_changed = base_a.intent.inferred_intent != head_a.intent.inferred_intent;
                    let behavior_changed = base_a.behavior.actual_behavior != head_a.behavior.actual_behavior;
                    let drift_increased = head_a.drift.drift_severity > base_a.drift.drift_severity;

                    if intent_changed || behavior_changed || drift_increased {
                        drift_findings.push((file.clone(), base_a, head_a, intent_changed, behavior_changed, drift_increased));
                    }
                }
            }
            (Err(_), Ok(_)) => {
                println!("  {} {} (new file)", style("+").green(), file.display());
            }
            (Ok(_), Err(_)) => {
                println!("  {} {} (deleted)", style("-").red(), file.display());
            }
            _ => {}
        }
    }

    // Print drift findings
    if drift_findings.is_empty() {
        println!("{} No significant drift detected in changes", style("✓").green());
    } else {
        println!("{} Drift detected in {} file(s):\n", style("!").yellow(), drift_findings.len());

        for (file, base_a, head_a, intent_changed, behavior_changed, drift_increased) in &drift_findings {
            let severity_icon = match head_a.drift.drift_severity {
                DriftSeverity::None => style("○").green(),
                DriftSeverity::Low => style("○").yellow(),
                DriftSeverity::Medium => style("●").yellow(),
                DriftSeverity::High => style("●").red(),
                DriftSeverity::Critical => style("●").red().bold(),
            };

            println!("{} {}", severity_icon, file.display());

            if *intent_changed {
                println!("  {} Intent changed:", style("→").dim());
                println!("    {} {}", style("-").red(), base_a.intent.inferred_intent);
                println!("    {} {}", style("+").green(), head_a.intent.inferred_intent);
            }

            if *behavior_changed {
                println!("  {} Behavior changed:", style("→").dim());
                println!("    {} {}", style("-").red(), base_a.behavior.actual_behavior);
                println!("    {} {}", style("+").green(), head_a.behavior.actual_behavior);
            }

            if *drift_increased {
                println!(
                    "  {} Drift severity: {:?} → {:?}",
                    style("↑").red(),
                    base_a.drift.drift_severity,
                    head_a.drift.drift_severity
                );
            }

            println!();
        }
    }

    // Summary
    println!("{}", style("Diff Summary").bold());
    println!("  Files changed: {}", changed_files.len());
    println!("  Files with drift: {}", drift_findings.len());

    Ok(())
}

fn validate_git_revision(revision: &str) -> Result<()> {
    if revision.is_empty() {
        anyhow::bail!("Git revision cannot be empty");
    }
    
    if !revision.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/') {
        anyhow::bail!("Invalid git revision: contains unsafe characters");
    }
    
    if revision.len() > 256 {
        anyhow::bail!("Git revision too long");
    }
    
    Ok(())
}

fn get_git_file_content(revision: &str, file: &PathBuf) -> Result<String> {
    validate_git_revision(revision)?;
    
    let file_str = file.to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?;
    
    let output = std::process::Command::new("git")
        .args(["show", &format!("{}:{}", revision, file_str)])
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn cmd_audit(
    config: &CliConfig,
    filter: &FileFilter,
    format: String,
    output: Option<PathBuf>,
) -> Result<()> {
    println!("{} Generating audit report...", style("→").cyan());

    let engine = build_engine(config, config.llm.enabled);

    let files = collect_files(&[PathBuf::from(".")], filter, config.analysis.max_file_size);

    println!("  Analyzing {} file(s)...", files.len());

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut analyses = vec![];
    let mut errors = vec![];

    for file in &files {
        match engine.analyze_file(file).await {
            Ok(analysis) => analyses.push(analysis),
            Err(e) => errors.push((file.clone(), e.to_string())),
        }
        pb.inc(1);
    }

    pb.finish_and_clear();

    // Build audit report
    let report = build_audit_report(&analyses, &errors);

    // Output based on format
    let report_content = match format.as_str() {
        "json" => serde_json::to_string_pretty(&report)?,
        "html" => generate_html_report(&report),
        _ => serde_json::to_string_pretty(&report)?,
    };

    if let Some(out_path) = output {
        tokio::fs::write(&out_path, &report_content).await?;
        println!("{} Audit report saved to {}", style("✓").green(), out_path.display());
    } else {
        println!("{}", report_content);
    }

    // Print summary
    println!("\n{}", style("Audit Summary").bold());
    println!("  Files analyzed: {}", analyses.len());
    println!("  Files with drift: {}", report.drift_summary.files_with_drift);
    println!("  Policy violations: {}", report.policy_summary.total_violations);
    println!("  Analysis errors: {}", errors.len());

    Ok(())
}

#[derive(serde::Serialize)]
struct AuditReport {
    generated_at: String,
    ctp_version: String,
    summary: AuditSummary,
    drift_summary: DriftSummary,
    policy_summary: PolicySummary,
    files: Vec<FileAudit>,
    errors: Vec<AuditError>,
}

#[derive(serde::Serialize)]
struct AuditSummary {
    total_files: usize,
    total_lines: usize,
    languages: std::collections::HashMap<String, usize>,
}

#[derive(serde::Serialize)]
struct DriftSummary {
    files_with_drift: usize,
    by_severity: std::collections::HashMap<String, usize>,
}

#[derive(serde::Serialize)]
struct PolicySummary {
    total_violations: usize,
    by_severity: std::collections::HashMap<String, usize>,
}

#[derive(serde::Serialize)]
struct FileAudit {
    path: String,
    language: String,
    lines: usize,
    drift_severity: String,
    violations: usize,
    intent: String,
}

#[derive(serde::Serialize)]
struct AuditError {
    path: String,
    error: String,
}

fn build_audit_report(analyses: &[ctp_core::ExplanationGraph], errors: &[(PathBuf, String)]) -> AuditReport {
    let mut languages: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut drift_by_severity: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut violation_by_severity: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total_lines = 0;
    let mut files_with_drift = 0;
    let mut total_violations = 0;

    let mut files = vec![];

    for analysis in analyses {
        // Count languages
        *languages.entry(analysis.module.language.clone()).or_insert(0) += 1;
        total_lines += analysis.module.lines_of_code;

        // Count drift
        let severity_str = format!("{:?}", analysis.drift.drift_severity);
        *drift_by_severity.entry(severity_str.clone()).or_insert(0) += 1;
        if analysis.drift.drift_detected {
            files_with_drift += 1;
        }

        // Count violations
        let file_violations: usize = analysis
            .policies
            .policy_results
            .iter()
            .map(|p| p.violations.len())
            .sum();
        total_violations += file_violations;

        for result in &analysis.policies.policy_results {
            for violation in &result.violations {
                let sev = format!("{:?}", violation.severity);
                *violation_by_severity.entry(sev).or_insert(0) += 1;
            }
        }

        files.push(FileAudit {
            path: analysis.module.path.clone(),
            language: analysis.module.language.clone(),
            lines: analysis.module.lines_of_code,
            drift_severity: severity_str,
            violations: file_violations,
            intent: analysis.intent.inferred_intent.clone(),
        });
    }

    AuditReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        ctp_version: "1.0.0".into(),
        summary: AuditSummary {
            total_files: analyses.len(),
            total_lines,
            languages,
        },
        drift_summary: DriftSummary {
            files_with_drift,
            by_severity: drift_by_severity,
        },
        policy_summary: PolicySummary {
            total_violations,
            by_severity: violation_by_severity,
        },
        files,
        errors: errors
            .iter()
            .map(|(p, e)| AuditError {
                path: p.display().to_string(),
                error: e.clone(),
            })
            .collect(),
    }
}

fn generate_html_report(report: &AuditReport) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>CodeTruth Audit Report</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }}
        h1 {{ color: #2563eb; }}
        .summary {{ background: #f3f4f6; padding: 20px; border-radius: 8px; margin: 20px 0; }}
        .metric {{ display: inline-block; margin-right: 40px; }}
        .metric-value {{ font-size: 2em; font-weight: bold; color: #1f2937; }}
        .metric-label {{ color: #6b7280; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #e5e7eb; }}
        th {{ background: #f9fafb; font-weight: 600; }}
        .drift-none {{ color: #10b981; }}
        .drift-low {{ color: #f59e0b; }}
        .drift-medium {{ color: #f97316; }}
        .drift-high {{ color: #ef4444; }}
        .drift-critical {{ color: #dc2626; font-weight: bold; }}
    </style>
</head>
<body>
    <h1>🔍 CodeTruth Audit Report</h1>
    <p>Generated: {}</p>
    
    <div class="summary">
        <div class="metric">
            <div class="metric-value">{}</div>
            <div class="metric-label">Files Analyzed</div>
        </div>
        <div class="metric">
            <div class="metric-value">{}</div>
            <div class="metric-label">Total Lines</div>
        </div>
        <div class="metric">
            <div class="metric-value">{}</div>
            <div class="metric-label">Files with Drift</div>
        </div>
        <div class="metric">
            <div class="metric-value">{}</div>
            <div class="metric-label">Policy Violations</div>
        </div>
    </div>

    <h2>File Analysis</h2>
    <table>
        <tr>
            <th>File</th>
            <th>Language</th>
            <th>Lines</th>
            <th>Drift</th>
            <th>Violations</th>
            <th>Intent</th>
        </tr>
        {}
    </table>
</body>
</html>"#,
        report.generated_at,
        report.summary.total_files,
        report.summary.total_lines,
        report.drift_summary.files_with_drift,
        report.policy_summary.total_violations,
        report.files.iter().map(|f| format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td class=\"drift-{}\">{}</td><td>{}</td><td>{}</td></tr>",
            f.path, f.language, f.lines, f.drift_severity.to_lowercase(), f.drift_severity, f.violations, f.intent
        )).collect::<Vec<_>>().join("\n        ")
    )
}

async fn cmd_lsp(config: &CliConfig, port: u16) -> Result<()> {
    println!("{} Starting LSP server on port {}...", style("→").cyan(), port);
    lsp::run_lsp(port, config.clone()).await
}

async fn cmd_ci_check(
    config: &CliConfig,
    filter: &FileFilter,
    min_drift_level: Option<String>,
    fail_on_violation: bool,
    paths: Vec<PathBuf>,
) -> Result<()> {
    let engine = build_engine(config, config.llm.enabled);

    let min_severity = parse_drift_severity(
        min_drift_level.as_deref().unwrap_or(&config.drift.min_severity),
        DriftSeverity::High,
    );

    if !config.policies.path.is_empty() {
        let policies_path = PathBuf::from(&config.policies.path);
        if policies_path.exists() {
            let _ = engine.load_policies(&policies_path);
        }
    }

    let files = if paths.is_empty() {
        collect_changed_files(filter, config.analysis.max_file_size)
            .unwrap_or_else(|_| collect_files(&[PathBuf::from(".")], filter, config.analysis.max_file_size))
    } else {
        collect_files(&paths, filter, config.analysis.max_file_size)
    };

    let mut violations = 0;
    let mut high_drift = 0;

    for file in &files {
        if let Ok(analysis) = engine.analyze_file(file).await {
            if analysis.drift.drift_severity >= min_severity {
                high_drift += 1;
                println!(
                    "{} {} - {:?} drift",
                    style("✗").red(),
                    file.display(),
                    analysis.drift.drift_severity
                );
            }

            violations += analysis
                .policies
                .policy_results
                .iter()
                .filter(|p| matches!(p.status, ctp_core::PolicyStatus::Fail))
                .count();
        }
    }

    println!("\n{}", style("CI Check Results").bold());
    println!("  Files checked: {}", files.len());
    println!("  High drift files: {}", high_drift);
    println!("  Policy violations: {}", violations);

    let fail_on_violation = fail_on_violation || config.policies.fail_on_violation;

    if high_drift > 0 || (fail_on_violation && violations > 0) {
        std::process::exit(1);
    }

    Ok(())
}

async fn cmd_context(
    config: &CliConfig,
    filter: &FileFilter,
    path: PathBuf,
    name: Option<String>,
    description: Option<String>,
    format: String,
    output: Option<PathBuf>,
    check_redundancy: bool,
) -> Result<()> {
    println!("{} Building hierarchical context map...", style("→").cyan());

    let engine = build_engine(config, config.llm.enabled);

    // Determine system name and description
    let system_name = name.unwrap_or_else(|| {
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "codebase".to_string())
    });
    let system_desc = description.unwrap_or_else(|| format!("{} codebase", system_name));

    // Initialize context builder
    let mut builder = CodebaseContextBuilder::new()
        .with_system(&system_name, &system_desc);

    let files = collect_files(&[path.clone()], filter, config.analysis.max_file_size);

    if files.is_empty() {
        println!("{} No source files found", style("!").yellow());
        return Ok(());
    }

    println!("  Found {} source file(s)", files.len());

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut analysis_errors = vec![];

    for file in &files {
        pb.set_message(file.file_name().unwrap_or_default().to_string_lossy().to_string());

        match engine.analyze_file(file).await {
            Ok(graph) => {
                if let Err(e) = builder.add_graph(&graph) {
                    // Log but continue - redundancy errors are informational
                    pb.suspend(|| {
                        println!("  {} {}: {}", style("!").yellow(), file.display(), e);
                    });
                }
            }
            Err(e) => {
                analysis_errors.push((file.clone(), e.to_string()));
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    // Get statistics
    let stats = builder.stats();

    // Check for redundancies if requested
    let redundancies = if check_redundancy {
        builder.find_redundancies()
    } else {
        vec![]
    };

    // Output based on format
    match format.as_str() {
        "json" => {
            let contexts = builder.contexts();
            let json_output = serde_json::json!({
                "system": {
                    "name": system_name,
                    "description": system_desc,
                },
                "stats": {
                    "total_components": stats.total_components,
                    "role_distribution": stats.role_distribution,
                    "total_invariants": stats.total_invariants,
                    "total_relationships": stats.total_relationships,
                    "redundancy_count": stats.redundancy_count,
                },
                "contexts": contexts.iter().map(|c| serde_json::json!({
                    "id": c.id.to_string(),
                    "level": c.level.as_str(),
                    "purpose": c.essence.purpose,
                    "role": c.essence.role.category(),
                    "constraints": c.essence.constraints,
                    "relationships": c.relationships.len(),
                    "invariants": c.invariants.len(),
                })).collect::<Vec<_>>(),
                "redundancies": redundancies.iter().map(|(a, b, sim)| serde_json::json!({
                    "component_a": a.to_string(),
                    "component_b": b.to_string(),
                    "similarity": sim,
                })).collect::<Vec<_>>(),
            });

            let json_str = serde_json::to_string_pretty(&json_output)?;
            if let Some(out) = output {
                tokio::fs::write(&out, &json_str).await?;
                println!("{} Context map saved to {}", style("✓").green(), out.display());
            } else {
                println!("{}", json_str);
            }
        }
        "summary" | _ => {
            println!("\n{}", style("Codebase Context Summary").bold());
            println!("  System: {} - {}", style(&system_name).cyan(), system_desc);
            println!();

            println!("{}", style("Component Statistics").bold());
            println!("  Total components: {}", stats.total_components);
            println!("  Total relationships: {}", stats.total_relationships);
            println!("  Total invariants: {}", stats.total_invariants);
            println!();

            println!("{}", style("Role Distribution").bold());
            for (role, count) in &stats.role_distribution {
                let bar_len = (*count as f64 / stats.total_components as f64 * 20.0) as usize;
                let bar = "█".repeat(bar_len);
                println!("  {:12} {} {} ({:.0}%)", 
                    role, 
                    style(&bar).cyan(),
                    count,
                    (*count as f64 / stats.total_components as f64) * 100.0
                );
            }

            if check_redundancy {
                println!();
                if redundancies.is_empty() {
                    println!("{} No redundancies detected", style("✓").green());
                } else {
                    println!("{} {} potential redundancies detected:", 
                        style("!").yellow(), 
                        redundancies.len()
                    );
                    for (a, b, similarity) in &redundancies {
                        println!("  {} ↔ {} ({:.0}% similar)", 
                            style(a.to_string()).dim(),
                            style(b.to_string()).dim(),
                            similarity * 100.0
                        );
                    }
                }
            }

            if !analysis_errors.is_empty() {
                println!();
                println!("{} {} file(s) failed to analyze", 
                    style("!").yellow(), 
                    analysis_errors.len()
                );
            }

            // Save JSON if output specified
            if let Some(out) = output {
                let contexts = builder.contexts();
                let json_output = serde_json::json!({
                    "system": { "name": system_name, "description": system_desc },
                    "stats": stats.total_components,
                    "contexts": contexts.len(),
                });
                tokio::fs::write(&out, serde_json::to_string_pretty(&json_output)?).await?;
                println!("\n{} Full context saved to {}", style("✓").green(), out.display());
            }
        }
    }

    Ok(())
}

async fn cmd_spec(
    action: SpecAction,
    llm_key: Option<&str>,
    llm_provider: Option<&str>,
) -> Result<()> {
    use spec_commands::*;
    
    match action {
        SpecAction::Generate { path, use_llm, output } => {
            cmd_spec_generate(path, use_llm, output, llm_key, llm_provider).await
        }
        SpecAction::Validate { spec_file, path } => {
            cmd_spec_validate(spec_file, path).await
        }
        SpecAction::Enrich { spec_file, output } => {
            cmd_spec_enrich(spec_file, output, llm_key, llm_provider).await
        }
        SpecAction::Show { spec_file } => {
            cmd_spec_show(spec_file).await
        }
    }
}

fn print_simple_results(results: &[ctp_core::ExplanationGraph]) {
    for result in results {
        let drift_icon = match result.drift.drift_severity {
            DriftSeverity::None => style("✓").green(),
            DriftSeverity::Low => style("○").yellow(),
            DriftSeverity::Medium => style("●").yellow(),
            DriftSeverity::High => style("✗").red(),
            DriftSeverity::Critical => style("✗✗").red().bold(),
        };

        println!(
            "{} {} [{}]",
            drift_icon,
            result.module.path,
            result.module.language
        );

        if !result.intent.declared_intent.is_empty() {
            println!("  Intent: {}", result.intent.declared_intent);
        }

        println!("  Behavior: {}", result.behavior.actual_behavior);
        println!(
            "  Drift: {:?} (confidence: {:.0}%)",
            result.drift.drift_severity,
            result.intent.confidence * 100.0
        );

        if !result.drift.drift_details.is_empty() {
            for detail in &result.drift.drift_details {
                println!(
                    "    {} {:?}: {}",
                    style("→").dim(),
                    detail.drift_type,
                    detail.remediation
                );
            }
        }

        println!();
    }
}
