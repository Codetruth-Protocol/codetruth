//! Critical Weight Scoring System
//!
//! Implements the three-pronged approach to critical functionality detection:
//! - Hot Path Discovery (H)
//! - Comments + Significant Usage (U)
//! - Annotations + Specs/Globs (A)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Critical weight score for a symbol/function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalWeight {
    /// Final critical weight (0.0 - 1.0)
    pub weight: f64,
    
    /// Classification level
    pub level: CriticalityLevel,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    
    /// Component scores
    pub scores: ComponentScores,
    
    /// Product weighting applied
    pub product_weight: f64,
}

/// Component scores for each signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    /// Annotation/Spec/Glob signal (0-1)
    pub annotation: f64,
    
    /// Hot path signal (0-1)
    pub hot_path: f64,
    
    /// Usage & comment signal (0-1)
    pub usage: f64,
}

/// Criticality level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CriticalityLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl CriticalityLevel {
    pub fn from_weight(weight: f64) -> Self {
        if weight >= 0.85 {
            CriticalityLevel::Critical
        } else if weight >= 0.70 {
            CriticalityLevel::High
        } else if weight >= 0.50 {
            CriticalityLevel::Medium
        } else {
            CriticalityLevel::Low
        }
    }
}

/// Critical weight calculator
pub struct CriticalWeightCalculator {
    /// Signal weights
    annotation_weight: f64,
    hot_path_weight: f64,
    usage_weight: f64,
    
    /// Classification thresholds
    critical_threshold: f64,
    high_threshold: f64,
    medium_threshold: f64,
    
    /// Product type weights
    product_weights: HashMap<String, HashMap<String, f64>>,
}

impl CriticalWeightCalculator {
    pub fn new() -> Self {
        Self {
            annotation_weight: 0.45,
            hot_path_weight: 0.35,
            usage_weight: 0.20,
            critical_threshold: 0.85,
            high_threshold: 0.70,
            medium_threshold: 0.50,
            product_weights: Self::default_product_weights(),
        }
    }

    /// Calculate critical weight for a symbol
    pub fn calculate(
        &self,
        _symbol: &SymbolInfo,
        call_frequency: f64,
        git_churn: f64,
        comment_signal: f64,
        usage_percentile: f64,
        annotation_score: f64,
        product_type: Option<&str>,
        entity_name: Option<&str>,
    ) -> CriticalWeight {
        // Calculate component scores
        let hot_path_score = self.calculate_hot_path_score(call_frequency, git_churn);
        let usage_score = self.calculate_usage_score(comment_signal, usage_percentile);
        
        // Calculate base weight
        let base_weight = (self.annotation_weight * annotation_score
            + self.hot_path_weight * hot_path_score
            + self.usage_weight * usage_score)
            .clamp(0.0, 1.0);
        
        // Apply product weighting
        let product_weight = self.get_product_weight(product_type, entity_name);
        let final_weight = (base_weight * product_weight).min(1.0);
        
        // Calculate confidence
        let confidence = self.calculate_confidence(
            annotation_score > 0.0,
            hot_path_score > 0.0,
            usage_score > 0.0,
        );
        
        CriticalWeight {
            weight: final_weight,
            level: CriticalityLevel::from_weight(final_weight),
            confidence,
            scores: ComponentScores {
                annotation: annotation_score,
                hot_path: hot_path_score,
                usage: usage_score,
            },
            product_weight,
        }
    }

    /// Calculate hot path score from call frequency and git churn
    fn calculate_hot_path_score(&self, call_frequency: f64, git_churn: f64) -> f64 {
        // Weight: 60% call frequency, 40% git churn
        (0.6 * call_frequency + 0.4 * git_churn).clamp(0.0, 1.0)
    }

    /// Calculate usage score from comment signal and usage percentile
    fn calculate_usage_score(&self, comment_signal: f64, usage_percentile: f64) -> f64 {
        // Weight: 40% comment signal, 40% usage percentile, 20% keyword density
        (0.4 * comment_signal + 0.4 * usage_percentile + 0.2 * comment_signal).clamp(0.0, 1.0)
    }

    /// Get product weight for entity
    fn get_product_weight(&self, product_type: Option<&str>, entity_name: Option<&str>) -> f64 {
        if let (Some(pt), Some(en)) = (product_type, entity_name) {
            if let Some(weights) = self.product_weights.get(pt) {
                if let Some(weight) = weights.get(en) {
                    return *weight;
                }
            }
        }
        1.0 // Default weight
    }

    /// Calculate confidence based on signal availability
    fn calculate_confidence(
        &self,
        has_annotation: bool,
        has_hot_path: bool,
        has_usage: bool,
    ) -> f64 {
        let signal_count = [has_annotation, has_hot_path, has_usage]
            .iter()
            .filter(|&&x| x)
            .count();
        
        match signal_count {
            3 => 1.0,
            2 => 0.7,
            1 => 0.4,
            _ => 0.1,
        }
    }

    /// Default product weights by product type
    fn default_product_weights() -> HashMap<String, HashMap<String, f64>> {
        let mut weights = HashMap::new();
        
        // E-commerce
        let mut ecommerce = HashMap::new();
        ecommerce.insert("orders".to_string(), 1.3);
        ecommerce.insert("order-lines".to_string(), 1.3);
        ecommerce.insert("checkout".to_string(), 1.3);
        ecommerce.insert("payments".to_string(), 1.3);
        ecommerce.insert("refunds".to_string(), 1.3);
        ecommerce.insert("catalog".to_string(), 1.15);
        ecommerce.insert("pricing".to_string(), 1.15);
        ecommerce.insert("inventory".to_string(), 1.15);
        ecommerce.insert("customers".to_string(), 1.05);
        weights.insert("ecommerce".to_string(), ecommerce);
        
        // B2C SaaS
        let mut b2c_saas = HashMap::new();
        b2c_saas.insert("authentication".to_string(), 1.2);
        b2c_saas.insert("session".to_string(), 1.2);
        b2c_saas.insert("billing".to_string(), 1.25);
        b2c_saas.insert("subscription".to_string(), 1.25);
        weights.insert("b2c_saas".to_string(), b2c_saas);
        
        // Financial Services
        let mut financial = HashMap::new();
        financial.insert("ledger".to_string(), 1.35);
        financial.insert("transactions".to_string(), 1.35);
        financial.insert("reconciliation".to_string(), 1.25);
        financial.insert("reporting".to_string(), 1.25);
        financial.insert("risk".to_string(), 1.3);
        financial.insert("fraud".to_string(), 1.3);
        weights.insert("financial_services".to_string(), financial);
        
        weights
    }
}

impl Default for CriticalWeightCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Symbol information for criticality calculation
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_criticality_level_from_weight() {
        assert_eq!(
            CriticalityLevel::from_weight(0.90),
            CriticalityLevel::Critical
        );
        assert_eq!(CriticalityLevel::from_weight(0.75), CriticalityLevel::High);
        assert_eq!(CriticalityLevel::from_weight(0.60), CriticalityLevel::Medium);
        assert_eq!(CriticalityLevel::from_weight(0.30), CriticalityLevel::Low);
    }

    #[test]
    fn test_critical_weight_calculation() {
        let calculator = CriticalWeightCalculator::new();
        let symbol = SymbolInfo {
            name: "processOrder".to_string(),
            file_path: "src/orders/processor.ts".to_string(),
            line_start: 10,
            line_end: 50,
        };

        let weight = calculator.calculate(
            &symbol,
            0.85, // High call frequency
            0.80, // High git churn
            0.90, // Strong comment signal
            0.75, // High usage percentile
            0.90, // Strong annotation
            Some("ecommerce"),
            Some("orders"),
        );

        assert!(weight.weight >= 0.85);
        assert_eq!(weight.level, CriticalityLevel::Critical);
        assert!(weight.confidence > 0.7);
    }
}
