//! MCP Tool implementations for CodeTruth
//!
//! Provides the following tools:
//! - analyze_file: Analyze a single file for intent, behavior, and drift
//! - check_compliance: Check files against compliance policies
//! - detect_drift: Detect drift between intent and implementation
//! - detect_stubs: Detect stubs, TODOs, and placeholders in code
//! - explain_violation: Get natural language explanation of violations
//! - analyze_codebase: Analyze entire codebase for compliance and redundancies

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use ctp_core::{CodeTruthEngine, ExplanationGraph};
use glob::Pattern;
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;
use tracing::{debug, info, instrument, warn};

use crate::cache::AnalysisCache;
use crate::metrics::{MetricsCollector, TimingGuard};
use crate::models::*;
use crate::security::{sanitize_path, check_file_size, validate_glob_pattern, ResourceLimits};

/// Default concurrent analysis limit
const MAX_CONCURRENT_ANALYSES: usize = 4;

/// Default analysis timeout (used by integration layer)
#[allow(dead_code)]
const ANALYSIS_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

/// Tool handler for MCP server
pub struct ToolHandler {
    engine: Arc<CodeTruthEngine>,
    cache: Arc<AnalysisCache>,
    /// Semaphore to limit concurrent analyses
    analysis_semaphore: Arc<Semaphore>,
    /// Resource limits for batch operations
    resource_limits: ResourceLimits,
    /// Metrics collector for observability
    metrics: Arc<MetricsCollector>,
}

impl ToolHandler {
    /// Create a new tool handler
    pub fn new(engine: Arc<CodeTruthEngine>, cache: Arc<AnalysisCache>) -> Self {
        Self { 
            engine, 
            cache,
            analysis_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_ANALYSES)),
            resource_limits: ResourceLimits::default(),
            metrics: Arc::new(MetricsCollector::new()),
        }
    }

    /// Create a tool handler with custom resource limits
    pub fn with_limits(
        engine: Arc<CodeTruthEngine>, 
        cache: Arc<AnalysisCache>,
        limits: ResourceLimits,
    ) -> Self {
        Self {
            engine,
            cache,
            analysis_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_ANALYSES)),
            resource_limits: limits,
            metrics: Arc::new(MetricsCollector::new()),
        }
    }

    /// Get metrics snapshot
    pub fn metrics(&self) -> crate::metrics::MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Analyze a single file
    #[instrument(skip(self), fields(file_path = %input.file_path))]
    pub async fn analyze_file(&self, input: AnalyzeFileInput) -> Result<AnalyzeFileOutput> {
        let path = Path::new(&input.file_path);
        
        if !path.exists() {
            anyhow::bail!("File not found: {}", input.file_path);
        }

        debug!("Analyzing file: {}", input.file_path);

        // Read file content
        let content = tokio::fs::read_to_string(path).await?;
        let content_hash = self.hash_content(&content);

        // Check cache first
        if let Some(cached) = self.cache.get(path, &content_hash).await {
            debug!("Using cached analysis for {}", input.file_path);
            return Ok(self.convert_to_output(input.file_path, cached));
        }

        // Perform analysis
        let graph = self.engine.analyze_file(path).await?;

        // Cache the result
        self.cache.put(path, &content_hash, graph.clone()).await;

        Ok(self.convert_to_output(input.file_path, graph))
    }

    /// Check compliance against policies
    #[instrument(skip(self), fields(path = %input.path))]
    pub async fn check_compliance(&self, input: CheckComplianceInput) -> Result<CheckComplianceOutput> {
        let path = Path::new(&input.path);
        
        if !path.exists() {
            anyhow::bail!("Path not found: {}", input.path);
        }

        // Load policies if directory provided
        if let Some(policy_dir) = &input.policy_dir {
            let policy_path = Path::new(policy_dir);
            if policy_path.exists() {
                let count = self.engine.load_policies(policy_path)?;
                info!("Loaded {} policies from {}", count, policy_dir);
            }
        }

        let mut violations = Vec::new();
        let mut files_analyzed = 0;

        if path.is_file() {
            // Single file analysis
            let result = self.analyze_file(AnalyzeFileInput {
                file_path: input.path.clone(),
                focus: None,
            }).await?;
            
            files_analyzed = 1;
            violations.extend(result.violations);
        } else {
            // Directory analysis - use recursive file collection
            let mut files = Vec::new();
            self.collect_files(path, &mut files, &None, &None).await?;
            
            for file_path in files {
                let file_path_str = file_path.display().to_string();
                
                match self.analyze_file(AnalyzeFileInput {
                    file_path: file_path_str.clone(),
                    focus: None,
                }).await {
                    Ok(result) => {
                        files_analyzed += 1;
                        // Filter by specific policy if requested
                        if let Some(ref target_policy) = input.policy {
                            let filtered: Vec<ComplianceViolation> = result.violations
                                .into_iter()
                                .filter(|v| v.policy == *target_policy)
                                .collect();
                            violations.extend(filtered);
                        } else {
                            violations.extend(result.violations);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to analyze {}: {}", file_path_str, e);
                    }
                }
            }
        }

        // Count by severity
        let mut counts = ViolationsBySeverity {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
        };

        for v in &violations {
            match v.severity.as_str() {
                "critical" => counts.critical += 1,
                "high" => counts.high += 1,
                "medium" => counts.medium += 1,
                "low" => counts.low += 1,
                _ => counts.low += 1,
            }
        }

        Ok(CheckComplianceOutput {
            path: input.path,
            files_analyzed,
            total_violations: violations.len(),
            violations_by_severity: counts,
            violations,
        })
    }

    /// Detect drift in code
    #[instrument(skip(self), fields(path = %input.path))]
    pub async fn detect_drift(&self, input: DetectDriftInput) -> Result<DetectDriftOutput> {
        let path = Path::new(&input.path);
        
        if !path.exists() {
            anyhow::bail!("Path not found: {}", input.path);
        }

        let mut findings = Vec::new();
        let mut files_analyzed = 0;

        let files_to_analyze = if path.is_file() {
            vec![path.to_path_buf()]
        } else {
            // Collect files from directory
            let mut files = Vec::new();
            let mut entries = tokio::fs::read_dir(path).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    files.push(entry_path);
                }
            }
            files
        };

        for file_path in files_to_analyze {
            match self.engine.analyze_file(&file_path).await {
                Ok(graph) => {
                    files_analyzed += 1;
                    
                    // Check for drift in the analysis
                    if graph.drift.drift_detected {
                        // Map drift severity to string
                        let drift_severity = match graph.drift.drift_severity {
                            ctp_core::DriftSeverity::Critical => "critical",
                            ctp_core::DriftSeverity::High => "high",
                            ctp_core::DriftSeverity::Medium => "medium",
                            ctp_core::DriftSeverity::Low => "low",
                            ctp_core::DriftSeverity::None => "none",
                        };
                        
                        for detail in &graph.drift.drift_details {
                            // Generate human-readable drift type description
                            let drift_type_desc = match detail.drift_type {
                                ctp_core::DriftType::Intent => "Intent mismatch",
                                ctp_core::DriftType::Policy => "Policy violation",
                                ctp_core::DriftType::Assumption => "Invalid assumption",
                                ctp_core::DriftType::Implementation => "Implementation drift",
                            };
                            
                            // Generate severity from impact or use drift severity
                            let severity = drift_severity.to_string();
                            
                            findings.push(DriftFinding {
                                file_path: file_path.display().to_string(),
                                line_number: detail.location.line_start,
                                drift_type: drift_type_desc.to_string(),
                                severity,
                                description: format!("Expected: {} but found: {}", detail.expected, detail.actual),
                                declared_intent: graph.intent.declared_intent.clone(),
                                actual_behavior: graph.behavior.actual_behavior.clone(),
                                suggestion: Some(detail.remediation.clone()),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to analyze {}: {}", file_path.display(), e);
                }
            }
        }

        Ok(DetectDriftOutput {
            path: input.path,
            has_drift: !findings.is_empty(),
            drift_count: findings.len(),
            files_analyzed,
            findings,
        })
    }

    /// Explain a violation in natural language
    #[instrument(skip(self))]
    pub async fn explain_violation(&self, input: ExplainViolationInput) -> Result<ExplainViolationOutput> {
        // Generate explanation based on violation details
        let explanation = format!(
            "The code in {} violates the '{}' policy. \
             {}. This indicates a mismatch between the intended behavior \
             and the actual implementation.",
            input.file_path,
            input.policy,
            input.description
        );

        let why_it_matters = match input.severity.as_str() {
            "critical" => "Critical violations can cause system failures, security breaches, or data loss. Immediate attention required.",
            "high" => "High severity violations may lead to bugs, performance issues, or maintenance difficulties.",
            "medium" => "Medium severity violations indicate code quality issues that should be addressed for maintainability.",
            "low" => "Low severity violations are minor issues that don't affect functionality but may impact readability.",
            _ => "This violation should be reviewed to ensure code quality and compliance.",
        }.to_string();

        let how_to_fix = "Review the code against the policy requirements and refactor to ensure compliance.".to_string();

        let prevention_tips = vec![
            "Add policy-compliant code examples to your documentation".to_string(),
            "Use automated linting tools to catch violations early".to_string(),
            "Conduct regular code reviews focusing on compliance".to_string(),
        ];

        Ok(ExplainViolationOutput {
            explanation,
            why_it_matters,
            how_to_fix,
            prevention_tips,
            references: vec![],
        })
    }

    /// Analyze entire codebase
    #[instrument(skip(self), fields(project = %input.project_name))]
    pub async fn analyze_codebase(&self, input: AnalyzeCodebaseInput) -> Result<AnalyzeCodebaseOutput> {
        let root_path = Path::new(&input.root_path);
        
        if !root_path.exists() {
            anyhow::bail!("Root path not found: {}", input.root_path);
        }

        // Collect all files
        let mut files = Vec::new();
        self.collect_files(root_path, &mut files, &input.include_patterns, &input.exclude_patterns).await?;

        info!("Analyzing {} files in codebase {}", files.len(), input.project_name);

        // Analyze files
        let mut all_violations = Vec::new();
        let mut successful_analyses = 0;
        let mut total_loc = 0;

        for file_path in &files {
            match self.engine.analyze_file(file_path).await {
                Ok(graph) => {
                    successful_analyses += 1;
                    total_loc += graph.module.lines_of_code;
                    all_violations.extend(
                        self.convert_violations(file_path.display().to_string(), &graph)
                    );
                }
                Err(e) => {
                    warn!("Failed to analyze {}: {}", file_path.display(), e);
                }
            }
        }

        let files_with_violations = all_violations.iter()
            .map(|v| &v.file_path)
            .collect::<std::collections::HashSet<_>>()
            .len();

        // Detect redundancies (simplified - would use ctp-context in full implementation)
        let redundancies = Vec::new();
        let critical_components = Vec::new();

        Ok(AnalyzeCodebaseOutput {
            project_name: input.project_name,
            total_files: successful_analyses,
            files_with_violations,
            total_violations: all_violations.len(),
            redundancies,
            critical_components,
            stats: CodebaseStats {
                total_components: successful_analyses,
                redundancy_count: 0,
                average_complexity: 0.0,
                total_lines_of_code: total_loc,
            },
        })
    }

    // Helper methods
    fn hash_content(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn convert_to_output(&self, file_path: String, graph: ExplanationGraph) -> AnalyzeFileOutput {
        let violations = self.convert_violations(file_path.clone(), &graph);

        AnalyzeFileOutput {
            file_path,
            language: graph.module.language,
            lines_of_code: graph.module.lines_of_code,
            complexity_score: graph.module.complexity_score,
            inferred_intent: graph.intent.inferred_intent,
            intent_confidence: graph.intent.confidence,
            entry_points: graph.behavior.entry_points.iter()
                .map(|e| e.function.clone())
                .collect(),
            side_effects: graph.behavior.side_effects.iter()
                .map(|s| {
                    let effect_type = match s.effect_type {
                        ctp_core::SideEffectType::Io => "I/O",
                        ctp_core::SideEffectType::Network => "Network",
                        ctp_core::SideEffectType::Database => "Database",
                        ctp_core::SideEffectType::StateMutation => "State Mutation",
                    };
                    let risk = match s.risk_level {
                        ctp_core::RiskLevel::Low => "low",
                        ctp_core::RiskLevel::Medium => "medium",
                        ctp_core::RiskLevel::High => "high",
                    };
                    format!("{}: {} (risk: {})", effect_type, s.description, risk)
                })
                .collect(),
            drift_detected: graph.drift.drift_detected,
            drift_details: if graph.drift.drift_detected {
                Some(format!("Found {} drift details", graph.drift.drift_details.len()))
            } else {
                None
            },
            violations,
        }
    }

    fn convert_violations(&self, file_path: String, graph: &ExplanationGraph) -> Vec<ComplianceViolation> {
        // Convert policy results to violations
        let mut violations = Vec::new();
        
        for policy_result in &graph.policies.policy_results {
            let policy_name = &policy_result.policy_name;
            // Include violations for both failed and warning policies
            if matches!(policy_result.status, ctp_core::PolicyStatus::Fail | ctp_core::PolicyStatus::Warning) {
                for violation in &policy_result.violations {
                    let severity = match violation.severity {
                        ctp_core::ViolationSeverity::Critical => "critical",
                        ctp_core::ViolationSeverity::Error => "high",
                        ctp_core::ViolationSeverity::Warning => "medium",
                        ctp_core::ViolationSeverity::Info => "low",
                    };
                    let suggestion = self.generate_suggestion(&violation.rule, severity);
                    violations.push(ComplianceViolation {
                        file_path: file_path.clone(),
                        line_number: Some(violation.location.line_start),
                        severity: severity.to_string(),
                        policy: policy_name.clone(),
                        description: violation.message.clone(),
                        suggestion: Some(suggestion),
                        rule_id: violation.rule.clone(),
                    });
                }
            }
        }
        
        violations
    }

    /// Generate contextual suggestion based on rule and severity
    fn generate_suggestion(&self, rule: &str, severity: &str) -> String {
        match (rule.to_lowercase().as_str(), severity) {
            (r, _) if r.contains("naming") => "Follow the project naming convention for consistency.".to_string(),
            (r, _) if r.contains("security") => "Review security requirements and apply secure coding practices.".to_string(),
            (r, _) if r.contains("doc") || r.contains("comment") => "Add comprehensive documentation explaining the code's purpose.".to_string(),
            (r, _) if r.contains("test") => "Add unit tests to verify this functionality works correctly.".to_string(),
            (r, _) if r.contains("complex") => "Refactor this code into smaller, more focused functions.".to_string(),
            (r, _) if r.contains("error") || r.contains("panic") => "Add proper error handling instead of unwrapping or panicking.".to_string(),
            (_, "critical") => "Address this issue immediately as it may cause system failures.".to_string(),
            (_, "high") => "Prioritize fixing this issue to prevent bugs or security problems.".to_string(),
            (_, "medium") => "Address this issue when convenient to improve code quality.".to_string(),
            _ => "Review the policy requirements and refactor accordingly.".to_string(),
        }
    }

    async fn collect_files(
        &self,
        dir: &Path,
        files: &mut Vec<std::path::PathBuf>,
        include: &Option<Vec<String>>,
        exclude: &Option<Vec<String>>,
    ) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        
        // Compile patterns once
        let include_patterns: Option<Vec<Pattern>> = include.as_ref().map(|p| {
            p.iter().filter_map(|s| Pattern::new(s).ok()).collect()
        });
        let exclude_patterns: Option<Vec<Pattern>> = exclude.as_ref().map(|p| {
            p.iter().filter_map(|s| Pattern::new(s).ok()).collect()
        });
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let path_str = path.display().to_string();
                
                // Check exclude patterns first
                if let Some(ref patterns) = exclude_patterns {
                    if patterns.iter().any(|p| p.matches(&path_str)) {
                        continue;
                    }
                }
                
                // Check include patterns (if specified, must match at least one)
                if let Some(ref patterns) = include_patterns {
                    if !patterns.iter().any(|p| p.matches(&path_str)) {
                        continue;
                    }
                }
                
                // Check if it's a source code file
                if self.is_source_file(&path) {
                    files.push(path);
                }
            } else if path.is_dir() {
                // Skip hidden directories and common non-source directories
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy();
                    if !name.starts_with('.') && 
                       !matches!(name.as_ref(), "target" | "node_modules" | "vendor" | "dist" | "build" | ".git" | "__pycache__" | ".venv" | "venv") {
                        Box::pin(self.collect_files(&path, files, include, exclude)).await?;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Detect stubs, TODOs, and placeholders in code
    #[instrument(skip(self), fields(path = %input.path))]
    pub async fn detect_stubs(&self, input: DetectStubsInput) -> Result<DetectStubsOutput> {
        let _timing = TimingGuard::new(&self.metrics);
        self.metrics.record_request();

        // Validate and sanitize the path
        let sanitized_path = match sanitize_path(&input.path, None) {
            Ok(path) => path,
            Err(e) => {
                self.metrics.record_failure();
                return Err(e.into());
            }
        };

        debug!("Detecting stubs in: {}", sanitized_path.display());

        // Validate glob patterns if provided
        if let Some(ref patterns) = input.include_patterns {
            for pattern in patterns {
                if let Err(e) = validate_glob_pattern(pattern) {
                    self.metrics.record_failure();
                    return Err(e.into());
                }
            }
        }
        if let Some(ref patterns) = input.exclude_patterns {
            for pattern in patterns {
                if let Err(e) = validate_glob_pattern(pattern) {
                    self.metrics.record_failure();
                    return Err(e.into());
                }
            }
        }

        // Acquire semaphore permit for concurrent limiting
        let _permit = match self.analysis_semaphore.acquire().await {
            Ok(p) => p,
            Err(_) => {
                self.metrics.record_failure();
                anyhow::bail!("Failed to acquire analysis permit");
            }
        };

        // Collect files to analyze
        let mut files_to_analyze = Vec::new();
        if sanitized_path.is_file() {
            // Check file size
            if let Err(e) = check_file_size(&sanitized_path) {
                self.metrics.record_failure();
                return Err(e.into());
            }
            files_to_analyze.push(sanitized_path.clone());
        } else {
            self.collect_files(&sanitized_path, &mut files_to_analyze, &input.include_patterns, &input.exclude_patterns).await?;
            
            // Check resource limits
            if files_to_analyze.len() > self.resource_limits.max_files {
                self.metrics.record_failure();
                anyhow::bail!(
                    "Too many files to analyze: {} exceeds limit of {}",
                    files_to_analyze.len(),
                    self.resource_limits.max_files
                );
            }
        }

        let mut all_findings: Vec<StubFindingDetail> = Vec::new();
        let mut files_analyzed = 0;
        let mut total_size: u64 = 0;
        let mut stubs_by_severity = StubsBySeverity {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
        };

        // Filter by minimum severity if provided
        let min_severity = input.min_severity.as_ref().map(|s| s.to_lowercase());

        for file_path in &files_to_analyze {
            // Check file size
            match check_file_size(file_path) {
                Ok(size) => {
                    total_size += size;
                    if total_size > self.resource_limits.max_total_size {
                        warn!("Total file size limit exceeded, stopping at {} files", files_analyzed);
                        break;
                    }
                }
                Err(e) => {
                    warn!("Skipping file {}: {}", file_path.display(), e);
                    continue;
                }
            }

            match tokio::fs::read_to_string(file_path).await {
                Ok(_content) => {
                    files_analyzed += 1;
                    let findings = self.engine.analyze_stubs(file_path);
                    
                    for finding in findings {
                        // Map stub severity to string
                        let severity_str = match finding.severity {
                            ctp_drift::StubSeverity::Critical => "critical",
                            ctp_drift::StubSeverity::High => "high",
                            ctp_drift::StubSeverity::Medium => "medium",
                            ctp_drift::StubSeverity::Low => "low",
                        };

                        // Filter by minimum severity
                        if let Some(ref min) = min_severity {
                            let should_include = match (min.as_str(), finding.severity) {
                                ("critical", ctp_drift::StubSeverity::Critical) => true,
                                ("critical", _) => false,
                                ("high", ctp_drift::StubSeverity::Critical) |
                                ("high", ctp_drift::StubSeverity::High) => true,
                                ("high", _) => false,
                                ("medium", ctp_drift::StubSeverity::Critical) |
                                ("medium", ctp_drift::StubSeverity::High) |
                                ("medium", ctp_drift::StubSeverity::Medium) => true,
                                ("medium", _) => false,
                                _ => true, // "low" or unknown includes all
                            };
                            if !should_include {
                                continue;
                            }
                        }

                        // Update severity counts
                        match finding.severity {
                            ctp_drift::StubSeverity::Critical => stubs_by_severity.critical += 1,
                            ctp_drift::StubSeverity::High => stubs_by_severity.high += 1,
                            ctp_drift::StubSeverity::Medium => stubs_by_severity.medium += 1,
                            ctp_drift::StubSeverity::Low => stubs_by_severity.low += 1,
                        }

                        // Determine stub type from the matched pattern
                        let stub_type = if finding.pattern_matched.to_lowercase().contains("todo") {
                            "TODO"
                        } else if finding.pattern_matched.to_lowercase().contains("fixme") {
                            "FIXME"
                        } else if finding.pattern_matched.to_lowercase().contains("placeholder") {
                            "PLACEHOLDER"
                        } else if finding.pattern_matched.to_lowercase().contains("unimplemented") {
                            "UNIMPLEMENTED"
                        } else {
                            "STUB"
                        };

                        all_findings.push(StubFindingDetail {
                            file_path: file_path.display().to_string(),
                            line_number: finding.line,
                            column: finding.column,
                            stub_type: stub_type.to_string(),
                            severity: severity_str.to_string(),
                            context: finding.context,
                            suggestion: if finding.suggestion.is_empty() { None } else { Some(finding.suggestion) },
                        });
                    }
                }
                Err(e) => {
                    warn!("Failed to read file {}: {}", file_path.display(), e);
                }
            }
        }

        self.metrics.record_success();
        self.metrics.record_files_analyzed(files_analyzed as u64);
        self.metrics.record_stubs(all_findings.len() as u64);

        let has_critical_stubs = stubs_by_severity.critical > 0;
        let total_stubs_found = all_findings.len();

        info!(
            "Stub detection complete: {} files analyzed, {} stubs found ({} critical)",
            files_analyzed, total_stubs_found, stubs_by_severity.critical
        );

        Ok(DetectStubsOutput {
            path: input.path,
            files_analyzed,
            total_stubs_found,
            has_critical_stubs,
            stubs_by_severity,
            findings: all_findings,
        })
    }

    /// Check if a file is a recognized source code file
    /// Note: This is a broad filter for file collection. Actual parsing support
    /// depends on ctp-parser feature flags. With all-languages feature enabled
    /// in ctp-mcp, all listed languages are supported.
    fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            matches!(ext.as_str(),
                // Systems languages
                "rs" | "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" |
                // Web
                "js" | "jsx" | "ts" | "tsx" | "html" | "css" | "scss" | "sass" | "less" |
                // Python
                "py" | "pyx" | "pxd" |
                // JVM
                "java" | "kt" | "scala" | "groovy" | "clj" |
                // .NET
                "cs" | "fs" | "vb" |
                // Functional
                "ml" | "mli" | "hs" | "lhs" | "elm" |
                // Other
                "go" | "rb" | "swift" | "m" | "mm" | "r" | "lua" | "pl" | "pm" |
                "sh" | "bash" | "zsh" | "ps1" | "sql" | "dart" | "zig" |
                // Config/Data
                "json" | "yaml" | "yml" | "toml" | "ini" | "xml"
            )
        } else {
            // Files without extension - check for shebang or specific names
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                matches!(name.as_ref(), "Makefile" | "Dockerfile" | "CMakeLists.txt")
            } else {
                false
            }
        }
    }
}
