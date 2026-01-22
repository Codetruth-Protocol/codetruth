//! Relationships between components
//!
//! Explicit tracking of how components relate to each other.
//! This prevents "hand doing leg work" by making dependencies visible.

use serde::{Deserialize, Serialize};

use crate::context::ContextId;

/// Relationship between two components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Target context this relationship points to
    pub target: ContextId,

    /// Type of relationship
    pub relationship_type: RelationshipType,

    /// Strength of the relationship (0.0-1.0)
    /// Higher = more tightly coupled
    pub strength: f64,

    /// Optional description of why this relationship exists
    pub reason: Option<String>,
}

impl Relationship {
    pub fn new(target: ContextId, relationship_type: RelationshipType) -> Self {
        Self {
            target,
            relationship_type,
            strength: 0.5, // Default medium strength
            reason: None,
        }
    }

    pub fn calls(target: ContextId) -> Self {
        Self::new(target, RelationshipType::Calls)
    }

    pub fn depends_on(target: ContextId) -> Self {
        Self::new(target, RelationshipType::DependsOn)
    }

    pub fn duplicates(target: ContextId) -> Self {
        Self::new(target, RelationshipType::Duplicates)
    }

    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    /// Check if this is a strong relationship (> 0.7)
    pub fn is_strong(&self) -> bool {
        self.strength > 0.7
    }

    /// Check if this indicates potential redundancy
    pub fn indicates_redundancy(&self) -> bool {
        matches!(self.relationship_type, RelationshipType::Duplicates)
    }
}

/// Types of relationships between components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// A calls B (direct invocation)
    Calls,

    /// A depends on B's types/interfaces (import dependency)
    DependsOn,

    /// A and B serve same purpose (REDUNDANCY WARNING)
    Duplicates,

    /// A extends/implements B (inheritance/implementation)
    Extends,

    /// A and B must change together (co-change pattern)
    CoChanges,

    /// A tests B (test relationship)
    Tests,

    /// A configures B (configuration relationship)
    Configures,

    /// A is parent of B in hierarchy
    Contains,

    /// A is sibling of B (same parent)
    Sibling,
}

impl RelationshipType {
    /// Check if this relationship type implies tight coupling
    pub fn implies_tight_coupling(&self) -> bool {
        matches!(
            self,
            Self::Extends | Self::CoChanges | Self::Duplicates
        )
    }

    /// Check if this is a hierarchical relationship
    pub fn is_hierarchical(&self) -> bool {
        matches!(self, Self::Contains | Self::Sibling)
    }

    /// Get the inverse relationship type (if applicable)
    pub fn inverse(&self) -> Option<Self> {
        match self {
            Self::Calls => Some(Self::Calls), // Called by
            Self::DependsOn => None, // Depended on by (not commonly tracked)
            Self::Extends => None, // Extended by
            Self::Tests => None, // Tested by
            Self::Contains => None, // Contained by (parent relationship)
            Self::Duplicates => Some(Self::Duplicates), // Symmetric
            Self::CoChanges => Some(Self::CoChanges), // Symmetric
            Self::Sibling => Some(Self::Sibling), // Symmetric
            Self::Configures => None,
        }
    }
}

/// Builder for creating relationship graphs
#[derive(Debug, Default)]
pub struct RelationshipBuilder {
    relationships: Vec<Relationship>,
}

impl RelationshipBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_call(mut self, from: &ContextId, to: ContextId) -> Self {
        self.relationships.push(Relationship::calls(to));
        self
    }

    pub fn add_dependency(mut self, from: &ContextId, to: ContextId) -> Self {
        self.relationships.push(Relationship::depends_on(to));
        self
    }

    pub fn add_duplicate(mut self, from: &ContextId, to: ContextId, similarity: f64) -> Self {
        self.relationships.push(
            Relationship::duplicates(to)
                .with_strength(similarity)
                .with_reason("Semantic similarity detected")
        );
        self
    }

    pub fn build(self) -> Vec<Relationship> {
        self.relationships
    }
}

/// Analyze relationships to find potential issues
pub struct RelationshipAnalyzer;

impl RelationshipAnalyzer {
    /// Find circular dependencies in a set of relationships
    pub fn find_cycles(
        contexts: &[(ContextId, Vec<Relationship>)],
    ) -> Vec<Vec<ContextId>> {
        // Simple cycle detection using DFS
        // TODO: Implement proper Tarjan's algorithm for large graphs
        let mut cycles = vec![];
        let mut visited = std::collections::HashSet::new();
        let mut path = vec![];

        for (id, _) in contexts {
            if !visited.contains(id) {
                Self::dfs_cycles(id, contexts, &mut visited, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycles(
        current: &ContextId,
        contexts: &[(ContextId, Vec<Relationship>)],
        visited: &mut std::collections::HashSet<ContextId>,
        path: &mut Vec<ContextId>,
        cycles: &mut Vec<Vec<ContextId>>,
    ) {
        if path.contains(current) {
            // Found a cycle
            let cycle_start = path.iter().position(|id| id == current).unwrap();
            cycles.push(path[cycle_start..].to_vec());
            return;
        }

        if visited.contains(current) {
            return;
        }

        visited.insert(current.clone());
        path.push(current.clone());

        // Find relationships for current context
        if let Some((_, rels)) = contexts.iter().find(|(id, _)| id == current) {
            for rel in rels {
                if matches!(rel.relationship_type, RelationshipType::Calls | RelationshipType::DependsOn) {
                    Self::dfs_cycles(&rel.target, contexts, visited, path, cycles);
                }
            }
        }

        path.pop();
    }

    /// Find all duplicate relationships (potential redundancy)
    pub fn find_duplicates(
        contexts: &[(ContextId, Vec<Relationship>)],
    ) -> Vec<(ContextId, ContextId, f64)> {
        let mut duplicates = vec![];

        for (id, rels) in contexts {
            for rel in rels {
                if rel.indicates_redundancy() {
                    duplicates.push((id.clone(), rel.target.clone(), rel.strength));
                }
            }
        }

        duplicates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_creation() {
        let target = ContextId::from_path("src/utils.rs");
        let rel = Relationship::calls(target.clone())
            .with_strength(0.9)
            .with_reason("Direct function call");

        assert!(rel.is_strong());
        assert_eq!(rel.target, target);
    }

    #[test]
    fn test_relationship_types() {
        assert!(RelationshipType::Extends.implies_tight_coupling());
        assert!(RelationshipType::Contains.is_hierarchical());
        assert!(!RelationshipType::Calls.implies_tight_coupling());
    }

    #[test]
    fn test_duplicate_detection() {
        let id1 = ContextId::from_path("src/a.rs");
        let id2 = ContextId::from_path("src/b.rs");

        let contexts = vec![
            (id1.clone(), vec![Relationship::duplicates(id2.clone()).with_strength(0.85)]),
            (id2.clone(), vec![]),
        ];

        let duplicates = RelationshipAnalyzer::find_duplicates(&contexts);
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].2, 0.85);
    }
}
