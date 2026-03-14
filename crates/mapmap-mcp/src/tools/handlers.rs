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

        "layer_load_media" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(media_path_val)) =
                    (args.get("layer_id"), args.get("media_path"))
                {
                    if let (Some(layer_id), Some(media_path_str)) =
                        (layer_id_val.as_u64(), media_path_val.as_str())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::LayerLoadMedia(
                                layer_id,
                                std::path::PathBuf::from(media_path_str),
                            )) {
                                error!("Failed to send LayerLoadMedia action: {}", e);
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
        "layer_set_media_time" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(time_val)) =
                    (args.get("layer_id"), args.get("time_seconds"))
                {
                    if let (Some(layer_id), Some(time)) = (layer_id_val.as_u64(), time_val.as_f64())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::LayerSetMediaTime(layer_id, time))
                            {
                                error!("Failed to send LayerSetMediaTime action: {}", e);
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
        "layer_set_playback_speed" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(speed_val)) =
                    (args.get("layer_id"), args.get("speed"))
                {
                    if let (Some(layer_id), Some(speed)) =
                        (layer_id_val.as_u64(), speed_val.as_f64())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::LayerSetPlaybackSpeed(
                                layer_id,
                                speed as f32,
                            )) {
                                error!("Failed to send LayerSetPlaybackSpeed action: {}", e);
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
        "layer_set_loop_mode" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(loop_val)) =
                    (args.get("layer_id"), args.get("loop_mode"))
                {
                    if let (Some(layer_id), Some(loop_mode)) =
                        (layer_id_val.as_u64(), loop_val.as_str())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::LayerSetLoopMode(
                                layer_id,
                                loop_mode.to_string(),
                            )) {
                                error!("Failed to send LayerSetLoopMode action: {}", e);
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
        "audio_bind_param" => {
            if let Some(args) = params.arguments {
                if let (Some(source_val), Some(layer_id_val), Some(param_val)) =
                    (args.get("source"), args.get("layer_id"), args.get("param"))
                {
                    if let (Some(source), Some(layer_id), Some(param)) = (
                        source_val.as_str(),
                        layer_id_val.as_u64(),
                        param_val.as_str(),
                    ) {
                        let min = args.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                        let max = args.get("max").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                        let smoothing = args
                            .get("smoothing")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.5) as f32;

                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::AudioBindParam {
                                source: source.to_string(),
                                layer_id,
                                param: param.to_string(),
                                min,
                                max,
                                smoothing,
                            }) {
                                error!("Failed to send AudioBindParam action: {}", e);
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
        "audio_set_sensitivity" => {
            if let Some(args) = params.arguments {
                if let (Some(band_val), Some(sens_val)) =
                    (args.get("frequency_band"), args.get("sensitivity"))
                {
                    if let (Some(band), Some(sens)) = (band_val.as_str(), sens_val.as_f64()) {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::AudioSetSensitivity(
                                band.to_string(),
                                sens as f32,
                            )) {
                                error!("Failed to send AudioSetSensitivity action: {}", e);
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
        "audio_set_threshold" => {
            if let Some(args) = params.arguments {
                if let Some(thresh_val) = args.get("threshold") {
                    if let Some(thresh) = thresh_val.as_f64() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::AudioSetThreshold(thresh as f32))
                            {
                                error!("Failed to send AudioSetThreshold action: {}", e);
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
        "effect_add" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(effect_val)) =
                    (args.get("layer_id"), args.get("effect_type"))
                {
                    if let (Some(layer_id), Some(effect_type)) =
                        (layer_id_val.as_u64(), effect_val.as_str())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::EffectAdd(
                                layer_id,
                                effect_type.to_string(),
                            )) {
                                error!("Failed to send EffectAdd action: {}", e);
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
        "effect_remove" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(effect_id_val)) =
                    (args.get("layer_id"), args.get("effect_id"))
                {
                    if let (Some(layer_id), Some(effect_id)) =
                        (layer_id_val.as_u64(), effect_id_val.as_u64())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::EffectRemove(layer_id, effect_id))
                            {
                                error!("Failed to send EffectRemove action: {}", e);
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
        "effect_set_param" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(effect_id_val), Some(param_val), Some(val_val)) = (
                    args.get("layer_id"),
                    args.get("effect_id"),
                    args.get("param_name"),
                    args.get("value"),
                ) {
                    if let (Some(layer_id), Some(effect_id), Some(param_name), Some(value)) = (
                        layer_id_val.as_u64(),
                        effect_id_val.as_u64(),
                        param_val.as_str(),
                        val_val.as_f64(),
                    ) {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::EffectSetParam(
                                layer_id,
                                effect_id,
                                param_name.to_string(),
                                value as f32,
                            )) {
                                error!("Failed to send EffectSetParam action: {}", e);
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
        "shader_load" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(path_val)) =
                    (args.get("layer_id"), args.get("shader_path"))
                {
                    if let (Some(layer_id), Some(path_str)) =
                        (layer_id_val.as_u64(), path_val.as_str())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::ShaderLoad(
                                layer_id,
                                std::path::PathBuf::from(path_str),
                            )) {
                                error!("Failed to send ShaderLoad action: {}", e);
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
        "timeline_add_keyframe" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(param_val), Some(time_val), Some(value_val)) = (
                    args.get("layer_id"),
                    args.get("param"),
                    args.get("time"),
                    args.get("value"),
                ) {
                    if let (Some(layer_id), Some(param), Some(time), Some(value)) = (
                        layer_id_val.as_u64(),
                        param_val.as_str(),
                        time_val.as_f64(),
                        value_val.as_f64(),
                    ) {
                        let easing = args
                            .get("easing")
                            .and_then(|v| v.as_str())
                            .unwrap_or("linear");

                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::TimelineAddKeyframe {
                                layer_id,
                                param: param.to_string(),
                                time,
                                value: value as f32,
                                easing: easing.to_string(),
                            }) {
                                error!("Failed to send TimelineAddKeyframe action: {}", e);
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
        "timeline_remove_keyframe" => {
            if let Some(args) = params.arguments {
                if let Some(key_val) = args.get("keyframe_id") {
                    if let Some(keyframe_id) = key_val.as_u64() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::TimelineRemoveKeyframe(keyframe_id))
                            {
                                error!("Failed to send TimelineRemoveKeyframe action: {}", e);
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
        "timeline_set_position" => {
            if let Some(args) = params.arguments {
                if let Some(time_val) = args.get("time_seconds") {
                    if let Some(time) = time_val.as_f64() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::TimelineSetPosition(time))
                            {
                                error!("Failed to send TimelineSetPosition action: {}", e);
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
        "surface_create" => {
            if let Some(args) = params.arguments {
                if let (Some(type_val), Some(corners_val)) =
                    (args.get("surface_type"), args.get("corners"))
                {
                    if let (Some(surface_type), Some(corners)) =
                        (type_val.as_str(), corners_val.as_str())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::SurfaceCreate(
                                surface_type.to_string(),
                                corners.to_string(),
                            )) {
                                error!("Failed to send SurfaceCreate action: {}", e);
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
        "surface_set_corners" => {
            if let Some(args) = params.arguments {
                if let (Some(id_val), Some(corners_val)) =
                    (args.get("surface_id"), args.get("corners"))
                {
                    if let (Some(surface_id), Some(corners)) =
                        (id_val.as_u64(), corners_val.as_str())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::SurfaceSetCorners(
                                surface_id,
                                corners.to_string(),
                            )) {
                                error!("Failed to send SurfaceSetCorners action: {}", e);
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
        "mask_create" => {
            if let Some(args) = params.arguments {
                if let (Some(layer_id_val), Some(type_val), Some(points_val)) = (
                    args.get("layer_id"),
                    args.get("mask_type"),
                    args.get("points"),
                ) {
                    if let (Some(layer_id), Some(mask_type), Some(points)) = (
                        layer_id_val.as_u64(),
                        type_val.as_str(),
                        points_val.as_str(),
                    ) {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::MaskCreate(
                                layer_id,
                                mask_type.to_string(),
                                points.to_string(),
                            )) {
                                error!("Failed to send MaskCreate action: {}", e);
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

        "set_module_source_path" => {
            if let Some(args) = params.arguments {
                if let (Some(m_val), Some(p_val), Some(path_val)) =
                    (args.get("module_id"), args.get("part_id"), args.get("path"))
                {
                    if let (Some(module_id), Some(part_id), Some(path_str)) =
                        (m_val.as_u64(), p_val.as_u64(), path_val.as_str())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::SetModuleSourcePath(
                                module_id,
                                part_id,
                                std::path::PathBuf::from(path_str),
                            )) {
                                error!("Failed to send SetModuleSourcePath action: {}", e);
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
        "media_library_list" => {
            let filter = params
                .arguments
                .as_ref()
                .and_then(|a| a.get("folder"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if let Some(sender) = &server.action_sender {
                if let Err(e) = sender.send(crate::McpAction::MediaLibraryList(filter)) {
                    error!("Failed to send MediaLibraryList action: {}", e);
                    return Some(crate::server::error_response(
                        id,
                        -32603,
                        "Internal error: Failed to send action",
                    ));
                }
            }
            Some(crate::server::success_response(
                id,
                serde_json::json!({"status": "queued"}),
            ))
        }
        "media_import" => {
            if let Some(args) = params.arguments {
                if let Some(src_val) = args.get("source_path") {
                    if let Some(src) = src_val.as_str() {
                        let dest = args
                            .get("destination_folder")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::MediaImport(
                                std::path::PathBuf::from(src),
                                dest,
                            )) {
                                error!("Failed to send MediaImport action: {}", e);
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
        "audio_unbind_param" => {
            if let Some(args) = params.arguments {
                if let Some(b_val) = args.get("binding_id") {
                    if let Some(binding_id) = b_val.as_u64() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::AudioUnbindParam(binding_id))
                            {
                                error!("Failed to send AudioUnbindParam action: {}", e);
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
        "effect_chain_get" => {
            if let Some(args) = params.arguments {
                if let Some(l_val) = args.get("layer_id") {
                    if let Some(layer_id) = l_val.as_u64() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::EffectChainGet(layer_id))
                            {
                                error!("Failed to send EffectChainGet action: {}", e);
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
        "shader_set_uniform" => {
            if let Some(args) = params.arguments {
                if let (Some(l_val), Some(u_val), Some(v_val)) = (
                    args.get("layer_id"),
                    args.get("uniform_name"),
                    args.get("value"),
                ) {
                    if let (Some(layer_id), Some(uniform), Some(value)) =
                        (l_val.as_u64(), u_val.as_str(), v_val.as_f64())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::ShaderSetUniform(
                                layer_id,
                                uniform.to_string(),
                                value as f32,
                            )) {
                                error!("Failed to send ShaderSetUniform action: {}", e);
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
        "timeline_get_keyframes" => {
            if let Some(args) = params.arguments {
                if let (Some(l_val), Some(p_val)) = (args.get("layer_id"), args.get("param")) {
                    if let (Some(layer_id), Some(param)) = (l_val.as_u64(), p_val.as_str()) {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::TimelineGetKeyframes(
                                layer_id,
                                param.to_string(),
                            )) {
                                error!("Failed to send TimelineGetKeyframes action: {}", e);
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
        "timeline_set_duration" => {
            if let Some(args) = params.arguments {
                if let Some(d_val) = args.get("duration_seconds") {
                    if let Some(duration) = d_val.as_f64() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::TimelineSetDuration(duration))
                            {
                                error!("Failed to send TimelineSetDuration action: {}", e);
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
        "surface_delete" => {
            if let Some(args) = params.arguments {
                if let Some(s_val) = args.get("surface_id") {
                    if let Some(surface_id) = s_val.as_u64() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::SurfaceDelete(surface_id))
                            {
                                error!("Failed to send SurfaceDelete action: {}", e);
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
        "surface_assign_layer" => {
            if let Some(args) = params.arguments {
                if let (Some(s_val), Some(l_val)) = (args.get("surface_id"), args.get("layer_id")) {
                    if let (Some(surface_id), Some(layer_id)) = (s_val.as_u64(), l_val.as_u64()) {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender
                                .send(crate::McpAction::SurfaceAssignLayer(surface_id, layer_id))
                            {
                                error!("Failed to send SurfaceAssignLayer action: {}", e);
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
        "mask_edit" => {
            if let Some(args) = params.arguments {
                if let (Some(m_val), Some(p_val)) = (args.get("mask_id"), args.get("points")) {
                    if let (Some(mask_id), Some(points)) = (m_val.as_u64(), p_val.as_str()) {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::MaskEdit(mask_id, points.to_string()))
                            {
                                error!("Failed to send MaskEdit action: {}", e);
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
        "scene_create" => {
            if let Some(args) = params.arguments {
                if let Some(n_val) = args.get("name") {
                    if let Some(name) = n_val.as_str() {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::SceneCreate(name.to_string()))
                            {
                                error!("Failed to send SceneCreate action: {}", e);
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
        "scene_switch" => {
            if let Some(args) = params.arguments {
                if let Some(s_val) = args.get("scene_id") {
                    if let Some(scene_id) = s_val.as_u64() {
                        let transition = args
                            .get("transition")
                            .and_then(|v| v.as_str())
                            .unwrap_or("fade");
                        let duration = args.get("duration").and_then(|v| v.as_f64()).unwrap_or(1.0);
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::SceneSwitch(
                                scene_id,
                                transition.to_string(),
                                duration as f32,
                            )) {
                                error!("Failed to send SceneSwitch action: {}", e);
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
        "preset_save" => {
            if let Some(args) = params.arguments {
                if let Some(n_val) = args.get("name") {
                    if let Some(name) = n_val.as_str() {
                        let scope = args
                            .get("scope")
                            .and_then(|v| v.as_str())
                            .unwrap_or("layer");
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::PresetSave(
                                name.to_string(),
                                scope.to_string(),
                            )) {
                                error!("Failed to send PresetSave action: {}", e);
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
        "preset_load" => {
            if let Some(args) = params.arguments {
                if let Some(p_val) = args.get("preset_id") {
                    if let Some(preset_id) = p_val.as_u64() {
                        let target = args
                            .get("target")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) =
                                sender.send(crate::McpAction::PresetLoad(preset_id, target))
                            {
                                error!("Failed to send PresetLoad action: {}", e);
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
        "audio_analysis_config" => {
            if let Some(args) = params.arguments {
                let fft_size = args
                    .get("fft_size")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1024) as u32;
                let smoothing = args
                    .get("smoothing")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5) as f32;
                let bands = args.get("bands").and_then(|v| v.as_u64()).unwrap_or(8) as u32;
                if let Some(sender) = &server.action_sender {
                    if let Err(e) = sender.send(crate::McpAction::AudioAnalysisConfig {
                        fft_size,
                        smoothing,
                        bands,
                    }) {
                        error!("Failed to send AudioAnalysisConfig action: {}", e);
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
            Some(crate::server::error_response(
                id,
                -32602,
                "Missing arguments",
            ))
        }
        "timeline_set_loop" => {
            if let Some(args) = params.arguments {
                if let (Some(s_val), Some(e_val), Some(en_val)) =
                    (args.get("start"), args.get("end"), args.get("enabled"))
                {
                    if let (Some(start), Some(end), Some(enabled)) =
                        (s_val.as_f64(), e_val.as_f64(), en_val.as_bool())
                    {
                        if let Some(sender) = &server.action_sender {
                            if let Err(e) = sender.send(crate::McpAction::TimelineSetLoop {
                                start,
                                end,
                                enabled,
                            }) {
                                error!("Failed to send TimelineSetLoop action: {}", e);
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
        "audio_bindings_list" => {
            if let Some(sender) = &server.action_sender {
                if let Err(e) = sender.send(crate::McpAction::AudioBindingsList) {
                    error!("Failed to send AudioBindingsList action: {}", e);
                    return Some(crate::server::error_response(
                        id,
                        -32603,
                        "Internal error: Failed to send action",
                    ));
                }
            }
            Some(crate::server::success_response(
                id,
                serde_json::json!({"status": "queued"}),
            ))
        }
        "effect_list" => {
            if let Some(sender) = &server.action_sender {
                if let Err(e) = sender.send(crate::McpAction::EffectList) {
                    error!("Failed to send EffectList action: {}", e);
                    return Some(crate::server::error_response(
                        id,
                        -32603,
                        "Internal error: Failed to send action",
                    ));
                }
            }
            Some(crate::server::success_response(
                id,
                serde_json::json!({"status": "queued"}),
            ))
        }
        "scene_list" => {
            if let Some(sender) = &server.action_sender {
                if let Err(e) = sender.send(crate::McpAction::SceneList) {
                    error!("Failed to send SceneList action: {}", e);
                    return Some(crate::server::error_response(
                        id,
                        -32603,
                        "Internal error: Failed to send action",
                    ));
                }
            }
            Some(crate::server::success_response(
                id,
                serde_json::json!({"status": "queued"}),
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
