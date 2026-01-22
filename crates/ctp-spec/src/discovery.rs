//! Pattern discovery from codebase analysis

use super::*;
use ctp_core::ExplanationGraph;

/// Discovered specification from code analysis
#[derive(Debug, Clone)]
pub struct DiscoveredSpec {
    pub functionalities: Vec<CoreFunctionality>,
    pub constraints: TechnicalConstraints,
    pub primary_language: String,
    pub confidence: f64,
}

/// Cluster of files with similar intent/purpose
#[derive(Debug, Clone)]
pub struct PatternCluster {
    pub key: String,
    pub graphs: Vec<ExplanationGraph>,
    pub confidence: f64,
}
