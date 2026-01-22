//! # CTP Context
//!
//! Hierarchical semantic context management for CodeTruth Protocol.
//!
//! This crate provides the foundation for handling large codebases without
//! losing essential information during summarization. It implements the
//! "Anatomy Model" - like documenting body parts for a doctor, ensuring:
//!
//! - **Essence is preserved**: Core purpose is never truncated
//! - **Redundancy is detected**: No duplicate components doing the same work
//! - **Hierarchy enables compression**: System → Domain → Module → Function
//! - **Relationships are explicit**: Components know what they connect to

pub mod context;
pub mod essence;
pub mod registry;
pub mod compression;
pub mod relationship;
pub mod error;
pub mod integration;

pub use context::{SemanticContext, ContextId, ContextLevel, NamingPatternMetadata};
pub use essence::{Essence, ComponentRole, BoundaryDirection};
pub use registry::{ComponentRegistry, RegistryEntry, RedundancyReport};
pub use compression::{ContextCompressor, CompressedContext, PriorityRule};
pub use relationship::{Relationship, RelationshipType};
pub use error::ContextError;

/// CTP Context version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
