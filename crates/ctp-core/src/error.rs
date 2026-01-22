//! Error types for CTP Core

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CTPError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Analysis error: {0}")]
    AnalysisError(String),

    #[error("Drift detection error: {0}")]
    DriftError(String),

    #[error("Policy evaluation error: {0}")]
    PolicyError(String),

    #[error("LLM error: {0}")]
    LLMError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type CTPResult<T> = Result<T, CTPError>;
