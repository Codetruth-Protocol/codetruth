//! # CTP Drift Detection
//!
//! Drift detection engine for CodeTruth Protocol.
//!
//! This crate provides algorithms for detecting drift between:
//! - Declared intent (from comments/docs) and inferred intent
//! - Intent and actual implementation behavior
//! - Current code and previous versions
//! - Error suppression patterns (hiding bugs instead of fixing them)

use std::collections::HashSet;

pub mod error_suppression;
pub mod workaround_detection;
pub mod stub_detection;
pub mod git_analysis;
pub mod documentation_drift;

pub use error_suppression::{
    ErrorSuppressionDetector, SuppressionConfig, SuppressionFinding, SuppressionType,
};
pub use workaround_detection::{
    WorkaroundDetector, WorkaroundFinding, WorkaroundType,
};
pub use stub_detection::{
    StubDetector, StubFinding, StubPattern, StubSeverity, StubStatistics,
};
pub use git_analysis::{
    GitAnalyzer, GitAnalysisConfig, GitDriftFinding, GitDriftType, GitDriftSeverity,
};
pub use documentation_drift::{
    DocumentationAnalyzer, DocumentationDriftFinding, DocDriftType, DocDriftSeverity,
};

use serde::{Deserialize, Serialize};
use tracing::debug;
use ctp_utils::text_similarity::jaccard_similarity;

/// Drift severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriftSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Types of drift that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriftType {
    /// Mismatch between declared and actual intent
    Intent,
    /// Policy violation
    Policy,
    /// Invalid or outdated assumptions
    Assumption,
    /// Implementation doesn't match specification
    Implementation,
}

/// A single drift finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftFinding {
    pub drift_type: DriftType,
    pub severity: DriftSeverity,
    pub expected: String,
    pub actual: String,
    pub location: Option<Location>,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
}

/// Result of drift analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub drift_detected: bool,
    pub overall_severity: DriftSeverity,
    pub findings: Vec<DriftFinding>,
    pub confidence: f64,
}

/// Configuration for drift detection
#[derive(Debug, Clone)]
pub struct DriftConfig {
    /// Minimum similarity threshold for "no drift"
    pub similarity_threshold: f64,
    /// Weight for semantic similarity
    pub semantic_weight: f64,
    /// Weight for structural similarity
    pub structural_weight: f64,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.7,
            semantic_weight: 0.6,
            structural_weight: 0.4,
        }
    }
}

/// Drift detection engine
pub struct DriftDetector {
    config: DriftConfig,
}

impl DriftDetector {
    pub fn new(config: DriftConfig) -> Self {
        Self { config }
    }

    /// Detect drift between declared intent and inferred intent
    pub fn detect_intent_drift(
        &self,
        declared_intent: &str,
        inferred_intent: &str,
    ) -> DriftReport {
        debug!("Detecting intent drift");

        let similarity = self.calculate_similarity(declared_intent, inferred_intent);
        let severity = self.severity_from_similarity(similarity);

        let mut findings = vec![];

        if severity != DriftSeverity::None {
            findings.push(DriftFinding {
                drift_type: DriftType::Intent,
                severity,
                expected: declared_intent.to_string(),
                actual: inferred_intent.to_string(),
                location: None,
                remediation: "Update documentation to match actual behavior or fix implementation".into(),
            });
        }

        DriftReport {
            drift_detected: !findings.is_empty(),
            overall_severity: severity,
            findings,
            confidence: similarity,
        }
    }

    /// Detect drift between intent and actual behavior
    pub fn detect_behavior_drift(
        &self,
        intent: &str,
        behavior: &str,
        side_effects: &[String],
    ) -> DriftReport {
        debug!("Detecting behavior drift");

        let mut findings = vec![];

        // Check for undocumented side effects
        if !side_effects.is_empty() && intent.is_empty() {
            findings.push(DriftFinding {
                drift_type: DriftType::Implementation,
                severity: DriftSeverity::Medium,
                expected: "Documented side effects".into(),
                actual: format!("Undocumented: {}", side_effects.join(", ")),
                location: None,
                remediation: "Add documentation describing the side effects".into(),
            });
        }

        // Check for dangerous operations without documentation
        let dangerous_patterns = ["delete", "drop", "truncate", "remove", "destroy"];
        let has_dangerous = behavior
            .to_lowercase()
            .split_whitespace()
            .any(|w| dangerous_patterns.iter().any(|p| w.contains(p)));

        if has_dangerous && !intent.to_lowercase().contains("delete")
            && !intent.to_lowercase().contains("remove")
        {
            findings.push(DriftFinding {
                drift_type: DriftType::Policy,
                severity: DriftSeverity::High,
                expected: "Dangerous operations should be documented".into(),
                actual: "Destructive operation without clear documentation".into(),
                location: None,
                remediation: "Add explicit documentation about destructive behavior".into(),
            });
        }

        let overall_severity = findings
            .iter()
            .map(|f| f.severity)
            .max()
            .unwrap_or(DriftSeverity::None);

        let confidence = if findings.is_empty() { 0.9 } else { 0.7 };
        DriftReport {
            drift_detected: !findings.is_empty(),
            overall_severity,
            findings,
            confidence,
        }
    }

    /// Detect drift between two versions of code
    pub fn detect_version_drift(
        &self,
        old_intent: &str,
        new_intent: &str,
        old_behavior: &str,
        new_behavior: &str,
    ) -> DriftReport {
        debug!("Detecting version drift");

        let mut findings = vec![];

        // Check if intent changed significantly
        let intent_similarity = self.calculate_similarity(old_intent, new_intent);
        if intent_similarity < self.config.similarity_threshold {
            findings.push(DriftFinding {
                drift_type: DriftType::Intent,
                severity: self.severity_from_similarity(intent_similarity),
                expected: old_intent.to_string(),
                actual: new_intent.to_string(),
                location: None,
                remediation: "Review intent change and update documentation".into(),
            });
        }

        // Check if behavior changed without intent change
        let behavior_similarity = self.calculate_similarity(old_behavior, new_behavior);
        if behavior_similarity < self.config.similarity_threshold
            && intent_similarity > self.config.similarity_threshold
        {
            findings.push(DriftFinding {
                drift_type: DriftType::Implementation,
                severity: DriftSeverity::Medium,
                expected: "Behavior consistent with unchanged intent".into(),
                actual: "Behavior changed without intent update".into(),
                location: None,
                remediation: "Update documentation to reflect behavior changes".into(),
            });
        }

        let overall_severity = findings
            .iter()
            .map(|f| f.severity)
            .max()
            .unwrap_or(DriftSeverity::None);

        DriftReport {
            drift_detected: !findings.is_empty(),
            overall_severity,
            findings,
            confidence: (intent_similarity + behavior_similarity) / 2.0,
        }
    }

    /// Calculate similarity between two text strings
    fn calculate_similarity(&self, a: &str, b: &str) -> f64 {
        jaccard_similarity(a, b)
    }

    /// Convert similarity score to severity level
    fn severity_from_similarity(&self, similarity: f64) -> DriftSeverity {
        match similarity {
            x if x >= 0.9 => DriftSeverity::None,
            x if x >= 0.7 => DriftSeverity::Low,
            x if x >= 0.5 => DriftSeverity::Medium,
            x if x >= 0.3 => DriftSeverity::High,
            _ => DriftSeverity::Critical,
        }
    }
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self::new(DriftConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_identical() {
        let detector = DriftDetector::default();
        let similarity = detector.calculate_similarity("hello world", "hello world");
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_similarity_different() {
        let detector = DriftDetector::default();
        let similarity = detector.calculate_similarity("hello world", "goodbye moon");
        assert!(similarity < 0.5);
    }

    #[test]
    fn test_intent_drift_none() {
        let detector = DriftDetector::default();
        let report = detector.detect_intent_drift(
            "Calculate the factorial of a number",
            "Calculate the factorial of a number",
        );
        assert!(report.overall_severity <= DriftSeverity::Low);
    }

    #[test]
    fn test_intent_drift_high() {
        let detector = DriftDetector::default();
        let report = detector.detect_intent_drift(
            "Send email notification",
            "Delete user account",
        );
        assert!(report.overall_severity >= DriftSeverity::High);
    }
}
