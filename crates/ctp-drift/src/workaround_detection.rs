//! Workaround Detection
//!
//! Detects when code changes implement workarounds instead of fixing root causes.
//! Example: Adding duplicate detection to a toast library instead of fixing
//! the code that calls toast() twice.

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{DriftFinding, DriftSeverity, DriftType, Location};

/// Types of workaround patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkaroundType {
    /// Adding deduplication logic instead of fixing duplicate calls
    Deduplication,
    /// Adding defensive null checks instead of ensuring valid data
    DefensiveNullCheck,
    /// Adding retry logic to mask flaky behavior
    RetryMasking,
    /// Adding timeouts to hide performance issues
    TimeoutWorkaround,
    /// Adding caching to hide inefficient queries
    CachingWorkaround,
    /// Adding feature flags to hide broken functionality
    FeatureFlagHiding,
    /// Adding complex state management to avoid fixing race conditions
    StateComplexity,
}

impl WorkaroundType {
    pub fn severity(&self) -> DriftSeverity {
        match self {
            WorkaroundType::Deduplication => DriftSeverity::Medium,
            WorkaroundType::DefensiveNullCheck => DriftSeverity::Medium,
            WorkaroundType::RetryMasking => DriftSeverity::High,
            WorkaroundType::TimeoutWorkaround => DriftSeverity::High,
            WorkaroundType::CachingWorkaround => DriftSeverity::Medium,
            WorkaroundType::FeatureFlagHiding => DriftSeverity::Critical,
            WorkaroundType::StateComplexity => DriftSeverity::High,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            WorkaroundType::Deduplication => "Added deduplication logic instead of fixing duplicate calls",
            WorkaroundType::DefensiveNullCheck => "Added defensive null checks instead of ensuring valid data",
            WorkaroundType::RetryMasking => "Added retry logic to mask flaky behavior",
            WorkaroundType::TimeoutWorkaround => "Added timeout to hide performance issues",
            WorkaroundType::CachingWorkaround => "Added caching to hide inefficient queries",
            WorkaroundType::FeatureFlagHiding => "Added feature flag to hide broken functionality",
            WorkaroundType::StateComplexity => "Added complex state management to avoid fixing race conditions",
        }
    }

    pub fn remediation(&self) -> &'static str {
        match self {
            WorkaroundType::Deduplication => "Find and fix the code that makes duplicate calls",
            WorkaroundType::DefensiveNullCheck => "Ensure data is valid at the source",
            WorkaroundType::RetryMasking => "Fix the underlying flakiness/race condition",
            WorkaroundType::TimeoutWorkaround => "Optimize the slow operation",
            WorkaroundType::CachingWorkaround => "Optimize the query or reduce call frequency",
            WorkaroundType::FeatureFlagHiding => "Fix the broken functionality or remove the feature",
            WorkaroundType::StateComplexity => "Fix the race condition at its source",
        }
    }
}

/// Detected workaround pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkaroundFinding {
    pub workaround_type: WorkaroundType,
    pub location: Location,
    pub code_snippet: String,
    pub severity: DriftSeverity,
    pub evidence: String,
}

/// Detects workaround patterns in code changes
pub struct WorkaroundDetector {
    dedup_patterns: Vec<Regex>,
    null_check_patterns: Vec<Regex>,
    retry_patterns: Vec<Regex>,
    timeout_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
}

impl WorkaroundDetector {
    pub fn new() -> Self {
        Self {
            dedup_patterns: vec![
                // Deduplication patterns
                Regex::new(r"(?i)(dedupe|dedup|duplicate.*check|seen.*set|prevent.*duplicate)").unwrap(),
                Regex::new(r"new Set\(\).*\.has\(").unwrap(),
                Regex::new(r"\.filter\(.*unique").unwrap(),
                Regex::new(r"if\s*\(\s*!?.*\.includes\(").unwrap(),
                Regex::new(r"const\s+\w+\s*=\s*\[\.\.\.\s*new\s+Set\(").unwrap(),
            ],
            null_check_patterns: vec![
                // Defensive null checks (especially in new code)
                Regex::new(r"if\s*\(\s*\w+\s*[!=]=\s*null\s*\|\|\s*\w+\s*[!=]=\s*undefined\s*\)").unwrap(),
                Regex::new(r"\?\?.*\?\?").unwrap(), // Multiple null coalescing
                Regex::new(r"\.?\..*\.?\.").unwrap(), // Multiple optional chaining
            ],
            retry_patterns: vec![
                Regex::new(r"(?i)(retry|retries|attempt|max.*attempts)").unwrap(),
                Regex::new(r"for\s*\(\s*let\s+\w+\s*=\s*0.*attempts").unwrap(),
                Regex::new(r"while\s*\(.*retry").unwrap(),
            ],
            timeout_patterns: vec![
                Regex::new(r"setTimeout\s*\(.*\d{3,}").unwrap(), // Timeouts > 100ms
                Regex::new(r"sleep\s*\(").unwrap(),
                Regex::new(r"await\s+new\s+Promise.*setTimeout").unwrap(),
            ],
            cache_patterns: vec![
                Regex::new(r"(?i)(cache|memoize|memo)").unwrap(),
                Regex::new(r"useMemo|useCallback").unwrap(),
                Regex::new(r"new\s+Map\(\).*\.set\(").unwrap(),
            ],
        }
    }

    /// Detect workarounds in new code
    pub fn detect(&self, old_code: &str, new_code: &str, file_path: &str) -> Vec<WorkaroundFinding> {
        let mut findings = vec![];

        // Get only the added lines
        let added_lines = self.get_added_lines(old_code, new_code);

        for (line_num, line) in added_lines {
            // Check for deduplication
            if self.dedup_patterns.iter().any(|p| p.is_match(line)) {
                // Check if this is in a utility/library file (more suspicious)
                let is_library = file_path.contains("/lib/") 
                    || file_path.contains("/utils/") 
                    || file_path.contains("/components/")
                    || file_path.contains("toast")
                    || file_path.contains("notification");

                if is_library {
                    findings.push(WorkaroundFinding {
                        workaround_type: WorkaroundType::Deduplication,
                        location: Location {
                            file: file_path.to_string(),
                            line_start: line_num,
                            line_end: line_num,
                        },
                        code_snippet: line.to_string(),
                        severity: DriftSeverity::High, // Higher severity in libraries
                        evidence: "Deduplication logic added to library/utility instead of fixing duplicate calls".to_string(),
                    });
                } else {
                    findings.push(WorkaroundFinding {
                        workaround_type: WorkaroundType::Deduplication,
                        location: Location {
                            file: file_path.to_string(),
                            line_start: line_num,
                            line_end: line_num,
                        },
                        code_snippet: line.to_string(),
                        severity: WorkaroundType::Deduplication.severity(),
                        evidence: "Deduplication logic added - verify this isn't masking duplicate calls".to_string(),
                    });
                }
            }

            // Check for excessive null checks
            let null_check_count = self.null_check_patterns.iter()
                .filter(|p| p.is_match(line))
                .count();
            
            if null_check_count >= 2 {
                findings.push(WorkaroundFinding {
                    workaround_type: WorkaroundType::DefensiveNullCheck,
                    location: Location {
                        file: file_path.to_string(),
                        line_start: line_num,
                        line_end: line_num,
                    },
                    code_snippet: line.to_string(),
                    severity: WorkaroundType::DefensiveNullCheck.severity(),
                    evidence: "Multiple defensive null checks - data should be validated at source".to_string(),
                });
            }

            // Check for retry logic
            if self.retry_patterns.iter().any(|p| p.is_match(line)) {
                findings.push(WorkaroundFinding {
                    workaround_type: WorkaroundType::RetryMasking,
                    location: Location {
                        file: file_path.to_string(),
                        line_start: line_num,
                        line_end: line_num,
                    },
                    code_snippet: line.to_string(),
                    severity: WorkaroundType::RetryMasking.severity(),
                    evidence: "Retry logic added - may be masking flaky behavior or race conditions".to_string(),
                });
            }

            // Check for timeout workarounds
            if self.timeout_patterns.iter().any(|p| p.is_match(line)) {
                findings.push(WorkaroundFinding {
                    workaround_type: WorkaroundType::TimeoutWorkaround,
                    location: Location {
                        file: file_path.to_string(),
                        line_start: line_num,
                        line_end: line_num,
                    },
                    code_snippet: line.to_string(),
                    severity: WorkaroundType::TimeoutWorkaround.severity(),
                    evidence: "Timeout/sleep added - may be hiding performance issues or race conditions".to_string(),
                });
            }

            // Check for caching workarounds
            if self.cache_patterns.iter().any(|p| p.is_match(line)) {
                // Only flag if this is a new cache in a component (not a utility)
                if !file_path.contains("/cache/") && !file_path.contains("/memo/") {
                    findings.push(WorkaroundFinding {
                        workaround_type: WorkaroundType::CachingWorkaround,
                        location: Location {
                            file: file_path.to_string(),
                            line_start: line_num,
                            line_end: line_num,
                        },
                        code_snippet: line.to_string(),
                        severity: WorkaroundType::CachingWorkaround.severity(),
                        evidence: "Caching added - verify this isn't hiding inefficient queries or excessive calls".to_string(),
                    });
                }
            }
        }

        findings
    }

    /// Extract added lines from diff
    fn get_added_lines<'a>(&self, old_code: &'a str, new_code: &'a str) -> Vec<(usize, &'a str)> {
        let old_lines: Vec<&str> = old_code.lines().collect();
        let new_lines: Vec<&str> = new_code.lines().collect();

        let mut added = vec![];

        // Simple diff: lines in new but not in old
        for (i, new_line) in new_lines.iter().enumerate() {
            let trimmed = new_line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Check if this line exists in old code
            if !old_lines.iter().any(|old_line| old_line.trim() == trimmed) {
                added.push((i + 1, *new_line));
            }
        }

        added
    }

    /// Convert workaround findings to drift findings
    pub fn to_drift_findings(&self, workarounds: Vec<WorkaroundFinding>) -> Vec<DriftFinding> {
        workarounds
            .into_iter()
            .map(|w| DriftFinding {
                drift_type: DriftType::Implementation,
                severity: w.severity,
                expected: "Root cause fix".to_string(),
                actual: format!("{}: {}", w.workaround_type.description(), w.evidence),
                location: Some(w.location),
                remediation: w.workaround_type.remediation().to_string(),
            })
            .collect()
    }
}

impl Default for WorkaroundDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_toast_deduplication() {
        let detector = WorkaroundDetector::new();
        
        let old_code = r#"
export function showToast(message: string) {
    toast.show(message);
}
"#;

        let new_code = r#"
const seenToasts = new Set<string>();

export function showToast(message: string) {
    if (seenToasts.has(message)) {
        return; // Prevent duplicate
    }
    seenToasts.add(message);
    toast.show(message);
}
"#;

        let findings = detector.detect(old_code, new_code, "src/lib/toast.ts");
        
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| {
            matches!(f.workaround_type, WorkaroundType::Deduplication)
                && f.severity == DriftSeverity::High // High because it's in a library
        }));
    }

    #[test]
    fn test_detect_retry_masking() {
        let detector = WorkaroundDetector::new();
        
        let old_code = "const data = await fetchData();";
        let new_code = r#"
let retries = 3;
let data;
while (retries > 0) {
    try {
        data = await fetchData();
        break;
    } catch (e) {
        retries--;
    }
}
"#;

        let findings = detector.detect(old_code, new_code, "src/api/client.ts");
        
        assert!(findings.iter().any(|f| {
            matches!(f.workaround_type, WorkaroundType::RetryMasking)
        }));
    }

    #[test]
    fn test_detect_defensive_null_checks() {
        let detector = WorkaroundDetector::new();
        
        let old_code = "const name = user.name;";
        let new_code = "const name = user?.name ?? user?.username ?? 'Unknown';";

        let findings = detector.detect(old_code, new_code, "src/components/UserProfile.tsx");
        
        assert!(findings.iter().any(|f| {
            matches!(f.workaround_type, WorkaroundType::DefensiveNullCheck)
        }));
    }

    #[test]
    fn test_no_false_positive_for_legitimate_cache() {
        let detector = WorkaroundDetector::new();
        
        let old_code = "";
        let new_code = r#"
export const memoizedCalculation = useMemo(() => {
    return expensiveCalculation(data);
}, [data]);
"#;

        let findings = detector.detect(old_code, new_code, "src/cache/calculations.ts");
        
        // Should not flag caching in a cache utility file
        assert!(findings.is_empty() || findings.iter().all(|f| {
            !matches!(f.workaround_type, WorkaroundType::CachingWorkaround)
        }));
    }
}
