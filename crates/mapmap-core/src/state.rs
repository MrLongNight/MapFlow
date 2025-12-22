//! Application State definitions
//!
//! This module defines the core state structures that are persisted to disk.

use crate::{
    AudioConfig, LayerManager, MappingManager, OscillatorConfig, OutputManager, PaintManager,
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

    /// Audio configuration
    pub audio_config: AudioConfig,

    /// Oscillator config
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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            dark_mode: true,
            ui_scale: 1.0,
            language: "en".to_string(),
        }
    }
}
