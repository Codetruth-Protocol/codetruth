//! Design Deviation Detection
//!
//! Detects deviations from the established design system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::tokens::{DesignTokens, ColorToken, SpacingToken};

/// Types of design deviations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeviationType {
    /// Color not in the design system palette
    UnknownColor {
        color: String,
        similar_to: Option<String>,
        distance: Option<f64>,
    },
    
    /// Typography doesn't match established patterns
    TypographyDeviation {
        element: String,
        property: String,
        expected: String,
        actual: String,
    },
    
    /// Spacing value not in the scale
    SpacingDeviation {
        value: String,
        nearest_scale: Option<String>,
    },
    
    /// Component variant not defined in design system
    UnknownComponentVariant {
        component: String,
        variant: String,
        known_variants: Vec<String>,
    },
    
    /// Hardcoded value instead of design token
    HardcodedValue {
        value: String,
        should_use: String,
        token_type: String,
    },
    
    /// Inconsistent styling for same element type
    InconsistentStyle {
        element: String,
        majority_style: String,
        deviation_style: String,
        majority_count: usize,
    },
}

/// A detected design deviation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignDeviation {
    pub deviation_type: DeviationType,
    pub file: String,
    pub line: usize,
    pub code_snippet: String,
    pub severity: DeviationSeverity,
    pub suggestion: String,
}

/// Severity of design deviation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviationSeverity {
    /// Minor inconsistency
    Info,
    /// Should be fixed but not critical
    Warning,
    /// Significant deviation from design system
    Error,
}

/// Configuration for deviation detection
#[derive(Debug, Clone)]
pub struct DeviationConfig {
    /// Color distance threshold for "similar" colors
    pub color_similarity_threshold: f64,
    /// Minimum usage count to consider something "established"
    pub min_usage_for_established: usize,
    /// Whether to flag hardcoded values
    pub flag_hardcoded_values: bool,
    /// Minimum severity to report
    pub min_severity: DeviationSeverity,
}

impl Default for DeviationConfig {
    fn default() -> Self {
        Self {
            color_similarity_threshold: 30.0,
            min_usage_for_established: 3,
            flag_hardcoded_values: true,
            min_severity: DeviationSeverity::Info,
        }
    }
}

/// Detects design system deviations
pub struct DeviationDetector {
    config: DeviationConfig,
    design_system: DesignTokens,
}

impl DeviationDetector {
    pub fn new(design_system: DesignTokens, config: DeviationConfig) -> Self {
        Self { config, design_system }
    }

    /// Detect deviations in a code snippet
    pub fn detect(&self, content: &str, file_path: &str) -> Vec<DesignDeviation> {
        let mut deviations = vec![];
        
        // Extract tokens from the new code
        let extractor = crate::extractor::DesignSystemExtractor::new();
        let file_tokens = extractor.extract_from_file(content, file_path);
        
        // Check colors
        for (hex, color) in &file_tokens.colors {
            if !self.design_system.has_color(hex) {
                let (similar, distance) = self.find_similar_color(hex);
                
                let severity = if distance.map(|d| d < self.config.color_similarity_threshold).unwrap_or(false) {
                    DeviationSeverity::Warning
                } else {
                    DeviationSeverity::Error
                };
                
                if severity >= self.config.min_severity {
                    deviations.push(DesignDeviation {
                        deviation_type: DeviationType::UnknownColor {
                            color: hex.clone(),
                            similar_to: similar.clone(),
                            distance,
                        },
                        file: file_path.to_string(),
                        line: 0, // Would need line tracking in extractor
                        code_snippet: hex.clone(),
                        severity,
                        suggestion: match &similar {
                            Some(s) => format!("Consider using '{}' instead of '{}'", s, hex),
                            None => format!("Add '{}' to design system or use an existing color", hex),
                        },
                    });
                }
            }
        }
        
        // Check spacing
        for (value, _spacing) in &file_tokens.spacing {
            if !self.is_valid_spacing(value) {
                let nearest = self.find_nearest_spacing(value);
                
                if self.config.min_severity <= DeviationSeverity::Warning {
                    deviations.push(DesignDeviation {
                        deviation_type: DeviationType::SpacingDeviation {
                            value: value.clone(),
                            nearest_scale: nearest.clone(),
                        },
                        file: file_path.to_string(),
                        line: 0,
                        code_snippet: value.clone(),
                        severity: DeviationSeverity::Warning,
                        suggestion: match &nearest {
                            Some(n) => format!("Use '{}' instead of '{}'", n, value),
                            None => format!("'{}' is not in the spacing scale", value),
                        },
                    });
                }
            }
        }
        
        // Check components
        for (name, component) in &file_tokens.components {
            if let Some(established) = self.design_system.components.get(name) {
                // Check for unknown variants
                for variant in &component.variants {
                    if !established.variants.contains(variant) {
                        if self.config.min_severity <= DeviationSeverity::Warning {
                            deviations.push(DesignDeviation {
                                deviation_type: DeviationType::UnknownComponentVariant {
                                    component: name.clone(),
                                    variant: variant.clone(),
                                    known_variants: established.variants.clone(),
                                },
                                file: file_path.to_string(),
                                line: 0,
                                code_snippet: format!("<{} variant=\"{}\">", name, variant),
                                severity: DeviationSeverity::Warning,
                                suggestion: format!(
                                    "Use one of the established variants: {:?}",
                                    established.variants
                                ),
                            });
                        }
                    }
                }
            }
        }
        
        deviations
    }

    /// Detect deviations between old and new code (for PR review)
    pub fn detect_new_deviations(
        &self,
        old_content: &str,
        new_content: &str,
        file_path: &str,
    ) -> Vec<DesignDeviation> {
        let old_deviations = self.detect(old_content, file_path);
        let new_deviations = self.detect(new_content, file_path);
        
        // Return only deviations that are new
        new_deviations
            .into_iter()
            .filter(|new_dev| {
                !old_deviations.iter().any(|old_dev| {
                    self.deviations_match(old_dev, new_dev)
                })
            })
            .collect()
    }

    fn deviations_match(&self, a: &DesignDeviation, b: &DesignDeviation) -> bool {
        match (&a.deviation_type, &b.deviation_type) {
            (
                DeviationType::UnknownColor { color: c1, .. },
                DeviationType::UnknownColor { color: c2, .. },
            ) => c1 == c2,
            (
                DeviationType::SpacingDeviation { value: v1, .. },
                DeviationType::SpacingDeviation { value: v2, .. },
            ) => v1 == v2,
            (
                DeviationType::UnknownComponentVariant { component: c1, variant: v1, .. },
                DeviationType::UnknownComponentVariant { component: c2, variant: v2, .. },
            ) => c1 == c2 && v1 == v2,
            _ => false,
        }
    }

    fn find_similar_color(&self, hex: &str) -> (Option<String>, Option<f64>) {
        if let Some(target) = ColorToken::from_hex(hex) {
            let mut closest: Option<(String, f64)> = None;
            
            for (name, token) in &self.design_system.colors.colors {
                let distance = target.distance(token);
                if closest.is_none() || distance < closest.as_ref().unwrap().1 {
                    closest = Some((name.clone(), distance));
                }
            }
            
            if let Some((name, dist)) = closest {
                return (Some(name), Some(dist));
            }
        }
        (None, None)
    }

    fn is_valid_spacing(&self, value: &str) -> bool {
        self.design_system.spacing.common_values.contains_key(value)
            || self.design_system.spacing.scale.iter().any(|s| s.value == value)
    }

    fn find_nearest_spacing(&self, value: &str) -> Option<String> {
        let target = SpacingToken::from_value(value);
        let target_px = target.pixels?;
        
        let mut nearest: Option<(String, f64)> = None;
        
        for spacing in &self.design_system.spacing.scale {
            if let Some(px) = spacing.pixels {
                let diff = (target_px - px).abs();
                if nearest.is_none() || diff < nearest.as_ref().unwrap().1 {
                    nearest = Some((spacing.value.clone(), diff));
                }
            }
        }
        
        nearest.map(|(v, _)| v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::ColorPalette;

    fn create_test_design_system() -> DesignTokens {
        let mut tokens = DesignTokens::new();
        
        // Add some colors
        tokens.colors.colors.insert(
            "#2563eb".to_string(),
            ColorToken::from_hex("#2563eb").unwrap(),
        );
        tokens.colors.colors.insert(
            "#1f2937".to_string(),
            ColorToken::from_hex("#1f2937").unwrap(),
        );
        
        tokens
    }

    #[test]
    fn test_detect_unknown_color() {
        let design_system = create_test_design_system();
        let detector = DeviationDetector::new(design_system, DeviationConfig::default());
        
        let content = r#"<div style={{ color: '#ff0000' }}>"#;
        let deviations = detector.detect(content, "test.tsx");
        
        // Should detect #ff0000 as unknown
        assert!(deviations.iter().any(|d| {
            matches!(&d.deviation_type, DeviationType::UnknownColor { color, .. } if color == "#ff0000")
        }));
    }

    #[test]
    fn test_no_deviation_for_known_color() {
        let design_system = create_test_design_system();
        let detector = DeviationDetector::new(design_system, DeviationConfig::default());
        
        let content = r#"<div style={{ color: '#2563eb' }}>"#;
        let deviations = detector.detect(content, "test.tsx");
        
        // Should not flag the known color
        assert!(deviations.iter().all(|d| {
            !matches!(&d.deviation_type, DeviationType::UnknownColor { color, .. } if color == "#2563eb")
        }));
    }
}
