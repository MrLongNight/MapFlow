pub mod protocol;
pub mod server;

use std::path::PathBuf;

pub use protocol::*;
pub use server::McpServer;

// Re-export for convenience
pub use anyhow::Result;

/// Actions internally triggered by the MCP Server to be handled by the main application.
#[derive(Debug, Clone)]
pub enum McpAction {
    // === Project Management ===
    /// Save the project.
    SaveProject(PathBuf),
    /// Load a project.
    LoadProject(PathBuf),

    // === Layer Management ===
    /// Add a new layer.
    AddLayer(String),
    /// Remove a layer by ID.
    RemoveLayer(u64),
    /// Set layer opacity (layer_id, opacity 0.0-1.0)
    SetLayerOpacity(u64, f32),
    /// Set layer visibility (layer_id, visible)
    SetLayerVisibility(u64, bool),
    /// Set layer blend mode (layer_id, blend_mode)
    SetLayerBlendMode(u64, String),

    // === Cue Management ===
    /// Trigger a cue by ID.
    TriggerCue(u64),
    /// Go to the next cue.
    NextCue,
    /// Go to the previous cue.
    PrevCue,

    // === Media Playback ===
    /// Start media playback
    MediaPlay,
    /// Pause media playback
    MediaPause,
    /// Stop media playback
    MediaStop,

    // === Phase 1: Media in Layers ===
    /// Load media into a layer (layer_id, media_path)
    LayerLoadMedia(u64, PathBuf),
    /// Set media playback time (layer_id, time_seconds)
    LayerSetMediaTime(u64, f64),
    /// Set playback speed (layer_id, speed)
    LayerSetPlaybackSpeed(u64, f32),
    /// Set loop mode (layer_id, loop_mode: "none", "loop", "ping-pong")
    LayerSetLoopMode(u64, String),
    /// Set module source path (part_id, path) - Used for async file picking
    SetModuleSourcePath(u64, PathBuf),
    /// List media library (optional folder filter)
    MediaLibraryList(Option<String>),
    /// Import media (source_path, destination_folder)
    MediaImport(PathBuf, Option<String>),

    // === Phase 2: Audio Reactivity ===
    /// Bind audio to parameter (source, layer_id, param, min, max, smoothing)
    AudioBindParam {
        source: String,
        layer_id: u64,
        param: String,
        min: f32,
        max: f32,
        smoothing: f32,
    },
    /// Unbind audio parameter (binding_id)
    AudioUnbindParam(u64),
    /// List all audio bindings
    AudioBindingsList,
    /// Set audio sensitivity (frequency_band, sensitivity)
    AudioSetSensitivity(String, f32),
    /// Set beat detection threshold
    AudioSetThreshold(f32),
    /// Configure audio analysis (fft_size, smoothing, bands)
    AudioAnalysisConfig {
        fft_size: u32,
        smoothing: f32,
        bands: u32,
    },

    // === Phase 3: Effects & Shaders ===
    /// Add effect to layer (layer_id, effect_type)
    EffectAdd(u64, String),
    /// Remove effect from layer (layer_id, effect_id)
    EffectRemove(u64, u64),
    /// Set effect parameter (layer_id, effect_id, param_name, value)
    EffectSetParam(u64, u64, String, f32),
    /// List available effects
    EffectList,
    /// Get effect chain for layer (layer_id)
    EffectChainGet(u64),
    /// Load custom shader (layer_id, shader_path)
    ShaderLoad(u64, PathBuf),
    /// Set shader uniform (layer_id, uniform_name, value)
    ShaderSetUniform(u64, String, f32),

    // === Phase 4: Timeline & Keyframes ===
    /// Add keyframe (layer_id, param, time, value, easing)
    TimelineAddKeyframe {
        layer_id: u64,
        param: String,
        time: f64,
        value: f32,
        easing: String,
    },
    /// Remove keyframe (keyframe_id)
    TimelineRemoveKeyframe(u64),
    /// Get keyframes for layer/param
    TimelineGetKeyframes(u64, String),
    /// Set timeline duration
    TimelineSetDuration(f64),
    /// Set timeline position
    TimelineSetPosition(f64),
    /// Set loop region (start, end, enabled)
    TimelineSetLoop { start: f64, end: f64, enabled: bool },

    // === Phase 5: Mapping & Scenes ===
    /// Create surface (type, corners as JSON)
    SurfaceCreate(String, String),
    /// Delete surface (surface_id)
    SurfaceDelete(u64),
    /// Set surface corners (surface_id, corners as JSON)
    SurfaceSetCorners(u64, String),
    /// Assign layer to surface (surface_id, layer_id)
    SurfaceAssignLayer(u64, u64),
    /// Create mask (layer_id, mask_type, points as JSON)
    MaskCreate(u64, String, String),
    /// Edit mask (mask_id, points as JSON)
    MaskEdit(u64, String),
    /// Create scene (name)
    SceneCreate(String),
    /// Switch to scene (scene_id, transition, duration)
    SceneSwitch(u64, String, f32),
    /// List all scenes
    SceneList,
    /// Save preset (name, scope)
    PresetSave(String, String),
    /// Load preset (preset_id, target)
    PresetLoad(u64, Option<String>),
}
