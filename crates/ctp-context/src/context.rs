//! Core semantic context types
//!
//! The `SemanticContext` is the fundamental unit of the hierarchical context system.
//! It represents a code component at any level (system, domain, module, function)
//! with its essence, relationships, and invariants.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::essence::Essence;
use crate::relationship::Relationship;

/// Unique identifier for a context node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextId(pub String);

impl ContextId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_path(path: &str) -> Self {
        Self(format!("path:{}", path))
    }

    pub fn from_name(name: &str, level: ContextLevel) -> Self {
        Self(format!("{}:{}", level.as_str(), name))
    }
}

impl Default for ContextId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ContextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Level in the context hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextLevel {
    /// Entire codebase - highest level
    System,
    /// Major subsystem (payments, auth, inventory)
    Domain,
    /// Single file or module
    Module,
    /// Individual function or method
    Function,
    /// Code block within a function
    Block,
}

impl ContextLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Domain => "domain",
            Self::Module => "module",
            Self::Function => "function",
            Self::Block => "block",
        }
    }

    /// Get the parent level (None for System)
    pub fn parent(&self) -> Option<Self> {
        match self {
            Self::System => None,
            Self::Domain => Some(Self::System),
            Self::Module => Some(Self::Domain),
            Self::Function => Some(Self::Module),
            Self::Block => Some(Self::Function),
        }
    }

    /// Get the child level (None for Block)
    pub fn child(&self) -> Option<Self> {
        match self {
            Self::System => Some(Self::Domain),
            Self::Domain => Some(Self::Module),
            Self::Module => Some(Self::Function),
            Self::Function => Some(Self::Block),
            Self::Block => None,
        }
    }
}

/// Invariant that must be preserved across changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    /// Unique identifier for this invariant
    pub id: String,

    /// Human-readable description
    pub description: String,

    /// How critical is this invariant?
    pub severity: InvariantSeverity,

    /// Code patterns that indicate violation
    pub violation_patterns: Vec<String>,

    /// Example of correct usage
    pub correct_example: Option<String>,
}

impl Invariant {
    pub fn critical(id: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            severity: InvariantSeverity::Critical,
            violation_patterns: vec![],
            correct_example: None,
        }
    }

    pub fn with_violation_pattern(mut self, pattern: &str) -> Self {
        self.violation_patterns.push(pattern.to_string());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvariantSeverity {
    /// System will crash or corrupt data
    Critical,
    /// Feature will malfunction
    High,
    /// Degraded behavior
    Medium,
    /// Style/convention violation
    Low,
}

/// Detail level for expanded context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailLevel {
    /// Full description (can be multiple paragraphs)
    pub full_description: String,

    /// Implementation notes
    pub implementation_notes: Vec<String>,

    /// Known limitations or edge cases
    pub limitations: Vec<String>,

    /// Usage examples
    pub examples: Vec<String>,

    /// Token count for this detail level
    pub token_count: usize,
}

impl DetailLevel {
    pub fn new(description: &str) -> Self {
        let token_count = estimate_tokens(description);
        Self {
            full_description: description.to_string(),
            implementation_notes: vec![],
            limitations: vec![],
            examples: vec![],
            token_count,
        }
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.implementation_notes.push(note.to_string());
        self.token_count += estimate_tokens(note);
        self
    }
}

/// Hierarchical context that can be compressed without losing essence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    /// Unique identifier for this context node
    pub id: ContextId,

    /// Level in the hierarchy
    pub level: ContextLevel,

    /// The "essence" - irreducible core meaning (NEVER truncated)
    pub essence: Essence,

    /// Expanded detail (can be compressed/expanded as needed)
    pub detail: Option<DetailLevel>,

    /// Relationships to other contexts
    pub relationships: Vec<Relationship>,

    /// Invariants that must be preserved
    pub invariants: Vec<Invariant>,

    /// Parent context ID (None for System level)
    pub parent_id: Option<ContextId>,

    /// Child context IDs
    pub children: Vec<ContextId>,

    /// Hash for change detection
    pub content_hash: String,

    /// When this context was created
    pub created_at: DateTime<Utc>,

    /// When this context was last updated
    pub updated_at: DateTime<Utc>,
}

impl SemanticContext {
    /// Create a new context with minimal required fields
    pub fn new(id: ContextId, level: ContextLevel, essence: Essence) -> Self {
        let now = Utc::now();
        let content_hash = essence.compute_hash();

        Self {
            id,
            level,
            essence,
            detail: None,
            relationships: vec![],
            invariants: vec![],
            parent_id: None,
            children: vec![],
            content_hash,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a system-level context
    pub fn system(name: &str, purpose: &str) -> Self {
        let id = ContextId::from_name(name, ContextLevel::System);
        let essence = Essence::new(purpose, crate::essence::ComponentRole::system());
        Self::new(id, ContextLevel::System, essence)
    }

    /// Create a domain-level context
    pub fn domain(name: &str, purpose: &str, parent: &ContextId) -> Self {
        let id = ContextId::from_name(name, ContextLevel::Domain);
        let essence = Essence::new(purpose, crate::essence::ComponentRole::core(name));
        let mut ctx = Self::new(id, ContextLevel::Domain, essence);
        ctx.parent_id = Some(parent.clone());
        ctx
    }

    /// Create a module-level context from a file path
    pub fn module(path: &str, purpose: &str, parent: &ContextId) -> Self {
        let id = ContextId::from_path(path);
        let essence = Essence::new(purpose, crate::essence::ComponentRole::core("module"));
        let mut ctx = Self::new(id, ContextLevel::Module, essence);
        ctx.parent_id = Some(parent.clone());
        ctx
    }

    /// Add a relationship to another context
    pub fn add_relationship(&mut self, relationship: Relationship) {
        self.relationships.push(relationship);
        self.updated_at = Utc::now();
    }

    /// Add an invariant
    pub fn add_invariant(&mut self, invariant: Invariant) {
        self.invariants.push(invariant);
        self.updated_at = Utc::now();
    }

    /// Add a child context
    pub fn add_child(&mut self, child_id: ContextId) {
        self.children.push(child_id);
        self.updated_at = Utc::now();
    }

    /// Set detail level
    pub fn with_detail(mut self, detail: DetailLevel) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Get total token count (essence + detail if present)
    pub fn token_count(&self) -> usize {
        let essence_tokens = self.essence.token_count();
        let detail_tokens = self.detail.as_ref().map(|d| d.token_count).unwrap_or(0);
        let invariant_tokens: usize = self.invariants.iter()
            .map(|i| estimate_tokens(&i.description))
            .sum();

        essence_tokens + detail_tokens + invariant_tokens
    }

    /// Check if this context has changed since last hash
    pub fn has_changed(&self, new_content: &str) -> bool {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(new_content.as_bytes());
        let new_hash = format!("{:x}", hasher.finalize());
        self.content_hash != new_hash
    }

    /// Get strongly related contexts (strength > threshold)
    pub fn strong_relationships(&self, threshold: f64) -> Vec<&Relationship> {
        self.relationships.iter()
            .filter(|r| r.strength > threshold)
            .collect()
    }
}

/// Estimate token count for a string (rough approximation: ~4 chars per token)
fn estimate_tokens(s: &str) -> usize {
    (s.len() + 3) / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_level_hierarchy() {
        assert_eq!(ContextLevel::System.parent(), None);
        assert_eq!(ContextLevel::Domain.parent(), Some(ContextLevel::System));
        assert_eq!(ContextLevel::Module.parent(), Some(ContextLevel::Domain));

        assert_eq!(ContextLevel::System.child(), Some(ContextLevel::Domain));
        assert_eq!(ContextLevel::Block.child(), None);
    }

    #[test]
    fn test_context_creation() {
        let ctx = SemanticContext::system("my-app", "E-commerce platform");
        assert_eq!(ctx.level, ContextLevel::System);
        assert!(ctx.parent_id.is_none());
        assert!(ctx.token_count() > 0);
    }

    #[test]
    fn test_context_id_formats() {
        let path_id = ContextId::from_path("src/main.rs");
        assert!(path_id.0.starts_with("path:"));

        let name_id = ContextId::from_name("payments", ContextLevel::Domain);
        assert!(name_id.0.starts_with("domain:"));
    }
}
