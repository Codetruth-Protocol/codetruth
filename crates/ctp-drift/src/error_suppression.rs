//! Error Suppression Detection
//!
//! Detects patterns where errors are hidden rather than properly handled.
//! This is a form of "silent drift" where bugs are masked instead of fixed.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{DriftFinding, DriftSeverity, DriftType, Location};

/// Types of error suppression patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionType {
    /// Empty catch/except block
    EmptyCatch,
    /// Catch that only logs but doesn't handle
    CatchAndLog,
    /// Promise .catch(() => {}) or similar
    SilentPromiseCatch,
    /// Removed validation/assertion
    RemovedValidation,
    /// Added unsafe unwrap/expect
    UnsafeUnwrap,
    /// Disabled linting rule
    DisabledLint,
    /// Null coalescing hiding errors
    SilentDefault,
    /// Broad exception catch (catch Exception, catch (...))
    BroadCatch,
}

impl SuppressionType {
    pub fn severity(&self) -> DriftSeverity {
        match self {
            SuppressionType::EmptyCatch => DriftSeverity::High,
            SuppressionType::CatchAndLog => DriftSeverity::Medium,
            SuppressionType::SilentPromiseCatch => DriftSeverity::High,
            SuppressionType::RemovedValidation => DriftSeverity::Critical,
            SuppressionType::UnsafeUnwrap => DriftSeverity::High,
            SuppressionType::DisabledLint => DriftSeverity::Medium,
            SuppressionType::SilentDefault => DriftSeverity::Low,
            SuppressionType::BroadCatch => DriftSeverity::Medium,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SuppressionType::EmptyCatch => "Empty catch block silently ignores errors",
            SuppressionType::CatchAndLog => "Error is logged but not handled",
            SuppressionType::SilentPromiseCatch => "Promise error silently ignored",
            SuppressionType::RemovedValidation => "Input validation was removed",
            SuppressionType::UnsafeUnwrap => "Unsafe unwrap may panic on error",
            SuppressionType::DisabledLint => "Linting rule disabled",
            SuppressionType::SilentDefault => "Error hidden by default value",
            SuppressionType::BroadCatch => "Overly broad exception catch",
        }
    }

    pub fn remediation(&self) -> &'static str {
        match self {
            SuppressionType::EmptyCatch => "Add proper error handling or re-throw the error",
            SuppressionType::CatchAndLog => "Handle the error appropriately or propagate it",
            SuppressionType::SilentPromiseCatch => "Handle promise rejection or let it propagate",
            SuppressionType::RemovedValidation => "Restore validation or document why it's not needed",
            SuppressionType::UnsafeUnwrap => "Use proper error handling (match, if let, ?) instead",
            SuppressionType::DisabledLint => "Fix the underlying issue instead of disabling the rule",
            SuppressionType::SilentDefault => "Handle the error case explicitly",
            SuppressionType::BroadCatch => "Catch specific exception types",
        }
    }
}

/// A detected suppression pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionFinding {
    pub suppression_type: SuppressionType,
    pub location: Location,
    pub code_snippet: String,
    pub severity: DriftSeverity,
    pub is_new: bool,
}

/// Configuration for suppression detection
#[derive(Debug, Clone)]
pub struct SuppressionConfig {
    /// Whether to flag catch-and-log as suppression
    pub flag_catch_and_log: bool,
    /// Whether to flag disabled lint rules
    pub flag_disabled_lint: bool,
    /// Minimum severity to report
    pub min_severity: DriftSeverity,
}

impl Default for SuppressionConfig {
    fn default() -> Self {
        Self {
            flag_catch_and_log: true,
            flag_disabled_lint: true,
            min_severity: DriftSeverity::Low,
        }
    }
}

/// Detects error suppression patterns in code
pub struct ErrorSuppressionDetector {
    config: SuppressionConfig,
    patterns: HashMap<String, Vec<SuppressionPattern>>,
}

#[derive(Clone)]
struct SuppressionPattern {
    suppression_type: SuppressionType,
    regex: Regex,
    context_lines: usize,
}

impl ErrorSuppressionDetector {
    pub fn new(config: SuppressionConfig) -> Self {
        let mut detector = Self {
            config,
            patterns: HashMap::new(),
        };
        detector.init_patterns();
        detector
    }

    fn init_patterns(&mut self) {
        // JavaScript/TypeScript patterns
        let js_patterns = vec![
            // Empty catch
            SuppressionPattern {
                suppression_type: SuppressionType::EmptyCatch,
                regex: Regex::new(r"catch\s*\([^)]*\)\s*\{\s*\}").unwrap(),
                context_lines: 1,
            },
            // Silent promise catch
            SuppressionPattern {
                suppression_type: SuppressionType::SilentPromiseCatch,
                regex: Regex::new(r"\.catch\s*\(\s*\(\s*\)\s*=>\s*\{\s*\}\s*\)").unwrap(),
                context_lines: 0,
            },
            SuppressionPattern {
                suppression_type: SuppressionType::SilentPromiseCatch,
                regex: Regex::new(r"\.catch\s*\(\s*\(\s*\)\s*=>\s*null\s*\)").unwrap(),
                context_lines: 0,
            },
            SuppressionPattern {
                suppression_type: SuppressionType::SilentPromiseCatch,
                regex: Regex::new(r"\.catch\s*\(\s*\(\s*\)\s*=>\s*undefined\s*\)").unwrap(),
                context_lines: 0,
            },
            // Disabled ESLint
            SuppressionPattern {
                suppression_type: SuppressionType::DisabledLint,
                regex: Regex::new(r"//\s*eslint-disable").unwrap(),
                context_lines: 0,
            },
            // TypeScript ignore
            SuppressionPattern {
                suppression_type: SuppressionType::DisabledLint,
                regex: Regex::new(r"//\s*@ts-ignore").unwrap(),
                context_lines: 0,
            },
            SuppressionPattern {
                suppression_type: SuppressionType::DisabledLint,
                regex: Regex::new(r"//\s*@ts-expect-error").unwrap(),
                context_lines: 0,
            },
            // Catch and log only
            SuppressionPattern {
                suppression_type: SuppressionType::CatchAndLog,
                regex: Regex::new(r"catch\s*\([^)]*\)\s*\{\s*console\.(log|error|warn)\s*\([^)]*\)\s*;?\s*\}").unwrap(),
                context_lines: 2,
            },
        ];
        self.patterns.insert("javascript".to_string(), js_patterns.clone());
        self.patterns.insert("typescript".to_string(), js_patterns);

        // Python patterns
        let py_patterns = vec![
            // Empty except
            SuppressionPattern {
                suppression_type: SuppressionType::EmptyCatch,
                regex: Regex::new(r"except.*:\s*pass\s*$").unwrap(),
                context_lines: 1,
            },
            // Bare except
            SuppressionPattern {
                suppression_type: SuppressionType::BroadCatch,
                regex: Regex::new(r"except\s*:").unwrap(),
                context_lines: 1,
            },
            // Catch Exception (too broad)
            SuppressionPattern {
                suppression_type: SuppressionType::BroadCatch,
                regex: Regex::new(r"except\s+Exception\s*:").unwrap(),
                context_lines: 1,
            },
            // Type ignore
            SuppressionPattern {
                suppression_type: SuppressionType::DisabledLint,
                regex: Regex::new(r"#\s*type:\s*ignore").unwrap(),
                context_lines: 0,
            },
            // noqa
            SuppressionPattern {
                suppression_type: SuppressionType::DisabledLint,
                regex: Regex::new(r"#\s*noqa").unwrap(),
                context_lines: 0,
            },
        ];
        self.patterns.insert("python".to_string(), py_patterns);

        // Rust patterns
        let rust_patterns = vec![
            // Unsafe unwrap
            SuppressionPattern {
                suppression_type: SuppressionType::UnsafeUnwrap,
                regex: Regex::new(r"\.unwrap\(\)").unwrap(),
                context_lines: 0,
            },
            // Unsafe expect without good message
            SuppressionPattern {
                suppression_type: SuppressionType::UnsafeUnwrap,
                regex: Regex::new(r#"\.expect\s*\(\s*"[^"]{0,20}"\s*\)"#).unwrap(),
                context_lines: 0,
            },
            // Allow unused
            SuppressionPattern {
                suppression_type: SuppressionType::DisabledLint,
                regex: Regex::new(r"#\[allow\(").unwrap(),
                context_lines: 0,
            },
            // Underscore to ignore Result
            SuppressionPattern {
                suppression_type: SuppressionType::SilentDefault,
                regex: Regex::new(r"let\s+_\s*=.*\?").unwrap(),
                context_lines: 0,
            },
        ];
        self.patterns.insert("rust".to_string(), rust_patterns);

        // Go patterns
        let go_patterns = vec![
            // Ignored error
            SuppressionPattern {
                suppression_type: SuppressionType::SilentDefault,
                regex: Regex::new(r",\s*_\s*:?=.*\(").unwrap(),
                context_lines: 0,
            },
            // Empty error check
            SuppressionPattern {
                suppression_type: SuppressionType::EmptyCatch,
                regex: Regex::new(r"if\s+err\s*!=\s*nil\s*\{\s*\}").unwrap(),
                context_lines: 1,
            },
        ];
        self.patterns.insert("go".to_string(), go_patterns);

        // Java patterns
        let java_patterns = vec![
            // Empty catch
            SuppressionPattern {
                suppression_type: SuppressionType::EmptyCatch,
                regex: Regex::new(r"catch\s*\([^)]+\)\s*\{\s*\}").unwrap(),
                context_lines: 1,
            },
            // Catch Exception (too broad)
            SuppressionPattern {
                suppression_type: SuppressionType::BroadCatch,
                regex: Regex::new(r"catch\s*\(\s*Exception\s+").unwrap(),
                context_lines: 1,
            },
            // SuppressWarnings
            SuppressionPattern {
                suppression_type: SuppressionType::DisabledLint,
                regex: Regex::new(r"@SuppressWarnings").unwrap(),
                context_lines: 0,
            },
        ];
        self.patterns.insert("java".to_string(), java_patterns);
    }

    /// Detect suppression patterns in code
    pub fn detect(&self, code: &str, language: &str, file_path: &str) -> Vec<SuppressionFinding> {
        let mut findings = vec![];

        let patterns = match self.patterns.get(language) {
            Some(p) => p,
            None => return findings,
        };

        let lines: Vec<&str> = code.lines().collect();

        for pattern in patterns {
            // Skip based on config
            if !self.config.flag_disabled_lint 
                && pattern.suppression_type == SuppressionType::DisabledLint {
                continue;
            }
            if !self.config.flag_catch_and_log 
                && pattern.suppression_type == SuppressionType::CatchAndLog {
                continue;
            }

            for (line_num, line) in lines.iter().enumerate() {
                if pattern.regex.is_match(line) {
                    let severity = pattern.suppression_type.severity();
                    
                    if severity < self.config.min_severity {
                        continue;
                    }

                    // Get context
                    let start = line_num.saturating_sub(pattern.context_lines);
                    let end = (line_num + pattern.context_lines + 1).min(lines.len());
                    let snippet: String = lines[start..end].join("\n");

                    findings.push(SuppressionFinding {
                        suppression_type: pattern.suppression_type,
                        location: Location {
                            file: file_path.to_string(),
                            line_start: line_num + 1,
                            line_end: line_num + 1,
                        },
                        code_snippet: snippet,
                        severity,
                        is_new: false,
                    });
                }
            }
        }

        findings
    }

    /// Detect NEW suppression patterns by comparing old and new code
    pub fn detect_new_suppressions(
        &self,
        old_code: &str,
        new_code: &str,
        language: &str,
        file_path: &str,
    ) -> Vec<SuppressionFinding> {
        let old_findings = self.detect(old_code, language, file_path);
        let mut new_findings = self.detect(new_code, language, file_path);

        // Mark findings that are new (not in old code)
        let old_snippets: std::collections::HashSet<_> = old_findings
            .iter()
            .map(|f| (&f.suppression_type, f.code_snippet.trim()))
            .collect();

        for finding in &mut new_findings {
            let key = (&finding.suppression_type, finding.code_snippet.trim());
            finding.is_new = !old_snippets.contains(&key);
        }

        // Return only new suppressions
        new_findings.into_iter().filter(|f| f.is_new).collect()
    }

    /// Convert suppression findings to drift findings
    pub fn to_drift_findings(&self, suppressions: Vec<SuppressionFinding>) -> Vec<DriftFinding> {
        suppressions
            .into_iter()
            .map(|s| DriftFinding {
                drift_type: DriftType::Implementation,
                severity: s.severity,
                expected: "Proper error handling".to_string(),
                actual: format!("{}: {}", 
                    s.suppression_type.description(),
                    s.code_snippet.lines().next().unwrap_or("")
                ),
                location: Some(s.location),
                remediation: s.suppression_type.remediation().to_string(),
            })
            .collect()
    }
}

impl Default for ErrorSuppressionDetector {
    fn default() -> Self {
        Self::new(SuppressionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_empty_catch_js() {
        let detector = ErrorSuppressionDetector::default();
        let code = r#"
try {
    doSomething();
} catch (e) { }
"#;
        let findings = detector.detect(code, "javascript", "test.js");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].suppression_type, SuppressionType::EmptyCatch);
    }

    #[test]
    fn test_detect_silent_promise_catch() {
        let detector = ErrorSuppressionDetector::default();
        let code = "fetchData().catch(() => {})";
        let findings = detector.detect(code, "javascript", "test.js");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].suppression_type, SuppressionType::SilentPromiseCatch);
    }

    #[test]
    fn test_detect_python_bare_except() {
        let detector = ErrorSuppressionDetector::default();
        let code = r#"
try:
    do_something()
except:
    pass
"#;
        let findings = detector.detect(code, "python", "test.py");
        assert!(findings.len() >= 1);
    }

    #[test]
    fn test_detect_rust_unwrap() {
        let detector = ErrorSuppressionDetector::default();
        let code = "let value = some_option.unwrap();";
        let findings = detector.detect(code, "rust", "test.rs");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].suppression_type, SuppressionType::UnsafeUnwrap);
    }

    #[test]
    fn test_detect_new_suppressions() {
        let detector = ErrorSuppressionDetector::default();
        let old_code = "let x = 1;";
        let new_code = r#"
try {
    doSomething();
} catch (e) { }
"#;
        let findings = detector.detect_new_suppressions(old_code, new_code, "javascript", "test.js");
        assert!(!findings.is_empty());
        assert!(findings[0].is_new);
    }

    #[test]
    fn test_eslint_disable() {
        let detector = ErrorSuppressionDetector::default();
        let code = "// eslint-disable-next-line no-unused-vars";
        let findings = detector.detect(code, "javascript", "test.js");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].suppression_type, SuppressionType::DisabledLint);
    }
}
