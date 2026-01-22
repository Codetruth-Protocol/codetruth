//! Integration with other CTP crates
//!
//! Provides conversion between `ctp-core` types and `ctp-context` types,
//! enabling gradual adoption of the hierarchical context system.

use crate::context::{ContextId, ContextLevel, SemanticContext, DetailLevel};
use crate::essence::{Essence, ComponentRole, BoundaryDirection};
use crate::relationship::{Relationship, RelationshipType};

/// Convert an ExplanationGraph from ctp-core into a SemanticContext
/// 
/// This enables backward compatibility - existing analyses can be
/// upgraded to the new hierarchical system.
pub trait FromExplanationGraph {
    /// Convert to a module-level SemanticContext
    fn to_semantic_context(&self) -> SemanticContext;
    
    /// Extract the essence from the explanation graph
    fn extract_essence(&self) -> Essence;
}

/// Trait for types that can provide context for LLM prompts
pub trait ContextProvider {
    /// Get compressed context suitable for LLM consumption
    fn get_llm_context(&self, token_budget: usize) -> String;
    
    /// Get the essence (never compressed)
    fn get_essence(&self) -> &Essence;
    
    /// Get relationships to other contexts
    fn get_relationships(&self) -> &[Relationship];
}

impl ContextProvider for SemanticContext {
    fn get_llm_context(&self, token_budget: usize) -> String {
        let mut parts = vec![];
        let mut tokens_used = 0;
        
        // Always include essence
        parts.push(format!("**Purpose**: {}", self.essence.purpose));
        parts.push(format!("**Role**: {}", self.essence.role.category()));
        tokens_used += self.essence.token_count();
        
        // Include constraints if budget allows
        if !self.essence.constraints.is_empty() && tokens_used < token_budget {
            parts.push(format!("**Constraints**: {}", self.essence.constraints.join("; ")));
            tokens_used += self.essence.constraints.iter().map(|c| c.len() / 4).sum::<usize>();
        }
        
        // Include detail if budget allows
        if let Some(detail) = &self.detail {
            if tokens_used + detail.token_count < token_budget {
                parts.push(format!("**Details**: {}", detail.full_description));
            }
        }
        
        // Include invariants if budget allows
        if !self.invariants.is_empty() && tokens_used < token_budget {
            let inv_str: Vec<_> = self.invariants.iter()
                .map(|i| format!("- {}", i.description))
                .collect();
            parts.push(format!("**Invariants**:\n{}", inv_str.join("\n")));
        }
        
        parts.join("\n")
    }
    
    fn get_essence(&self) -> &Essence {
        &self.essence
    }
    
    fn get_relationships(&self) -> &[Relationship] {
        &self.relationships
    }
}

/// Helper to infer component role from code patterns
pub fn infer_role_from_patterns(
    file_path: &str,
    behavior_description: &str,
    has_io: bool,
    has_network: bool,
    has_db: bool,
) -> ComponentRole {
    let path_lower = file_path.to_lowercase();
    let behavior_lower = behavior_description.to_lowercase();
    
    // Check for test files
    if path_lower.contains("test") || path_lower.contains("spec") {
        return ComponentRole::test();
    }
    
    // Check for config files
    if path_lower.contains("config") || path_lower.contains("settings") {
        return ComponentRole::Config;
    }
    
    // Check for API/boundary patterns
    if path_lower.contains("api") || path_lower.contains("handler") || 
       path_lower.contains("controller") || path_lower.contains("endpoint") {
        return ComponentRole::boundary_in();
    }
    
    // Check for client/outbound patterns
    if path_lower.contains("client") || has_network {
        return ComponentRole::boundary_out();
    }
    
    // Check for data patterns
    if path_lower.contains("model") || path_lower.contains("schema") || 
       path_lower.contains("entity") || has_db {
        return ComponentRole::data();
    }
    
    // Check for utility patterns
    if path_lower.contains("util") || path_lower.contains("helper") || 
       path_lower.contains("common") {
        return ComponentRole::utility();
    }
    
    // Check for orchestration patterns
    if behavior_lower.contains("orchestrat") || behavior_lower.contains("coordinat") ||
       path_lower.contains("service") || path_lower.contains("manager") {
        return ComponentRole::orchestrator();
    }
    
    // Default to core with domain inferred from path
    let domain = infer_domain_from_path(file_path);
    ComponentRole::core(&domain)
}

/// Infer domain from file path
pub fn infer_domain_from_path(file_path: &str) -> String {
    // Extract the first meaningful directory after src/
    let path_parts: Vec<&str> = file_path.split(['/', '\\']).collect();
    
    // Find 'src' and get next part
    if let Some(src_idx) = path_parts.iter().position(|&p| p == "src") {
        if src_idx + 1 < path_parts.len() {
            let next_part = path_parts[src_idx + 1];
            
            // Check if it's a directory (not a file with extension)
            if !next_part.contains('.') {
                // Skip common non-domain directories
                if !["lib", "main", "index", "app"].contains(&next_part) {
                    return next_part.to_string();
                }
            }
        }
    }
    
    // Fallback: use filename without extension
    if let Some(filename) = path_parts.last() {
        // Try to strip common extensions
        let name = filename
            .strip_suffix(".rs")
            .or_else(|| filename.strip_suffix(".ts"))
            .or_else(|| filename.strip_suffix(".tsx"))
            .or_else(|| filename.strip_suffix(".js"))
            .or_else(|| filename.strip_suffix(".jsx"))
            .or_else(|| filename.strip_suffix(".py"))
            .or_else(|| filename.strip_suffix(".go"))
            .or_else(|| filename.strip_suffix(".java"))
            .unwrap_or(filename);
        return name.to_string();
    }
    
    "unknown".to_string()
}

/// Infer context level from file path depth and patterns
pub fn infer_level_from_path(file_path: &str) -> ContextLevel {
    let path_parts: Vec<&str> = file_path.split(['/', '\\']).collect();
    let depth = path_parts.len();
    
    // Very shallow paths are likely domain-level
    if depth <= 2 {
        return ContextLevel::Domain;
    }
    
    // Check for index/mod files (often domain or module level)
    let filename = path_parts.last().unwrap_or(&"");
    if filename.starts_with("index") || filename.starts_with("mod") || 
       filename.starts_with("lib") || filename.starts_with("main") {
        if depth <= 3 {
            return ContextLevel::Domain;
        }
    }
    
    // Default to module level for regular files
    ContextLevel::Module
}

/// Build relationships from import/dependency analysis
pub fn build_relationships_from_imports(
    imports: &[String],
    file_path: &str,
) -> Vec<Relationship> {
    imports.iter()
        .filter_map(|import| {
            // Convert import path to context ID
            let target_id = ContextId::from_path(import);
            
            // Skip self-references
            if import == file_path {
                return None;
            }
            
            Some(Relationship::depends_on(target_id).with_strength(0.5))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_role_from_patterns() {
        assert!(matches!(
            infer_role_from_patterns("src/api/handler.rs", "handles requests", false, false, false),
            ComponentRole::Boundary { .. }
        ));
        
        assert!(matches!(
            infer_role_from_patterns("src/utils/string.rs", "string utilities", false, false, false),
            ComponentRole::Utility { .. }
        ));
        
        assert!(matches!(
            infer_role_from_patterns("tests/test_payment.rs", "tests payment", false, false, false),
            ComponentRole::Test { .. }
        ));
    }

    #[test]
    fn test_infer_domain_from_path() {
        assert_eq!(infer_domain_from_path("src/payments/charge.rs"), "payments");
        assert_eq!(infer_domain_from_path("src/auth/login.rs"), "auth");
        // When no domain directory found, falls back to filename without extension
        assert_eq!(infer_domain_from_path("crates/ctp-core/src/engine.rs"), "engine");
    }

    #[test]
    fn test_infer_level_from_path() {
        assert_eq!(infer_level_from_path("src/lib.rs"), ContextLevel::Domain);
        assert_eq!(infer_level_from_path("src/payments/charge.rs"), ContextLevel::Module);
        assert_eq!(infer_level_from_path("src/index.ts"), ContextLevel::Domain);
    }
}
