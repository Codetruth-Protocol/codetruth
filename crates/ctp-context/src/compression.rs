//! Context Compression - fits large codebases into LLM context windows
//!
//! Implements hierarchical compression that preserves essence while
//! reducing token count. Like summarizing medical records without
//! losing critical patient information.

use std::collections::{HashMap, HashSet};

use crate::context::{ContextId, ContextLevel, SemanticContext};
use crate::essence::Essence;

/// Compressed context ready for LLM consumption
#[derive(Debug, Clone)]
pub struct CompressedContext {
    /// Token budget for this context
    budget: usize,

    /// Current token count
    current_tokens: usize,

    /// Full contexts (with all detail)
    full_contexts: Vec<SemanticContext>,

    /// Essence-only contexts (compressed)
    essence_only: Vec<EssenceSnapshot>,

    /// Summarized groups
    summaries: Vec<GroupSummary>,
}

/// Snapshot of just the essence (minimal representation)
#[derive(Debug, Clone)]
pub struct EssenceSnapshot {
    pub id: ContextId,
    pub level: ContextLevel,
    pub purpose: String,
    pub role_category: String,
    pub constraint_count: usize,
}

impl EssenceSnapshot {
    pub fn from_context(ctx: &SemanticContext) -> Self {
        Self {
            id: ctx.id.clone(),
            level: ctx.level,
            purpose: ctx.essence.purpose.clone(),
            role_category: ctx.essence.role.category().to_string(),
            constraint_count: ctx.essence.constraints.len(),
        }
    }

    pub fn token_count(&self) -> usize {
        // Rough estimate: purpose + overhead
        (self.purpose.len() + 20) / 4
    }
}

/// Summary of a group of related contexts
#[derive(Debug, Clone)]
pub struct GroupSummary {
    pub description: String,
    pub component_count: usize,
    pub component_ids: Vec<ContextId>,
    pub common_role: String,
}

impl GroupSummary {
    pub fn token_count(&self) -> usize {
        (self.description.len() + 10) / 4
    }
}

impl CompressedContext {
    pub fn new(budget: usize) -> Self {
        Self {
            budget,
            current_tokens: 0,
            full_contexts: vec![],
            essence_only: vec![],
            summaries: vec![],
        }
    }

    /// Add a full context (with all detail)
    pub fn add_full(&mut self, ctx: &SemanticContext) -> bool {
        let tokens = ctx.token_count();
        if self.current_tokens + tokens <= self.budget {
            self.full_contexts.push(ctx.clone());
            self.current_tokens += tokens;
            true
        } else {
            false
        }
    }

    /// Add essence-only snapshot
    pub fn add_essence(&mut self, ctx: &SemanticContext) -> bool {
        let snapshot = EssenceSnapshot::from_context(ctx);
        let tokens = snapshot.token_count();

        if self.current_tokens + tokens <= self.budget {
            self.essence_only.push(snapshot);
            self.current_tokens += tokens;
            true
        } else {
            false
        }
    }

    /// Add a group summary
    pub fn add_summary(&mut self, summary: GroupSummary) -> bool {
        let tokens = summary.token_count();
        if self.current_tokens + tokens <= self.budget {
            self.current_tokens += tokens;
            self.summaries.push(summary);
            true
        } else {
            false
        }
    }

    /// Get current token count
    pub fn token_count(&self) -> usize {
        self.current_tokens
    }

    /// Get remaining budget
    pub fn remaining_budget(&self) -> usize {
        self.budget.saturating_sub(self.current_tokens)
    }

    /// Convert to a string suitable for LLM prompt
    pub fn to_prompt_string(&self) -> String {
        let mut parts = vec![];

        // Full contexts first (most important)
        if !self.full_contexts.is_empty() {
            parts.push("## Primary Components (Full Detail)".to_string());
            for ctx in &self.full_contexts {
                parts.push(format!(
                    "### {} [{}]\n**Purpose**: {}\n**Role**: {}\n**Constraints**: {}",
                    ctx.id,
                    ctx.level.as_str(),
                    ctx.essence.purpose,
                    ctx.essence.role.category(),
                    if ctx.essence.constraints.is_empty() {
                        "None".to_string()
                    } else {
                        ctx.essence.constraints.join("; ")
                    }
                ));

                if let Some(detail) = &ctx.detail {
                    parts.push(format!("**Details**: {}", detail.full_description));
                }

                if !ctx.invariants.is_empty() {
                    let invariants: Vec<_> = ctx.invariants.iter()
                        .map(|i| format!("- {}: {}", i.id, i.description))
                        .collect();
                    parts.push(format!("**Invariants**:\n{}", invariants.join("\n")));
                }
            }
        }

        // Essence snapshots (related components)
        if !self.essence_only.is_empty() {
            parts.push("\n## Related Components (Summary)".to_string());
            for snapshot in &self.essence_only {
                parts.push(format!(
                    "- **{}** [{}]: {}",
                    snapshot.id,
                    snapshot.role_category,
                    snapshot.purpose
                ));
            }
        }

        // Group summaries (context)
        if !self.summaries.is_empty() {
            parts.push("\n## System Context".to_string());
            for summary in &self.summaries {
                parts.push(format!(
                    "- {} ({} components): {}",
                    summary.common_role,
                    summary.component_count,
                    summary.description
                ));
            }
        }

        parts.join("\n\n")
    }
}

/// Priority rule for context compression
#[derive(Debug, Clone)]
pub struct PriorityRule {
    /// Name of the rule
    pub name: String,

    /// Priority score (higher = more important to include)
    pub priority: i32,

    /// Condition for this rule
    pub condition: PriorityCondition,
}

/// Conditions for priority rules
#[derive(Debug, Clone)]
pub enum PriorityCondition {
    /// Context is at a specific level
    Level(ContextLevel),

    /// Context has a specific role category
    RoleCategory(String),

    /// Context is directly related to focus
    DirectlyRelated,

    /// Context is an ancestor of focus
    Ancestor,

    /// Context has critical invariants
    HasCriticalInvariants,

    /// Context has high relationship strength
    StrongRelationship(f64),
}

/// Compresses context to fit within token budget while preserving essence
pub struct ContextCompressor {
    /// Priority rules for what to keep
    priority_rules: Vec<PriorityRule>,

    /// Default token budget
    default_budget: usize,
}

impl Default for ContextCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextCompressor {
    pub fn new() -> Self {
        Self {
            priority_rules: Self::default_rules(),
            default_budget: 4000, // ~16K chars, reasonable for most LLMs
        }
    }

    pub fn with_budget(mut self, budget: usize) -> Self {
        self.default_budget = budget;
        self
    }

    pub fn with_rules(mut self, rules: Vec<PriorityRule>) -> Self {
        self.priority_rules = rules;
        self
    }

    fn default_rules() -> Vec<PriorityRule> {
        vec![
            PriorityRule {
                name: "focus_context".into(),
                priority: 100,
                condition: PriorityCondition::DirectlyRelated,
            },
            PriorityRule {
                name: "ancestors".into(),
                priority: 80,
                condition: PriorityCondition::Ancestor,
            },
            PriorityRule {
                name: "critical_invariants".into(),
                priority: 70,
                condition: PriorityCondition::HasCriticalInvariants,
            },
            PriorityRule {
                name: "strong_relationships".into(),
                priority: 60,
                condition: PriorityCondition::StrongRelationship(0.7),
            },
            PriorityRule {
                name: "system_level".into(),
                priority: 50,
                condition: PriorityCondition::Level(ContextLevel::System),
            },
            PriorityRule {
                name: "domain_level".into(),
                priority: 40,
                condition: PriorityCondition::Level(ContextLevel::Domain),
            },
        ]
    }

    /// Compress a set of contexts to fit budget
    pub fn compress(
        &self,
        contexts: &[SemanticContext],
        focus: &ContextId,
        budget: Option<usize>,
    ) -> CompressedContext {
        let budget = budget.unwrap_or(self.default_budget);
        let mut result = CompressedContext::new(budget);

        // Build lookup maps
        let context_map: HashMap<&ContextId, &SemanticContext> = contexts.iter()
            .map(|c| (&c.id, c))
            .collect();

        // Find focus context
        let focus_ctx = match context_map.get(focus) {
            Some(ctx) => *ctx,
            None => return result, // Focus not found, return empty
        };

        // Phase 1: Always include focus context with full detail
        result.add_full(focus_ctx);

        // Phase 2: Include ancestors (system → domain → module chain)
        let ancestors = self.get_ancestors(contexts, focus);
        for ancestor in ancestors {
            if !result.add_essence(ancestor) {
                break; // Out of budget
            }
        }

        // Phase 3: Include strongly related contexts
        let related = self.get_related(contexts, focus_ctx, 0.7);
        for rel_ctx in related {
            if rel_ctx.id != *focus {
                if !result.add_essence(rel_ctx) {
                    break;
                }
            }
        }

        // Phase 4: Summarize remaining contexts by role
        let remaining: Vec<_> = contexts.iter()
            .filter(|c| {
                c.id != *focus &&
                !result.full_contexts.iter().any(|fc| fc.id == c.id) &&
                !result.essence_only.iter().any(|es| es.id == c.id)
            })
            .collect();

        if !remaining.is_empty() {
            let summaries = self.summarize_by_role(&remaining);
            for summary in summaries {
                if !result.add_summary(summary) {
                    break;
                }
            }
        }

        result
    }

    /// Get ancestor contexts (parent chain)
    fn get_ancestors<'a>(
        &self,
        contexts: &'a [SemanticContext],
        focus: &ContextId,
    ) -> Vec<&'a SemanticContext> {
        let context_map: HashMap<&ContextId, &SemanticContext> = contexts.iter()
            .map(|c| (&c.id, c))
            .collect();

        let mut ancestors = vec![];
        let mut current = context_map.get(focus).and_then(|c| c.parent_id.as_ref());

        while let Some(parent_id) = current {
            if let Some(parent) = context_map.get(parent_id) {
                ancestors.push(*parent);
                current = parent.parent_id.as_ref();
            } else {
                break;
            }
        }

        // Reverse so system comes first
        ancestors.reverse();
        ancestors
    }

    /// Get related contexts by relationship strength
    fn get_related<'a>(
        &self,
        contexts: &'a [SemanticContext],
        focus: &SemanticContext,
        min_strength: f64,
    ) -> Vec<&'a SemanticContext> {
        let context_map: HashMap<&ContextId, &SemanticContext> = contexts.iter()
            .map(|c| (&c.id, c))
            .collect();

        let related_ids: HashSet<_> = focus.relationships.iter()
            .filter(|r| r.strength >= min_strength)
            .map(|r| &r.target)
            .collect();

        contexts.iter()
            .filter(|c| related_ids.contains(&c.id))
            .collect()
    }

    /// Summarize contexts by role category
    fn summarize_by_role(&self, contexts: &[&SemanticContext]) -> Vec<GroupSummary> {
        let mut by_role: HashMap<String, Vec<&SemanticContext>> = HashMap::new();

        for ctx in contexts {
            let role = ctx.essence.role.category().to_string();
            by_role.entry(role).or_default().push(ctx);
        }

        by_role.into_iter()
            .map(|(role, ctxs)| {
                let purposes: Vec<_> = ctxs.iter()
                    .take(3) // Sample up to 3 for description
                    .map(|c| c.essence.purpose.as_str())
                    .collect();

                let description = if ctxs.len() <= 3 {
                    purposes.join("; ")
                } else {
                    format!("{} (and {} more)", purposes.join("; "), ctxs.len() - 3)
                };

                GroupSummary {
                    description,
                    component_count: ctxs.len(),
                    component_ids: ctxs.iter().map(|c| c.id.clone()).collect(),
                    common_role: role,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::essence::ComponentRole;

    fn create_test_context(name: &str, level: ContextLevel, purpose: &str) -> SemanticContext {
        let id = ContextId::from_name(name, level);
        let essence = Essence::new(purpose, ComponentRole::core("test"));
        SemanticContext::new(id, level, essence)
    }

    #[test]
    fn test_compression_includes_focus() {
        let compressor = ContextCompressor::new();

        let focus = create_test_context("focus", ContextLevel::Module, "The focus component");
        let other = create_test_context("other", ContextLevel::Module, "Another component");

        let contexts = vec![focus.clone(), other];
        let result = compressor.compress(&contexts, &focus.id, Some(1000));

        assert_eq!(result.full_contexts.len(), 1);
        assert_eq!(result.full_contexts[0].id, focus.id);
    }

    #[test]
    fn test_compression_respects_budget() {
        let compressor = ContextCompressor::new();

        let contexts: Vec<_> = (0..100)
            .map(|i| create_test_context(
                &format!("ctx_{}", i),
                ContextLevel::Module,
                &format!("Component {} with some description text", i),
            ))
            .collect();

        let focus_id = contexts[0].id.clone();
        let result = compressor.compress(&contexts, &focus_id, Some(500));

        assert!(result.token_count() <= 500);
    }

    #[test]
    fn test_prompt_generation() {
        let compressor = ContextCompressor::new();

        let mut system = create_test_context("system", ContextLevel::System, "E-commerce platform");
        let mut domain = create_test_context("payments", ContextLevel::Domain, "Payment processing");
        domain.parent_id = Some(system.id.clone());
        system.children.push(domain.id.clone());

        let mut focus = create_test_context("charge", ContextLevel::Module, "Handles charging");
        focus.parent_id = Some(domain.id.clone());

        let contexts = vec![system, domain, focus.clone()];
        let result = compressor.compress(&contexts, &focus.id, Some(2000));

        let prompt = result.to_prompt_string();
        assert!(prompt.contains("Handles charging"));
        assert!(prompt.contains("Primary Components"));
    }
}
