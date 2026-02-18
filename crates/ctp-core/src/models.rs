//! Core data models for CodeTruth Protocol
//!
//! These structures follow the CTP specification v1.0

use serde::{Deserialize, Serialize};

/// The main explanation graph - core output of CTP analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationGraph {
    /// Protocol version
    pub ctp_version: String,

    /// Unique identifier for this explanation
    pub explanation_id: String,

    /// Code module being explained
    pub module: Module,

    /// Intent reconstruction
    pub intent: Intent,

    /// Actual behavior analysis
    pub behavior: Behavior,

    /// Drift detection results
    pub drift: DriftAnalysis,

    /// Policy compliance results
    pub policies: PolicyResults,

    /// Historical context
    pub history: History,

    /// Analysis metadata
    pub metadata: Metadata,
}

/// Information about the analyzed code module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub path: String,
    pub language: String,
    pub lines_of_code: usize,
    pub complexity_score: f64,
    pub content_hash: String,
}

/// Intent reconstruction from code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// What the code claims to do (from comments/docs)
    pub declared_intent: String,

    /// What we infer it's supposed to do (from context)
    pub inferred_intent: String,

    /// Confidence in our inference (0.0 - 1.0)
    pub confidence: f64,

    /// Business purpose
    pub business_context: String,

    /// Technical rationale
    pub technical_rationale: String,
}

/// Actual behavior analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Behavior {
    /// What the code actually does
    pub actual_behavior: String,

    /// Entry points
    pub entry_points: Vec<EntryPoint>,

    /// Exit points
    pub exit_points: Vec<ExitPoint>,

    /// Side effects
    pub side_effects: Vec<SideEffect>,

    /// Dependencies
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub function: String,
    pub parameters: Vec<String>,
    pub preconditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitPoint {
    pub return_type: String,
    pub possible_values: Vec<String>,
    pub postconditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    pub effect_type: SideEffectType,
    pub description: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideEffectType {
    Io,
    Network,
    Database,
    StateMutation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub module: String,
    pub reason: String,
    pub coupling_type: CouplingType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouplingType {
    Tight,
    Loose,
}

/// Drift detection results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAnalysis {
    pub drift_detected: bool,
    pub drift_severity: DriftSeverity,
    pub drift_details: Vec<DriftDetail>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriftSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetail {
    pub drift_type: DriftType,
    pub expected: String,
    pub actual: String,
    pub location: Location,
    pub impact: Impact,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriftType {
    Intent,
    Policy,
    Assumption,
    Implementation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Impact {
    pub functional: String,
    pub security: String,
    pub performance: String,
    pub maintainability: String,
}

/// Policy evaluation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResults {
    pub evaluated_at: String,
    pub policy_results: Vec<PolicyResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub policy_id: String,
    pub policy_name: String,
    pub status: PolicyStatus,
    pub violations: Vec<Violation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyStatus {
    Pass,
    Fail,
    Warning,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule: String,
    pub severity: ViolationSeverity,
    pub message: String,
    pub location: Location,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Historical tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub previous_versions: Vec<PreviousVersion>,
    pub evolution: Evolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviousVersion {
    pub version_id: String,
    pub analyzed_at: String,
    pub commit_hash: String,
    pub drift_from_previous: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evolution {
    pub created_at: String,
    pub last_modified: String,
    pub modification_count: usize,
    pub stability_score: f64,
}

/// Analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub generated_at: String,
    pub generator: Generator,
    pub extensions: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generator {
    pub name: String,
    pub version: String,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
}

// ============================================================================
// Minimal Mode Models (for 90% of use cases)
// ============================================================================

/// Simplified analysis for minimal mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalAnalysis {
    pub ctp_version: String,
    pub file_hash: String,
    pub intent: String,
    pub behavior: String,
    pub drift: DriftSeverity,
    pub confidence: f64,
}

/// Result of a full codebase analysis with cross-file context
#[derive(Debug, Clone)]
pub struct CodebaseAnalysis {
    /// Individual file analysis results
    pub graphs: Vec<ExplanationGraph>,
    /// Hierarchical semantic contexts built from the analysis
    pub contexts: Vec<ctp_context::SemanticContext>,
    /// System-level context (root of hierarchy)
    pub system_context: Option<ctp_context::SemanticContext>,
    /// Detected redundancies: (source_id, target_id, similarity)
    pub redundancies: Vec<(ctp_context::ContextId, ctp_context::ContextId, f64)>,
    /// Codebase-level statistics
    pub stats: crate::context_bridge::CodebaseStats,
    /// Files that failed to analyze: (path, error message)
    pub errors: Vec<(String, String)>,
}

impl From<ExplanationGraph> for MinimalAnalysis {
    fn from(graph: ExplanationGraph) -> Self {
        MinimalAnalysis {
            ctp_version: graph.ctp_version,
            file_hash: graph.explanation_id,
            intent: graph.intent.inferred_intent,
            behavior: graph.behavior.actual_behavior,
            drift: graph.drift.drift_severity,
            confidence: graph.intent.confidence,
        }
    }
}
