//! Detector module for code analysis
//!
//! Detectors identify specific patterns, issues, and violations in code.

mod semantic_similarity;
mod function_name_similarity;

pub use semantic_similarity::SemanticSimilarityDetector;
pub use function_name_similarity::FunctionNameSimilarityDetector;

use crate::models::{DriftDetail, DriftType, Impact, Location};

/// A finding from a detector
pub struct Finding {
    pub message: String,
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
    pub remediation: String,
}

/// Trait for all detectors
pub trait Detector: Send + Sync {
    /// Get the name of this detector
    fn name(&self) -> &'static str;

    /// Analyze a file and return findings
    fn analyze(&self, file_path: &str, content: &str) -> Vec<Finding>;
}

/// Registry for managing detectors
pub struct DetectorsRegistry {
    detectors: Vec<Box<dyn Detector>>,
}

impl DetectorsRegistry {
    pub fn new() -> Self {
        let mut reg = Self { detectors: vec![] };
        // Register basic built-in detectors
        reg.register(Box::new(DuplicateToastDetector::default()));
        reg.register(Box::new(SemanticSimilarityDetector::default()));
        reg.register(Box::new(FunctionNameSimilarityDetector::default()));
        reg
    }

    pub fn register(&mut self, detector: Box<dyn Detector>) {
        self.detectors.push(detector);
    }

    pub fn run(&self, file_path: &str, content: &str) -> Vec<DriftDetail> {
        let mut details = vec![];
        for d in &self.detectors {
            for f in d.analyze(file_path, content) {
                details.push(DriftDetail {
                    drift_type: DriftType::Implementation,
                    expected: String::from("No detector-triggered issues"),
                    actual: f.message,
                    location: Location {
                        file: f.file,
                        line_start: f.line_start,
                        line_end: f.line_end,
                    },
                    impact: Impact {
                        functional: "Review required".into(),
                        security: "Unknown".into(),
                        performance: "Unknown".into(),
                        maintainability: "Needs attention".into(),
                    },
                    remediation: f.remediation,
                });
            }
        }
        details
    }
}

// -------------------------
// Built-in basic detectors
// -------------------------

#[derive(Default)]
struct DuplicateToastDetector;

impl Detector for DuplicateToastDetector {
    fn name(&self) -> &'static str {
        "ui_duplicate_toast"
    }

    fn analyze(&self, file_path: &str, content: &str) -> Vec<Finding> {
        let mut count = 0usize;
        let mut lines_with_toast = vec![];
        
        for (line_num, line) in content.lines().enumerate() {
            let l = line.to_lowercase();
            if l.contains("toast(") 
                || l.contains("showtoast(") 
                || l.contains("notify(")
                || l.contains("alert(")
                || l.contains("shownotification(")
                || l.contains("message(")
                || l.contains("showmessage(") {
                count += 1;
                lines_with_toast.push(line_num + 1);
            }
        }
        
        if count > 1 {
            vec![Finding {
                message: format!(
                    "Multiple toast/notification calls found ({} calls); possible duplicate user message",
                    count
                ),
                file: file_path.to_string(),
                line_start: *lines_with_toast.first().unwrap_or(&1),
                line_end: *lines_with_toast.last().unwrap_or(&1),
                remediation: "Ensure only one user-facing toast/notification is triggered per action path. Consider using a notification manager that deduplicates messages.".into(),
            }]
        } else {
            vec![]
        }
    }
}
