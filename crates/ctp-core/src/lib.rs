//! # CTP Core
//!
//! Core analysis engine for CodeTruth Protocol.
//!
//! This crate provides the main `CodeTruthEngine` that orchestrates:
//! - AST parsing via `ctp-parser`
//! - Drift detection via `ctp-drift`
//! - Policy evaluation via `ctp-policy`
//! - Optional LLM enhancement via `ctp-llm`

pub mod engine;
pub mod models;
pub mod error;
pub mod context_bridge;

pub use engine::{CodeTruthEngine, EngineConfig};
pub use models::DriftSeverity;
pub use models::*;
pub use error::CTPError;

/// CTP Protocol version
pub const CTP_VERSION: &str = "1.0.0";

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
