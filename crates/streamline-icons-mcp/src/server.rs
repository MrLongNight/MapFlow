//! MCP Server implementation for Streamline Icons API
//!
//! Handles JSON-RPC 2.0 requests over stdio.

use crate::api::{
    DownloadPngParams, DownloadSvgParams, FamilySearchParams, ProductType, SearchParams,
    StreamlineClient,
};
use crate::protocol::*;
use anyhow::Result;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

/// MCP Server for Streamline Icons
pub struct McpServer {
    client: StreamlineClient,
}

impl McpServer {
    /// Create a new server using the ICON_API_KEY environment variable
    pub fn from_env() -> Result<Self> {
        let client = StreamlineClient::from_env()?;
        Ok(Self { client })
    }

    /// Create a new server with a specific API key
    pub fn new(api_key: String) -> Self {
        Self {
            client: StreamlineClient::new(api_key),
        }
    }

    /// Run the MCP server over stdio
    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        info!("Streamline Icons MCP Server started");

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            if let Some(response) = self.handle_request(&line).await {
                let response_str = serde_json::to_string(&response)?;
                stdout.write_all(response_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        Ok(())
    }

    /// Handle a single JSON-RPC request
    pub async fn handle_request(&self, request_str: &str) -> Option<JsonRpcResponse> {
        let request: JsonRpcRequest = match serde_json::from_str(request_str) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse request: {}", e);
                return Some(error_response(None, -32700, "Parse error"));
            }
        };

        let id = request.id.clone();

        match request.method.as_str() {
            "initialize" => Some(self.handle_initialize(id)),
            "initialized" => None, // Notification, no response needed
            "tools/list" => Some(self.handle_tools_list(id)),
            "tools/call" => self.handle_tool_call(id, request.params).await,
            _ => Some(error_response(id, -32601, "Method not found")),
        }
    }

    fn handle_initialize(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(json!({})),
                resources: None,
                prompts: None,
            },
            server_info: ServerInfo {
                name: "streamline-icons-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        success_response(id, serde_json::to_value(result).unwrap())
    }

    fn handle_tools_list(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let tools = vec![
            Tool {
                name: "search".to_string(),
                description: Some("Search for icons, illustrations, emojis, or elements from all families.".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "productType": {
                            "type": "string",
                            "enum": ["icons", "illustrations", "emojis", "elements"],
                            "description": "Product type for the search"
                        },
                        "query": {
                            "type": "string",
                            "description": "Search term to find icons"
                        },
                        "offset": {
                            "type": "number",
                            "description": "Number of items to skip before returning results",
                            "default": 0
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of items to return (max 50)",
                            "default": 10
                        },
                        "productTier": {
                            "type": "string",
                            "enum": ["all", "free", "premium"],
                            "description": "Filter by price tier",
                            "default": "all"
                        },
                        "style": {
                            "type": "string",
                            "description": "Filter for the style of the sets (e.g., line, solid, flat, duo)"
                        }
                    },
                    "required": ["productType", "query"]
                }),
            },
            Tool {
                name: "family_search".to_string(),
                description: Some("Search for icons within a specific family.".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "familySlug": {
                            "type": "string",
                            "description": "Family slug obtained from a global search"
                        },
                        "query": {
                            "type": "string",
                            "description": "Search term to find icons"
                        },
                        "offset": {
                            "type": "number",
                            "description": "Number of items to skip",
                            "default": 0
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of items to return (max 50)",
                            "default": 10
                        }
                    },
                    "required": ["familySlug", "query"]
                }),
            },
            Tool {
                name: "get_icon_by_hash".to_string(),
                description: Some("Retrieve detailed information about a specific icon by its hash.".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "iconHash": {
                            "type": "string",
                            "description": "Icon hash obtained from a search response"
                        }
                    },
                    "required": ["iconHash"]
                }),
            },
            Tool {
                name: "download_svg".to_string(),
                description: Some("Download an icon as SVG with optional modifications.".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "iconHash": {
                            "type": "string",
                            "description": "Icon hash obtained from a search response"
                        },
                        "size": {
                            "type": "number",
                            "description": "Image size in pixels (square)"
                        },
                        "colors": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of HEX or named colors for export"
                        },
                        "backgroundColor": {
                            "type": "string",
                            "description": "Background color in HEX or named color",
                            "default": "#ffffff00"
                        },
                        "responsive": {
                            "type": "boolean",
                            "description": "Scales SVG with container; removes width/height",
                            "default": false
                        },
                        "strokeWidth": {
                            "type": "number",
                            "description": "Adjusts vector path thickness"
                        },
                        "strokeToFill": {
                            "type": "boolean",
                            "description": "Converts strokes to fills",
                            "default": false
                        },
                        "base64": {
                            "type": "boolean",
                            "description": "Return SVG as base64 string instead of raw data"
                        }
                    },
                    "required": ["iconHash", "size"]
                }),
            },
            Tool {
                name: "download_png".to_string(),
                description: Some("Download an icon as PNG with optional modifications.".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "iconHash": {
                            "type": "string",
                            "description": "Icon hash obtained from a search response"
                        },
                        "size": {
                            "type": "number",
                            "description": "Image size in pixels (square)"
                        },
                        "colors": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of HEX or named colors for export"
                        },
                        "backgroundColor": {
                            "type": "string",
                            "description": "Background color in HEX or named color",
                            "default": "#ffffff00"
                        },
                        "strokeWidth": {
                            "type": "number",
                            "description": "Adjusts vector path thickness"
                        }
                    },
                    "required": ["iconHash", "size"]
                }),
            },
        ];

        success_response(id, json!({ "tools": tools }))
    }

    async fn handle_tool_call(
        &self,
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> Option<JsonRpcResponse> {
        let params = match params {
            Some(p) => p,
            None => return Some(error_response(id, -32602, "Invalid params: missing params")),
        };

        let call_params: CallToolParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return Some(error_response(
                    id,
                    -32602,
                    &format!("Invalid params: {}", e),
                ))
            }
        };

        let args = call_params.arguments.unwrap_or(json!({}));

        let result = match call_params.name.as_str() {
            "search" => self.handle_search(&args).await,
            "family_search" => self.handle_family_search(&args).await,
            "get_icon_by_hash" => self.handle_get_icon_by_hash(&args).await,
            "download_svg" => self.handle_download_svg(&args).await,
            "download_png" => self.handle_download_png(&args).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", call_params.name)),
        };

        Some(match result {
            Ok(content) => success_response(
                id,
                serde_json::to_value(CallToolResult {
                    content: vec![ToolContent::Text { text: content }],
                    is_error: None,
                })
                .unwrap(),
            ),
            Err(e) => success_response(
                id,
                serde_json::to_value(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: format!("Error: {}", e),
                    }],
                    is_error: Some(true),
                })
                .unwrap(),
            ),
        })
    }

    async fn handle_search(&self, args: &serde_json::Value) -> Result<String> {
        let product_type_str = args
            .get("productType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: productType"))?;

        let product_type: ProductType = product_type_str.parse()?;

        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?
            .to_string();

        let params = SearchParams {
            product_type,
            query,
            offset: args.get("offset").and_then(|v| v.as_u64()).map(|v| v as u32),
            limit: args.get("limit").and_then(|v| v.as_u64()).map(|v| v as u32),
            product_tier: args.get("productTier").and_then(|v| v.as_str()).map(String::from),
            style: args.get("style").and_then(|v| v.as_str()).map(String::from),
        };

        let response = self.client.search(params).await?;
        Ok(serde_json::to_string_pretty(&response)?)
    }

    async fn handle_family_search(&self, args: &serde_json::Value) -> Result<String> {
        let family_slug = args
            .get("familySlug")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: familySlug"))?
            .to_string();

        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?
            .to_string();

        let params = FamilySearchParams {
            family_slug,
            query,
            offset: args.get("offset").and_then(|v| v.as_u64()).map(|v| v as u32),
            limit: args.get("limit").and_then(|v| v.as_u64()).map(|v| v as u32),
        };

        let response = self.client.family_search(params).await?;
        Ok(serde_json::to_string_pretty(&response)?)
    }

    async fn handle_get_icon_by_hash(&self, args: &serde_json::Value) -> Result<String> {
        let icon_hash = args
            .get("iconHash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: iconHash"))?;

        let response = self.client.get_icon_by_hash(icon_hash).await?;
        Ok(serde_json::to_string_pretty(&response)?)
    }

    async fn handle_download_svg(&self, args: &serde_json::Value) -> Result<String> {
        let icon_hash = args
            .get("iconHash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: iconHash"))?
            .to_string();

        let size = args
            .get("size")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: size"))? as u32;

        let colors = args
            .get("colors")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        let params = DownloadSvgParams {
            icon_hash,
            size,
            colors,
            background_color: args.get("backgroundColor").and_then(|v| v.as_str()).map(String::from),
            responsive: args.get("responsive").and_then(|v| v.as_bool()),
            stroke_width: args.get("strokeWidth").and_then(|v| v.as_f64()).map(|v| v as f32),
            stroke_to_fill: args.get("strokeToFill").and_then(|v| v.as_bool()),
            base64: args.get("base64").and_then(|v| v.as_bool()),
        };

        self.client.download_svg(params).await
    }

    async fn handle_download_png(&self, args: &serde_json::Value) -> Result<String> {
        let icon_hash = args
            .get("iconHash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: iconHash"))?
            .to_string();

        let size = args
            .get("size")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: size"))? as u32;

        let colors = args
            .get("colors")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        let params = DownloadPngParams {
            icon_hash,
            size,
            colors,
            background_color: args.get("backgroundColor").and_then(|v| v.as_str()).map(String::from),
            stroke_width: args.get("strokeWidth").and_then(|v| v.as_f64()).map(|v| v as f32),
        };

        let bytes = self.client.download_png(params).await?;
        // Return as base64 for PNG
        use std::io::Write;
        let mut encoder = base64_encoder();
        encoder.write_all(&bytes)?;
        Ok(encoder.finish())
    }
}

fn base64_encoder() -> Base64Encoder {
    Base64Encoder::new()
}

struct Base64Encoder {
    data: Vec<u8>,
}

impl Base64Encoder {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn finish(self) -> String {
        use std::fmt::Write;
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();
        
        for chunk in self.data.chunks(3) {
            let b0 = chunk[0] as usize;
            let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
            let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
            
            let _ = write!(result, "{}", ALPHABET[(b0 >> 2) & 0x3F] as char);
            let _ = write!(result, "{}", ALPHABET[((b0 << 4) | (b1 >> 4)) & 0x3F] as char);
            
            if chunk.len() > 1 {
                let _ = write!(result, "{}", ALPHABET[((b1 << 2) | (b2 >> 6)) & 0x3F] as char);
            } else {
                result.push('=');
            }
            
            if chunk.len() > 2 {
                let _ = write!(result, "{}", ALPHABET[b2 & 0x3F] as char);
            } else {
                result.push('=');
            }
        }
        
        result
    }
}

impl std::io::Write for Base64Encoder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn success_response(id: Option<serde_json::Value>, result: serde_json::Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id,
    }
}

fn error_response(id: Option<serde_json::Value>, code: i32, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
            data: None,
        }),
        id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_list() {
        // We can't test from_env without API key, so just verify the structure compiles
        let api_key = "test_key".to_string();
        let server = McpServer::new(api_key);
        let response = server.handle_tools_list(Some(json!(1)));
        
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_initialize() {
        let server = McpServer::new("test_key".to_string());
        let response = server.handle_initialize(Some(json!(1)));
        
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_base64_encoder() {
        let mut encoder = Base64Encoder::new();
        use std::io::Write;
        encoder.write_all(b"Hello").unwrap();
        let result = encoder.finish();
        assert_eq!(result, "SGVsbG8=");
    }
}
