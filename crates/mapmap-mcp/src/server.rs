use crate::protocol::*;
use crate::McpAction;
use anyhow::Result;
use crossbeam_channel::Sender;
use mapmap_control::osc::client::OscClient;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

/// Validate that a path is safe for file operations
///
/// Prevents:
/// - Path traversal (.. components)
/// - Absolute paths (restricts to working directory)
fn validate_safe_path(path_str: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path_str);

    // Check for absolute path
    if path.is_absolute() {
        return Err("Absolute paths are not allowed".to_string());
    }

    // Check for path traversal components
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err("Path traversal (..) is not allowed".to_string());
        }
    }

    Ok(path)
}

/// Validate that a path is safe and has an allowed extension
fn validate_path_with_extensions(
    path_str: &str,
    allowed_extensions: &[&str],
) -> Result<PathBuf, String> {
    let path = validate_safe_path(path_str)?;

    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            if allowed_extensions.contains(&ext_str.to_lowercase().as_str()) {
                return Ok(path);
            }
            return Err(format!("Extension '{}' is not allowed", ext_str));
        }
    }

    Err("Invalid file extension".to_string())
}

pub struct McpServer {
    // Optional OSC client (currently unused but will be used for OSC tools)
    #[allow(dead_code)]
    osc_client: Option<OscClient>,
    // Channel to send actions to main app
    action_sender: Option<Sender<McpAction>>,
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
                Some(success_response(id, serde_json::to_value(result).unwrap()))
            }
            "notifications/initialized" => None,
            "tools/list" => {
                let tools = vec![
                    // === Basic Tools ===
                    Tool {
                        name: "send_osc".to_string(),
                        description: Some(
                            "Send an Open Sound Control (OSC) message to MapFlow".to_string(),
                        ),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "address": { "type": "string", "description": "OSC Address" },
                                "args": { "type": "array", "items": { "type": "number" } }
                            },
                            "required": ["address", "args"]
                        }),
                    },
                    // === Layer Management ===
                    Tool {
                        name: "layer_create".to_string(),
                        description: Some("Create a new layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "name": { "type": "string", "description": "Name for the new layer" }
                            },
                            "required": ["name"]
                        }),
                    },
                    Tool {
                        name: "layer_delete".to_string(),
                        description: Some("Delete a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "layer_id": { "type": "integer" } },
                            "required": ["layer_id"]
                        }),
                    },
                    Tool {
                        name: "layer_list".to_string(),
                        description: Some("List all layers".to_string()),
                        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
                    },
                    Tool {
                        name: "layer_set_opacity".to_string(),
                        description: Some("Set layer opacity (0.0-1.0)".to_string()),
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
                        description: Some("Set layer visibility".to_string()),
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
                        name: "layer_set_blend_mode".to_string(),
                        description: Some("Set layer blend mode".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "blend_mode": { "type": "string", "enum": ["normal", "add", "multiply", "screen", "overlay"] }
                            },
                            "required": ["layer_id", "blend_mode"]
                        }),
                    },
                    // === Cue Management ===
                    Tool {
                        name: "cue_trigger".to_string(),
                        description: Some("Trigger a specific cue".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "cue_id": { "type": "integer" } },
                            "required": ["cue_id"]
                        }),
                    },
                    Tool {
                        name: "cue_next".to_string(),
                        description: Some("Go to the next cue".to_string()),
                        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
                    },
                    Tool {
                        name: "cue_previous".to_string(),
                        description: Some("Go to the previous cue".to_string()),
                        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
                    },
                    // === Media Playback ===
                    Tool {
                        name: "media_play".to_string(),
                        description: Some("Start media playback".to_string()),
                        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
                    },
                    Tool {
                        name: "media_pause".to_string(),
                        description: Some("Pause media playback".to_string()),
                        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
                    },
                    Tool {
                        name: "media_stop".to_string(),
                        description: Some("Stop media playback".to_string()),
                        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
                    },
                    // === Project Management ===
                    Tool {
                        name: "project_save".to_string(),
                        description: Some("Save the current project".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "path": { "type": "string" } },
                            "required": ["path"]
                        }),
                    },
                    Tool {
                        name: "project_load".to_string(),
                        description: Some("Load a project from disk".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "path": { "type": "string" } },
                            "required": ["path"]
                        }),
                    },
                    // === Phase 1: Media in Layers ===
                    Tool {
                        name: "layer_load_media".to_string(),
                        description: Some("Load media file into a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "media_path": { "type": "string", "description": "Path to media file" }
                            },
                            "required": ["layer_id", "media_path"]
                        }),
                    },
                    Tool {
                        name: "layer_set_media_time".to_string(),
                        description: Some("Set media playback position in seconds".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "time_seconds": { "type": "number", "minimum": 0.0 }
                            },
                            "required": ["layer_id", "time_seconds"]
                        }),
                    },
                    Tool {
                        name: "layer_set_playback_speed".to_string(),
                        description: Some("Set media playback speed".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "speed": { "type": "number", "description": "1.0 = normal, 0.5 = half, 2.0 = double" }
                            },
                            "required": ["layer_id", "speed"]
                        }),
                    },
                    Tool {
                        name: "layer_set_loop_mode".to_string(),
                        description: Some("Set layer loop mode".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "loop_mode": { "type": "string", "enum": ["none", "loop", "ping-pong"] }
                            },
                            "required": ["layer_id", "loop_mode"]
                        }),
                    },
                    Tool {
                        name: "media_library_list".to_string(),
                        description: Some("List available media in library".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "folder": { "type": "string", "description": "Optional folder filter" }
                            }
                        }),
                    },
                    Tool {
                        name: "media_import".to_string(),
                        description: Some("Import media file into library".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "source_path": { "type": "string" },
                                "destination_folder": { "type": "string" }
                            },
                            "required": ["source_path"]
                        }),
                    },
                    // === Phase 2: Audio Reactivity ===
                    Tool {
                        name: "audio_bind_param".to_string(),
                        description: Some(
                            "Bind audio frequency/beat to a layer parameter".to_string(),
                        ),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "source": { "type": "string", "enum": ["bass", "mid", "high", "beat", "volume"] },
                                "layer_id": { "type": "integer" },
                                "param": { "type": "string", "enum": ["opacity", "scale", "rotation", "hue", "saturation"] },
                                "min": { "type": "number", "default": 0.0 },
                                "max": { "type": "number", "default": 1.0 },
                                "smoothing": { "type": "number", "minimum": 0.0, "maximum": 1.0, "default": 0.5 }
                            },
                            "required": ["source", "layer_id", "param"]
                        }),
                    },
                    Tool {
                        name: "audio_unbind_param".to_string(),
                        description: Some("Remove an audio parameter binding".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "binding_id": { "type": "integer" } },
                            "required": ["binding_id"]
                        }),
                    },
                    Tool {
                        name: "audio_bindings_list".to_string(),
                        description: Some("List all active audio bindings".to_string()),
                        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
                    },
                    Tool {
                        name: "audio_set_sensitivity".to_string(),
                        description: Some("Set sensitivity for a frequency band".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "frequency_band": { "type": "string", "enum": ["bass", "mid", "high"] },
                                "sensitivity": { "type": "number", "minimum": 0.0, "maximum": 2.0 }
                            },
                            "required": ["frequency_band", "sensitivity"]
                        }),
                    },
                    Tool {
                        name: "audio_set_threshold".to_string(),
                        description: Some("Set beat detection threshold".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "threshold": { "type": "number", "minimum": 0.0, "maximum": 1.0 }
                            },
                            "required": ["threshold"]
                        }),
                    },
                    Tool {
                        name: "audio_analysis_config".to_string(),
                        description: Some("Configure audio analysis parameters".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "fft_size": { "type": "integer", "enum": [256, 512, 1024, 2048, 4096] },
                                "smoothing": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
                                "bands": { "type": "integer", "minimum": 3, "maximum": 32 }
                            }
                        }),
                    },
                    // === Phase 3: Effects & Shaders ===
                    Tool {
                        name: "effect_add".to_string(),
                        description: Some("Add an effect to a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "effect_type": { "type": "string", "enum": ["blur", "glow", "chromatic_aberration", "distortion", "color_correction", "edge_detection", "pixelate", "mirror", "kaleidoscope"] }
                            },
                            "required": ["layer_id", "effect_type"]
                        }),
                    },
                    Tool {
                        name: "effect_remove".to_string(),
                        description: Some("Remove an effect from a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "effect_id": { "type": "integer" }
                            },
                            "required": ["layer_id", "effect_id"]
                        }),
                    },
                    Tool {
                        name: "effect_set_param".to_string(),
                        description: Some("Set an effect parameter".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "effect_id": { "type": "integer" },
                                "param_name": { "type": "string" },
                                "value": { "type": "number" }
                            },
                            "required": ["layer_id", "effect_id", "param_name", "value"]
                        }),
                    },
                    Tool {
                        name: "effect_list".to_string(),
                        description: Some("List all available effects".to_string()),
                        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
                    },
                    Tool {
                        name: "effect_chain_get".to_string(),
                        description: Some("Get the effect chain for a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "layer_id": { "type": "integer" } },
                            "required": ["layer_id"]
                        }),
                    },
                    Tool {
                        name: "shader_load".to_string(),
                        description: Some("Load a custom shader for a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "shader_path": { "type": "string" }
                            },
                            "required": ["layer_id", "shader_path"]
                        }),
                    },
                    Tool {
                        name: "shader_set_uniform".to_string(),
                        description: Some("Set a shader uniform value".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "uniform_name": { "type": "string" },
                                "value": { "type": "number" }
                            },
                            "required": ["layer_id", "uniform_name", "value"]
                        }),
                    },
                    // === Phase 4: Timeline & Keyframes ===
                    Tool {
                        name: "timeline_add_keyframe".to_string(),
                        description: Some("Add a keyframe to the timeline".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "param": { "type": "string" },
                                "time": { "type": "number", "description": "Time in seconds" },
                                "value": { "type": "number" },
                                "easing": { "type": "string", "enum": ["linear", "ease-in", "ease-out", "ease-in-out", "bounce"], "default": "linear" }
                            },
                            "required": ["layer_id", "param", "time", "value"]
                        }),
                    },
                    Tool {
                        name: "timeline_remove_keyframe".to_string(),
                        description: Some("Remove a keyframe".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "keyframe_id": { "type": "integer" } },
                            "required": ["keyframe_id"]
                        }),
                    },
                    Tool {
                        name: "timeline_get_keyframes".to_string(),
                        description: Some("Get keyframes for a layer parameter".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "param": { "type": "string" }
                            },
                            "required": ["layer_id", "param"]
                        }),
                    },
                    Tool {
                        name: "timeline_set_duration".to_string(),
                        description: Some("Set the timeline duration".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "duration_seconds": { "type": "number", "minimum": 0.0 } },
                            "required": ["duration_seconds"]
                        }),
                    },
                    Tool {
                        name: "timeline_set_position".to_string(),
                        description: Some("Set the timeline playback position".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "time_seconds": { "type": "number", "minimum": 0.0 } },
                            "required": ["time_seconds"]
                        }),
                    },
                    Tool {
                        name: "timeline_set_loop".to_string(),
                        description: Some("Set timeline loop region".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "start": { "type": "number", "minimum": 0.0 },
                                "end": { "type": "number", "minimum": 0.0 },
                                "enabled": { "type": "boolean" }
                            },
                            "required": ["start", "end", "enabled"]
                        }),
                    },
                    // === Phase 5: Mapping & Scenes ===
                    Tool {
                        name: "surface_create".to_string(),
                        description: Some("Create a new mapping surface".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "surface_type": { "type": "string", "enum": ["quad", "bezier", "mesh", "triangle"] },
                                "corners": { "type": "string", "description": "JSON array of corner points [[x,y], ...]" }
                            },
                            "required": ["surface_type", "corners"]
                        }),
                    },
                    Tool {
                        name: "surface_delete".to_string(),
                        description: Some("Delete a mapping surface".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "surface_id": { "type": "integer" } },
                            "required": ["surface_id"]
                        }),
                    },
                    Tool {
                        name: "surface_set_corners".to_string(),
                        description: Some("Update surface corner positions".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "surface_id": { "type": "integer" },
                                "corners": { "type": "string", "description": "JSON array of corner points" }
                            },
                            "required": ["surface_id", "corners"]
                        }),
                    },
                    Tool {
                        name: "surface_assign_layer".to_string(),
                        description: Some("Assign a layer to a surface".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "surface_id": { "type": "integer" },
                                "layer_id": { "type": "integer" }
                            },
                            "required": ["surface_id", "layer_id"]
                        }),
                    },
                    Tool {
                        name: "mask_create".to_string(),
                        description: Some("Create a mask for a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "mask_type": { "type": "string", "enum": ["polygon", "ellipse", "rectangle", "bezier"] },
                                "points": { "type": "string", "description": "JSON array of points" }
                            },
                            "required": ["layer_id", "mask_type", "points"]
                        }),
                    },
                    Tool {
                        name: "mask_edit".to_string(),
                        description: Some("Edit mask points".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "mask_id": { "type": "integer" },
                                "points": { "type": "string", "description": "JSON array of points" }
                            },
                            "required": ["mask_id", "points"]
                        }),
                    },
                    Tool {
                        name: "scene_create".to_string(),
                        description: Some("Create a new scene".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": { "name": { "type": "string" } },
                            "required": ["name"]
                        }),
                    },
                    Tool {
                        name: "scene_switch".to_string(),
                        description: Some("Switch to a different scene".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "scene_id": { "type": "integer" },
                                "transition": { "type": "string", "enum": ["cut", "fade", "wipe", "dissolve"], "default": "fade" },
                                "duration": { "type": "number", "default": 1.0 }
                            },
                            "required": ["scene_id"]
                        }),
                    },
                    Tool {
                        name: "scene_list".to_string(),
                        description: Some("List all scenes".to_string()),
                        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
                    },
                    Tool {
                        name: "preset_save".to_string(),
                        description: Some("Save current state as a preset".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "scope": { "type": "string", "enum": ["layer", "effect", "project"], "default": "layer" }
                            },
                            "required": ["name"]
                        }),
                    },
                    Tool {
                        name: "preset_load".to_string(),
                        description: Some("Load a saved preset".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "preset_id": { "type": "integer" },
                                "target": { "type": "string", "description": "Optional target (e.g., layer_id)" }
                            },
                            "required": ["preset_id"]
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

                match params.name.as_str() {
                    "project_save" => {
                        if let Some(args) = params.arguments {
                            if let Some(path_val) = args.get("path") {
                                if let Some(path_str) = path_val.as_str() {
                                    match validate_path_with_extensions(
                                        path_str,
                                        &["mapmap", "json", "ron", "mflow"],
                                    ) {
                                        Ok(path) => {
                                            if let Some(sender) = &self.action_sender {
                                                let _ = sender
                                                    .send(crate::McpAction::SaveProject(path));
                                            }
                                            return Some(success_response(
                                                id,
                                                serde_json::json!({"status":"queued"}),
                                            ));
                                        }
                                        Err(e) => {
                                            return Some(error_response(
                                                id,
                                                -32602,
                                                &format!("Invalid path: {}", e),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing path"))
                    }
                    "project_load" => {
                        if let Some(args) = params.arguments {
                            if let Some(path_val) = args.get("path") {
                                if let Some(path_str) = path_val.as_str() {
                                    match validate_path_with_extensions(
                                        path_str,
                                        &["mapmap", "json", "ron", "mflow"],
                                    ) {
                                        Ok(path) => {
                                            if let Some(sender) = &self.action_sender {
                                                let _ = sender
                                                    .send(crate::McpAction::LoadProject(path));
                                            }
                                            return Some(success_response(
                                                id,
                                                serde_json::json!({"status":"queued"}),
                                            ));
                                        }
                                        Err(e) => {
                                            return Some(error_response(
                                                id,
                                                -32602,
                                                &format!("Invalid path: {}", e),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing path"))
                    }
                    "layer_create" => {
                        if let Some(args) = params.arguments {
                            if let Some(name_val) = args.get("name") {
                                if let Some(name_str) = name_val.as_str() {
                                    if let Some(sender) = &self.action_sender {
                                        let _ = sender
                                            .send(crate::McpAction::AddLayer(name_str.to_string()));
                                    }
                                    return Some(success_response(
                                        id,
                                        serde_json::json!({"status":"queued"}),
                                    ));
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing layer name"))
                    }
                    "layer_delete" => {
                        if let Some(args) = params.arguments {
                            if let Some(layer_id_val) = args.get("layer_id") {
                                if let Some(layer_id) = layer_id_val.as_u64() {
                                    if let Some(sender) = &self.action_sender {
                                        let _ =
                                            sender.send(crate::McpAction::RemoveLayer(layer_id));
                                    }
                                    return Some(success_response(
                                        id,
                                        serde_json::json!({"status":"queued"}),
                                    ));
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing layer_id"))
                    }
                    "layer_set_opacity" => {
                        if let Some(args) = params.arguments {
                            if let (Some(layer_id_val), Some(opacity_val)) =
                                (args.get("layer_id"), args.get("opacity"))
                            {
                                if let (Some(layer_id), Some(opacity)) =
                                    (layer_id_val.as_u64(), opacity_val.as_f64())
                                {
                                    if let Some(sender) = &self.action_sender {
                                        let _ = sender.send(crate::McpAction::SetLayerOpacity(
                                            layer_id,
                                            opacity as f32,
                                        ));
                                    }
                                    return Some(success_response(
                                        id,
                                        serde_json::json!({"status": "queued"}),
                                    ));
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing arguments"))
                    }
                    "layer_set_visibility" => {
                        if let Some(args) = params.arguments {
                            if let (Some(layer_id_val), Some(visible_val)) =
                                (args.get("layer_id"), args.get("visible"))
                            {
                                if let (Some(layer_id), Some(visible)) =
                                    (layer_id_val.as_u64(), visible_val.as_bool())
                                {
                                    if let Some(sender) = &self.action_sender {
                                        let _ = sender.send(crate::McpAction::SetLayerVisibility(
                                            layer_id, visible,
                                        ));
                                    }
                                    return Some(success_response(
                                        id,
                                        serde_json::json!({"status": "queued"}),
                                    ));
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing arguments"))
                    }
                    "cue_trigger" => {
                        if let Some(args) = params.arguments {
                            if let Some(cue_id_val) = args.get("cue_id") {
                                if let Some(cue_id) = cue_id_val.as_u64() {
                                    if let Some(sender) = &self.action_sender {
                                        let _ = sender.send(crate::McpAction::TriggerCue(cue_id));
                                    }
                                    return Some(success_response(
                                        id,
                                        serde_json::json!({"status":"queued"}),
                                    ));
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing cue_id"))
                    }
                    "cue_next" => {
                        if let Some(sender) = &self.action_sender {
                            let _ = sender.send(crate::McpAction::NextCue);
                        }
                        Some(success_response(id, serde_json::json!({"status":"queued"})))
                    }
                    "cue_previous" => {
                        if let Some(sender) = &self.action_sender {
                            let _ = sender.send(crate::McpAction::PrevCue);
                        }
                        Some(success_response(id, serde_json::json!({"status":"queued"})))
                    }
                    "media_play" => {
                        if let Some(sender) = &self.action_sender {
                            let _ = sender.send(crate::McpAction::MediaPlay);
                        }
                        Some(success_response(id, serde_json::json!({"status":"queued"})))
                    }
                    "media_pause" => {
                        if let Some(sender) = &self.action_sender {
                            let _ = sender.send(crate::McpAction::MediaPause);
                        }
                        Some(success_response(id, serde_json::json!({"status":"queued"})))
                    }
                    "media_stop" => {
                        if let Some(sender) = &self.action_sender {
                            let _ = sender.send(crate::McpAction::MediaStop);
                        }
                        Some(success_response(id, serde_json::json!({"status":"queued"})))
                    }
                    "layer_list" => {
                        // Mock empty list for now
                        let layers: Vec<String> = vec![];
                        Some(success_response(id, serde_json::json!({"layers": layers})))
                    }
                    "send_osc" => {
                        if let Some(args) = params.arguments {
                            self.handle_send_osc(id, &serde_json::to_value(args).unwrap())
                        } else {
                            Some(error_response(id, -32602, "Missing arguments for send_osc"))
                        }
                    }
                    "layer_load_media" => {
                        if let Some(args) = params.arguments {
                            if let (Some(layer_id_val), Some(media_path_val)) =
                                (args.get("layer_id"), args.get("media_path"))
                            {
                                if let (Some(layer_id), Some(media_path_str)) =
                                    (layer_id_val.as_u64(), media_path_val.as_str())
                                {
                                    match validate_path_with_extensions(
                                        media_path_str,
                                        &[
                                            "mp4", "mov", "avi", "mkv", "webm", "png", "jpg", "jpeg", "webp",
                                        ],
                                    ) {
                                        Ok(path) => {
                                            if let Some(sender) = &self.action_sender {
                                                let _ = sender.send(crate::McpAction::LayerLoadMedia(
                                                    layer_id, path,
                                                ));
                                            }
                                            return Some(success_response(
                                                id,
                                                serde_json::json!({"status":"queued"}),
                                            ));
                                        }
                                        Err(e) => {
                                            return Some(error_response(
                                                id,
                                                -32602,
                                                &format!("Invalid media path: {}", e),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing arguments"))
                    }
                    "shader_load" => {
                        if let Some(args) = params.arguments {
                            if let (Some(layer_id_val), Some(shader_path_val)) =
                                (args.get("layer_id"), args.get("shader_path"))
                            {
                                if let (Some(layer_id), Some(shader_path_str)) =
                                    (layer_id_val.as_u64(), shader_path_val.as_str())
                                {
                                    match validate_path_with_extensions(
                                        shader_path_str,
                                        &["wgsl", "glsl", "frag", "vert"],
                                    ) {
                                        Ok(path) => {
                                            if let Some(sender) = &self.action_sender {
                                                let _ = sender.send(crate::McpAction::ShaderLoad(
                                                    layer_id, path,
                                                ));
                                            }
                                            return Some(success_response(
                                                id,
                                                serde_json::json!({"status":"queued"}),
                                            ));
                                        }
                                        Err(e) => {
                                            return Some(error_response(
                                                id,
                                                -32602,
                                                &format!("Invalid shader path: {}", e),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing arguments"))
                    }
                    _ => Some(error_response(id, -32601, "Tool not found")),
                }
            }
            _ => Some(error_response(id, -32601, "Method not found")),
        }
    }

    #[allow(dead_code)]
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
        if let McpAction::AddLayer(name) = action {
            assert_eq!(name, "Test Layer");
        } else {
            panic!("Expected AddLayer action");
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
        if let McpAction::RemoveLayer(id) = action {
            assert_eq!(id, 42);
        } else {
            panic!("Expected RemoveLayer action");
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
        if let McpAction::TriggerCue(id) = action {
            assert_eq!(id, 5);
        } else {
            panic!("Expected TriggerCue action");
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
        if let McpAction::SaveProject(path) = action {
            assert_eq!(path.to_str().unwrap(), "test.mapmap");
        } else {
            panic!("Expected SaveProject action");
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
        if let McpAction::LoadProject(path) = action {
            assert_eq!(path.to_str().unwrap(), "other.mapmap");
        } else {
            panic!("Expected LoadProject action");
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
        assert!(error.message.contains("Path traversal"));

        // Verify NO action was sent
        assert!(rx.try_recv().is_err());

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
        if let McpAction::SaveProject(path) = valid_action {
            assert_eq!(path.to_str().unwrap(), "good_project.mapmap");
        } else {
            panic!("Expected SaveProject action");
        }
    }

    #[tokio::test]
    async fn test_security_extensions() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        // Test project_save with invalid extension
        let invalid_save = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "project_save",
                "arguments": {
                    "path": "malware.sh"
                }
            }
        });
        let resp = server.handle_request(&invalid_save.to_string()).await.unwrap();
        assert!(resp.error.is_some(), "Should reject .sh extension");
        let error = resp.error.unwrap();
        assert!(error.message.contains("Extension 'sh' is not allowed"));
        assert!(rx.try_recv().is_err(), "Should not send action");

        // Test project_save with valid extension
        let valid_save = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "project_save",
                "arguments": {
                    "path": "project.mapmap"
                }
            }
        });
        let resp = server.handle_request(&valid_save.to_string()).await.unwrap();
        assert!(resp.error.is_none());
        assert!(matches!(rx.try_recv().unwrap(), McpAction::SaveProject(_)));

        // Test layer_load_media with invalid extension
        let invalid_media = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "layer_load_media",
                "arguments": {
                    "layer_id": 1,
                    "media_path": "script.txt"
                }
            }
        });
        let resp = server.handle_request(&invalid_media.to_string()).await.unwrap();
        assert!(resp.error.is_some(), "Should reject .txt for media");
        let error = resp.error.unwrap();
        assert!(error.message.contains("Extension 'txt' is not allowed"));

        // Test layer_load_media with valid extension
        let valid_media = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "layer_load_media",
                "arguments": {
                    "layer_id": 1,
                    "media_path": "video.mp4"
                }
            }
        });
        let resp = server.handle_request(&valid_media.to_string()).await.unwrap();
        assert!(resp.error.is_none());
        if let McpAction::LayerLoadMedia(id, path) = rx.try_recv().unwrap() {
            assert_eq!(id, 1);
            assert_eq!(path.to_str().unwrap(), "video.mp4");
        } else {
            panic!("Expected LayerLoadMedia");
        }
    }
}
