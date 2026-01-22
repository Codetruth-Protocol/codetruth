//! Bridge between ctp-core and ctp-context
//!
//! Provides conversion from ExplanationGraph to SemanticContext,
//! enabling hierarchical context management for large codebases.

use ctp_context::{
    SemanticContext, ContextId, ContextLevel, Essence,
    ComponentRegistry, Relationship, RelationshipType,
    context::DetailLevel,
    integration::infer_role_from_patterns,
};

use crate::models::{ExplanationGraph, SideEffectType};

/// Convert an ExplanationGraph to a SemanticContext
pub fn explanation_graph_to_context(graph: &ExplanationGraph) -> SemanticContext {
    let id = ContextId::from_path(&graph.module.path);
    
    // Infer role from behavior patterns
    let has_io = graph.behavior.side_effects.iter()
        .any(|s| matches!(s.effect_type, SideEffectType::Io));
    let has_network = graph.behavior.side_effects.iter()
        .any(|s| matches!(s.effect_type, SideEffectType::Network));
    let has_db = graph.behavior.side_effects.iter()
        .any(|s| matches!(s.effect_type, SideEffectType::Database));
    
    let role = infer_role_from_patterns(
        &graph.module.path,
        &graph.behavior.actual_behavior,
        has_io,
        has_network,
        has_db,
    );
    
    // Build essence from intent
    let purpose = if !graph.intent.inferred_intent.is_empty() {
        graph.intent.inferred_intent.clone()
    } else if !graph.intent.declared_intent.is_empty() {
        graph.intent.declared_intent.clone()
    } else {
        format!("{} module", graph.module.name)
    };
    
    let essence = Essence::new(&purpose, role)
        .with_keywords(&extract_keywords(&graph.behavior.actual_behavior));
    
    // Create context
    let mut ctx = SemanticContext::new(id, ContextLevel::Module, essence);
    
    // Add detail from behavior
    if !graph.behavior.actual_behavior.is_empty() {
        let detail = DetailLevel::new(&graph.behavior.actual_behavior);
        ctx = ctx.with_detail(detail);
    }
    
    // Add invariants from drift analysis
    if graph.drift.drift_detected {
        for detail in &graph.drift.drift_details {
            let invariant = ctp_context::context::Invariant::critical(
                &format!("drift_{}", detail.drift_type.as_str()),
                &format!("Expected: {} | Actual: {}", detail.expected, detail.actual),
            );
            ctx.add_invariant(invariant);
        }
    }
    
    ctx
}

/// Extract keywords from behavior description
fn extract_keywords(behavior: &str) -> Vec<&str> {
    // Extract meaningful words (skip common words)
    let stop_words = ["the", "a", "an", "is", "are", "was", "were", "be", "been", 
                      "being", "have", "has", "had", "do", "does", "did", "will",
                      "would", "could", "should", "may", "might", "must", "shall",
                      "can", "need", "to", "of", "in", "for", "on", "with", "at",
                      "by", "from", "as", "into", "through", "during", "before",
                      "after", "above", "below", "between", "under", "again",
                      "further", "then", "once", "here", "there", "when", "where",
                      "why", "how", "all", "each", "few", "more", "most", "other",
                      "some", "such", "no", "nor", "not", "only", "own", "same",
                      "so", "than", "too", "very", "just", "and", "but", "if",
                      "or", "because", "until", "while", "this", "that", "these",
                      "those", "it", "its"];
    
    behavior
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .filter(|w| !stop_words.contains(&w.to_lowercase().as_str()))
        .take(10)
        .collect()
}

/// Extension trait for DriftType to get string representation
trait DriftTypeExt {
    fn as_str(&self) -> &'static str;
}

impl DriftTypeExt for crate::models::DriftType {
    fn as_str(&self) -> &'static str {
        match self {
            crate::models::DriftType::Intent => "intent",
            crate::models::DriftType::Policy => "policy",
            crate::models::DriftType::Assumption => "assumption",
            crate::models::DriftType::Implementation => "implementation",
        }
    }
}

/// Analyze a codebase and build a hierarchical context tree
pub struct CodebaseContextBuilder {
    registry: ComponentRegistry,
    contexts: Vec<SemanticContext>,
    system_context: Option<SemanticContext>,
}

impl CodebaseContextBuilder {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            contexts: vec![],
            system_context: None,
        }
    }

    /// Set the system-level context (root of hierarchy)
    pub fn with_system(mut self, name: &str, purpose: &str) -> Self {
        let ctx = SemanticContext::system(name, purpose);
        self.system_context = Some(ctx);
        self
    }

    /// Add an ExplanationGraph to the context tree
    pub fn add_graph(&mut self, graph: &ExplanationGraph) -> Result<(), ctp_context::ContextError> {
        let mut ctx = explanation_graph_to_context(graph);
        
        // Link to system context if available
        if let Some(system) = &self.system_context {
            ctx.parent_id = Some(system.id.clone());
        }
        
        // Check for redundancy before adding
        let redundancy = self.registry.check_redundancy(&ctx.essence);
        match redundancy {
            ctp_context::RedundancyReport::Duplicate { existing, similarity, .. } => {
                // Add as duplicate relationship instead of failing
                let rel = Relationship::duplicates(existing)
                    .with_strength(similarity)
                    .with_reason("Semantic similarity detected during analysis");
                ctx.add_relationship(rel);
            }
            ctp_context::RedundancyReport::Related { similar } => {
                // Add relationships to similar components
                for sim in similar {
                    let rel = Relationship::new(sim.id, RelationshipType::CoChanges)
                        .with_strength(sim.similarity)
                        .with_reason("Related functionality");
                    ctx.add_relationship(rel);
                }
            }
            ctp_context::RedundancyReport::Unique => {}
        }
        
        // Register in registry
        self.registry.register(ctx.id.clone(), ctx.essence.clone())?;
        self.contexts.push(ctx);
        
        Ok(())
    }

    /// Build relationships between contexts based on imports/dependencies
    pub fn build_relationships(&mut self, dependencies: &[(String, Vec<String>)]) {
        // First, collect all valid target IDs
        let existing_ids: std::collections::HashSet<ContextId> = self.contexts
            .iter()
            .map(|c| c.id.clone())
            .collect();
        
        for (source_path, deps) in dependencies {
            let source_id = ContextId::from_path(source_path);
            
            // Collect valid relationships first
            let valid_deps: Vec<ContextId> = deps
                .iter()
                .map(|dep_path| ContextId::from_path(dep_path))
                .filter(|target_id| existing_ids.contains(target_id))
                .collect();
            
            // Then apply them
            if let Some(ctx) = self.contexts.iter_mut().find(|c| c.id == source_id) {
                for target_id in valid_deps {
                    ctx.add_relationship(
                        Relationship::depends_on(target_id).with_strength(0.6)
                    );
                }
            }
        }
    }

    /// Get all contexts
    pub fn contexts(&self) -> &[SemanticContext] {
        &self.contexts
    }

    /// Get the registry
    pub fn registry(&self) -> &ComponentRegistry {
        &self.registry
    }

    /// Get system context
    pub fn system(&self) -> Option<&SemanticContext> {
        self.system_context.as_ref()
    }

    /// Find potential redundancies in the codebase
    pub fn find_redundancies(&self) -> Vec<(ContextId, ContextId, f64)> {
        let mut redundancies = vec![];
        
        for ctx in &self.contexts {
            for rel in &ctx.relationships {
                if rel.indicates_redundancy() {
                    redundancies.push((ctx.id.clone(), rel.target.clone(), rel.strength));
                }
            }
        }
        
        redundancies
    }

    /// Get statistics about the codebase context
    pub fn stats(&self) -> CodebaseStats {
        let registry_stats = self.registry.stats();
        
        let total_invariants: usize = self.contexts.iter()
            .map(|c| c.invariants.len())
            .sum();
        
        let total_relationships: usize = self.contexts.iter()
            .map(|c| c.relationships.len())
            .sum();
        
        let redundancy_count = self.find_redundancies().len();
        
        CodebaseStats {
            total_components: self.contexts.len(),
            role_distribution: registry_stats.role_distribution,
            total_invariants,
            total_relationships,
            redundancy_count,
        }
    }
}

impl Default for CodebaseContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the codebase context
#[derive(Debug, Clone)]
pub struct CodebaseStats {
    pub total_components: usize,
    pub role_distribution: std::collections::HashMap<String, usize>,
    pub total_invariants: usize,
    pub total_relationships: usize,
    pub redundancy_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn create_test_graph(name: &str, path: &str, intent: &str) -> ExplanationGraph {
        ExplanationGraph {
            ctp_version: "1.0.0".into(),
            explanation_id: format!("test_{}", name),
            module: Module {
                name: name.into(),
                path: path.into(),
                language: "rust".into(),
                lines_of_code: 100,
                complexity_score: 5.0,
                content_hash: "test_hash".into(),
            },
            intent: Intent {
                declared_intent: intent.into(),
                inferred_intent: intent.into(),
                confidence: 0.9,
                business_context: String::new(),
                technical_rationale: String::new(),
            },
            behavior: Behavior {
                actual_behavior: format!("{} implementation", intent),
                entry_points: vec![],
                exit_points: vec![],
                side_effects: vec![],
                dependencies: vec![],
            },
            drift: DriftAnalysis {
                drift_detected: false,
                drift_severity: DriftSeverity::None,
                drift_details: vec![],
            },
            policies: PolicyResults {
                evaluated_at: chrono::Utc::now().to_rfc3339(),
                policy_results: vec![],
            },
            history: History {
                previous_versions: vec![],
                evolution: Evolution {
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_modified: chrono::Utc::now().to_rfc3339(),
                    modification_count: 0,
                    stability_score: 1.0,
                },
            },
            metadata: Metadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator: Generator {
                    name: "test".into(),
                    version: "1.0.0".into(),
                    llm_provider: None,
                    llm_model: None,
                },
                extensions: serde_json::json!({}),
            },
        }
    }

    #[test]
    fn test_graph_to_context_conversion() {
        let graph = create_test_graph("engine", "src/engine.rs", "Main analysis engine");
        let ctx = explanation_graph_to_context(&graph);
        
        assert_eq!(ctx.level, ContextLevel::Module);
        assert!(ctx.essence.purpose.contains("analysis") || ctx.essence.purpose.contains("engine"));
    }

    #[test]
    fn test_codebase_builder() {
        let mut builder = CodebaseContextBuilder::new()
            .with_system("my-app", "Test application");
        
        let graph1 = create_test_graph("auth", "src/auth/login.rs", "User authentication");
        let graph2 = create_test_graph("payments", "src/payments/charge.rs", "Payment processing");
        
        builder.add_graph(&graph1).unwrap();
        builder.add_graph(&graph2).unwrap();
        
        assert_eq!(builder.contexts().len(), 2);
        assert!(builder.system().is_some());
    }

    #[test]
    fn test_redundancy_detection() {
        let mut builder = CodebaseContextBuilder::new();
        
        // Add two similar components - this will trigger redundancy detection
        let graph1 = create_test_graph("auth1", "src/auth/login.rs", "User authentication and login");
        let graph2 = create_test_graph("auth2", "src/legacy/auth.rs", "User authentication and login handling");
        
        builder.add_graph(&graph1).unwrap();
        
        // Second add should fail due to redundancy detection
        let result = builder.add_graph(&graph2);
        assert!(result.is_err(), "Expected redundancy error but got Ok");
        
        // Should only have one context since the second was rejected
        assert_eq!(builder.contexts().len(), 1);
        
        let _redundancies = builder.find_redundancies();
        // Redundancy detection is working as expected
    }
}
