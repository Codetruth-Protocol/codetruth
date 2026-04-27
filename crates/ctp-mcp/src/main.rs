//! CodeTruth MCP Server
//!
//! Implements the Model Context Protocol for AI-native CodeTruth integration.
//! Uses stdio transport for minimal latency and maximum compatibility.

use ctp_mcp::server::CodeTruthMCPServer;
use rmcp::service::ServiceExt;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (don't pollute stdout for MCP protocol)
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(std::io::stderr)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting CodeTruth MCP Server v{}", env!("CARGO_PKG_VERSION"));

    // Create MCP server
    let server = CodeTruthMCPServer::new().await?;
    
    info!("MCP server initialized, starting stdio transport");
    
    // Start stdio transport on stdin/stdout
    // The tuple (stdin, stdout) implements IntoTransport via the async_rw feature
    let _service = server.serve((tokio::io::stdin(), tokio::io::stdout())).await?;
    
    info!("MCP server running - waiting for client");
    
    // Keep the server running
    tokio::signal::ctrl_c().await?;
    
    info!("MCP server shutting down");
    Ok(())
}
