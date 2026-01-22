//! Main CodeTruth analysis engine

use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use tracing::{debug, info, instrument, warn};

use ctp_drift::{DriftDetector, DriftConfig};
use ctp_parser::CTPParser;
use ctp_policy::PolicyEngine;

use crate::error::{CTPError, CTPResult};
use crate::models::*;
use crate::detectors::DetectorsRegistry;
use crate::naming_patterns::{NamingPatternDetector, NamingAnalysisResult};

struct IntentInferenceResult {
    inferred_intent: String,
    confidence: f64,
    business_context: String,
    technical_rationale: String,
}
use crate::CTP_VERSION;

/// Configuration for the CodeTruth engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Enable LLM enhancement for complex analysis
    pub enable_llm: bool,

    /// LLM provider (anthropic, openai, local)
    pub llm_provider: Option<String>,

    /// LLM model name
    pub llm_model: Option<String>,

    /// API key for LLM provider
    pub llm_api_key: Option<String>,

    /// Maximum file size to analyze (bytes)
    pub max_file_size: usize,

    /// Supported languages
    pub languages: Vec<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            enable_llm: false,
            llm_provider: None,
            llm_model: None,
            llm_api_key: None,
            max_file_size: 10 * 1024 * 1024, // 10MB
            languages: vec![
                "python".into(),
                "javascript".into(),
                "typescript".into(),
                "rust".into(),
                "go".into(),
                "java".into(),
            ],
        }
    }
}

/// The main CodeTruth analysis engine
pub struct CodeTruthEngine {
    config: EngineConfig,
    #[allow(dead_code)] // Will be used for AST-based analysis in future
    parser: Arc<RwLock<CTPParser>>,
    drift_detector: DriftDetector,
    policy_engine: Arc<RwLock<PolicyEngine>>,
    naming_detector: Arc<RwLock<NamingPatternDetector>>,
    detectors: DetectorsRegistry,
}

impl CodeTruthEngine {
    /// Create a new engine with the given configuration
    pub fn new(config: EngineConfig) -> Self {
        info!("Initializing CodeTruth engine v{}", CTP_VERSION);
        
        let parser = match CTPParser::new() {
            Ok(p) => Arc::new(RwLock::new(p)),
            Err(e) => {
                warn!("Failed to initialize parser: {}, using fallback", e);
                Arc::new(RwLock::new(CTPParser::default()))
            }
        };
        
        let drift_detector = DriftDetector::new(DriftConfig::default());
        let policy_engine = Arc::new(RwLock::new(PolicyEngine::new()));
        let naming_detector = Arc::new(RwLock::new(NamingPatternDetector::new()));
        let detectors = DetectorsRegistry::new();
        
        Self {
            config,
            parser,
            drift_detector,
            policy_engine,
            naming_detector,
            detectors,
        }
    }

    /// Create a new engine with default configuration
    pub fn default() -> Self {
        Self::new(EngineConfig::default())
    }
    
    /// Load policies from a directory
    pub fn load_policies(&self, policy_dir: &Path) -> CTPResult<usize> {
        let mut engine = self.policy_engine.write();
        let mut count = 0;
        
        if policy_dir.is_dir() {
            for entry in std::fs::read_dir(policy_dir).map_err(CTPError::IoError)? {
                let entry = entry.map_err(CTPError::IoError)?;
                let path = entry.path();
                if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                    let content = std::fs::read_to_string(&path).map_err(CTPError::IoError)?;
                    if let Err(e) = engine.load_policy_from_str(&content) {
                        warn!("Failed to load policy {:?}: {}", path, e);
                    } else {
                        count += 1;
                    }
                }
            }
        }
        
        Ok(count)
    }
    
    /// Analyze multiple files in parallel
    pub async fn analyze_files(&self, paths: &[&Path]) -> Vec<CTPResult<ExplanationGraph>> {
        paths
            .par_iter()
            .map(|path| {
                tokio::runtime::Handle::current()
                    .block_on(self.analyze_file(path))
            })
            .collect()
    }

    /// Analyze naming patterns in a directory
    pub fn analyze_naming_patterns(&self, directory_path: &Path) -> CTPResult<NamingAnalysisResult> {
        let mut detector = self.naming_detector.write();
        detector.analyze_directory(directory_path)
            .map_err(|e| CTPError::AnalysisError(format!("Naming pattern analysis failed: {}", e)))
    }

    /// Analyze a single file and generate explanation graph
    #[instrument(skip(self))]
    pub async fn analyze_file(&self, path: &Path) -> CTPResult<ExplanationGraph> {
        let start = std::time::Instant::now();
        debug!("Analyzing file: {}", path.display());

        // Read file content
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| CTPError::IoError(e))?;

        // Check file size
        if content.len() > self.config.max_file_size {
            return Err(CTPError::AnalysisError(format!(
                "File too large: {} bytes (max: {})",
                content.len(),
                self.config.max_file_size
            )));
        }

        // Detect language
        let language = self.detect_language(path)?;

        // Generate content hash
        let content_hash = self.hash_content(&content);

        // Parse with tree-sitter for complexity metrics
        let complexity_score = {
            let mut parser = self.parser.write();
            if let Ok(lang) = ctp_parser::SupportedLanguage::from_extension(
                path.extension().and_then(|e| e.to_str()).unwrap_or("")
            ).ok_or(()) {
                if let Ok(parsed) = parser.parse(&content, lang) {
                    // Normalize complexity: cyclomatic / functions, capped at 10
                    let func_count = parsed.complexity.function_count.max(1);
                    (parsed.complexity.cyclomatic as f64 / func_count as f64).min(10.0)
                } else {
                    0.0
                }
            } else {
                0.0
            }
        };

        // Extract module info
        let module = Module {
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            path: path.display().to_string(),
            language: language.clone(),
            lines_of_code: content.lines().count(),
            complexity_score,
            content_hash: content_hash.clone(),
        };

        // Extract declared intent from comments/docstrings
        let declared_intent = self.extract_declared_intent(&content, &language);

        // Analyze actual behavior
        let behavior = self.analyze_behavior(&content, &language);

        // Infer intent (rule-based for now, LLM optional)
        let intent = if self.config.enable_llm {
            let llm_result = self.llm_infer_intent(&content, &declared_intent).await?;
            Intent {
                declared_intent: declared_intent.clone(),
                inferred_intent: llm_result.inferred_intent,
                confidence: llm_result.confidence,
                business_context: llm_result.business_context,
                technical_rationale: llm_result.technical_rationale,
            }
        } else {
            let inferred_intent = self.rule_based_infer_intent(&content, &declared_intent);
            let confidence = self.calculate_confidence(&declared_intent, &inferred_intent);
            Intent {
                declared_intent,
                inferred_intent,
                confidence,
                business_context: String::new(),
                technical_rationale: String::new(),
            }
        };

        // Detect drift using ctp-drift
        let side_effect_strs: Vec<String> = behavior
            .side_effects
            .iter()
            .map(|s| s.description.clone())
            .collect();
        
        let drift_report = self.drift_detector.detect_behavior_drift(
            &intent.declared_intent,
            &behavior.actual_behavior,
            &side_effect_strs,
        );
        
        let mut drift = DriftAnalysis {
            drift_detected: drift_report.drift_detected,
            drift_severity: match drift_report.overall_severity {
                ctp_drift::DriftSeverity::None => DriftSeverity::None,
                ctp_drift::DriftSeverity::Low => DriftSeverity::Low,
                ctp_drift::DriftSeverity::Medium => DriftSeverity::Medium,
                ctp_drift::DriftSeverity::High => DriftSeverity::High,
                ctp_drift::DriftSeverity::Critical => DriftSeverity::Critical,
            },
            drift_details: drift_report
                .findings
                .into_iter()
                .map(|f| DriftDetail {
                    drift_type: match f.drift_type {
                        ctp_drift::DriftType::Intent => DriftType::Intent,
                        ctp_drift::DriftType::Policy => DriftType::Policy,
                        ctp_drift::DriftType::Assumption => DriftType::Assumption,
                        ctp_drift::DriftType::Implementation => DriftType::Implementation,
                    },
                    expected: f.expected,
                    actual: f.actual,
                    location: Location {
                        file: path.display().to_string(),
                        line_start: f.location.as_ref().map(|l| l.line_start).unwrap_or(0),
                        line_end: f.location.as_ref().map(|l| l.line_end).unwrap_or(0),
                    },
                    impact: Impact {
                        functional: "Review required".into(),
                        security: "Review required".into(),
                        performance: "Unknown".into(),
                        maintainability: "Review required".into(),
                    },
                    remediation: f.remediation,
                })
                .collect(),
        };

        // Run singular adapter detectors and append any implementation drift details
        let detector_details = self.detectors.run(&path.display().to_string(), &content);
        if !detector_details.is_empty() {
            drift.drift_detected = true;
            drift.drift_details.extend(detector_details);
            // Severity unchanged here; policy engine can escalate via violations.
        }

        // Evaluate policies
        let policy_results = {
            let engine = self.policy_engine.read();
            engine.evaluate(&path.display().to_string(), &content)
        };
        
        let policies = PolicyResults {
            evaluated_at: chrono::Utc::now().to_rfc3339(),
            policy_results: policy_results
                .into_iter()
                .map(|r| crate::models::PolicyResult {
                    policy_id: r.policy_id,
                    policy_name: r.policy_name,
                    status: match r.status {
                        ctp_policy::PolicyStatus::Pass => PolicyStatus::Pass,
                        ctp_policy::PolicyStatus::Fail => PolicyStatus::Fail,
                        ctp_policy::PolicyStatus::Warning => PolicyStatus::Warning,
                        ctp_policy::PolicyStatus::Skip => PolicyStatus::Skip,
                    },
                    violations: r.violations.into_iter().map(|v| Violation {
                        rule: v.rule_id,
                        severity: match v.severity {
                            ctp_policy::PolicySeverity::Info => ViolationSeverity::Info,
                            ctp_policy::PolicySeverity::Warning => ViolationSeverity::Warning,
                            ctp_policy::PolicySeverity::Error => ViolationSeverity::Error,
                            ctp_policy::PolicySeverity::Critical => ViolationSeverity::Critical,
                        },
                        message: v.message,
                        location: Location {
                            file: v.file,
                            line_start: v.line.unwrap_or(0),
                            line_end: v.line.unwrap_or(0),
                        },
                        evidence: v.evidence,
                    }).collect(),
                })
                .collect(),
        };

        // Build history (empty for first analysis)
        let history = History {
            previous_versions: vec![],
            evolution: Evolution {
                created_at: chrono::Utc::now().to_rfc3339(),
                last_modified: chrono::Utc::now().to_rfc3339(),
                modification_count: 0,
                stability_score: 1.0,
            },
        };

        // Build metadata
        let metadata = Metadata {
            generated_at: chrono::Utc::now().to_rfc3339(),
            generator: Generator {
                name: "CodeTruth".into(),
                version: crate::VERSION.into(),
                llm_provider: self.config.llm_provider.clone(),
                llm_model: self.config.llm_model.clone(),
            },
            extensions: serde_json::json!({}),
        };

        let elapsed = start.elapsed();
        info!("Analysis completed in {:?}", elapsed);

        Ok(ExplanationGraph {
            ctp_version: CTP_VERSION.into(),
            explanation_id: content_hash,
            module,
            intent,
            behavior,
            drift,
            policies,
            history,
            metadata,
        })
    }

    /// Analyze code from a string (useful for WASM)
    pub async fn analyze_string(
        &self,
        content: &str,
        language: &str,
        name: &str,
    ) -> CTPResult<ExplanationGraph> {
        let content_hash = self.hash_content(content);

        let module = Module {
            name: name.into(),
            path: String::new(),
            language: language.into(),
            lines_of_code: content.lines().count(),
            complexity_score: 0.0,
            content_hash: content_hash.clone(),
        };

        let declared_intent = self.extract_declared_intent(content, language);
        let behavior = self.analyze_behavior(content, language);
        let inferred_intent = self.rule_based_infer_intent(content, &declared_intent);
        let confidence = self.calculate_confidence(&declared_intent, &inferred_intent);

        let intent = Intent {
            declared_intent,
            inferred_intent,
            confidence,
            business_context: String::new(),
            technical_rationale: String::new(),
        };

        let drift = self.detect_drift_for_string(&intent, &behavior);

        Ok(ExplanationGraph {
            ctp_version: CTP_VERSION.into(),
            explanation_id: content_hash,
            module,
            intent,
            behavior,
            drift,
            policies: PolicyResults {
                evaluated_at: chrono::Utc::now().to_rfc3339(),
                policy_results: vec![],
            },
            history: History {
                previous_versions: vec![],
                evolution: Evolution {
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_modified: chrono::Utc::now().to_rfc3339(),
                    modification_count: 0,
                    stability_score: 1.0,
                },
            },
            metadata: Metadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator: Generator {
                    name: "CodeTruth".into(),
                    version: crate::VERSION.into(),
                    llm_provider: None,
                    llm_model: None,
                },
                extensions: serde_json::json!({}),
            },
        })
    }

    /// Detect programming language from file extension
    fn detect_language(&self, path: &Path) -> CTPResult<String> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let language = match ext {
            "py" => "python",
            "js" | "mjs" | "cjs" => "javascript",
            "ts" | "tsx" => "typescript",
            "rs" => "rust",
            "go" => "go",
            "java" => "java",
            "rb" => "ruby",
            "php" => "php",
            "c" | "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" => "cpp",
            "cs" => "csharp",
            _ => return Err(CTPError::UnsupportedLanguage(ext.into())),
        };

        Ok(language.into())
    }

    /// Generate SHA256 hash of content
    fn hash_content(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }

    /// Extract declared intent from comments and docstrings
    fn extract_declared_intent(&self, content: &str, language: &str) -> String {
        // Simple regex-based extraction for MVP
        // TODO: Use tree-sitter for proper AST-based extraction

        match language {
            "python" => {
                // Extract Python docstrings
                if let Some(start) = content.find("\"\"\"") {
                    if let Some(end) = content[start + 3..].find("\"\"\"") {
                        let docstring = &content[start + 3..start + 3 + end];
                        return docstring.trim().chars().take(280).collect();
                    }
                }
            }
            "javascript" | "typescript" => {
                // Extract JSDoc comments
                if let Some(start) = content.find("/**") {
                    if let Some(end) = content[start..].find("*/") {
                        let comment = &content[start + 3..start + end];
                        let cleaned: String = comment
                            .lines()
                            .map(|l| l.trim().trim_start_matches('*').trim())
                            .collect::<Vec<_>>()
                            .join(" ");
                        return cleaned.chars().take(280).collect();
                    }
                }
            }
            "rust" => {
                // Extract Rust doc comments
                let doc_lines: Vec<&str> = content
                    .lines()
                    .take_while(|l| l.trim().starts_with("///") || l.trim().starts_with("//!"))
                    .map(|l| l.trim().trim_start_matches("///").trim_start_matches("//!").trim())
                    .collect();
                if !doc_lines.is_empty() {
                    return doc_lines.join(" ").chars().take(280).collect();
                }
            }
            _ => {}
        }

        // Fallback: extract first comment
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with("//") {
                return trimmed
                    .trim_start_matches('#')
                    .trim_start_matches("//")
                    .trim()
                    .chars()
                    .take(280)
                    .collect();
            }
        }

        String::new()
    }

    /// Analyze actual behavior of the code
    fn analyze_behavior(&self, content: &str, language: &str) -> Behavior {
        // Try to parse with tree-sitter for better extraction
        let (entry_points, exit_points) = self.extract_entry_exit_points(content, language);

        let lines: Vec<&str> = content.lines().collect();

        // Count functions/methods
        let func_patterns = match language {
            "python" => vec!["def ", "async def "],
            "javascript" | "typescript" => vec!["function ", "const ", "async function "],
            "rust" => vec!["fn ", "pub fn ", "async fn "],
            "go" => vec!["func "],
            "java" => vec!["public ", "private ", "protected "],
            _ => vec!["function", "def", "fn"],
        };

        let func_count = lines
            .iter()
            .filter(|l| func_patterns.iter().any(|p| l.trim().starts_with(p)))
            .count();

        // Detect I/O operations
        let io_patterns = ["open(", "read(", "write(", "fetch(", "axios.", "fs.", "File::"];
        let has_io = lines
            .iter()
            .any(|l| io_patterns.iter().any(|p| l.contains(p)));

        // Detect database operations
        let db_patterns = ["SELECT ", "INSERT ", "UPDATE ", "DELETE ", "mongodb", "redis", "prisma", "sqlx"];
        let has_db = lines
            .iter()
            .any(|l| db_patterns.iter().any(|p| l.to_uppercase().contains(&p.to_uppercase())));

        // Detect network operations
        let net_patterns = ["http", "https", "fetch", "axios", "request", "socket"];
        let has_network = lines
            .iter()
            .any(|l| net_patterns.iter().any(|p| l.to_lowercase().contains(p)));

        // Build behavior description
        let mut parts = vec![];
        if func_count > 0 {
            parts.push(format!("{} function(s)", func_count));
        }
        if has_io {
            parts.push("file I/O".into());
        }
        if has_db {
            parts.push("database operations".into());
        }
        if has_network {
            parts.push("network calls".into());
        }

        let actual_behavior = if parts.is_empty() {
            "Simple logic".into()
        } else {
            format!("Performs {}", parts.join(", "))
        };

        // Build side effects
        let mut side_effects = vec![];
        if has_io {
            side_effects.push(SideEffect {
                effect_type: SideEffectType::Io,
                description: "File system operations detected".into(),
                risk_level: RiskLevel::Medium,
            });
        }
        if has_db {
            side_effects.push(SideEffect {
                effect_type: SideEffectType::Database,
                description: "Database operations detected".into(),
                risk_level: RiskLevel::High,
            });
        }
        if has_network {
            side_effects.push(SideEffect {
                effect_type: SideEffectType::Network,
                description: "Network operations detected".into(),
                risk_level: RiskLevel::Medium,
            });
        }

        Behavior {
            actual_behavior,
            entry_points,
            exit_points,
            side_effects,
            dependencies: vec![],
        }
    }

    /// Extract entry and exit points from code
    fn extract_entry_exit_points(&self, content: &str, language: &str) -> (Vec<EntryPoint>, Vec<ExitPoint>) {
        let mut entry_points = vec![];
        let mut exit_points = vec![];

        // Try to use tree-sitter parser
        if let Some(lang) = ctp_parser::SupportedLanguage::from_extension(
            match language {
                "python" => "py",
                "javascript" => "js",
                "typescript" => "ts",
                "rust" => "rs",
                "go" => "go",
                "java" => "java",
                _ => "",
            }
        ) {
            let mut parser = self.parser.write();
            if let Ok(parsed) = parser.parse(content, lang) {
                // Extract entry points from functions
                for func in &parsed.functions {
                    entry_points.push(EntryPoint {
                        function: func.name.clone(),
                        parameters: func.parameters.clone(),
                        preconditions: vec![],
                    });
                }

                // Infer exit points from return patterns
                let return_patterns = match language {
                    "python" => vec!["return "],
                    "javascript" | "typescript" => vec!["return ", "=> "],
                    "rust" => vec!["return ", "-> "],
                    "go" => vec!["return "],
                    "java" => vec!["return "],
                    _ => vec!["return "],
                };

                let has_returns = content.lines().any(|line| {
                    return_patterns.iter().any(|p| line.contains(p))
                });

                if has_returns {
                    exit_points.push(ExitPoint {
                        return_type: "mixed".into(),
                        possible_values: vec![],
                        postconditions: vec![],
                    });
                }
            }
        }

        (entry_points, exit_points)
    }

    /// Rule-based intent inference (no LLM)
    fn rule_based_infer_intent(&self, content: &str, declared_intent: &str) -> String {
        if !declared_intent.is_empty() {
            return declared_intent.to_string();
        }

        // Infer from code patterns
        let content_lower = content.to_lowercase();

        if content_lower.contains("test") || content_lower.contains("assert") {
            return "Test code for verifying functionality".into();
        }
        if content_lower.contains("auth") || content_lower.contains("login") {
            return "Authentication/authorization logic".into();
        }
        if content_lower.contains("payment") || content_lower.contains("charge") {
            return "Payment processing logic".into();
        }
        if content_lower.contains("api") || content_lower.contains("endpoint") {
            return "API endpoint handler".into();
        }
        if content_lower.contains("database") || content_lower.contains("query") {
            return "Database interaction logic".into();
        }

        "General purpose code module".into()
    }

    /// LLM-enhanced intent inference
    #[cfg(feature = "llm")]
    async fn llm_infer_intent(&self, content: &str, declared_intent: &str) -> CTPResult<IntentInferenceResult> {
        use ctp_llm::{LLMClient, LLMConfig, LLMProvider, IntentInference};
        
        // Check if we have LLM configuration
        let (provider, model, api_key) = match (
            &self.config.llm_provider,
            &self.config.llm_model,
            &self.config.llm_api_key,
        ) {
            (Some(provider), Some(model), Some(key)) => (provider.clone(), model.clone(), key.clone()),
            _ => {
                debug!("LLM not configured, falling back to rule-based inference");
                return Ok(self.rule_based_infer_intent(content, declared_intent));
            }
        };

        let llm_provider = match provider.to_lowercase().as_str() {
            "anthropic" => LLMProvider::Anthropic,
            "openai" => LLMProvider::OpenAI,
            "ollama" => LLMProvider::Ollama,
            _ => {
                warn!("Unknown LLM provider: {}, falling back to rule-based", provider);
                return Ok(self.rule_based_infer_intent(content, declared_intent));
            }
        };

        let config = LLMConfig {
            provider: llm_provider,
            model,
            api_key: Some(api_key),
            base_url: None,
            max_tokens: 1000,
        };

        let client = LLMClient::new(config);
        
        match client.infer_intent(content, declared_intent).await {
            Ok(inference) => {
                info!("LLM inference successful, confidence: {}", inference.confidence);
                Ok(IntentInferenceResult {
                    inferred_intent: inference.inferred_intent,
                    confidence: inference.confidence,
                    business_context: inference.business_context,
                    technical_rationale: inference.technical_rationale,
                })
            }
            Err(e) => {
                warn!("LLM inference failed: {}, falling back to rule-based", e);
                let inferred = self.rule_based_infer_intent(content, declared_intent);
                Ok(IntentInferenceResult {
                    inferred_intent: inferred,
                    confidence: 0.5,
                    business_context: String::new(),
                    technical_rationale: "Rule-based inference (LLM unavailable)".into(),
                })
            }
        }
    }

    #[cfg(not(feature = "llm"))]
    async fn llm_infer_intent(&self, _content: &str, declared_intent: &str) -> CTPResult<IntentInferenceResult> {
        let inferred = self.rule_based_infer_intent(_content, declared_intent);
        Ok(IntentInferenceResult {
            inferred_intent: inferred,
            confidence: 0.5,
            business_context: String::new(),
            technical_rationale: "Rule-based inference".into(),
        })
    }

    /// Calculate confidence score
    fn calculate_confidence(&self, declared: &str, inferred: &str) -> f64 {
        if declared.is_empty() {
            return 0.5; // Low confidence without declared intent
        }

        // Simple word overlap similarity
        let declared_lower = declared.to_lowercase();
        let declared_words: std::collections::HashSet<&str> =
            declared_lower.split_whitespace().collect();
        let inferred_lower = inferred.to_lowercase();
        let inferred_words: std::collections::HashSet<&str> =
            inferred_lower.split_whitespace().collect();

        let intersection = declared_words.intersection(&inferred_words).count();
        let union = declared_words.union(&inferred_words).count();

        if union == 0 {
            0.5
        } else {
            0.5 + (intersection as f64 / union as f64) * 0.5
        }
    }

    /// Detect drift between intent and behavior (for string analysis without file path)
    fn detect_drift_for_string(&self, intent: &Intent, behavior: &Behavior) -> DriftAnalysis {
        let side_effect_strs: Vec<String> = behavior
            .side_effects
            .iter()
            .map(|s| s.description.clone())
            .collect();
        
        let drift_report = self.drift_detector.detect_behavior_drift(
            &intent.declared_intent,
            &behavior.actual_behavior,
            &side_effect_strs,
        );
        
        DriftAnalysis {
            drift_detected: drift_report.drift_detected,
            drift_severity: match drift_report.overall_severity {
                ctp_drift::DriftSeverity::None => DriftSeverity::None,
                ctp_drift::DriftSeverity::Low => DriftSeverity::Low,
                ctp_drift::DriftSeverity::Medium => DriftSeverity::Medium,
                ctp_drift::DriftSeverity::High => DriftSeverity::High,
                ctp_drift::DriftSeverity::Critical => DriftSeverity::Critical,
            },
            drift_details: drift_report
                .findings
                .into_iter()
                .map(|f| DriftDetail {
                    drift_type: match f.drift_type {
                        ctp_drift::DriftType::Intent => DriftType::Intent,
                        ctp_drift::DriftType::Policy => DriftType::Policy,
                        ctp_drift::DriftType::Assumption => DriftType::Assumption,
                        ctp_drift::DriftType::Implementation => DriftType::Implementation,
                    },
                    expected: f.expected,
                    actual: f.actual,
                    location: Location {
                        file: String::new(),
                        line_start: f.location.as_ref().map(|l| l.line_start).unwrap_or(0),
                        line_end: f.location.as_ref().map(|l| l.line_end).unwrap_or(0),
                    },
                    impact: Impact {
                        functional: "Review required".into(),
                        security: "Review required".into(),
                        performance: "Unknown".into(),
                        maintainability: "Review required".into(),
                    },
                    remediation: f.remediation,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_string() {
        let engine = CodeTruthEngine::default();
        let code = r#"
def factorial(n):
    """Calculate the factorial of a number."""
    if n <= 1:
        return 1
    return n * factorial(n - 1)
"#;

        let result = engine
            .analyze_string(code, "python", "factorial.py")
            .await
            .unwrap();

        assert_eq!(result.module.language, "python");
        assert!(!result.intent.declared_intent.is_empty());
    }
}
