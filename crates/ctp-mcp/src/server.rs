//! MCP Server implementation for CodeTruth
//!
//! Implements the Model Context Protocol server specification
//! for integration with Claude Code and other AI assistants.

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::handler::server::ServerHandler;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::Error as McpError;
use serde_json::json;
use tracing::{debug, info, instrument, warn};

use ctp_core::{CodeTruthEngine, EngineConfig};

use crate::cache::AnalysisCache;
use crate::models::*;
use crate::tools::ToolHandler;
use crate::{SERVER_NAME, SERVER_VERSION};

/// CodeTruth MCP Server
#[derive(Clone)]
pub struct CodeTruthMCPServer {
    tool_handler: Arc<ToolHandler>,
}

impl CodeTruthMCPServer {
    /// Create a new MCP server instance
    pub async fn new() -> anyhow::Result<Self> {
        info!("Initializing CodeTruth MCP Server");

        // Initialize engine with default config
        let config = EngineConfig::default();
        let engine = Arc::new(CodeTruthEngine::new(config));
        let cache = Arc::new(AnalysisCache::new());
        
        let tool_handler = Arc::new(ToolHandler::new(engine, cache));

        Ok(Self { tool_handler })
    }

    /// Convert serde_json::Value to Arc<JsonObject>
    fn to_json_object(value: serde_json::Value) -> Arc<serde_json::Map<String, serde_json::Value>> {
        match value {
            serde_json::Value::Object(map) => Arc::new(map),
            _ => Arc::new(serde_json::Map::new()),
        }
    }

    /// Execute a tool call
    #[instrument(skip(self, request), fields(tool = %request.name))]
    async fn execute_tool(&self, request: CallToolRequestParam) -> anyhow::Result<CallToolResult> {
        debug!("Executing tool: {}", request.name);

        let result = match request.name.as_ref() {
            "analyze_file" => {
                let input: AnalyzeFileInput = serde_json::from_value(
                    request.arguments.map(|a| serde_json::Value::Object(a)).unwrap_or_default()
                )?;
                let output = self.tool_handler.analyze_file(input).await?;
                serde_json::to_value(output)?
            }
            "check_compliance" => {
                let input: CheckComplianceInput = serde_json::from_value(
                    request.arguments.map(|a| serde_json::Value::Object(a)).unwrap_or_default()
                )?;
                let output = self.tool_handler.check_compliance(input).await?;
                serde_json::to_value(output)?
            }
            "detect_drift" => {
                let input: DetectDriftInput = serde_json::from_value(
                    request.arguments.map(|a| serde_json::Value::Object(a)).unwrap_or_default()
                )?;
                let output = self.tool_handler.detect_drift(input).await?;
                serde_json::to_value(output)?
            }
            "detect_stubs" => {
                let input: DetectStubsInput = serde_json::from_value(
                    request.arguments.map(|a| serde_json::Value::Object(a)).unwrap_or_default()
                )?;
                let output = self.tool_handler.detect_stubs(input).await?;
                serde_json::to_value(output)?
            }
            "explain_violation" => {
                let input: ExplainViolationInput = serde_json::from_value(
                    request.arguments.map(|a| serde_json::Value::Object(a)).unwrap_or_default()
                )?;
                let output = self.tool_handler.explain_violation(input).await?;
                serde_json::to_value(output)?
            }
            "analyze_codebase" => {
                let input: AnalyzeCodebaseInput = serde_json::from_value(
                    request.arguments.map(|a| serde_json::Value::Object(a)).unwrap_or_default()
                )?;
                let output = self.tool_handler.analyze_codebase(input).await?;
                serde_json::to_value(output)?
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown tool: {}", request.name));
            }
        };

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&result)?)],
            is_error: Some(false),
        })
    }
}

impl ServerHandler for CodeTruthMCPServer {
    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("MCP client initializing connection");

        Ok(InitializeResult {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            server_info: Implementation {
                name: SERVER_NAME.to_string(),
                version: SERVER_VERSION.to_string(),
            },
            instructions: Some("CodeTruth MCP Server - AI-native code analysis and compliance checking".to_string()),
        })
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        debug!("Listing available tools");

        let tools = vec![
            Tool::new(
                Cow::Borrowed("analyze_file"),
                Cow::Borrowed("Analyze a single code file to infer intent, detect behavior, and identify drift. Returns intent inference, complexity metrics, entry points, side effects, and any detected violations."),
                Self::to_json_object(json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Absolute path to the file to analyze"
                        },
                        "focus": {
                            "type": "string",
                            "description": "Optional focus area: 'security', 'performance', 'compliance'"
                        }
                    },
                    "required": ["file_path"]
                })),
            ),
            Tool::new(
                Cow::Borrowed("check_compliance"),
                Cow::Borrowed("Check code files or directories against compliance policies. Returns violations grouped by severity with descriptions and suggestions."),
                Self::to_json_object(json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to file or directory to check"
                        },
                        "policy": {
                            "type": "string",
                            "description": "Optional specific policy name to check"
                        },
                        "policy_dir": {
                            "type": "string",
                            "description": "Optional path to policy directory"
                        }
                    },
                    "required": ["path"]
                })),
            ),
            Tool::new(
                Cow::Borrowed("detect_drift"),
                Cow::Borrowed("Detect drift between declared intent and actual implementation. Identifies semantic mismatches, missing validations, and behavior deviations."),
                Self::to_json_object(json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to analyze for drift"
                        },
                        "baseline": {
                            "type": "string",
                            "description": "Optional baseline specification to compare against"
                        },
                        "drift_types": {
                            "type": "array",
                            "description": "Optional specific drift types to detect",
                            "items": { "type": "string" }
                        }
                    },
                    "required": ["path"]
                })),
            ),
            Tool::new(
                Cow::Borrowed("detect_stubs"),
                Cow::Borrowed("Detect stubs, TODOs, placeholders, and unimplemented code in files or directories. Returns findings by severity with line numbers and suggested fixes. Use this to audit AI-generated code for incomplete implementations before production."),
                Self::to_json_object(json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to file or directory to analyze for stubs"
                        },
                        "include_patterns": {
                            "type": "array",
                            "description": "Optional glob patterns for files to include (e.g., ['**/*.rs', '**/*.ts'])",
                            "items": { "type": "string" }
                        },
                        "exclude_patterns": {
                            "type": "array",
                            "description": "Optional glob patterns for files to exclude (e.g., ['**/tests/**', '**/vendor/**'])",
                            "items": { "type": "string" }
                        },
                        "min_severity": {
                            "type": "string",
                            "description": "Optional minimum severity to report: 'low', 'medium', 'high', 'critical'. Default: 'low' (all)",
                            "enum": ["low", "medium", "high", "critical"]
                        }
                    },
                    "required": ["path"]
                })),
            ),
            Tool::new(
                Cow::Borrowed("explain_violation"),
                Cow::Borrowed("Get a natural language explanation of a compliance violation. Provides context on why it matters and how to fix it."),
                Self::to_json_object(json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "File path of the violation"
                        },
                        "policy": {
                            "type": "string",
                            "description": "Policy that was violated"
                        },
                        "description": {
                            "type": "string",
                            "description": "Violation description"
                        },
                        "severity": {
                            "type": "string",
                            "description": "Severity level"
                        },
                        "code_context": {
                            "type": "string",
                            "description": "Optional surrounding code for better explanation"
                        }
                    },
                    "required": ["file_path", "policy", "description", "severity"]
                })),
            ),
            Tool::new(
                Cow::Borrowed("analyze_codebase"),
                Cow::Borrowed("Analyze an entire codebase for compliance, redundancies, and critical components. Provides organization-wide visibility into code quality."),
                Self::to_json_object(json!({
                    "type": "object",
                    "properties": {
                        "root_path": {
                            "type": "string",
                            "description": "Root directory of the codebase"
                        },
                        "project_name": {
                            "type": "string",
                            "description": "Name of the project"
                        },
                        "project_purpose": {
                            "type": "string",
                            "description": "Description of what the project does"
                        },
                        "include_patterns": {
                            "type": "array",
                            "description": "Optional glob patterns to include",
                            "items": { "type": "string" }
                        },
                        "exclude_patterns": {
                            "type": "array",
                            "description": "Optional glob patterns to exclude",
                            "items": { "type": "string" }
                        }
                    },
                    "required": ["root_path", "project_name", "project_purpose"]
                })),
            ),
        ];

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match self.execute_tool(request).await {
            Ok(response) => Ok(response),
            Err(e) => {
                warn!("Tool execution failed: {}", e);
                Ok(CallToolResult {
                    content: vec![Content::text(format!("Error: {}", e))],
                    is_error: Some(true),
                })
            }
        }
    }

}
