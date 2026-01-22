//! Documentation drift detection
//!
//! Detects when documentation becomes stale or inconsistent with code

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationDriftFinding {
    pub file_path: String,
    pub finding_type: DocDriftType,
    pub severity: DocDriftSeverity,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocDriftType {
    MissingSection,      // Required section not found
    OutdatedExample,     // Code example doesn't match current API
    BrokenLink,          // Link to non-existent file/section
    InconsistentVersion, // Version mismatch
    StaleContent,        // Content hasn't been updated in a long time
    MissingFunctionality, // Core functionality not documented
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocDriftSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

pub struct DocumentationAnalyzer {
    required_sections: Vec<String>,
    core_functionalities: Vec<String>,
}

impl DocumentationAnalyzer {
    pub fn new(required_sections: Vec<String>, core_functionalities: Vec<String>) -> Self {
        Self {
            required_sections,
            core_functionalities,
        }
    }

    /// Analyze README file for drift
    pub fn analyze_readme(&self, content: &str, file_path: &Path) -> Vec<DocumentationDriftFinding> {
        let mut findings = vec![];

        // Check for required sections
        let content_lower = content.to_lowercase();
        for section in &self.required_sections {
            let section_lower = section.to_lowercase();
            if !content_lower.contains(&section_lower) {
                findings.push(DocumentationDriftFinding {
                    file_path: file_path.display().to_string(),
                    finding_type: DocDriftType::MissingSection,
                    severity: DocDriftSeverity::Medium,
                    description: format!("Required section '{}' not found", section),
                    suggestion: format!("Add '{}' section to README", section),
                });
            }
        }

        // Check for core functionalities mentioned
        for functionality in &self.core_functionalities {
            let func_lower = functionality.to_lowercase();
            if !content_lower.contains(&func_lower) {
                findings.push(DocumentationDriftFinding {
                    file_path: file_path.display().to_string(),
                    finding_type: DocDriftType::MissingFunctionality,
                    severity: DocDriftSeverity::High,
                    description: format!("Core functionality '{}' not documented", functionality),
                    suggestion: format!("Document '{}' in README or user guide", functionality),
                });
            }
        }

        // Check for broken internal links
        let broken_links = self.detect_broken_links(content);
        for link in broken_links {
            findings.push(DocumentationDriftFinding {
                file_path: file_path.display().to_string(),
                finding_type: DocDriftType::BrokenLink,
                severity: DocDriftSeverity::Low,
                description: format!("Potentially broken link: {}", link),
                suggestion: format!("Verify link exists: {}", link),
            });
        }

        findings
    }

    /// Detect potentially broken links in markdown
    fn detect_broken_links(&self, content: &str) -> Vec<String> {
        let mut broken = vec![];
        
        // Simple regex-like detection for markdown links
        for line in content.lines() {
            if let Some(start) = line.find("](") {
                if let Some(end) = line[start+2..].find(')') {
                    let link = &line[start+2..start+2+end];
                    // Check if it's a relative file link
                    if link.starts_with("./") || link.starts_with("../") || link.ends_with(".md") {
                        // Would need actual file system check in real implementation
                        // For now, just collect them
                        broken.push(link.to_string());
                    }
                }
            }
        }
        
        broken
    }

    /// Compare documented API with actual code exports
    pub fn compare_api_documentation(
        &self,
        doc_content: &str,
        actual_exports: &[String],
    ) -> Vec<DocumentationDriftFinding> {
        let mut findings = vec![];

        // Extract documented functions/classes from doc
        let documented = self.extract_documented_items(doc_content);
        let actual_set: HashSet<_> = actual_exports.iter().collect();
        let documented_set: HashSet<_> = documented.iter().collect();

        // Find undocumented exports
        for export in actual_exports {
            if !documented_set.contains(&export) {
                findings.push(DocumentationDriftFinding {
                    file_path: "API Documentation".into(),
                    finding_type: DocDriftType::MissingFunctionality,
                    severity: DocDriftSeverity::Medium,
                    description: format!("Exported '{}' is not documented", export),
                    suggestion: format!("Add documentation for '{}'", export),
                });
            }
        }

        // Find documented items that no longer exist
        for doc_item in &documented {
            if !actual_set.contains(&doc_item) {
                findings.push(DocumentationDriftFinding {
                    file_path: "API Documentation".into(),
                    finding_type: DocDriftType::OutdatedExample,
                    severity: DocDriftSeverity::High,
                    description: format!("Documented '{}' no longer exists in code", doc_item),
                    suggestion: format!("Remove or update documentation for '{}'", doc_item),
                });
            }
        }

        findings
    }

    /// Extract function/class names from documentation
    fn extract_documented_items(&self, content: &str) -> Vec<String> {
        let mut items = vec![];
        
        // Look for code blocks and function signatures
        let mut in_code_block = false;
        for line in content.lines() {
            if line.trim().starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            
            if in_code_block {
                // Extract function names (simple heuristic)
                if line.contains("fn ") || line.contains("function ") || line.contains("def ") {
                    if let Some(name) = self.extract_function_name(line) {
                        items.push(name);
                    }
                }
            }
        }
        
        items
    }

    /// Extract function name from a line
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let line = line.trim();
        
        // Rust: fn name(
        if let Some(pos) = line.find("fn ") {
            let after = &line[pos+3..];
            if let Some(paren) = after.find('(') {
                return Some(after[..paren].trim().to_string());
            }
        }
        
        // Python: def name(
        if let Some(pos) = line.find("def ") {
            let after = &line[pos+4..];
            if let Some(paren) = after.find('(') {
                return Some(after[..paren].trim().to_string());
            }
        }
        
        // JavaScript: function name(
        if let Some(pos) = line.find("function ") {
            let after = &line[pos+9..];
            if let Some(paren) = after.find('(') {
                return Some(after[..paren].trim().to_string());
            }
        }
        
        None
    }
}

impl Default for DocumentationAnalyzer {
    fn default() -> Self {
        Self::new(
            vec![
                "Installation".into(),
                "Usage".into(),
                "API".into(),
                "Contributing".into(),
            ],
            vec![],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_missing_section() {
        let analyzer = DocumentationAnalyzer::new(
            vec!["Installation".into(), "Usage".into()],
            vec![],
        );
        
        let readme = "# My Project\n\nSome description";
        let findings = analyzer.analyze_readme(readme, &PathBuf::from("README.md"));
        
        assert_eq!(findings.len(), 2);
        assert!(findings.iter().any(|f| matches!(f.finding_type, DocDriftType::MissingSection)));
    }

    #[test]
    fn test_missing_functionality() {
        let analyzer = DocumentationAnalyzer::new(
            vec![],
            vec!["authentication".into()],
        );
        
        let readme = "# My Project\n\nBasic features";
        let findings = analyzer.analyze_readme(readme, &PathBuf::from("README.md"));
        
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].finding_type, DocDriftType::MissingFunctionality);
    }

    #[test]
    fn test_api_drift() {
        let analyzer = DocumentationAnalyzer::default();
        
        let doc = "```rust\nfn old_function() {}\n```";
        let actual = vec!["new_function".to_string()];
        
        let findings = analyzer.compare_api_documentation(doc, &actual);
        
        assert!(findings.len() >= 1);
    }

    #[test]
    fn test_extract_function_name() {
        let analyzer = DocumentationAnalyzer::default();
        
        assert_eq!(
            analyzer.extract_function_name("fn test_function() {"),
            Some("test_function".to_string())
        );
        
        assert_eq!(
            analyzer.extract_function_name("def my_func(param):"),
            Some("my_func".to_string())
        );
    }
}
