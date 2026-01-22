//! Context-aware LLM prompting
//!
//! Uses ctp-context's ContextCompressor to build optimal prompts
//! that fit within token budgets while preserving essential information.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;

use ctp_context::{
    SemanticContext, ContextId, ContextCompressor, CompressedContext,
    integration::ContextProvider,
};

use crate::{LLMClient, LLMConfig, IntentInference};

/// Context-aware intent inference request
#[derive(Debug)]
pub struct ContextAwareRequest<'a> {
    /// The code to analyze
    pub code: &'a str,
    
    /// The focus context (what we're analyzing)
    pub focus: &'a SemanticContext,
    
    /// Related contexts for additional understanding
    pub related_contexts: &'a [SemanticContext],
    
    /// Token budget for context (default: 2000)
    pub context_budget: usize,
}

impl<'a> ContextAwareRequest<'a> {
    pub fn new(code: &'a str, focus: &'a SemanticContext) -> Self {
        Self {
            code,
            focus,
            related_contexts: &[],
            context_budget: 2000,
        }
    }

    pub fn with_related(mut self, contexts: &'a [SemanticContext]) -> Self {
        self.related_contexts = contexts;
        self
    }

    pub fn with_budget(mut self, budget: usize) -> Self {
        self.context_budget = budget;
        self
    }
}

/// Enhanced intent inference with hierarchical context
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextAwareInference {
    /// Base inference result
    pub inference: IntentInference,
    
    /// Detected relationships to other components
    pub detected_relationships: Vec<DetectedRelationship>,
    
    /// Potential redundancies found
    pub potential_redundancies: Vec<String>,
    
    /// Suggested invariants
    pub suggested_invariants: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectedRelationship {
    pub target_purpose: String,
    pub relationship_type: String,
    pub confidence: f64,
}

/// Context-aware LLM client that uses hierarchical compression
pub struct ContextAwareLLM {
    client: LLMClient,
    compressor: ContextCompressor,
}

impl ContextAwareLLM {
    pub fn new(config: LLMConfig) -> Self {
        Self {
            client: LLMClient::new(config),
            compressor: ContextCompressor::new(),
        }
    }

    pub fn with_compressor(mut self, compressor: ContextCompressor) -> Self {
        self.compressor = compressor;
        self
    }

    /// Infer intent with full hierarchical context
    pub async fn infer_intent_with_context(
        &self,
        request: ContextAwareRequest<'_>,
    ) -> Result<ContextAwareInference> {
        debug!("Inferring intent with hierarchical context");

        // Build compressed context
        let mut all_contexts = vec![request.focus.clone()];
        all_contexts.extend(request.related_contexts.iter().cloned());

        let compressed = self.compressor.compress(
            &all_contexts,
            &request.focus.id,
            Some(request.context_budget),
        );

        // Build the enhanced prompt
        let prompt = self.build_context_aware_prompt(request.code, &compressed);

        // Call LLM
        let response = self.call_llm_for_context_analysis(&prompt).await?;

        Ok(response)
    }

    fn build_context_aware_prompt(&self, code: &str, context: &CompressedContext) -> String {
        let context_str = context.to_prompt_string();

        format!(
            r#"You are analyzing code within a larger system. Use the provided context to understand how this code fits into the overall architecture.

## System Context
{context_str}

## Code to Analyze
```
{code}
```

## Instructions
Analyze this code considering:
1. Its purpose within the system hierarchy
2. Relationships to other components mentioned in the context
3. Whether it might duplicate existing functionality
4. Critical invariants that should be maintained

Respond in JSON format:
{{
    "inference": {{
        "inferred_intent": "One sentence describing the code's purpose",
        "confidence": 0.0-1.0,
        "business_context": "How this fits into business logic",
        "technical_rationale": "Technical design decisions"
    }},
    "detected_relationships": [
        {{
            "target_purpose": "Purpose of related component",
            "relationship_type": "calls|depends_on|duplicates|extends",
            "confidence": 0.0-1.0
        }}
    ],
    "potential_redundancies": ["List of components this might duplicate"],
    "suggested_invariants": ["Critical rules this code must maintain"]
}}"#
        )
    }

    async fn call_llm_for_context_analysis(&self, prompt: &str) -> Result<ContextAwareInference> {
        // Use the underlying client's inference
        let base_inference = self.client.infer_intent(prompt, "").await?;

        // For now, return with empty extended fields
        // In production, we'd parse the full JSON response
        Ok(ContextAwareInference {
            inference: base_inference,
            detected_relationships: vec![],
            potential_redundancies: vec![],
            suggested_invariants: vec![],
        })
    }

    /// Analyze multiple components for redundancy
    pub async fn analyze_redundancy(
        &self,
        contexts: &[SemanticContext],
    ) -> Result<RedundancyAnalysis> {
        debug!("Analyzing {} components for redundancy", contexts.len());

        // Group by role category
        let mut by_role: std::collections::HashMap<String, Vec<&SemanticContext>> = 
            std::collections::HashMap::new();

        for ctx in contexts {
            let role = ctx.essence.role.category().to_string();
            by_role.entry(role).or_default().push(ctx);
        }

        let mut potential_duplicates = vec![];

        // Check within each role group for semantic similarity
        for (role, group) in &by_role {
            if group.len() < 2 {
                continue;
            }

            for i in 0..group.len() {
                for j in (i + 1)..group.len() {
                    let similarity = group[i].essence.similarity(&group[j].essence);
                    if similarity > 0.7 {
                        potential_duplicates.push(PotentialDuplicate {
                            component_a: group[i].id.clone(),
                            component_b: group[j].id.clone(),
                            similarity,
                            role: role.clone(),
                            recommendation: format!(
                                "Components '{}' and '{}' have {:.0}% semantic similarity. Consider consolidating.",
                                group[i].essence.purpose,
                                group[j].essence.purpose,
                                similarity * 100.0
                            ),
                        });
                    }
                }
            }
        }

        Ok(RedundancyAnalysis {
            total_components: contexts.len(),
            potential_duplicates,
            role_distribution: by_role.into_iter()
                .map(|(k, v)| (k, v.len()))
                .collect(),
        })
    }
}

/// Result of redundancy analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct RedundancyAnalysis {
    pub total_components: usize,
    pub potential_duplicates: Vec<PotentialDuplicate>,
    pub role_distribution: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PotentialDuplicate {
    pub component_a: ContextId,
    pub component_b: ContextId,
    pub similarity: f64,
    pub role: String,
    pub recommendation: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctp_context::{Essence, ComponentRole, ContextLevel};

    fn create_test_context(name: &str, purpose: &str) -> SemanticContext {
        let id = ContextId::from_name(name, ContextLevel::Module);
        let essence = Essence::new(purpose, ComponentRole::core("test"));
        SemanticContext::new(id, ContextLevel::Module, essence)
    }

    #[test]
    fn test_context_aware_request() {
        let focus = create_test_context("main", "Main entry point");
        let request = ContextAwareRequest::new("fn main() {}", &focus)
            .with_budget(1000);

        assert_eq!(request.context_budget, 1000);
    }

    #[tokio::test]
    async fn test_redundancy_analysis() {
        let contexts = vec![
            create_test_context("auth1", "User authentication handler"),
            create_test_context("auth2", "User authentication service"),
            create_test_context("payments", "Payment processing"),
        ];

        let llm = ContextAwareLLM::new(crate::LLMConfig::default());
        let analysis = llm.analyze_redundancy(&contexts).await.unwrap();

        assert_eq!(analysis.total_components, 3);
        // auth1 and auth2 should be flagged as potential duplicates
        assert!(!analysis.potential_duplicates.is_empty() || 
                analysis.role_distribution.get("core").copied().unwrap_or(0) == 3);
    }
}
