use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as JsonResult;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, Hover, HoverContents,
    HoverProviderCapability, InitializeParams, InitializeResult, MarkupContent, MarkupKind,
    MessageType, NumberOrString, Position, Range, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Url,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

use ctp_core::{CodeTruthEngine, DriftSeverity, ExplanationGraph, PolicyStatus, ViolationSeverity};

use crate::config::{CliConfig, FileFilter};
use crate::engine_config::build_engine;

pub async fn run_lsp(port: u16, config: CliConfig) -> Result<()> {
    let filter = FileFilter::new(&config)?;
    let engine = build_engine(&config, config.llm.enabled);

    if !config.policies.path.is_empty() {
        let policies_path = PathBuf::from(&config.policies.path);
        if policies_path.exists() {
            let _ = engine.load_policies(&policies_path);
        }
    }

    let engine = Arc::new(engine);
    let min_severity = parse_drift_severity(&config.drift.min_severity, DriftSeverity::Low);
    let max_file_size = config.analysis.max_file_size;

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("CTP LSP listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let engine = Arc::clone(&engine);
        let filter = filter.clone();

        tokio::spawn(async move {
            let (service, socket) = LspService::new(|client| {
                Backend::new(client, engine, filter, min_severity, max_file_size)
            });

            let (read, write) = tokio::io::split(stream);
            Server::new(read, write, socket).serve(service).await;
        });
    }
}

struct Backend {
    client: Client,
    engine: Arc<CodeTruthEngine>,
    filter: FileFilter,
    min_drift_severity: DriftSeverity,
    max_file_size: usize,
    cache: RwLock<HashMap<Url, ExplanationGraph>>,
}

impl Backend {
    fn new(
        client: Client,
        engine: Arc<CodeTruthEngine>,
        filter: FileFilter,
        min_drift_severity: DriftSeverity,
        max_file_size: usize,
    ) -> Self {
        Self {
            client,
            engine,
            filter,
            min_drift_severity,
            max_file_size,
            cache: RwLock::new(HashMap::new()),
        }
    }

    async fn handle_document(&self, uri: Url, content: Option<String>) {
        let Some(path) = uri.to_file_path().ok() else {
            return;
        };

        if !self.filter.is_allowed_file(&path) {
            self.client.publish_diagnostics(uri, Vec::new(), None).await;
            return;
        }

        if let Some(ref text) = content {
            if text.len() > self.max_file_size {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("CTP skipped large file: {}", path.display()),
                    )
                    .await;
                return;
            }
        }

        let analysis = self
            .analyze_path(&path, content.as_deref())
            .await
            .map_err(|err| err.to_string());

        match analysis {
            Ok(Some(graph)) => {
                let diagnostics = diagnostics_from_analysis(&graph, self.min_drift_severity);
                self.client
                    .publish_diagnostics(uri.clone(), diagnostics, None)
                    .await;
                self.cache.write().await.insert(uri, graph);
            }
            Ok(None) => {
                self.client.publish_diagnostics(uri.clone(), Vec::new(), None).await;
                self.cache.write().await.remove(&uri);
            }
            Err(message) => {
                self.client
                    .log_message(MessageType::ERROR, format!("CTP analysis failed: {}", message))
                    .await;
            }
        }
    }

    async fn analyze_path(
        &self,
        path: &Path,
        content: Option<&str>,
    ) -> Result<Option<ExplanationGraph>> {
        let language = self.filter.language_for_path(path).unwrap_or_default();
        if language.is_empty() {
            return Ok(None);
        }

        if let Some(text) = content {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            let analysis = self.engine.analyze_string(text, &language, &name).await?;
            return Ok(Some(analysis));
        }

        let analysis = self.engine.analyze_file(path).await?;
        Ok(Some(analysis))
    }

    async fn hover_for_uri(&self, uri: &Url) -> Option<Hover> {
        let cached = { self.cache.read().await.get(uri).cloned() };
        let analysis = if let Some(analysis) = cached {
            Some(analysis)
        } else {
            let path = uri.to_file_path().ok()?;
            self.analyze_path(&path, None).await.ok().flatten()
        }?;

        let drift = analysis.drift.drift_severity;
        let intent = analysis.intent;

        let value = format!(
            "**Declared Intent**: {}\n\n**Inferred Intent**: {}\n\n**Confidence**: {:.0}%\n\n**Drift**: {:?}",
            if intent.declared_intent.is_empty() {
                "(none)"
            } else {
                intent.declared_intent.as_str()
            },
            if intent.inferred_intent.is_empty() {
                "(none)"
            } else {
                intent.inferred_intent.as_str()
            },
            intent.confidence * 100.0,
            drift
        );

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: None,
        })
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> JsonResult<InitializeResult> {
        let sync_options = TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::FULL),
            save: Some(TextDocumentSyncSaveOptions::Supported(true)),
            ..Default::default()
        };

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "ctp".into(),
                version: Some(ctp_core::VERSION.into()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(sync_options)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: tower_lsp::lsp_types::InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "CTP LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> JsonResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.handle_document(uri, Some(text)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params
            .content_changes
            .last()
            .map(|change| change.text.clone());
        self.handle_document(uri, text).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        self.handle_document(uri, params.text).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.client.publish_diagnostics(uri.clone(), Vec::new(), None).await;
        self.cache.write().await.remove(&uri);
    }

    async fn hover(&self, params: tower_lsp::lsp_types::HoverParams) -> JsonResult<Option<Hover>> {
        Ok(self.hover_for_uri(&params.text_document_position_params.text_document.uri).await)
    }
}

fn diagnostics_from_analysis(
    analysis: &ExplanationGraph,
    min_drift_severity: DriftSeverity,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let drift_severity = analysis.drift.drift_severity;

    if drift_severity >= min_drift_severity && drift_severity != DriftSeverity::None {
        let diag_severity = drift_severity_to_diagnostic(drift_severity);
        let details = &analysis.drift.drift_details;

        if details.is_empty() {
            diagnostics.push(Diagnostic {
                range: default_range(),
                severity: diag_severity,
                message: format!("Drift detected: {:?}", drift_severity),
                source: Some("ctp".into()),
                ..Default::default()
            });
        } else {
            for detail in details {
                diagnostics.push(Diagnostic {
                    range: range_from_location(&detail.location),
                    severity: diag_severity,
                    message: format!(
                        "Drift ({:?}): expected {}, got {}",
                        detail.drift_type, detail.expected, detail.actual
                    ),
                    source: Some("ctp".into()),
                    ..Default::default()
                });
            }
        }
    }

    for policy in &analysis.policies.policy_results {
        if matches!(policy.status, PolicyStatus::Fail | PolicyStatus::Warning) {
            if policy.violations.is_empty() {
                diagnostics.push(Diagnostic {
                    range: default_range(),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!("Policy {} reported issues", policy.policy_id),
                    source: Some("ctp".into()),
                    ..Default::default()
                });
                continue;
            }

            for violation in &policy.violations {
                diagnostics.push(Diagnostic {
                    range: range_from_location(&violation.location),
                    severity: Some(violation_severity_to_diagnostic(violation.severity.clone())),
                    code: Some(NumberOrString::String(violation.rule.clone())),
                    message: format!("Policy {}: {}", policy.policy_id, violation.message),
                    source: Some("ctp".into()),
                    ..Default::default()
                });
            }
        }
    }

    diagnostics
}

fn parse_drift_severity(value: &str, fallback: DriftSeverity) -> DriftSeverity {
    match value.to_lowercase().as_str() {
        "none" => DriftSeverity::None,
        "low" => DriftSeverity::Low,
        "medium" => DriftSeverity::Medium,
        "high" => DriftSeverity::High,
        "critical" => DriftSeverity::Critical,
        _ => fallback,
    }
}

fn drift_severity_to_diagnostic(severity: DriftSeverity) -> Option<DiagnosticSeverity> {
    match severity {
        DriftSeverity::None => None,
        DriftSeverity::Low => Some(DiagnosticSeverity::HINT),
        DriftSeverity::Medium => Some(DiagnosticSeverity::INFORMATION),
        DriftSeverity::High => Some(DiagnosticSeverity::WARNING),
        DriftSeverity::Critical => Some(DiagnosticSeverity::ERROR),
    }
}

fn violation_severity_to_diagnostic(severity: ViolationSeverity) -> DiagnosticSeverity {
    match severity {
        ViolationSeverity::Info => DiagnosticSeverity::INFORMATION,
        ViolationSeverity::Warning => DiagnosticSeverity::WARNING,
        ViolationSeverity::Error => DiagnosticSeverity::ERROR,
        ViolationSeverity::Critical => DiagnosticSeverity::ERROR,
    }
}

fn range_from_location(location: &ctp_core::Location) -> Range {
    let start_line = location.line_start.saturating_sub(1) as u32;
    let end_line = location.line_end.saturating_sub(1) as u32;

    Range::new(
        Position::new(start_line, 0),
        Position::new(end_line.max(start_line), 0),
    )
}

fn default_range() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 0))
}
