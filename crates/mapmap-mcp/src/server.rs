use crate::protocol::*;
use crate::McpAction;
use anyhow::Result;
use crossbeam_channel::Sender;
use mapmap_control::osc::client::OscClient;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

pub struct McpServer {
    // Optional OSC client (currently unused but will be used for OSC tools)
    #[allow(dead_code)]
    osc_client: Option<OscClient>,
    // Channel to send actions to main app
    pub action_sender: Option<Sender<McpAction>>,
}

impl McpServer {
    pub fn new(action_sender: Option<crossbeam_channel::Sender<crate::McpAction>>) -> Self {
        // Try to connect to default MapFlow OSC port
        let osc_client = match OscClient::new("127.0.0.1:8000") {
            Ok(client) => {
                info!("MCP Server connected to OSC at 127.0.0.1:8000");
                Some(client)
            }
            Err(e) => {
                error!("Failed to create OSC client: {}", e);
                None
            }
        };
        Self {
            osc_client,
            action_sender,
        }
    }

    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                break; // EOF
            }

            let response = self.handle_request(&line).await;

            if let Some(resp) = response {
                let json = serde_json::to_string(&resp)?;
                stdout.write_all(json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        Ok(())
    }

    async fn handle_request(&self, request_str: &str) -> Option<JsonRpcResponse> {
        let request: JsonRpcRequest = match serde_json::from_str(request_str) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                return Some(error_response(None, -32700, "Parse error"));
            }
        };

        let id = request.id.clone();

        match request.method.as_str() {
            "initialize" => {
                let result = InitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: ServerCapabilities {
                        tools: Some(serde_json::json!({
                            "listChanged": true
                        })),
                        resources: None,
                        prompts: None,
                    },
                    server_info: ServerInfo {
                        name: "MapFlow-mcp".to_string(),
                        version: "0.1.0".to_string(),
                    },
                };
                match serde_json::to_value(result) {
                    Ok(val) => Some(success_response(id, val)),
                    Err(e) => {
                        error!("Failed to serialize initialize result: {}", e);
                        Some(error_response(id, -32603, "Internal error"))
                    }
                }
            }
            "notifications/initialized" => None,
            "tools/list" => {
                let tools = crate::tools::registry::get_tools();

                Some(success_response(
                    id,
                    serde_json::json!({
                        "tools": tools
                    }),
                ))
            }
            "resources/list" => {
                let resources = vec![
                    serde_json::json!({
                        "uri": "project://current",
                        "name": "Current Project",
                        "mimeType": "application/json",
                        "description": "The current MapFlow project state"
                    }),
                    serde_json::json!({
                        "uri": "layer://list",
                        "name": "Layer List",
                        "mimeType": "application/json",
                        "description": "List of all layers"
                    }),
                ];
                Some(success_response(
                    id,
                    serde_json::json!({ "resources": resources }),
                ))
            }
            "resources/read" => {
                // Parse params
                let params: Option<serde_json::Value> =
                    serde_json::from_value(request.params.unwrap_or(serde_json::Value::Null)).ok();
                let uri = params
                    .and_then(|p| p.get("uri").and_then(|v| v.as_str()).map(|s| s.to_string()));

                if let Some(uri) = uri {
                    match uri.as_str() {
                        "project://current" => {
                            // TODO: Implement shared state reading
                            Some(success_response(
                                id,
                                serde_json::json!({
                                    "contents": [{
                                        "uri": uri,
                                        "mimeType": "application/json",
                                        "text": "{\"error\": \"Shared state access not yet implemented\"}"
                                    }]
                                }),
                            ))
                        }
                        _ => Some(error_response(id, -32602, "Resource not found")),
                    }
                } else {
                    Some(error_response(id, -32602, "Missing uri parameter"))
                }
            }
            "prompts/list" => {
                let prompts = vec![
                    serde_json::json!({
                        "name": "create_mapping",
                        "description": "Assist in creating a new projection mapping",
                        "arguments": []
                    }),
                    serde_json::json!({
                         "name": "troubleshoot",
                         "description": "Diagnose common problems",
                         "arguments": []
                    }),
                ];
                Some(success_response(
                    id,
                    serde_json::json!({ "prompts": prompts }),
                ))
            }
            "prompts/get" => {
                let params: Option<serde_json::Value> =
                    serde_json::from_value(request.params.unwrap_or(serde_json::Value::Null)).ok();
                let name = params.and_then(|p| {
                    p.get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });

                if let Some(name_str) = name {
                    match name_str.as_str() {
                        "create_mapping" => Some(success_response(
                            id,
                            serde_json::json!({
                                "description": "Create a new mapping",
                                "messages": [
                                    {
                                        "role": "user",
                                        "content": {
                                            "type": "text",
                                            "text": "I want to create a new mapping for a surface. Please guide me through the steps affecting layers and meshes."
                                        }
                                    }
                                ]
                            }),
                        )),
                        "troubleshoot" => Some(success_response(
                            id,
                            serde_json::json!({
                                "description": "Troubleshoot MapFlow",
                                "messages": [
                                    {
                                        "role": "user",
                                        "content": {
                                            "type": "text",
                                            "text": "Analyze the current state and logs for any errors or misconfigurations."
                                        }
                                    }
                                ]
                            }),
                        )),
                        _ => Some(error_response(id, -32601, "Prompt not found")),
                    }
                } else {
                    Some(error_response(id, -32602, "Missing name parameter"))
                }
            }
            // Handle tool calls
            "tools/call" => {
                // Parse params
                let params: CallToolParams = match serde_json::from_value(
                    request.params.clone().unwrap_or(serde_json::Value::Null),
                ) {
                    Ok(p) => p,
                    Err(_) => return Some(error_response(id, -32602, "Invalid params")),
                };

                crate::tools::handlers::handle_tool_call(self, id, params)
            }
            _ => Some(error_response(id, -32601, "Method not found")),
        }
    }

    #[allow(dead_code)]
    pub fn handle_send_osc(
        &self,
        id: Option<serde_json::Value>,
        args: &serde_json::Value,
    ) -> Option<JsonRpcResponse> {
        if let (Some(address_val), Some(args_val)) = (args.get("address"), args.get("args")) {
            if let (Some(address), Some(args_array)) = (address_val.as_str(), args_val.as_array()) {
                let mut osc_args = Vec::new();
                for arg in args_array {
                    if let Some(f) = arg.as_f64() {
                        osc_args.push(rosc::OscType::Float(f as f32));
                    }
                }
                return self.send_osc_msg(address, osc_args, id);
            }
        }
        Some(error_response(
            id,
            -32602,
            "Missing address or args argument",
        ))
    }

    #[allow(dead_code)]
    fn send_osc_msg(
        &self,
        address: &str,
        args: Vec<rosc::OscType>,
        id: Option<serde_json::Value>,
    ) -> Option<JsonRpcResponse> {
        if let Some(client) = &self.osc_client {
            match client.send_message(address, args) {
                Ok(_) => Some(success_response(
                    id,
                    serde_json::json!(CallToolResult {
                        content: vec![ToolContent::Text {
                            text: format!("Sent OSC message to {}", address)
                        }],
                        is_error: Some(false)
                    }),
                )),
                Err(e) => Some(success_response(
                    id,
                    serde_json::json!(CallToolResult {
                        content: vec![ToolContent::Text {
                            text: format!("OSC Error: {}", e)
                        }],
                        is_error: Some(true)
                    }),
                )),
            }
        } else {
            Some(error_response(id, -32000, "OSC Client not initialized"))
        }
    }
}

pub fn success_response(
    id: Option<serde_json::Value>,
    result: serde_json::Value,
) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id,
    }
}

pub fn error_response(id: Option<serde_json::Value>, code: i32, message: &str) -> JsonRpcResponse {
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
    use crossbeam_channel::unbounded;
    use serde_json::json;

    #[tokio::test]
    async fn test_handle_layer_create() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "layer_create",
                "arguments": {
                    "name": "Test Layer"
                }
            }
        });

        let response = server.handle_request(&request.to_string()).await;
        assert!(response.is_some());

        let action = rx.try_recv().unwrap();
        match action {
            McpAction::AddLayer(name) => assert_eq!(name, "Test Layer"),
            other => panic!("Expected AddLayer action, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_handle_layer_delete() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "layer_delete",
                "arguments": {
                    "layer_id": 42
                }
            }
        });

        server.handle_request(&request.to_string()).await;
        let action = rx.try_recv().unwrap();
        match action {
            McpAction::RemoveLayer(id) => assert_eq!(id, 42),
            other => panic!("Expected RemoveLayer action, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_handle_cue_trigger() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "cue_trigger",
                "arguments": {
                    "cue_id": 5
                }
            }
        });

        server.handle_request(&request.to_string()).await;
        let action = rx.try_recv().unwrap();
        match action {
            McpAction::TriggerCue(id) => assert_eq!(id, 5),
            other => panic!("Expected TriggerCue action, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_handle_cue_navigation() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        // Test Next
        let next_req = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "cue_next",
                "arguments": {}
            }
        });
        server.handle_request(&next_req.to_string()).await;
        assert!(matches!(rx.try_recv().unwrap(), McpAction::NextCue));

        // Test Previous
        let prev_req = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "cue_previous",
                "arguments": {}
            }
        });
        server.handle_request(&prev_req.to_string()).await;
        assert!(matches!(rx.try_recv().unwrap(), McpAction::PrevCue));
    }

    #[tokio::test]
    async fn test_handle_project_save_load() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        // Test Save
        let save_req = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "project_save",
                "arguments": {
                    "path": "test.mapmap"
                }
            }
        });
        server.handle_request(&save_req.to_string()).await;
        let action = rx.try_recv().unwrap();
        match action {
            McpAction::SaveProject(path) => assert_eq!(path.to_str().unwrap(), "test.mapmap"),
            other => panic!("Expected SaveProject action, got {:?}", other),
        }

        // Test Load
        let load_req = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {
                "name": "project_load",
                "arguments": {
                    "path": "other.mapmap"
                }
            }
        });
        server.handle_request(&load_req.to_string()).await;
        let action = rx.try_recv().unwrap();
        match action {
            McpAction::LoadProject(path) => assert_eq!(path.to_str().unwrap(), "other.mapmap"),
            other => panic!("Expected LoadProject action, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_handle_send_osc() {
        let (tx, _rx) = unbounded();
        let server = McpServer::new(Some(tx));

        let request = json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "tools/call",
            "params": {
                "name": "send_osc",
                "arguments": {
                    "address": "/test/addr",
                    "args": ["hello", 123, 1.5]
                }
            }
        });

        let response = server.handle_request(&request.to_string()).await;
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(
            resp.error.is_none(),
            "Response should not be an error: {:?}",
            resp.error
        );

        let result = resp.result.unwrap();
        // result is a CallToolResult
        assert_eq!(result["isError"], false);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Sent OSC"));
    }

    #[tokio::test]
    async fn test_security_path_traversal() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        // Test vulnerable path traversal
        let save_req = json!({
            "jsonrpc": "2.0",
            "id": 99,
            "method": "tools/call",
            "params": {
                "name": "project_save",
                "arguments": {
                    "path": "../evil.txt"
                }
            }
        });

        let response = server.handle_request(&save_req.to_string()).await;
        let resp = response.unwrap();

        // Should fail now
        assert!(resp.error.is_some(), "Should fail after fix (secure)");

        let error = resp.error.unwrap();
        assert!(error.message.contains("Invalid path"));
        // Could be traversal or extension error depending on order, but we check generic "Invalid path" prefix
        assert!(error.message.contains("Path traversal") || error.message.contains("Extension"));

        // Verify NO action was sent
        assert!(rx.try_recv().is_err());

        // Test invalid extension
        let ext_req = json!({
            "jsonrpc": "2.0",
            "id": 101,
            "method": "tools/call",
            "params": {
                "name": "project_save",
                "arguments": {
                    "path": "script.sh"
                }
            }
        });
        let ext_resp = server.handle_request(&ext_req.to_string()).await.unwrap();
        assert!(ext_resp.error.is_some());
        assert!(ext_resp
            .error
            .unwrap()
            .message
            .contains("Extension 'sh' is not allowed"));

        // Test valid path
        let valid_req = json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "tools/call",
            "params": {
                "name": "project_save",
                "arguments": {
                    "path": "good_project.mapmap"
                }
            }
        });

        let valid_response = server.handle_request(&valid_req.to_string()).await;
        let valid_resp = valid_response.unwrap();
        assert!(valid_resp.error.is_none());
        assert_eq!(valid_resp.result.unwrap()["status"], "queued");

        // Verify valid action sent
        let valid_action = rx.try_recv().unwrap();
        match valid_action {
            McpAction::SaveProject(path) => {
                assert_eq!(path.to_str().unwrap(), "good_project.mapmap")
            }
            other => panic!("Expected SaveProject action, got {:?}", other),
        }
    }
}
