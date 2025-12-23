//! Streamline Icons MCP Server
//!
//! A standalone MCP server that provides access to the Streamline Icons API.
//!
//! # Usage
//!
//! Set the ICON_API_KEY environment variable and run:
//! ```bash
//! streamline-icons-mcp
//! ```

use streamline_icons_mcp::McpServer;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (stdout is used for MCP communication)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .with_writer(std::io::stderr)
        .init();

    let server = McpServer::from_env()?;
    server.run_stdio().await
}
