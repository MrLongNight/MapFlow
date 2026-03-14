use crate::protocol::*;

use anyhow::Result;
use std::path::PathBuf;
use tracing::error;

/// Validate that a path is safe for file operations
pub fn validate_path_with_extensions(
    path_str: &str,
    allowed_extensions: &[&str],
) -> Result<PathBuf, String> {
    let path = PathBuf::from(path_str);

    if path.is_absolute() {
        return Err("Absolute paths are not allowed".to_string());
    }

    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err("Path traversal (..) is not allowed".to_string());
        }
    }

    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            if !allowed_extensions.contains(&ext_str.to_lowercase().as_str()) {
                return Err(format!(
                    "Extension '{}' is not allowed. Allowed: {:?}",
                    ext_str, allowed_extensions
                ));
            }
        } else {
            return Err("Invalid file extension".to_string());
        }
    } else {
        return Err("File must have an extension".to_string());
    }

    Ok(path)
}

pub fn handle_tool_call(
    server: &crate::server::McpServer,
    id: Option<serde_json::Value>,
    params: CallToolParams,
) -> Option<JsonRpcResponse> {
    match params.name.as_str() {
        "Application.CaptureScreenshot" => {
            if let Some(args) = params.arguments {
                if let Some(name_val) = args.get("test_name") {
                    if let Some(test_name) = name_val.as_str() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::ApplicationCaptureScreenshot(
                                    test_name.to_string(),
                                ))
                            {
                                error!("Failed to send ApplicationCaptureScreenshot action: {}", e);
                                return Some(crate::server::error_response(
                                    id,
                                    -32603,
                                    "Internal error: Failed to send action",
                                ));
                            }
                        }
                        return Some(crate::server::success_response(
                            id,
                            serde_json::json!({"status": "queued", "message": format!("Capture initiated for test_name '{}'", test_name)}),
                        ));
                    }
                }
            }
            Some(crate::server::error_response(
                id,
                -32602,
                "Missing test_name",
            ))
        }
        "project_save" => {
            if let Some(args) = params.arguments {
                if let Some(path_val) = args.get("path") {
                    if let Some(path_str) = path_val.as_str() {
                        match crate::tools::handlers::validate_path_with_extensions(
                            path_str,
                            &["mapmap", "json"],
                        ) {
                            Ok(path) => {
                                if let Some(sender) = &server.action_sender {
                                    if let Err(e) = sender.send(crate::McpAction::SaveProject(path))
                                    {
                                        error!("Failed to send SaveProject action: {}", e);
                                        return Some(crate::server::error_response(
                                            id,
                                            -32603,
                                            "Internal error: Failed to send action",
                                        ));
                                    }
                                }
                                return Some(crate::server::success_response(
                                    id,
                                    serde_json::json!({"status":"queued"}),
                                ));
                            }
                            Err(e) => {
                                return Some(crate::server::error_response(
                                    id,
                                    -32602,
                                    &format!("Invalid path: {}", e),
                                ));
                            }
                        }
                    }
                }
            }
            Some(crate::server::error_response(id, -32602, "Missing path"))
        }
        "project_load" => {
            if let Some(args) = params.arguments {
                if let Some(path_val) = args.get("path") {
                    if let Some(path_str) = path_val.as_str() {
                        match crate::tools::handlers::validate_path_with_extensions(
                            path_str,
                            &["mapmap", "json"],
                        ) {
                            Ok(path) => {
                                if let Some(sender) = &server.action_sender {
                                    if let Err(e) = sender.send(crate::McpAction::LoadProject(path))
                                    {
                                        error!("Failed to send LoadProject action: {}", e);
                                        return Some(crate::server::error_response(
                                            id,
                                            -32603,
                                            "Internal error: Failed to send action",
                                        ));
                                    }
                                }
                                return Some(crate::server::success_response(
                                    id,
                                    serde_json::json!({"status":"queued"}),
                                ));
                            }
                            Err(e) => {
                                return Some(crate::server::error_response(
                                    id,
                                    -32602,
                                    &format!("Invalid path: {}", e),
                                ));
                            }
                        }
                    }
                }
            }
            Some(crate::server::error_response(id, -32602, "Missing path"))
        }
        "layer_create" => {
            if let Some(args) = params.arguments {
                if let Some(name_val) = args.get("name") {
                    if let Some(name_str) = name_val.as_str() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::AddLayer(name_str.to_string()))
                            {
                                error!("Failed to send AddLayer action: {}", e);
                                return Some(crate::server::error_response(
                                    id,
                                    -32603,
                                    "Internal error: Failed to send action",
                                ));
                            }
                        }
                        return Some(crate::server::success_response(
                            id,
                            serde_json::json!({"status":"queued"}),
                        ));
                    }
                }
            }
            Some(crate::server::error_response(
                id,
                -32602,
                "Missing layer name",
            ))
        }
        "layer_delete" => {
            if let Some(args) = params.arguments {
                if let Some(layer_id_val) = args.get("layer_id") {
                    if let Some(layer_id) = layer_id_val.as_u64() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::RemoveLayer(layer_id)) {
                                error!("Failed to send RemoveLayer action: {}", e);
                                return Some(crate::server::error_response(
                                    id,
                                    -32603,
                                    "Internal error: Failed to send action",
                                ));
                            }
                        }
                        return Some(crate::server::success_response(
                            id,
                            serde_json::json!({"status":"queued"}),
                        ));
                    }
                }
            }
            Some(crate::server::error_response(
                id,
                -32602,
                "Missing layer_id",
            ))
        }
        "layer_set_opacity" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(opacity_val)) =
                    (args.get("layer_id"), args.get("opacity"))
                {
                    if let (Some(layer_id), Some(opacity)) =
                        (layer_id_val.as_u64(), opacity_val.as_f64())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender
                                .send(crate::McpAction::SetLayerOpacity(layer_id, opacity as f32))
                            {
                                error!("Failed to send SetLayerOpacity action: {}", e);
                                return Some(crate::server::error_response(
                                    id,
                                    -32603,
                                    "Internal error: Failed to send action",
                                ));
                            }
                        }
                        return Some(crate::server::success_response(
                            id,
                            serde_json::json!({"status": "queued"}),
                        ));
                    }
                }
            }
            Some(crate::server::error_response(
                id,
                -32602,
                "Missing arguments",
            ))
        }
        "layer_set_visibility" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(visible_val)) =
                    (args.get("layer_id"), args.get("visible"))
                {
                    if let (Some(layer_id), Some(visible)) =
                        (layer_id_val.as_u64(), visible_val.as_bool())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::SetLayerVisibility(layer_id, visible))
                            {
                                error!("Failed to send SetLayerVisibility action: {}", e);
                                return Some(crate::server::error_response(
                                    id,
                                    -32603,
                                    "Internal error: Failed to send action",
                                ));
                            }
                        }
                        return Some(crate::server::success_response(
                            id,
                            serde_json::json!({"status": "queued"}),
                        ));
                    }
                }
            }
            Some(crate::server::error_response(
                id,
                -32602,
                "Missing arguments",
            ))
        }
        "cue_trigger" => {
            if let Some(args) = params.arguments {
                if let Some(cue_id_val) = args.get("cue_id") {
                    if let Some(cue_id) = cue_id_val.as_u64() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::TriggerCue(cue_id)) {
                                error!("Failed to send TriggerCue action: {}", e);
                                return Some(crate::server::error_response(
                                    id,
                                    -32603,
                                    "Internal error: Failed to send action",
                                ));
                            }
                        }
                        return Some(crate::server::success_response(
                            id,
                            serde_json::json!({"status":"queued"}),
                        ));
                    }
                }
            }
            Some(crate::server::error_response(id, -32602, "Missing cue_id"))
        }
        "cue_next" => {
            if let Some(sender) = &server.action_sender {
                if let Err(e) = sender.send(crate::McpAction::NextCue) {
                    error!("Failed to send NextCue action: {}", e);
                    return Some(crate::server::error_response(
                        id,
                        -32603,
                        "Internal error: Failed to send action",
                    ));
                }
            }
            Some(crate::server::success_response(
                id,
                serde_json::json!({"status":"queued"}),
            ))
        }
        "cue_previous" => {
            if let Some(sender) = &server.action_sender {
                if let Err(e) = sender.send(crate::McpAction::PrevCue) {
                    error!("Failed to send PrevCue action: {}", e);
                    return Some(crate::server::error_response(
                        id,
                        -32603,
                        "Internal error: Failed to send action",
                    ));
                }
            }
            Some(crate::server::success_response(
                id,
                serde_json::json!({"status":"queued"}),
            ))
        }
        "media_play" => {
            if let Some(sender) = &server.action_sender {
                if let Err(e) = sender.send(crate::McpAction::MediaPlay) {
                    error!("Failed to send MediaPlay action: {}", e);
                    return Some(crate::server::error_response(
                        id,
                        -32603,
                        "Internal error: Failed to send action",
                    ));
                }
            }
            Some(crate::server::success_response(
                id,
                serde_json::json!({"status":"queued"}),
            ))
        }
        "media_pause" => {
            if let Some(sender) = &server.action_sender {
                if let Err(e) = sender.send(crate::McpAction::MediaPause) {
                    error!("Failed to send MediaPause action: {}", e);
                    return Some(crate::server::error_response(
                        id,
                        -32603,
                        "Internal error: Failed to send action",
                    ));
                }
            }
            Some(crate::server::success_response(
                id,
                serde_json::json!({"status":"queued"}),
            ))
        }
        "media_stop" => {
            if let Some(sender) = &server.action_sender {
                if let Err(e) = sender.send(crate::McpAction::MediaStop) {
                    error!("Failed to send MediaStop action: {}", e);
                    return Some(crate::server::error_response(
                        id,
                        -32603,
                        "Internal error: Failed to send action",
                    ));
                }
            }
            Some(crate::server::success_response(
                id,
                serde_json::json!({"status":"queued"}),
            ))
        }
        "layer_list" => {
            // Mock empty list for now
            let layers: Vec<String> = vec![];
            Some(crate::server::success_response(
                id,
                serde_json::json!({"layers": layers}),
            ))
        }
        "send_osc" => {
            if let Some(args) = params.arguments {
                match serde_json::to_value(args) {
                    Ok(val) => server.handle_send_osc(id, &val),
                    Err(e) => {
                        error!("Failed to serialize arguments for send_osc: {}", e);
                        Some(crate::server::error_response(id, -32603, "Internal error"))
                    }
                }
            } else {
                Some(crate::server::error_response(
                    id,
                    -32602,
                    "Missing arguments for send_osc",
                ))
            }
        }
        _ => Some(crate::server::error_response(id, -32601, "Tool not found")),
    }
}
