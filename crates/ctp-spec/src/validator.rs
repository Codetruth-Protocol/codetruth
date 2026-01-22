//! Product specification validation against codebase

use super::*;
use ctp_core::ExplanationGraph;

pub struct SpecValidator;

impl SpecValidator {
    pub fn new() -> Self {
        Self
    }
    
    /// Validate spec against actual code
    pub fn validate(
        &self,
        spec: &ProductMetadata,
        analyses: &[ExplanationGraph],
    ) -> Result<ValidationReport> {
        let mut missing_functionalities = vec![];
        let mut undocumented_code = vec![];
        let mut spec_drift = vec![];
        
        // Check if all spec functionalities are implemented
        for func in &spec.core_functionalities {
            let implemented = analyses.iter().any(|a| {
                func.entry_points.iter().any(|ep| a.module.path.contains(&ep.file))
            });
            
            if !implemented {
                missing_functionalities.push(func.name.clone());
            }
        }
        
        // Check for code not in spec
        for analysis in analyses {
            let in_spec = spec.core_functionalities.iter().any(|func| {
                func.entry_points.iter().any(|ep| analysis.module.path.contains(&ep.file))
            });
            
            if !in_spec && !analysis.intent.inferred_intent.is_empty() {
                undocumented_code.push(analysis.module.path.clone());
            }
        }
        
        // Calculate confidence
        let total_funcs = spec.core_functionalities.len();
        let implemented_funcs = total_funcs - missing_functionalities.len();
        let confidence = if total_funcs > 0 {
            implemented_funcs as f64 / total_funcs as f64
        } else {
            1.0
        };
        
        Ok(ValidationReport {
            is_valid: missing_functionalities.is_empty(),
            confidence,
            missing_functionalities,
            undocumented_code,
            spec_drift,
        })
    }
}

impl Default for SpecValidator {
    fn default() -> Self {
        Self::new()
    }
}
