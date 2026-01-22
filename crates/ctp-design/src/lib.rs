//! # CTP Design System
//!
//! Design system extraction and deviation detection for CodeTruth Protocol.
//!
//! This crate provides:
//! - Automatic extraction of design tokens from codebases (colors, typography, spacing)
//! - Detection of design system deviations in new code
//! - Git-like tracking of design system evolution

pub mod tokens;
pub mod extractor;
pub mod deviation;
pub mod baseline;

pub use tokens::{DesignTokens, ColorToken, TypographyToken, SpacingToken, ComponentPattern};
pub use extractor::DesignSystemExtractor;
pub use deviation::{DesignDeviation, DeviationType, DeviationDetector};
pub use baseline::{DesignBaseline, BaselineManager};
