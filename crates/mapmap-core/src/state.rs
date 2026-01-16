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
    fn test_app_state_serialization_roundtrip() {
        let original = AppState::new("Test Project");

        // Serialize
        let serialized = serde_json::to_string(&original).expect("Failed to serialize AppState");

        // Deserialize
        let deserialized: AppState =
            serde_json::from_str(&serialized).expect("Failed to deserialize AppState");

        // Note: We cannot compare 'dirty' flag as it is skipped in serialization
        // But everything else should be equal
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.version, deserialized.version);
        assert_eq!(original.settings, deserialized.settings);

        // Check key manager states
        assert_eq!(original.paint_manager, deserialized.paint_manager);
        assert_eq!(original.layer_manager, deserialized.layer_manager);
        // Note: ModuleManager derived PartialEq check
        assert_eq!(original.module_manager, deserialized.module_manager);
    }

    #[test]
    fn test_app_state_new_initialization() {
        let state = AppState::new("My Cool Project");

        assert_eq!(state.name, "My Cool Project");
        assert_eq!(state.version, "0.1.0");
        assert_eq!(state.settings.output_count, 1);

        // Managers should be initialized empty but ready
        assert_eq!(state.paint_manager.paints().len(), 0);
        assert_eq!(state.layer_manager.len(), 0);
        assert_eq!(state.mapping_manager.mappings().len(), 0);

        // Default configs
        assert!(!state.dirty);
    }

    #[test]
    fn test_app_settings_custom_values() {
        let settings = AppSettings {
            master_volume: 0.5,
            dark_mode: false,
            ui_scale: 1.5,
            language: "de".to_string(),
            log_config: LogConfig::default(),
            output_count: 3,
        };

        assert_eq!(settings.master_volume, 0.5);
        assert_eq!(settings.dark_mode, false);
        assert_eq!(settings.ui_scale, 1.5);
        assert_eq!(settings.language, "de");
        assert_eq!(settings.output_count, 3);
    }

    #[test]
    fn test_app_state_serialization_skip_dirty() {
        let mut original = AppState::new("Dirty Project");
        original.dirty = true;

        let serialized = serde_json::to_string(&original).expect("Failed to serialize");

        // "dirty" field should NOT be present in JSON
        assert!(!serialized.contains("\"dirty\""));

        let deserialized: AppState =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Deserialized struct uses Default for skipped fields -> dirty should be false
        assert_eq!(deserialized.dirty, false);
    }

    #[test]
    fn test_deep_structure_serialization() {
        let mut original = AppState::new("Complex Project");

        // Add some complexity
        original
            .shader_graphs
            .insert(1, crate::ShaderGraph::new(1, "Test Graph".to_string()));
        original
            .effect_chain
            .add_effect(crate::effects::EffectType::Blur);

        let serialized = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: AppState =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert!(deserialized.shader_graphs.contains_key(&1));
        assert_eq!(deserialized.effect_chain.effects.len(), 1);
        assert_eq!(
            deserialized.effect_chain.effects[0].effect_type,
            crate::effects::EffectType::Blur
        );
    }
}
