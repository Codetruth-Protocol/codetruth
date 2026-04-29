//! Function name similarity detector
//!
//! Detects functions with similar names and similar implementations,
//! e.g., formatFileSize, formatFileBytes, formatBytes

use super::{Detector, Finding};
use ctp_utils::text_similarity::jaccard_similarity;
use regex::Regex;

/// Function name similarity detector
pub struct FunctionNameSimilarityDetector {
    name_similarity_threshold: f64,
    implementation_similarity_threshold: f64,
    levenshtein_distance_max: usize,
}

impl FunctionNameSimilarityDetector {
    pub fn new(
        name_similarity_threshold: f64,
        implementation_similarity_threshold: f64,
        levenshtein_distance_max: usize,
    ) -> Self {
        Self {
            name_similarity_threshold,
            implementation_similarity_threshold,
            levenshtein_distance_max,
        }
    }

    pub fn default() -> Self {
        Self::new(0.8, 0.85, 3)
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// Check if two function names are similar
    fn names_similar(&self, name1: &str, name2: &str) -> bool {
        let name1_lower = name1.to_lowercase();
        let name2_lower = name2.to_lowercase();

        // Check Jaccard similarity
        let jaccard = jaccard_similarity(&name1_lower, &name2_lower);
        if jaccard >= self.name_similarity_threshold {
            return true;
        }

        // Check Levenshtein distance
        let distance = self.levenshtein_distance(&name1_lower, &name2_lower);
        if distance <= self.levenshtein_distance_max {
            return true;
        }

        false
    }

    /// Extract function bodies (simplified)
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        let mut functions = vec![];
        let lines: Vec<&str> = content.lines().collect();

        let function_patterns = vec![
            (r"^\s*(?:pub\s+)?fn\s+(\w+)\s*\(", "rust"),
            (r"^\s*def\s+(\w+)\s*\(", "python"),
            (r"^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\(", "javascript"),
            (r"^\s*func\s+(\w+)\s*\(", "go"),
        ];

        for (pattern, _lang) in function_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        if let Some(name) = caps.get(1) {
                            let func_name = name.as_str().to_string();
                            let body = self.extract_function_body(&lines, line_num);
                            
                            functions.push(FunctionInfo {
                                name: func_name,
                                body,
                                line_start: line_num + 1,
                            });
                        }
                    }
                }
            }
        }

        functions
    }

    fn extract_function_body(&self, lines: &[&str], start_line: usize) -> String {
        let mut body = String::new();
        let mut brace_count = 0;
        let mut in_function = false;

        for line in lines.iter().skip(start_line) {
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
        let normalized = code.to_string();
        Regex::new(r"\s+")
            .unwrap()
            .replace_all(&normalized, " ")
            .to_string()
            .trim()
            .to_string()
    }

    /// Calculate implementation similarity
    fn implementation_similarity(&self, body1: &str, body2: &str) -> f64 {
        let norm1 = self.normalize_code(body1);
        let norm2 = self.normalize_code(body2);
        jaccard_similarity(&norm1, &norm2)
    }
}

impl Detector for FunctionNameSimilarityDetector {
    fn name(&self) -> &'static str {
        "function_name_similarity"
    }

    fn analyze(&self, file_path: &str, content: &str) -> Vec<Finding> {
        let functions = self.extract_functions(content);
        let mut findings = vec![];

        // Compare all pairs
        for i in 0..functions.len() {
            for j in (i + 1)..functions.len() {
                if self.names_similar(&functions[i].name, &functions[j].name) {
                    let impl_similarity = self.implementation_similarity(
                        &functions[i].body,
                        &functions[j].body,
                    );

                    if impl_similarity >= self.implementation_similarity_threshold {
                        findings.push(Finding {
                            message: format!(
                                "Functions with similar names and implementations detected: '{}' and '{}' ({:.1}% implementation similarity)",
                                functions[i].name,
                                functions[j].name,
                                impl_similarity * 100.0
                            ),
                            file: file_path.to_string(),
                            line_start: functions[i].line_start,
                            line_end: functions[j].line_start,
                            remediation: format!(
                                "Consolidate functions '{}' and '{}' into a single function with a clear, unique name. If functions serve different purposes, rename them to reflect their differences.",
                                functions[i].name,
                                functions[j].name
                            ),
                        });
                    }
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
}
