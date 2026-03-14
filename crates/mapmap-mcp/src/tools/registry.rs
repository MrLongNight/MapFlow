use crate::protocol::Tool;

pub fn get_tools() -> Vec<Tool> {
    vec![
        // === Basic Tools ===
        Tool {
            name: "send_osc".to_string(),
            description: Some("Send an Open Sound Control (OSC) message to MapFlow".to_string()),
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
            description: Some("Bind audio frequency/beat to a layer parameter".to_string()),
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
        Tool {
            name: "set_module_source_path".to_string(),
            description: Some("Set module source path for async file picking".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "module_id": { "type": "integer" },
                    "part_id": { "type": "integer" },
                    "path": { "type": "string" }
                },
                "required": ["module_id", "part_id", "path"]
            }),
        },
        Tool {
            name: "media_library_list".to_string(),
            description: Some("List available media in library".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "folder": { "type": "string" } }
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
            name: "effect_chain_get".to_string(),
            description: Some("Get the effect chain for a layer".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "layer_id": { "type": "integer" } },
                "required": ["layer_id"]
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
            name: "surface_delete".to_string(),
            description: Some("Delete a mapping surface".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "surface_id": { "type": "integer" } },
                "required": ["surface_id"]
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
            name: "mask_edit".to_string(),
            description: Some("Edit mask points".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "mask_id": { "type": "integer" },
                    "points": { "type": "string" }
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
                    "transition": { "type": "string" },
                    "duration": { "type": "number" }
                },
                "required": ["scene_id"]
            }),
        },
        Tool {
            name: "preset_save".to_string(),
            description: Some("Save current state as a preset".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "scope": { "type": "string" }
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
                    "target": { "type": "string" }
                },
                "required": ["preset_id"]
            }),
        },
        Tool {
            name: "audio_analysis_config".to_string(),
            description: Some("Configure audio analysis parameters".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "fft_size": { "type": "integer" },
                    "smoothing": { "type": "number" },
                    "bands": { "type": "integer" }
                }
            }),
        },
        Tool {
            name: "timeline_set_loop".to_string(),
            description: Some("Set timeline loop region".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "start": { "type": "number" },
                    "end": { "type": "number" },
                    "enabled": { "type": "boolean" }
                },
                "required": ["start", "end", "enabled"]
            }),
        },
        Tool {
            name: "audio_bindings_list".to_string(),
            description: Some("List all active audio bindings".to_string()),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
        Tool {
            name: "effect_list".to_string(),
            description: Some("List all available effects".to_string()),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
        Tool {
            name: "scene_list".to_string(),
            description: Some("List all scenes".to_string()),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
    ]
}
