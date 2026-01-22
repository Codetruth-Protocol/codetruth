//! Component Registry - prevents redundancy
//!
//! The registry tracks all known components and detects when new code
//! duplicates existing functionality ("adding new hands to the body").

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ContextId;
use crate::essence::Essence;
use crate::error::ContextError;

/// Semantic hash for similarity indexing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticHash(pub String);

/// Entry in the component registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Context ID
    pub id: ContextId,

    /// The essence of this component
    pub essence: Essence,

    /// Semantic hash for similarity matching
    pub hash: SemanticHash,

    /// When this entry was registered
    pub registered_at: DateTime<Utc>,

    /// How many times this component has been analyzed
    pub analysis_count: usize,

    /// Last analysis timestamp
    pub last_analyzed: DateTime<Utc>,
}

/// Result of redundancy check
#[derive(Debug, Clone)]
pub enum RedundancyReport {
    /// Component is unique, no duplicates found
    Unique,

    /// Component duplicates an existing one
    Duplicate {
        existing: ContextId,
        similarity: f64,
        recommendation: String,
    },

    /// Component is related but not duplicate
    Related {
        similar: Vec<SimilarComponent>,
    },
}

/// A component similar to the one being checked
#[derive(Debug, Clone)]
pub struct SimilarComponent {
    pub id: ContextId,
    pub similarity: f64,
    pub purpose: String,
}

/// Registry of all components - prevents "adding new hands"
#[derive(Debug, Default)]
pub struct ComponentRegistry {
    /// All known components indexed by ID
    components: HashMap<ContextId, RegistryEntry>,

    /// Semantic hash index for fast similarity lookup
    hash_index: HashMap<SemanticHash, ContextId>,

    /// Similarity threshold for duplicate detection
    duplicate_threshold: f64,

    /// Similarity threshold for "related" detection
    related_threshold: f64,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            hash_index: HashMap::new(),
            duplicate_threshold: 0.85,
            related_threshold: 0.5,
        }
    }

    pub fn with_thresholds(duplicate: f64, related: f64) -> Self {
        Self {
            components: HashMap::new(),
            hash_index: HashMap::new(),
            duplicate_threshold: duplicate,
            related_threshold: related,
        }
    }

    /// Check if new code duplicates existing functionality
    pub fn check_redundancy(&self, new_essence: &Essence) -> RedundancyReport {
        let mut similar_components = vec![];

        for entry in self.components.values() {
            let similarity = new_essence.similarity(&entry.essence);

            if similarity >= self.duplicate_threshold {
                return RedundancyReport::Duplicate {
                    existing: entry.id.clone(),
                    similarity,
                    recommendation: format!(
                        "Consider using existing component '{}' instead of creating a duplicate",
                        entry.essence.purpose
                    ),
                };
            }

            if similarity >= self.related_threshold {
                similar_components.push(SimilarComponent {
                    id: entry.id.clone(),
                    similarity,
                    purpose: entry.essence.purpose.clone(),
                });
            }
        }

        if similar_components.is_empty() {
            RedundancyReport::Unique
        } else {
            // Sort by similarity descending
            similar_components.sort_by(|a, b| {
                b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal)
            });
            RedundancyReport::Related { similar: similar_components }
        }
    }

    /// Register a new component
    pub fn register(&mut self, id: ContextId, essence: Essence) -> Result<(), ContextError> {
        // Check for redundancy first
        let redundancy = self.check_redundancy(&essence);

        if let RedundancyReport::Duplicate { existing, similarity, .. } = redundancy {
            return Err(ContextError::RedundantComponent {
                new_id: id,
                existing_id: existing,
                similarity,
            });
        }

        let hash = SemanticHash(essence.compute_hash());
        let now = Utc::now();

        let entry = RegistryEntry {
            id: id.clone(),
            essence,
            hash: hash.clone(),
            registered_at: now,
            analysis_count: 1,
            last_analyzed: now,
        };

        self.hash_index.insert(hash, id.clone());
        self.components.insert(id, entry);

        Ok(())
    }

    /// Update an existing component (re-analysis)
    pub fn update(&mut self, id: &ContextId, essence: Essence) -> Result<(), ContextError> {
        if let Some(entry) = self.components.get_mut(id) {
            // Remove old hash
            self.hash_index.remove(&entry.hash);

            // Update entry
            let new_hash = SemanticHash(essence.compute_hash());
            entry.essence = essence;
            entry.hash = new_hash.clone();
            entry.analysis_count += 1;
            entry.last_analyzed = Utc::now();

            // Add new hash
            self.hash_index.insert(new_hash, id.clone());

            Ok(())
        } else {
            Err(ContextError::ComponentNotFound(id.clone()))
        }
    }

    /// Get a component by ID
    pub fn get(&self, id: &ContextId) -> Option<&RegistryEntry> {
        self.components.get(id)
    }

    /// Get all components
    pub fn all(&self) -> impl Iterator<Item = &RegistryEntry> {
        self.components.values()
    }

    /// Get component count
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Find components by role category
    pub fn find_by_role(&self, role_category: &str) -> Vec<&RegistryEntry> {
        self.components.values()
            .filter(|e| e.essence.role.category() == role_category)
            .collect()
    }

    /// Find components matching keywords
    pub fn find_by_keywords(&self, keywords: &[&str]) -> Vec<&RegistryEntry> {
        self.components.values()
            .filter(|e| {
                keywords.iter().any(|kw| {
                    e.essence.purpose.to_lowercase().contains(&kw.to_lowercase()) ||
                    e.essence.keywords.iter().any(|k| k.to_lowercase() == kw.to_lowercase())
                })
            })
            .collect()
    }

    /// Get statistics about the registry
    pub fn stats(&self) -> RegistryStats {
        let mut role_counts: HashMap<&str, usize> = HashMap::new();

        for entry in self.components.values() {
            *role_counts.entry(entry.essence.role.category()).or_insert(0) += 1;
        }

        RegistryStats {
            total_components: self.components.len(),
            role_distribution: role_counts.into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }

    /// Export registry to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let entries: Vec<&RegistryEntry> = self.components.values().collect();
        serde_json::to_string_pretty(&entries)
    }

    /// Import registry from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let entries: Vec<RegistryEntry> = serde_json::from_str(json)?;
        let mut registry = Self::new();

        for entry in entries {
            registry.hash_index.insert(entry.hash.clone(), entry.id.clone());
            registry.components.insert(entry.id.clone(), entry);
        }

        Ok(registry)
    }
}

/// Statistics about the registry
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_components: usize,
    pub role_distribution: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::essence::ComponentRole;

    #[test]
    fn test_registry_registration() {
        let mut registry = ComponentRegistry::new();

        let id = ContextId::from_path("src/payments/charge.rs");
        let essence = Essence::new("Handles payment charging", ComponentRole::core("payments"));

        assert!(registry.register(id.clone(), essence).is_ok());
        assert_eq!(registry.len(), 1);
        assert!(registry.get(&id).is_some());
    }

    #[test]
    fn test_duplicate_detection() {
        let mut registry = ComponentRegistry::new();

        let id1 = ContextId::from_path("src/payments/charge.rs");
        let essence1 = Essence::new("Handles payment charging", ComponentRole::core("payments"))
            .with_keywords(&["payment", "charge", "transaction"]);

        registry.register(id1, essence1).unwrap();

        // Try to register a duplicate
        let id2 = ContextId::from_path("src/billing/charge.rs");
        let essence2 = Essence::new("Handles payment charging logic", ComponentRole::core("billing"))
            .with_keywords(&["payment", "charge", "billing"]);

        let result = registry.check_redundancy(&essence2);
        assert!(matches!(result, RedundancyReport::Duplicate { .. } | RedundancyReport::Related { .. }));
    }

    #[test]
    fn test_unique_component() {
        let mut registry = ComponentRegistry::new();

        let id1 = ContextId::from_path("src/payments/charge.rs");
        let essence1 = Essence::new("Handles payment charging", ComponentRole::core("payments"));
        registry.register(id1, essence1).unwrap();

        // Check a completely different component
        let essence2 = Essence::new("Manages user authentication", ComponentRole::core("auth"));
        let result = registry.check_redundancy(&essence2);

        assert!(matches!(result, RedundancyReport::Unique));
    }

    #[test]
    fn test_find_by_role() {
        let mut registry = ComponentRegistry::new();

        registry.register(
            ContextId::from_path("src/api/handler.rs"),
            Essence::new("API endpoint handler", ComponentRole::boundary_in()),
        ).unwrap();

        registry.register(
            ContextId::from_path("src/core/logic.rs"),
            Essence::new("Core business logic", ComponentRole::core("business")),
        ).unwrap();

        let boundaries = registry.find_by_role("boundary");
        assert_eq!(boundaries.len(), 1);

        let cores = registry.find_by_role("core");
        assert_eq!(cores.len(), 1);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut registry = ComponentRegistry::new();

        registry.register(
            ContextId::from_path("src/test.rs"),
            Essence::new("Test component", ComponentRole::utility()),
        ).unwrap();

        let json = registry.to_json().unwrap();
        let restored = ComponentRegistry::from_json(&json).unwrap();

        assert_eq!(restored.len(), 1);
    }
}
