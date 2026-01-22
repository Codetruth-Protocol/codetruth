//! Essence - the irreducible core meaning of a component
//!
//! Like "hand grips objects" - this is the fundamental purpose that
//! MUST be preserved during any summarization or compression.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

use crate::context::ContextId;

/// The irreducible core meaning - like "hand grips objects"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Essence {
    /// One-sentence purpose (MUST fit in 100 chars)
    pub purpose: String,

    /// Role in the system (what would break if this disappeared?)
    pub role: ComponentRole,

    /// Critical constraints (things that MUST remain true)
    pub constraints: Vec<String>,

    /// Keywords for semantic matching
    pub keywords: Vec<String>,
}

impl Essence {
    /// Create a new essence with purpose and role
    pub fn new(purpose: &str, role: ComponentRole) -> Self {
        // Enforce 100 char limit on purpose - but preserve meaning
        let purpose = if purpose.len() > 100 {
            Self::smart_truncate(purpose, 100)
        } else {
            purpose.to_string()
        };

        Self {
            purpose,
            role,
            constraints: vec![],
            keywords: vec![],
        }
    }

    /// Smart truncation that preserves meaning
    fn smart_truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            return s.to_string();
        }

        // Reserve 3 chars for "..."
        let target_len = max_len.saturating_sub(3);
        
        // Try to break at word boundary
        if let Some(last_space) = s[..target_len].rfind(' ') {
            if last_space > target_len / 2 {
                return format!("{}...", &s[..last_space]);
            }
        }

        // Hard truncate if no good word boundary
        format!("{}...", &s[..target_len])
    }

    /// Add a constraint
    pub fn with_constraint(mut self, constraint: &str) -> Self {
        self.constraints.push(constraint.to_string());
        self
    }

    /// Add keywords for semantic matching
    pub fn with_keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords.extend(keywords.iter().map(|s| s.to_string()));
        self
    }

    /// Compute a hash of the essence for change detection
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.purpose.as_bytes());
        hasher.update(format!("{:?}", self.role).as_bytes());
        for constraint in &self.constraints {
            hasher.update(constraint.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Estimate token count
    pub fn token_count(&self) -> usize {
        let purpose_tokens = (self.purpose.len() + 3) / 4;
        let constraint_tokens: usize = self.constraints.iter()
            .map(|c| (c.len() + 3) / 4)
            .sum();
        purpose_tokens + constraint_tokens + 10 // overhead for role
    }

    /// Check semantic similarity with another essence
    pub fn similarity(&self, other: &Essence) -> f64 {
        // Simple keyword overlap for now
        // TODO: Use embeddings for better semantic matching
        let self_lower = self.purpose.to_lowercase();
        let other_lower = other.purpose.to_lowercase();
        
        let self_words: std::collections::HashSet<String> = self_lower
            .split_whitespace()
            .map(|s| s.to_string())
            .chain(self.keywords.iter().cloned())
            .collect();

        let other_words: std::collections::HashSet<String> = other_lower
            .split_whitespace()
            .map(|s| s.to_string())
            .chain(other.keywords.iter().cloned())
            .collect();

        let intersection = self_words.intersection(&other_words).count();
        let union = self_words.union(&other_words).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

/// Role of a component in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ComponentRole {
    /// System-level orchestrator
    System,

    /// Entry point - receives external input
    Boundary {
        direction: BoundaryDirection,
    },

    /// Core logic - implements business rules
    Core {
        domain: String,
    },

    /// Utility - supports other components
    Utility {
        used_by: Vec<ContextId>,
    },

    /// Orchestrator - coordinates other components
    Orchestrator {
        coordinates: Vec<ContextId>,
    },

    /// Data - holds/transforms data
    Data {
        schema_hash: Option<String>,
    },

    /// Test - verifies other components
    Test {
        tests: Vec<ContextId>,
    },

    /// Configuration - system settings
    Config,
}

impl ComponentRole {
    pub fn system() -> Self {
        Self::System
    }

    pub fn boundary_in() -> Self {
        Self::Boundary { direction: BoundaryDirection::Inbound }
    }

    pub fn boundary_out() -> Self {
        Self::Boundary { direction: BoundaryDirection::Outbound }
    }

    pub fn core(domain: &str) -> Self {
        Self::Core { domain: domain.to_string() }
    }

    pub fn utility() -> Self {
        Self::Utility { used_by: vec![] }
    }

    pub fn orchestrator() -> Self {
        Self::Orchestrator { coordinates: vec![] }
    }

    pub fn data() -> Self {
        Self::Data { schema_hash: None }
    }

    pub fn test() -> Self {
        Self::Test { tests: vec![] }
    }

    /// Get a human-readable category name
    pub fn category(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Boundary { .. } => "boundary",
            Self::Core { .. } => "core",
            Self::Utility { .. } => "utility",
            Self::Orchestrator { .. } => "orchestrator",
            Self::Data { .. } => "data",
            Self::Test { .. } => "test",
            Self::Config => "config",
        }
    }

    /// Check if this role is critical (system would break without it)
    pub fn is_critical(&self) -> bool {
        matches!(self, Self::System | Self::Core { .. } | Self::Boundary { .. })
    }
}

/// Direction of a boundary component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryDirection {
    /// Receives input from outside (API endpoints, CLI, etc.)
    Inbound,
    /// Sends output to outside (external APIs, databases, etc.)
    Outbound,
    /// Both directions (websockets, bidirectional streams)
    Bidirectional,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_essence_creation() {
        let essence = Essence::new(
            "Handles payment retry with idempotency",
            ComponentRole::core("payments"),
        );

        assert!(essence.purpose.len() <= 100);
        assert!(essence.token_count() > 0);
    }

    #[test]
    fn test_smart_truncate() {
        let long_text = "This is a very long description that exceeds the maximum allowed length for an essence purpose field and continues with even more text to ensure it definitely gets truncated";
        let essence = Essence::new(long_text, ComponentRole::utility());

        // Should be truncated to 100 chars max (including "...")
        assert!(essence.purpose.len() <= 100, "Purpose length {} exceeds 100", essence.purpose.len());
        assert!(essence.purpose.ends_with("..."), "Purpose doesn't end with '...': {}", essence.purpose);
        // Verify it's actually truncated
        assert!(essence.purpose.len() < long_text.len());
    }

    #[test]
    fn test_essence_similarity() {
        let e1 = Essence::new("Handles payment processing", ComponentRole::core("payments"))
            .with_keywords(&["payment", "charge", "transaction"]);

        let e2 = Essence::new("Processes payment transactions", ComponentRole::core("payments"))
            .with_keywords(&["payment", "process", "transaction"]);

        let e3 = Essence::new("Manages user authentication", ComponentRole::core("auth"))
            .with_keywords(&["auth", "login", "user"]);

        let sim_12 = e1.similarity(&e2);
        let sim_13 = e1.similarity(&e3);

        assert!(sim_12 > sim_13, "Similar essences should have higher similarity");
    }

    #[test]
    fn test_role_criticality() {
        assert!(ComponentRole::system().is_critical());
        assert!(ComponentRole::core("payments").is_critical());
        assert!(ComponentRole::boundary_in().is_critical());
        assert!(!ComponentRole::utility().is_critical());
        assert!(!ComponentRole::test().is_critical());
    }
}
