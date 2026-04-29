//! Error types for MCP server
//!
//! Provides structured error handling with specific error categories
//! for production-grade observability and debugging.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for MCP operations
#[derive(Error, Debug)]
pub enum MCPError {
    /// File not found at specified path
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),

    /// Permission denied when accessing file
    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),

    /// Path is invalid or contains traversal attempts
    #[error("invalid path: {0}")]
    InvalidPath(String),

    /// Analysis engine failed
    #[error("analysis failed: {0}")]
    AnalysisFailed(String),

    /// Invalid input parameters
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Operation was cancelled
    #[error("operation cancelled")]
    Cancelled,

    /// Timeout exceeded
    #[error("operation timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),

    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl MCPError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            MCPError::Timeout(_) | MCPError::Io(_) | MCPError::AnalysisFailed(_)
        )
    }

    /// Get error category for metrics/logging
    pub fn category(&self) -> &'static str {
        match self {
            MCPError::FileNotFound(_) => "file_not_found",
            MCPError::PermissionDenied(_) => "permission_denied",
            MCPError::InvalidPath(_) => "invalid_path",
            MCPError::AnalysisFailed(_) => "analysis_failed",
            MCPError::InvalidInput(_) => "invalid_input",
            MCPError::Cancelled => "cancelled",
            MCPError::Timeout(_) => "timeout",
            MCPError::Internal(_) => "internal",
            MCPError::Serialization(_) => "serialization",
            MCPError::Io(_) => "io",
        }
    }

    /// Get HTTP-like status code for error
    pub fn status_code(&self) -> u16 {
        match self {
            MCPError::FileNotFound(_) => 404,
            MCPError::PermissionDenied(_) => 403,
            MCPError::InvalidPath(_) => 400,
            MCPError::InvalidInput(_) => 400,
            MCPError::Cancelled => 499, // Client Closed Request
            MCPError::Timeout(_) => 408,
            MCPError::AnalysisFailed(_) => 422,
            MCPError::Serialization(_) => 400,
            MCPError::Io(_) => 500,
            MCPError::Internal(_) => 500,
        }
    }
}

/// Result type alias for MCP operations
pub type MCPResult<T> = Result<T, MCPError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let err = MCPError::FileNotFound(PathBuf::from("/test"));
        assert_eq!(err.category(), "file_not_found");
        assert!(!err.is_retryable());

        let err = MCPError::Timeout(std::time::Duration::from_secs(30));
        assert_eq!(err.category(), "timeout");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(MCPError::FileNotFound(PathBuf::from("/test")).status_code(), 404);
        assert_eq!(MCPError::PermissionDenied(PathBuf::from("/test")).status_code(), 403);
        assert_eq!(MCPError::InvalidInput("bad".into()).status_code(), 400);
        assert_eq!(MCPError::Internal("oops".into()).status_code(), 500);
    }
}
