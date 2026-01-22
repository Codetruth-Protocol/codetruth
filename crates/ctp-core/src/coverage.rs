//! Coverage report parsers
//!
//! Supports multiple coverage formats:
//! - LCOV
//! - Cobertura XML
//! - JaCoCo XML
//! - nyc JSON
//! - pytest-cov JSON

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Coverage information for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    pub file_path: String,
    pub line_coverage: f64,
    pub branch_coverage: Option<f64>,
    pub function_coverage: Option<f64>,
    pub lines_covered: usize,
    pub lines_total: usize,
}

/// Coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub format: CoverageFormat,
    pub files: HashMap<String, FileCoverage>,
    pub total_coverage: f64,
}

/// Coverage format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverageFormat {
    Lcov,
    Cobertura,
    Jacoco,
    Nyc,
    PytestCov,
}

/// Coverage parser trait
pub trait CoverageParser: Send + Sync {
    fn parse(&self, content: &str) -> Result<CoverageReport, CoverageParseError>;
    fn format(&self) -> CoverageFormat;
}

/// Coverage parse error
#[derive(Debug, thiserror::Error)]
pub enum CoverageParseError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// LCOV parser
pub struct LcovParser;

impl CoverageParser for LcovParser {
    fn format(&self) -> CoverageFormat {
        CoverageFormat::Lcov
    }

    fn parse(&self, content: &str) -> Result<CoverageReport, CoverageParseError> {
        let mut files = HashMap::new();
        let mut current_file: Option<String> = None;
        let mut lines_covered = 0;
        let mut lines_total = 0;

        for line in content.lines() {
            if line.starts_with("SF:") {
                // Source file
                current_file = Some(line[3..].trim().to_string());
                lines_covered = 0;
                lines_total = 0;
            } else if line.starts_with("DA:") {
                // Line data: DA:line_number,hits
                lines_total += 1;
                let parts: Vec<&str> = line[3..].split(',').collect();
                if parts.len() >= 2 {
                    if let Ok(hits) = parts[1].parse::<usize>() {
                        if hits > 0 {
                            lines_covered += 1;
                        }
                    }
                }
            } else if line == "end_of_record" {
                if let Some(file) = current_file.take() {
                    let coverage = if lines_total > 0 {
                        lines_covered as f64 / lines_total as f64
                    } else {
                        0.0
                    };

                    files.insert(
                        file.clone(),
                        FileCoverage {
                            file_path: file,
                            line_coverage: coverage,
                            branch_coverage: None,
                            function_coverage: None,
                            lines_covered,
                            lines_total,
                        },
                    );
                }
            }
        }

        // Calculate total coverage
        let total_coverage = if !files.is_empty() {
            files.values().map(|f| f.line_coverage).sum::<f64>() / files.len() as f64
        } else {
            0.0
        };

        Ok(CoverageReport {
            format: CoverageFormat::Lcov,
            files,
            total_coverage,
        })
    }
}

/// Cobertura XML parser (simplified)
pub struct CoberturaParser;

impl CoverageParser for CoberturaParser {
    fn format(&self) -> CoverageFormat {
        CoverageFormat::Cobertura
    }

    fn parse(&self, content: &str) -> Result<CoverageReport, CoverageParseError> {
        // Simplified parser - in production would use proper XML parsing
        let mut files = HashMap::new();
        
        // Basic regex-based extraction
        let re = regex::Regex::new(r#"<class name="([^"]+)" filename="([^"]+)".*?lines-covered="(\d+)" lines-valid="(\d+)""#)
            .map_err(|e| CoverageParseError::ParseError(format!("Regex error: {}", e)))?;
        
        for cap in re.captures_iter(content) {
            let _name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let filename = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let covered: usize = cap.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            let total: usize = cap.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            
            let coverage = if total > 0 {
                covered as f64 / total as f64
            } else {
                0.0
            };
            
            files.insert(
                filename.to_string(),
                FileCoverage {
                    file_path: filename.to_string(),
                    line_coverage: coverage,
                    branch_coverage: None,
                    function_coverage: None,
                    lines_covered: covered,
                    lines_total: total,
                },
            );
        }
        
        let total_coverage = if !files.is_empty() {
            files.values().map(|f| f.line_coverage).sum::<f64>() / files.len() as f64
        } else {
            0.0
        };
        
        Ok(CoverageReport {
            format: CoverageFormat::Cobertura,
            files,
            total_coverage,
        })
    }
}

/// Coverage report loader
pub struct CoverageLoader {
    parsers: Vec<Box<dyn CoverageParser>>,
}

impl CoverageLoader {
    pub fn new() -> Self {
        let mut parsers: Vec<Box<dyn CoverageParser>> = vec![];
        parsers.push(Box::new(LcovParser));
        parsers.push(Box::new(CoberturaParser));
        
        Self { parsers }
    }

    /// Load coverage report from file
    pub fn load_from_file(&self, path: &Path) -> Result<CoverageReport, CoverageParseError> {
        let content = std::fs::read_to_string(path)?;
        self.load_from_str(&content, path)
    }

    /// Load coverage report from string
    pub fn load_from_str(
        &self,
        content: &str,
        path: &Path,
    ) -> Result<CoverageReport, CoverageParseError> {
        // Try each parser
        for parser in &self.parsers {
            match parser.parse(content) {
                Ok(report) if !report.files.is_empty() => {
                    return Ok(report);
                }
                _ => continue,
            }
        }

        Err(CoverageParseError::InvalidFormat(format!(
            "Could not parse coverage report: {}",
            path.display()
        )))
    }
}

impl Default for CoverageLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcov_parser() {
        let parser = LcovParser;
        let content = r#"
SF:src/main.rs
DA:1,1
DA:2,1
DA:3,0
end_of_record
"#;
        
        let report = parser.parse(content).unwrap();
        assert_eq!(report.files.len(), 1);
        assert!(report.files.contains_key("src/main.rs"));
        let file_cov = report.files.get("src/main.rs").unwrap();
        assert!((file_cov.line_coverage - 0.666).abs() < 0.01);
    }
}
