//! Semantic similarity detector for code duplication
//!
//! Detects functions/modules with high semantic similarity using
//! AST-normalized comparison and token shingles.

use super::{Detector, Finding};
use ctp_utils::text_similarity::jaccard_similarity;
use regex::Regex;

/// Semantic similarity detector
pub struct SemanticSimilarityDetector {
    warn_threshold: f64,
    error_threshold: f64,
    min_function_size: usize,
    normalize_whitespace: bool,
    ignore_comments: bool,
}

impl SemanticSimilarityDetector {
    pub fn new(
        warn_threshold: f64,
        error_threshold: f64,
        min_function_size: usize,
        normalize_whitespace: bool,
        ignore_comments: bool,
    ) -> Self {
        Self {
            warn_threshold,
            error_threshold,
            min_function_size,
            normalize_whitespace,
            ignore_comments,
        }
    }

    pub fn default() -> Self {
        Self::new(0.7, 0.9, 5, true, false)
    }

    /// Extract function bodies from code
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        let mut functions = vec![];
        let lines: Vec<&str> = content.lines().collect();

        // Patterns for different languages
        let function_patterns = vec![
            (r"^\s*(?:pub\s+)?fn\s+(\w+)\s*\(", "rust"), // Rust
            (r"^\s*def\s+(\w+)\s*\(", "python"),        // Python
            (r"^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\(", "javascript"), // JS/TS
            (r"^\s*(?:public\s+|private\s+|protected\s+)?function\s+(\w+)\s*\(", "php"), // PHP
            (r"^\s*func\s+(\w+)\s*\(", "go"),           // Go
        ];

        for (pattern, _lang) in function_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        if let Some(name) = caps.get(1) {
                            let func_name = name.as_str().to_string();
                            let body = self.extract_function_body(&lines, line_num);
                            let body_line_count = body.clone().lines().count();
                            
                            if body_line_count >= self.min_function_size {
                                functions.push(FunctionInfo {
                                    name: func_name,
                                    body,
                                    line_start: line_num + 1,
                                    line_end: line_num + 1 + body_line_count,
                                });
                            }
                        }
                    }
                }
            }
        }

        functions
    }

    /// Extract function body (simplified - in production would use AST)
    fn extract_function_body(&self, lines: &[&str], start_line: usize) -> String {
        let mut body = String::new();
        let mut brace_count = 0;
        let mut in_function = false;

        for _line in lines.iter().skip(start_line) {
            let line = _line;
            let trimmed = line.trim();
            
            if !in_function {
                if trimmed.contains('{') || trimmed.contains(':') {
                    in_function = true;
                    brace_count += trimmed.matches('{').count() as i32;
                    brace_count -= trimmed.matches('}').count() as i32;
                    body.push_str(line);
                    body.push('\n');
                    continue;
                }
            }

            if in_function {
                brace_count += trimmed.matches('{').count() as i32;
                brace_count -= trimmed.matches('}').count() as i32;
                
                body.push_str(line);
                body.push('\n');

                if brace_count <= 0 {
                    break;
                }
            }
        }

        body
    }

    /// Normalize code for comparison
    fn normalize_code(&self, code: &str) -> String {
        let mut normalized = code.to_string();

        if self.normalize_whitespace {
            // Normalize whitespace
            normalized = Regex::new(r"\s+")
                .unwrap()
                .replace_all(&normalized, " ")
                .to_string();
        }

        if self.ignore_comments {
            // Remove comments (simplified)
            normalized = Regex::new(r"//.*$|/\*.*?\*/")
                .unwrap()
                .replace_all(&normalized, "")
                .to_string();
        }

        normalized.trim().to_string()
    }

    /// Calculate similarity between two function bodies
    fn calculate_similarity(&self, func1: &FunctionInfo, func2: &FunctionInfo) -> f64 {
        let body1 = self.normalize_code(&func1.body);
        let body2 = self.normalize_code(&func2.body);

        // Use Jaccard similarity on token shingles
        jaccard_similarity(&body1, &body2)
    }
}

impl Detector for SemanticSimilarityDetector {
    fn name(&self) -> &'static str {
        "semantic_similarity"
    }

    fn analyze(&self, file_path: &str, content: &str) -> Vec<Finding> {
        let functions = self.extract_functions(content);
        let mut findings = vec![];

        // Compare all pairs of functions
        for i in 0..functions.len() {
            for j in (i + 1)..functions.len() {
                let similarity = self.calculate_similarity(&functions[i], &functions[j]);

                if similarity >= self.error_threshold {
                    findings.push(Finding {
                        message: format!(
                            "High code duplication detected: {:.1}% similarity between functions '{}' and '{}'",
                            similarity * 100.0,
                            functions[i].name,
                            functions[j].name
                        ),
                        file: file_path.to_string(),
                        line_start: functions[i].line_start,
                        line_end: functions[j].line_end,
                        remediation: format!(
                            "Consolidate functions '{}' and '{}' into a shared utility. Consider creating a common function that accepts parameters to handle variations.",
                            functions[i].name,
                            functions[j].name
                        ),
                    });
                } else if similarity >= self.warn_threshold {
                    findings.push(Finding {
                        message: format!(
                            "Potential code duplication: {:.1}% similarity between functions '{}' and '{}'",
                            similarity * 100.0,
                            functions[i].name,
                            functions[j].name
                        ),
                        file: file_path.to_string(),
                        line_start: functions[i].line_start,
                        line_end: functions[j].line_end,
                        remediation: format!(
                            "Review functions '{}' and '{}' for consolidation opportunities.",
                            functions[i].name,
                            functions[j].name
                        ),
                    });
                }
            }
        }

        findings
    }
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    body: String,
    line_start: usize,
    line_end: usize,
}
