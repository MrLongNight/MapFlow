//! Streamline Icons MCP Server
//!
//! A Model Context Protocol server that provides access to the Streamline Icons API.
//! Icons can be searched, retrieved, and downloaded as SVG or PNG.

pub mod api;
pub mod protocol;
pub mod server;

pub use api::StreamlineClient;
pub use protocol::*;
pub use server::McpServer;

pub use anyhow::Result;
