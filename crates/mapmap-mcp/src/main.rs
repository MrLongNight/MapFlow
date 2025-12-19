use anyhow::Result;
use mapmap_mcp::McpServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (stderr only, as stdout is used for MCP protocol)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let server = McpServer::new(None);
    eprintln!("Starting MapMap MCP Server on stdio...");

    server.run_stdio().await?;

    Ok(())
}
