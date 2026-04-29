//! Data models for MCP server

use serde::{Deserialize, Serialize};

/// Input for file analysis tool
#[derive(Debug, Clone, Deserialize)]
pub struct AnalyzeFileInput {
    /// Absolute path to the file to analyze
    pub file_path: String,
    /// Optional: specific focus area (e.g., "security", "performance")
    pub focus: Option<String>,
}

/// Output from file analysis
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzeFileOutput {
    /// File path that was analyzed
    pub file_path: String,
    /// Detected language
    pub language: String,
    /// Lines of code
    pub lines_of_code: usize,
    /// Complexity score (0.0 - 10.0)
    pub complexity_score: f64,
    /// Inferred intent of the code
    pub inferred_intent: String,
    /// Confidence in intent inference (0.0 - 1.0)
    pub intent_confidence: f64,
    /// Detected entry points
    pub entry_points: Vec<String>,
    /// Detected side effects
    pub side_effects: Vec<String>,
    /// Detected drift between intent and behavior
    pub drift_detected: bool,
    /// Drift details if detected
    pub drift_details: Option<String>,
    /// Compliance violations found
    pub violations: Vec<ComplianceViolation>,
}

/// Input for compliance check tool
#[derive(Debug, Clone, Deserialize)]
pub struct CheckComplianceInput {
    /// Absolute path to file or directory to check
    pub path: String,
    /// Optional: specific policy to check against
    pub policy: Option<String>,
    /// Optional: policy directory path
    pub policy_dir: Option<String>,
}

/// Output from compliance check
#[derive(Debug, Clone, Serialize)]
pub struct CheckComplianceOutput {
    /// Path that was checked
    pub path: String,
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Total violations found
    pub total_violations: usize,
    /// Violations by severity
    pub violations_by_severity: ViolationsBySeverity,
    /// Detailed violation list
    pub violations: Vec<ComplianceViolation>,
}

/// Violation counts by severity
#[derive(Debug, Clone, Serialize)]
pub struct ViolationsBySeverity {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Compliance violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    /// File path where violation occurred
    pub file_path: String,
    /// Line number (if applicable)
    pub line_number: Option<usize>,
    /// Violation severity
    pub severity: String,
    /// Policy name that was violated
    pub policy: String,
    /// Human-readable description
    pub description: String,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Rule ID for reference
    pub rule_id: String,
}

/// Input for drift detection tool
#[derive(Debug, Clone, Deserialize)]
pub struct DetectDriftInput {
    /// Path to analyze for drift
    pub path: String,
    /// Optional: baseline or specification to compare against
    pub baseline: Option<String>,
    /// Optional: specific drift types to detect
    pub drift_types: Option<Vec<String>>,
}

/// Output from drift detection
#[derive(Debug, Clone, Serialize)]
pub struct DetectDriftOutput {
    /// Path that was analyzed
    pub path: String,
    /// Whether drift was detected
    pub has_drift: bool,
    /// Number of drift instances
    pub drift_count: usize,
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Detailed drift findings
    pub findings: Vec<DriftFinding>,
}

/// Individual drift finding
#[derive(Debug, Clone, Serialize)]
pub struct DriftFinding {
    /// File path
    pub file_path: String,
    /// Line number
    pub line_number: usize,
    /// Type of drift
    pub drift_type: String,
    /// Severity level
    pub severity: String,
    /// Description of the drift
    pub description: String,
    /// Declared intent (from comments/docs)
    pub declared_intent: String,
    /// Actual behavior detected
    pub actual_behavior: String,
    /// Suggested correction
    pub suggestion: Option<String>,
}

/// Input for violation explanation tool
#[derive(Debug, Clone, Deserialize)]
pub struct ExplainViolationInput {
    /// File path of the violation
    pub file_path: String,
    /// Policy that was violated
    pub policy: String,
    /// Violation description
    pub description: String,
    /// Severity level
    pub severity: String,
    /// Optional: code context for better explanation
    pub code_context: Option<String>,
}

/// Output from violation explanation
#[derive(Debug, Clone, Serialize)]
pub struct ExplainViolationOutput {
    /// Natural language explanation of the violation
    pub explanation: String,
    /// Why this matters (business/technical impact)
    pub why_it_matters: String,
    /// How to fix (step by step)
    pub how_to_fix: String,
    /// Best practices to prevent recurrence
    pub prevention_tips: Vec<String>,
    /// Related documentation links (if available)
    pub references: Vec<String>,
}

/// Input for codebase-wide analysis
#[derive(Debug, Clone, Deserialize)]
pub struct AnalyzeCodebaseInput {
    /// Root directory of the codebase
    pub root_path: String,
    /// Project name
    pub project_name: String,
    /// Project purpose/description
    pub project_purpose: String,
    /// Optional: specific files to include
    pub include_patterns: Option<Vec<String>>,
    /// Optional: files to exclude
    pub exclude_patterns: Option<Vec<String>>,
}

/// Output from codebase analysis
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzeCodebaseOutput {
    /// Project name
    pub project_name: String,
    /// Total files analyzed
    pub total_files: usize,
    /// Files with violations
    pub files_with_violations: usize,
    /// Total violations
    pub total_violations: usize,
    /// Redundancy findings
    pub redundancies: Vec<RedundancyFinding>,
    /// Critical components identified
    pub critical_components: Vec<CriticalComponent>,
    /// Summary statistics
    pub stats: CodebaseStats,
}

/// Redundancy finding
#[derive(Debug, Clone, Serialize)]
pub struct RedundancyFinding {
    /// Type of redundancy
    pub redundancy_type: String,
    /// Description
    pub description: String,
    /// Files involved
    pub files: Vec<String>,
    /// Confidence score
    pub confidence: f64,
}

/// Critical component info
#[derive(Debug, Clone, Serialize)]
pub struct CriticalComponent {
    /// Component name
    pub name: String,
    /// File path
    pub file_path: String,
    /// Why it's critical
    pub rationale: String,
    /// Criticality level
    pub level: String,
}

/// Codebase statistics
#[derive(Debug, Clone, Serialize)]
pub struct CodebaseStats {
    pub total_components: usize,
    pub redundancy_count: usize,
    pub average_complexity: f64,
    pub total_lines_of_code: usize,
}

/// Input for stub detection tool
#[derive(Debug, Clone, Deserialize)]
pub struct DetectStubsInput {
    /// Path to file or directory to analyze for stubs
    pub path: String,
    /// Optional: include files matching these patterns
    pub include_patterns: Option<Vec<String>>,
    /// Optional: exclude files matching these patterns
    pub exclude_patterns: Option<Vec<String>>,
    /// Optional: minimum severity to report ("low", "medium", "high", "critical")
    pub min_severity: Option<String>,
}

/// Output from stub detection
#[derive(Debug, Clone, Serialize)]
pub struct DetectStubsOutput {
    /// Path that was analyzed
    pub path: String,
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Total stub/placeholder findings
    pub total_stubs_found: usize,
    /// Whether any critical stubs were found
    pub has_critical_stubs: bool,
    /// Stub counts by severity
    pub stubs_by_severity: StubsBySeverity,
    /// Detailed stub findings
    pub findings: Vec<StubFindingDetail>,
}

/// Stub counts by severity
#[derive(Debug, Clone, Serialize)]
pub struct StubsBySeverity {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Individual stub finding detail
#[derive(Debug, Clone, Serialize)]
pub struct StubFindingDetail {
    /// File path where stub was found
    pub file_path: String,
    /// Line number (1-indexed)
    pub line_number: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Type of stub detected (TODO, FIXME, PLACEHOLDER, UNIMPLEMENTED, etc.)
    pub stub_type: String,
    /// Severity level
    pub severity: String,
    /// The matched pattern/context
    pub context: String,
    /// Suggested remediation
    pub suggestion: Option<String>,
}
