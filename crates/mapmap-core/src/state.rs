//! Application State definitions
//!
//! This module defines the core state structures that are persisted to disk.

use crate::{
    assignment::AssignmentManager, logging::LogConfig, module::ModuleManager, AudioConfig,
    LayerManager, MappingManager, OscillatorConfig, OutputManager, PaintManager,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Global application state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,

    /// Paint manager (media sources)
    pub paint_manager: Arc<PaintManager>,

    /// Mapping manager (geometry mapping)
    pub mapping_manager: Arc<MappingManager>,

    /// Layer manager (compositing)
    pub layer_manager: Arc<LayerManager>,

    /// Output manager (display configuration)
    pub output_manager: Arc<OutputManager>,

    /// Module manager (show control)
    #[serde(default)]
    pub module_manager: Arc<ModuleManager>,

    /// Effect automation
    #[serde(default)]
    pub effect_animator: Arc<crate::EffectParameterAnimator>,

    /// Custom shader graphs
    #[serde(default)]
    pub shader_graphs: Arc<std::collections::HashMap<crate::GraphId, crate::ShaderGraph>>,

    /// Effect chain
    #[serde(default)]
    pub effect_chain: Arc<crate::effects::EffectChain>,

    /// Assignment manager (MIDI, OSC, etc.)
    #[serde(default)]
    pub assignment_manager: Arc<AssignmentManager>,

    /// Audio configuration
    pub audio_config: AudioConfig,

    /// Oscillator configuration
    pub oscillator_config: OscillatorConfig,

    /// Application settings
    #[serde(default)]
    pub settings: Arc<AppSettings>,

    /// Dirty flag (has changes?) - Not serialized
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            version: "0.1.0".to_string(),
            paint_manager: Arc::new(PaintManager::new()),
            mapping_manager: Arc::new(MappingManager::new()),
            layer_manager: Arc::new(LayerManager::new()),
            output_manager: Arc::new(OutputManager::new((1920, 1080))),
            module_manager: Arc::new(ModuleManager::default()),
            effect_animator: Arc::new(crate::EffectParameterAnimator::default()),
            shader_graphs: Arc::new(std::collections::HashMap::new()),
            effect_chain: Arc::new(crate::effects::EffectChain::new()),
            assignment_manager: Arc::new(AssignmentManager::default()),
            audio_config: AudioConfig::default(),
            oscillator_config: OscillatorConfig::default(),
            settings: Arc::new(AppSettings::default()),
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

    /// Get mutable reference to PaintManager (CoW)
    pub fn paint_manager_mut(&mut self) -> &mut PaintManager {
        Arc::make_mut(&mut self.paint_manager)
    }

    /// Get mutable reference to MappingManager (CoW)
    pub fn mapping_manager_mut(&mut self) -> &mut MappingManager {
        Arc::make_mut(&mut self.mapping_manager)
    }

    /// Get mutable reference to LayerManager (CoW)
    pub fn layer_manager_mut(&mut self) -> &mut LayerManager {
        Arc::make_mut(&mut self.layer_manager)
    }

    /// Get mutable reference to OutputManager (CoW)
    pub fn output_manager_mut(&mut self) -> &mut OutputManager {
        Arc::make_mut(&mut self.output_manager)
    }

    /// Get mutable reference to ModuleManager (CoW)
    pub fn module_manager_mut(&mut self) -> &mut ModuleManager {
        Arc::make_mut(&mut self.module_manager)
    }

    /// Get mutable reference to EffectParameterAnimator (CoW)
    pub fn effect_animator_mut(&mut self) -> &mut crate::EffectParameterAnimator {
        Arc::make_mut(&mut self.effect_animator)
    }

    /// Get mutable reference to ShaderGraphs (CoW)
    pub fn shader_graphs_mut(
        &mut self,
    ) -> &mut std::collections::HashMap<crate::GraphId, crate::ShaderGraph> {
        Arc::make_mut(&mut self.shader_graphs)
    }

    /// Get mutable reference to EffectChain (CoW)
    pub fn effect_chain_mut(&mut self) -> &mut crate::effects::EffectChain {
        Arc::make_mut(&mut self.effect_chain)
    }

    /// Get mutable reference to AssignmentManager (CoW)
    pub fn assignment_manager_mut(&mut self) -> &mut AssignmentManager {
        Arc::make_mut(&mut self.assignment_manager)
    }

    /// Get mutable reference to AppSettings (CoW)
    pub fn settings_mut(&mut self) -> &mut AppSettings {
        Arc::make_mut(&mut self.settings)
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
    fn test_app_state_cow_behavior() {
        let state1 = AppState::new("COW Test");
        let mut state2 = state1.clone();

        // Initially they share the same PaintManager (Arc)
        assert_eq!(Arc::strong_count(&state1.paint_manager), 2);

        // Mutate state2's paint manager
        state2
            .paint_manager_mut()
            .add_paint(crate::Paint::color(1, "Test", [1.0, 0.0, 0.0, 1.0]));

        // Now they should have split
        assert_eq!(Arc::strong_count(&state1.paint_manager), 1);
        assert_eq!(Arc::strong_count(&state2.paint_manager), 1);

        // state1 should be empty, state2 should have 1 paint
        assert!(state1.paint_manager.paints().is_empty());
        assert_eq!(state2.paint_manager.paints().len(), 1);
    }

    #[test]
    fn test_all_managers_cow_behavior() {
        let state1 = AppState::new("All COW Test");

        // 1. Mapping Manager
        {
            let mut state2 = state1.clone();
            assert_eq!(Arc::strong_count(&state1.mapping_manager), 2);
            state2
                .mapping_manager_mut()
                .add_mapping(crate::mapping::Mapping::quad(1, "Test", 1));
            assert_eq!(Arc::strong_count(&state1.mapping_manager), 1);
            assert!(state1.mapping_manager.mappings().is_empty());
            assert!(!state2.mapping_manager.mappings().is_empty());
        }

        // 2. Layer Manager
        {
            let mut state3 = state1.clone();
            assert_eq!(Arc::strong_count(&state1.layer_manager), 2);
            state3.layer_manager_mut().create_layer("Test Layer");
            assert_eq!(Arc::strong_count(&state1.layer_manager), 1);
            assert!(state1.layer_manager.is_empty());
            assert!(!state3.layer_manager.is_empty());
        }

        // 3. Module Manager
        {
            let mut state4 = state1.clone();
            assert_eq!(Arc::strong_count(&state1.module_manager), 2);
            state4
                .module_manager_mut()
                .create_module("Test Module".to_string());
            assert_eq!(Arc::strong_count(&state1.module_manager), 1);
            assert!(state1.module_manager.list_modules().is_empty());
            assert!(!state4.module_manager.list_modules().is_empty());
        }

        // 4. Output Manager
        {
            let mut state5 = state1.clone();
            assert_eq!(Arc::strong_count(&state1.output_manager), 2);
            state5.output_manager_mut().set_canvas_size(100, 100);
            assert_eq!(Arc::strong_count(&state1.output_manager), 1);
            assert_eq!(state1.output_manager.canvas_size(), (1920, 1080));
            assert_eq!(state5.output_manager.canvas_size(), (100, 100));
        }

        // 5. Assignment Manager
        {
            let mut state6 = state1.clone();
            assert_eq!(Arc::strong_count(&state1.assignment_manager), 2);
            let _ = state6.assignment_manager_mut();
            assert_eq!(Arc::strong_count(&state1.assignment_manager), 1);
        }
    }
}




