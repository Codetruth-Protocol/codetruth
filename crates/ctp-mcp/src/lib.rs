//! # CodeTruth MCP Server
//!
//! Implements the Model Context Protocol (MCP) for AI-native integration
//! with Claude Code and other AI coding assistants.
//!
//! ## Architecture
//!
//! - Uses stdio transport for minimal latency
//! - On-demand analysis (no background processes)
//! - Result caching for repeated queries
//! - Lazy model loading for efficiency

pub mod error;
pub mod metrics;
pub mod security;
pub mod server;
pub mod tools;
pub mod cache;
pub mod models;

pub use server::CodeTruthMCPServer;
pub use models::*;

/// MCP Protocol version compliance
pub const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

/// Server capabilities
pub const SERVER_NAME: &str = "codetruth";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
