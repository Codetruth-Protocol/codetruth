//! Component Criticality
//!
//! Maps code paths to criticality levels for impact analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use glob::Pattern;

/// Criticality level for a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriticalityLevel {
    /// Minimal impact if broken
    Low,
    /// Important but not blocking
    Medium,
    /// Affects core user experience
    High,
    /// Downtime = immediate revenue/safety loss
    Critical,
}

impl CriticalityLevel {
    /// Get impact multiplier for this level
    pub fn impact_multiplier(&self) -> f64 {
        match self {
            CriticalityLevel::Low => 0.25,
            CriticalityLevel::Medium => 0.5,
            CriticalityLevel::High => 1.0,
            CriticalityLevel::Critical => 2.0,
        }
    }

    /// Get recommended review level
    pub fn review_level(&self) -> ReviewLevel {
        match self {
            CriticalityLevel::Low => ReviewLevel::Standard,
            CriticalityLevel::Medium => ReviewLevel::Standard,
            CriticalityLevel::High => ReviewLevel::Enhanced,
            CriticalityLevel::Critical => ReviewLevel::Critical,
        }
    }
}

impl Default for CriticalityLevel {
    fn default() -> Self {
        CriticalityLevel::Medium
    }
}

/// Review level recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewLevel {
    /// Auto-merge OK with passing tests
    Standard,
    /// Needs senior engineer review
    Enhanced,
    /// Needs architecture/security review
    Critical,
    /// Needs compliance/legal review
    Regulated,
}

/// Criticality mapping for a codebase
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CriticalityMap {
    /// Patterns mapped to criticality levels
    patterns: HashMap<CriticalityLevel, Vec<String>>,
    
    /// Compiled patterns (not serialized)
    #[serde(skip)]
    compiled: HashMap<CriticalityLevel, Vec<Pattern>>,
}

impl CriticalityMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pattern at a criticality level
    pub fn add_pattern(&mut self, level: CriticalityLevel, pattern: &str) {
        self.patterns.entry(level).or_default().push(pattern.to_string());
        
        if let Ok(compiled) = Pattern::new(pattern) {
            self.compiled.entry(level).or_default().push(compiled);
        }
    }

    /// Compile patterns (call after deserialization)
    pub fn compile(&mut self) {
        self.compiled.clear();
        for (level, patterns) in &self.patterns {
            for pattern in patterns {
                if let Ok(compiled) = Pattern::new(pattern) {
                    self.compiled.entry(*level).or_default().push(compiled);
                }
            }
        }
    }

    /// Get criticality level for a file path
    pub fn get_criticality(&self, file_path: &str) -> CriticalityLevel {
        // Collect all matching patterns with their levels and specificity
        let mut matches: Vec<(CriticalityLevel, usize)> = vec![];
        
        for level in [
            CriticalityLevel::Critical,
            CriticalityLevel::High,
            CriticalityLevel::Medium,
            CriticalityLevel::Low,
        ] {
            if let Some(patterns) = self.compiled.get(&level) {
                for pattern in patterns {
                    if pattern.matches(file_path) {
                        // Calculate specificity (more specific = fewer wildcards)
                        let pattern_str = pattern.as_str();
                        let specificity = pattern_str.len() - pattern_str.matches('*').count();
                        matches.push((level, specificity));
                    }
                }
            }
        }
        
        if matches.is_empty() {
            return CriticalityLevel::Medium;
        }
        
        // Sort by: 1) specificity (descending), 2) criticality level (descending)
        matches.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0))
        });
        
        // Return the most specific match (or highest criticality if tied)
        matches[0].0
    }

    /// Get all files at a specific criticality level from a list
    pub fn filter_by_criticality<'a>(
        &self,
        files: &'a [String],
        level: CriticalityLevel,
    ) -> Vec<&'a String> {
        files
            .iter()
            .filter(|f| self.get_criticality(f) == level)
            .collect()
    }

    /// Create a default map with common patterns
    pub fn with_defaults() -> Self {
        let mut map = Self::new();
        
        // Critical paths
        map.add_pattern(CriticalityLevel::Critical, "**/payments/**");
        map.add_pattern(CriticalityLevel::Critical, "**/checkout/**");
        map.add_pattern(CriticalityLevel::Critical, "**/auth/**");
        map.add_pattern(CriticalityLevel::Critical, "**/security/**");
        map.add_pattern(CriticalityLevel::Critical, "**/billing/**");
        
        // High criticality
        map.add_pattern(CriticalityLevel::High, "**/api/**");
        map.add_pattern(CriticalityLevel::High, "**/core/**");
        map.add_pattern(CriticalityLevel::High, "**/database/**");
        map.add_pattern(CriticalityLevel::High, "**/models/**");
        map.add_pattern(CriticalityLevel::High, "**/orders/**");
        map.add_pattern(CriticalityLevel::High, "**/users/**");
        
        // Medium criticality
        map.add_pattern(CriticalityLevel::Medium, "**/features/**");
        map.add_pattern(CriticalityLevel::Medium, "**/components/**");
        map.add_pattern(CriticalityLevel::Medium, "**/services/**");
        
        // Low criticality
        map.add_pattern(CriticalityLevel::Low, "**/tests/**");
        map.add_pattern(CriticalityLevel::Low, "**/docs/**");
        map.add_pattern(CriticalityLevel::Low, "**/scripts/**");
        map.add_pattern(CriticalityLevel::Low, "**/marketing/**");
        map.add_pattern(CriticalityLevel::Low, "**/*.md");
        
        map
    }
}

/// Criticality information for a specific component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCriticality {
    /// File path
    pub path: String,
    
    /// Criticality level
    pub level: CriticalityLevel,
    
    /// Reason for this criticality
    pub reason: Option<String>,
    
    /// Dependencies on other critical components
    pub critical_dependencies: Vec<String>,
    
    /// Components that depend on this one
    pub dependents: Vec<String>,
}

impl ComponentCriticality {
    pub fn new(path: &str, level: CriticalityLevel) -> Self {
        Self {
            path: path.to_string(),
            level,
            reason: None,
            critical_dependencies: vec![],
            dependents: vec![],
        }
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    /// Calculate effective criticality considering dependencies
    pub fn effective_criticality(&self) -> CriticalityLevel {
        if !self.critical_dependencies.is_empty() {
            // If this component has critical dependencies, bump up criticality
            match self.level {
                CriticalityLevel::Low => CriticalityLevel::Medium,
                CriticalityLevel::Medium => CriticalityLevel::High,
                other => other,
            }
        } else {
            self.level
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_criticality_map() {
        let map = CriticalityMap::with_defaults();
        
        assert_eq!(
            map.get_criticality("src/payments/stripe.ts"),
            CriticalityLevel::Critical
        );
        assert_eq!(
            map.get_criticality("src/auth/login.ts"),
            CriticalityLevel::Critical
        );
        assert_eq!(
            map.get_criticality("src/api/users.ts"),
            CriticalityLevel::High
        );
        assert_eq!(
            map.get_criticality("src/marketing/banner.ts"),
            CriticalityLevel::Low
        );
    }

    #[test]
    fn test_criticality_ordering() {
        assert!(CriticalityLevel::Critical > CriticalityLevel::High);
        assert!(CriticalityLevel::High > CriticalityLevel::Medium);
        assert!(CriticalityLevel::Medium > CriticalityLevel::Low);
    }

    #[test]
    fn test_impact_multiplier() {
        assert!(
            CriticalityLevel::Critical.impact_multiplier()
                > CriticalityLevel::Low.impact_multiplier()
        );
    }
}
