//! Error types for ctp-context

use thiserror::Error;

use crate::context::ContextId;

/// Errors that can occur in context operations
#[derive(Debug, Error)]
pub enum ContextError {
    /// Attempted to register a component that duplicates existing functionality
    #[error("Redundant component: '{new_id}' duplicates '{existing_id}' (similarity: {similarity:.2})")]
    RedundantComponent {
        new_id: ContextId,
        existing_id: ContextId,
        similarity: f64,
    },

    /// Component not found in registry
    #[error("Component not found: {0}")]
    ComponentNotFound(ContextId),

    /// Invalid context hierarchy
    #[error("Invalid hierarchy: {0}")]
    InvalidHierarchy(String),

    /// Circular dependency detected
    #[error("Circular dependency detected: {0:?}")]
    CircularDependency(Vec<ContextId>),

    /// Token budget exceeded
    #[error("Token budget exceeded: {used} > {budget}")]
    BudgetExceeded { used: usize, budget: usize },

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Invalid essence (e.g., empty purpose)
    #[error("Invalid essence: {0}")]
    InvalidEssence(String),
}

/// Result type for context operations
pub type ContextResult<T> = Result<T, ContextError>;
