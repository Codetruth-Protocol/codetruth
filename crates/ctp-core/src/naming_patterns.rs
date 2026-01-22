//! Naming pattern detection and analysis module
//!
//! This module provides functionality to detect, learn, and monitor naming conventions
//! across a codebase. It integrates with the ctp-context crate to maintain semantic
//! awareness of naming patterns and detect drift from established conventions.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Pattern types for naming conventions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternType {
    /// kebab-case (e.g., "my-file-name")
    KebabCase,
    /// snake_case (e.g., "my_file_name") 
    SnakeCase,
    /// camelCase (e.g., "myFileName")
    CamelCase,
    /// PascalCase (e.g., "MyFileName")
    PascalCase,
    /// UPPERCASE (e.g., "MY_FILE_NAME")
    UpperCase,
    /// Mixed/unknown pattern
    Mixed,
}

/// Naming pattern detected for a directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingPattern {
    /// The dominant pattern type
    pub pattern_type: PatternType,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Number of files analyzed
    pub sample_size: usize,
    /// Exceptions found (files that don't follow the pattern)
    pub exceptions: Vec<String>,
    /// File extensions included in analysis
    pub extensions: Vec<String>,
}

/// Analysis result for naming pattern compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingAnalysisResult {
    /// Directory analyzed
    pub directory: String,
    /// Detected pattern
    pub detected_pattern: NamingPattern,
    /// Files violating the pattern
    pub violations: Vec<NamingViolation>,
    /// Overall compliance score (0.0 - 1.0)
    pub compliance_score: f64,
}

/// Individual naming violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingViolation {
    /// File name that violates the pattern
    pub file_name: String,
    /// Expected pattern
    pub expected_pattern: PatternType,
    /// Actual pattern detected
    pub actual_pattern: PatternType,
    /// Severity of the violation
    pub severity: ViolationSeverity,
}

/// Severity levels for naming violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Minor inconsistency
    Low,
    /// Noticeable inconsistency
    Medium,
    /// Major inconsistency
    High,
}

/// Main naming pattern detector
pub struct NamingPatternDetector {
    /// Cached patterns per directory
    pattern_cache: HashMap<String, NamingPattern>,
    /// Minimum sample size for pattern detection
    min_sample_size: usize,
    /// Confidence threshold for pattern detection
    confidence_threshold: f64,
}

impl NamingPatternDetector {
    /// Create a new naming pattern detector
    pub fn new() -> Self {
        Self {
            pattern_cache: HashMap::new(),
            min_sample_size: 3,
            confidence_threshold: 0.7,
        }
    }

    /// Analyze naming patterns in a directory
    pub fn analyze_directory(&mut self, directory_path: &Path) -> Result<NamingAnalysisResult, Box<dyn std::error::Error>> {
        let dir_str = directory_path.to_string_lossy().to_string();
        
        // Check cache first
        if let Some(cached_pattern) = self.pattern_cache.get(&dir_str) {
            return self.analyze_with_cached_pattern(directory_path, cached_pattern.clone());
        }

        // Detect pattern from existing files
        let detected_pattern = self.detect_pattern(directory_path)?;
        
        // Cache the detected pattern
        self.pattern_cache.insert(dir_str.clone(), detected_pattern.clone());

        // Analyze for violations
        self.analyze_with_cached_pattern(directory_path, detected_pattern)
    }

    /// Detect the dominant naming pattern in a directory
    fn detect_pattern(&self, directory_path: &Path) -> Result<NamingPattern, Box<dyn std::error::Error>> {
        let mut pattern_counts: HashMap<PatternType, usize> = HashMap::new();
        let mut file_names = Vec::new();
        let mut extensions = std::collections::HashSet::new();

        // Collect file names and count patterns
        for entry in std::fs::read_dir(directory_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        extensions.insert(ext.to_string());
                    }
                    
                    let pattern = self.detect_file_pattern(file_name);
                    *pattern_counts.entry(pattern).or_insert(0) += 1;
                    file_names.push(file_name.to_string());
                }
            }
        }

        if file_names.len() < self.min_sample_size {
            return Ok(NamingPattern {
                pattern_type: PatternType::Mixed,
                confidence: 0.0,
                sample_size: file_names.len(),
                exceptions: file_names,
                extensions: extensions.into_iter().collect(),
            });
        }

        // Find dominant pattern
        let total_files = file_names.len();
        let (dominant_pattern, count) = pattern_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .unwrap_or((&PatternType::Mixed, &0));

        let confidence = *count as f64 / total_files as f64;
        
        // Find exceptions
        let exceptions: Vec<String> = file_names
            .into_iter()
            .filter(|name| self.detect_file_pattern(name) != *dominant_pattern)
            .collect();

        Ok(NamingPattern {
            pattern_type: dominant_pattern.clone(),
            confidence,
            sample_size: total_files,
            exceptions,
            extensions: extensions.into_iter().collect(),
        })
    }

    /// Detect pattern for a single file name
    fn detect_file_pattern(&self, file_name: &str) -> PatternType {
        // Remove extension for analysis
        let name_without_ext = match file_name.rfind('.') {
            Some(pos) => &file_name[..pos],
            None => file_name,
        };

        // Check for UPPERCASE
        if name_without_ext.chars().all(|c| c.is_uppercase() || c == '_') {
            return PatternType::UpperCase;
        }

        // Check for kebab-case
        if name_without_ext.contains('-') && !name_without_ext.contains('_') {
            return PatternType::KebabCase;
        }

        // Check for snake_case
        if name_without_ext.contains('_') && !name_without_ext.contains('-') {
            return PatternType::SnakeCase;
        }

        // Check for PascalCase (starts with uppercase, no separators)
        if name_without_ext.chars().next().map_or(false, |c| c.is_uppercase()) 
            && !name_without_ext.contains('_') 
            && !name_without_ext.contains('-') {
            return PatternType::PascalCase;
        }

        // Check for camelCase (starts with lowercase, no separators)
        if name_without_ext.chars().next().map_or(false, |c| c.is_lowercase()) 
            && !name_without_ext.contains('_') 
            && !name_without_ext.contains('-') 
            && name_without_ext.chars().any(|c| c.is_uppercase()) {
            return PatternType::CamelCase;
        }

        PatternType::Mixed
    }

    /// Analyze directory using a cached pattern
    fn analyze_with_cached_pattern(&self, directory_path: &Path, pattern: NamingPattern) -> Result<NamingAnalysisResult, Box<dyn std::error::Error>> {
        let mut violations = Vec::new();

        // Check each file against the cached pattern
        for entry in std::fs::read_dir(directory_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let actual_pattern = self.detect_file_pattern(file_name);
                    
                    if actual_pattern != pattern.pattern_type {
                        let severity = self.calculate_violation_severity(&pattern, &actual_pattern);
                        
                        violations.push(NamingViolation {
                            file_name: file_name.to_string(),
                            expected_pattern: pattern.pattern_type.clone(),
                            actual_pattern,
                            severity,
                        });
                    }
                }
            }
        }

        let compliance_score = if violations.is_empty() {
            1.0
        } else {
            1.0 - (violations.len() as f64 / pattern.sample_size as f64)
        };

        Ok(NamingAnalysisResult {
            directory: directory_path.to_string_lossy().to_string(),
            detected_pattern: pattern,
            violations,
            compliance_score,
        })
    }

    /// Calculate violation severity based on pattern difference
    fn calculate_violation_severity(&self, expected: &NamingPattern, actual: &PatternType) -> ViolationSeverity {
        if expected.confidence < self.confidence_threshold {
            return ViolationSeverity::Low; // Low confidence pattern = low severity
        }

        match (expected.pattern_type.clone(), actual) {
            (PatternType::KebabCase, PatternType::SnakeCase) |
            (PatternType::SnakeCase, PatternType::KebabCase) => ViolationSeverity::Medium,
            (PatternType::CamelCase, PatternType::PascalCase) |
            (PatternType::PascalCase, PatternType::CamelCase) => ViolationSeverity::Medium,
            (PatternType::UpperCase, _) | (_, PatternType::UpperCase) => ViolationSeverity::High,
            (PatternType::Mixed, _) => ViolationSeverity::Low,
            _ => ViolationSeverity::Medium,
        }
    }

    /// Update cached pattern for a directory
    pub fn update_pattern(&mut self, directory: &str, pattern: NamingPattern) {
        self.pattern_cache.insert(directory.to_string(), pattern);
    }

    /// Get cached pattern for a directory
    pub fn get_pattern(&self, directory: &str) -> Option<&NamingPattern> {
        self.pattern_cache.get(directory)
    }

    /// Clear pattern cache
    pub fn clear_cache(&mut self) {
        self.pattern_cache.clear();
    }
}

impl Default for NamingPatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_pattern_detection() {
        let detector = NamingPatternDetector::new();
        
        assert_eq!(detector.detect_file_pattern("my-file-name.md"), PatternType::KebabCase);
        assert_eq!(detector.detect_file_pattern("my_file_name.rs"), PatternType::SnakeCase);
        assert_eq!(detector.detect_file_pattern("myFileName.ts"), PatternType::CamelCase);
        assert_eq!(detector.detect_file_pattern("MyFileName.ts"), PatternType::PascalCase);
        assert_eq!(detector.detect_file_pattern("MY_FILE_NAME"), PatternType::UpperCase);
        assert_eq!(detector.detect_file_pattern("mixed-Case_Name"), PatternType::Mixed);
    }

    #[test]
    fn test_directory_analysis() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();
        
        // Create test files with kebab-case pattern
        fs::write(dir_path.join("file-one.md"), "content")?;
        fs::write(dir_path.join("file-two.md"), "content")?;
        fs::write(dir_path.join("file-three.md"), "content")?;
        // Add one violation
        fs::write(dir_path.join("UPPER_CASE.md"), "content")?;
        
        let mut detector = NamingPatternDetector::new();
        let result = detector.analyze_directory(dir_path)?;
        
        assert_eq!(result.detected_pattern.pattern_type, PatternType::KebabCase);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].file_name, "UPPER_CASE.md");
        
        Ok(())
    }
}
