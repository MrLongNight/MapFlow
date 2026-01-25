//! Application State definitions
//!
//! This module defines the core state structures that are persisted to disk.

use crate::{
    assignment::AssignmentManager, logging::LogConfig, module::ModuleManager, AudioConfig,
    LayerManager, MappingManager, OscillatorConfig, OutputManager, PaintManager,
};
use serde::{Deserialize, Serialize};

/// Global application state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,

    /// Paint manager (media sources)
    pub paint_manager: PaintManager,

    /// Mapping manager (geometry mapping)
    pub mapping_manager: MappingManager,

    /// Layer manager (compositing)
    pub layer_manager: LayerManager,

    /// Output manager (display configuration)
    pub output_manager: OutputManager,

    /// Module manager (show control)
    #[serde(default)]
    pub module_manager: ModuleManager,

    /// Effect automation
    #[serde(default)]
    pub effect_animator: crate::EffectParameterAnimator,

    /// Custom shader graphs
    #[serde(default)]
    pub shader_graphs: std::collections::HashMap<crate::GraphId, crate::ShaderGraph>,

    /// Effect chain
    #[serde(default)]
    pub effect_chain: crate::effects::EffectChain,

    /// Assignment manager (MIDI, OSC, etc.)
    #[serde(default)]
    pub assignment_manager: AssignmentManager,

    /// Audio configuration
    pub audio_config: AudioConfig,

    /// Oscillator configuration
    pub oscillator_config: OscillatorConfig,

    /// Application settings
    #[serde(default)]
    pub settings: AppSettings,

    /// Dirty flag (has changes?) - Not serialized
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            version: "0.1.0".to_string(),
            paint_manager: PaintManager::new(),
            mapping_manager: MappingManager::new(),
            layer_manager: LayerManager::new(),
            output_manager: OutputManager::new((1920, 1080)),
            module_manager: ModuleManager::default(),
            effect_animator: crate::EffectParameterAnimator::default(),
            shader_graphs: std::collections::HashMap::new(),
            effect_chain: crate::effects::EffectChain::new(),
            assignment_manager: AssignmentManager::default(),
            audio_config: AudioConfig::default(),
            oscillator_config: OscillatorConfig::default(),
            settings: AppSettings::default(),
            dirty: false,
        }
    }
}

impl AppState {
    /// Create a new empty project state
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

/// Global application settings (not strictly project, but persisted with it or separately in user config)
/// For now, we include it in project file for simplicity, or we can split it later.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    /// Global master volume
    pub master_volume: f32,
    /// Dark mode toggle
    pub dark_mode: bool,
    /// UI scale factor
    pub ui_scale: f32,
    /// UI Language code (en, de)
    pub language: String,
    /// Logging configuration
    #[serde(default)]
    pub log_config: LogConfig,
    /// Number of output windows (projectors/beamers)
    #[serde(default = "default_output_count")]
    pub output_count: u8,
}

fn default_output_count() -> u8 {
    1
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            dark_mode: true,
            ui_scale: 1.0,
            language: "en".to_string(),
            log_config: LogConfig::default(),
            output_count: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_defaults() {
        let state = AppState::default();
        assert_eq!(state.name, "Untitled Project");
        assert_eq!(state.version, "0.1.0");
        assert!(!state.dirty);
        assert_eq!(state.output_manager.canvas_size(), (1920, 1080));

        // Check deeper defaults
        assert!(state.paint_manager.paints().is_empty());
        assert!(state.layer_manager.is_empty());
        assert!(state.module_manager.list_modules().is_empty());
        assert!(state.shader_graphs.is_empty());
        assert!(state.assignment_manager.assignments().is_empty());
    }

    #[test]
    fn test_app_settings_defaults() {
        let settings = AppSettings::default();
        assert_eq!(settings.master_volume, 1.0);
        assert!(settings.dark_mode);
        assert_eq!(settings.ui_scale, 1.0);
        assert_eq!(settings.language, "en");
        assert_eq!(settings.output_count, 1);
    }

    #[test]
    fn test_app_settings_serialization() {
        let settings = AppSettings {
            master_volume: 0.5,
            dark_mode: false,
            ui_scale: 1.5,
            language: "de".to_string(),
            log_config: LogConfig::default(),
            output_count: 3,
        };

        let serialized = serde_json::to_string(&settings).expect("Failed to serialize AppSettings");
        let deserialized: AppSettings =
            serde_json::from_str(&serialized).expect("Failed to deserialize AppSettings");

        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_app_state_serialization_roundtrip() {
        let original = AppState::new("Test Project");

        // Serialize
        let serialized = serde_json::to_string(&original).expect("Failed to serialize AppState");

        // Deserialize
        let deserialized: AppState =
            serde_json::from_str(&serialized).expect("Failed to deserialize AppState");

        // Note: 'dirty' flag is skipped in serialization, but since original is fresh (dirty=false)
        // and default deserialization is dirty=false, they should match completely.
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_dirty_flag_excluded() {
        let mut original = AppState::new("Dirty Project");
        original.dirty = true;

        // Serialize
        let serialized = serde_json::to_string(&original).expect("Failed to serialize AppState");

        // Deserialize
        let deserialized: AppState =
            serde_json::from_str(&serialized).expect("Failed to deserialize AppState");

        // Name should match
        assert_eq!(original.name, deserialized.name);

        // Dirty flag should NOT match (original=true, deserialized=false default)
        assert!(original.dirty);
        assert!(!deserialized.dirty);
        assert_ne!(original, deserialized);

        // Check that "dirty" key is not in the JSON string
        assert!(!serialized.contains("\"dirty\""));
    }

    #[test]
    fn test_app_state_deep_defaults() {
        let state = AppState::default();

        // Verify Oscillator defaults within AppState
        assert!(state.oscillator_config.enabled);
        assert_eq!(state.oscillator_config.frequency_min, 0.5);

        // Verify Effect Animator is default (empty)
        assert!(state.effect_animator.bindings().is_empty());
    }

    #[test]
    fn test_app_state_dirty_flag_skip() {
        let mut state = AppState::new("Dirty Project");
        state.dirty = true;

        let serialized = serde_json::to_string(&state).expect("Failed to serialize");

        // Ensure "dirty" field is NOT in the JSON string
        assert!(!serialized.contains("\"dirty\""));

        let deserialized: AppState =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Deserialized object should have default dirty state (false), NOT true
        assert!(!deserialized.dirty);
    }
}
