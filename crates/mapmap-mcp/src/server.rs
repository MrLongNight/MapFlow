use crate::protocol::*;
use crate::McpAction;
use anyhow::Result;
use crossbeam_channel::Sender;
use mapmap_control::osc::client::OscClient;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

pub struct McpServer {
    osc_client: Option<OscClient>,
    action_sender: Option<Sender<McpAction>>,
}

impl McpServer {
    pub fn new(action_sender: Option<Sender<McpAction>>) -> Self {
        // Try to connect to default VJMapper OSC port
        let osc_client = match OscClient::new("127.0.0.1:8000") {
            Ok(client) => {
                info!("MCP Server connected regarding OSC to 127.0.0.1:8000");
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
                        name: "vjmapper-mcp".to_string(),
                        version: "0.1.0".to_string(),
                    },
                };
                Some(success_response(id, serde_json::to_value(result).unwrap()))
            }
            "notifications/initialized" => None,
            "tools/list" => {
                let tools = vec![
                    Tool {
                        name: "send_osc".to_string(),
                        description: Some(
                            "Send an Open Sound Control (OSC) message to the running VJMapper instance"
                                .to_string(),
                        ),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "address": {
                                    "type": "string",
                                    "description": "OSC Address (e.g. /mapmap/layer/1/opacity)"
                                },
                                "args": {
                                    "type": "array",
                                    "items": { "type": "number" },
                                    "description": "List of numeric arguments (floats)"
                                }
                            },
                            "required": ["address", "args"]
                        }),
                    },
                    Tool {
                        name: "layer_set_opacity".to_string(),
                        description: Some("Set the opacity of a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "opacity": { "type": "number", "minimum": 0.0, "maximum": 1.0 }
                            },
                            "required": ["layer_id", "opacity"]
                        }),
                    },
                    Tool {
                        name: "layer_set_visibility".to_string(),
                        description: Some("Set the visibility of a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "visible": { "type": "boolean" }
                            },
                            "required": ["layer_id", "visible"]
                        }),
                    },
                    Tool {
                        name: "media_play".to_string(),
                        description: Some("Start media playback".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {},
                        }),
                    },
                    Tool {
                        name: "media_pause".to_string(),
                        description: Some("Pause media playback".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {},
                        }),
                    },
                    Tool {
                        name: "media_stop".to_string(),
                        description: Some("Stop media playback".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {},
                        }),
                    },
                    Tool {
                        name: "project_save".to_string(),
                        description: Some("Save the current project".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" }
                            },
                            "required": ["path"]
                        }),
                    },
                    Tool {
                        name: "project_load".to_string(),
                        description: Some("Load a project from disk".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" }
                            },
                            "required": ["path"]
                        }),
                    },
                ];

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
                        "description": "The current VJMapper project state"
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
                                "description": "Troubleshoot VJMapper",
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
            "tools/call" => {
                // Parse params
                let params: CallToolParams = match serde_json::from_value(
                    request.params.clone().unwrap_or(serde_json::Value::Null),
                ) {
                    Ok(p) => p,
                    Err(_) => return Some(error_response(id, -32602, "Invalid params")),
                };

                let args = params.arguments.unwrap_or(serde_json::json!({}));

                match params.name.as_str() {
                    "send_osc" => self.handle_send_osc(id, &args),
                    "layer_set_opacity" => {
                        if let (Some(layer_id), Some(opacity)) = (
                            args.get("layer_id").and_then(|v| v.as_u64()),
                            args.get("opacity").and_then(|v| v.as_f64()),
                        ) {
                            self.send_osc_msg(
                                &format!("/mapmap/layer/{}/opacity", layer_id),
                                vec![rosc::OscType::Float(opacity as f32)],
                                id,
                            )
                        } else {
                            Some(error_response(id, -32602, "Invalid arguments"))
                        }
                    }
                    "layer_set_visibility" => {
                        if let (Some(layer_id), Some(visible)) = (
                            args.get("layer_id").and_then(|v| v.as_u64()),
                            args.get("visible").and_then(|v| v.as_bool()),
                        ) {
                            let val = if visible { 1.0 } else { 0.0 };
                            // Or use boolean type for OSC if supported by App
                            self.send_osc_msg(
                                &format!("/mapmap/layer/{}/visible", layer_id),
                                vec![rosc::OscType::Float(val)], // Using float for now as bool support varies
                                id,
                            )
                        } else {
                            Some(error_response(id, -32602, "Invalid arguments"))
                        }
                    }
                    "media_play" => self.send_osc_msg("/mapmap/playback/play", vec![], id),
                    "media_pause" => self.send_osc_msg("/mapmap/playback/pause", vec![], id),
                    "media_stop" => self.send_osc_msg("/mapmap/playback/stop", vec![], id),
                    "project_save" => {
                        if let Some(path_str) = args.get("path").and_then(|v| v.as_str()) {
                            let path = PathBuf::from(path_str);
                            if let Some(sender) = &self.action_sender {
                                let _ = sender.send(McpAction::SaveProject(path));
                                Some(success_response(
                                    id,
                                    serde_json::json!(CallToolResult {
                                        content: vec![ToolContent::Text {
                                            text: "Save triggered".to_string()
                                        }],
                                        is_error: Some(false),
                                    }),
                                ))
                            } else {
                                Some(error_response(
                                    id,
                                    -32000,
                                    "Internal error: Action sender not connected",
                                ))
                            }
                        } else {
                            Some(error_response(id, -32602, "Invalid arguments"))
                        }
                    }
                    "project_load" => {
                        if let Some(path_str) = args.get("path").and_then(|v| v.as_str()) {
                            let path = PathBuf::from(path_str);
                            if let Some(sender) = &self.action_sender {
                                let _ = sender.send(McpAction::LoadProject(path));
                                Some(success_response(
                                    id,
                                    serde_json::json!(CallToolResult {
                                        content: vec![ToolContent::Text {
                                            text: "Load triggered".to_string()
                                        }],
                                        is_error: Some(false),
                                    }),
                                ))
                            } else {
                                Some(error_response(
                                    id,
                                    -32000,
                                    "Internal error: Action sender not connected",
                                ))
                            }
                        } else {
                            Some(error_response(id, -32602, "Invalid arguments"))
                        }
                    }
                    _ => Some(error_response(id, -32601, "Tool not found")),
                }
            }
            _ => Some(error_response(id, -32601, "Method not found")),
        }
    }

    fn handle_send_osc(
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
