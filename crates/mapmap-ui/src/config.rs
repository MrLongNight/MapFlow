//! User configuration management
//!
//! Handles saving and loading user preferences including language settings.

use crate::theme::ThemeConfig;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;

/// Style for the audio meter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AudioMeterStyle {
    #[default]
    Retro,
    Digital,
}

impl fmt::Display for AudioMeterStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retro => write!(f, "Retro (Analog)"),
            Self::Digital => write!(f, "Digital (LED)"),
        }
    }
}

/// MIDI element assignment target
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiAssignmentTarget {
    /// Assigned to MapFlow internal control
    MapFlow(String), // Control target ID
    /// Assigned to Streamer.bot function
    StreamerBot(String), // Function name
    /// Assigned to Mixxx function
    Mixxx(String), // Function name
}

impl fmt::Display for MidiAssignmentTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MapFlow(id) => write!(f, "MapFlow: {}", id),
            Self::StreamerBot(func) => write!(f, "Streamer.bot: {}", func),
            Self::Mixxx(func) => write!(f, "Mixxx: {}", func),
        }
    }
}

/// A single MIDI element assignment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiAssignment {
    /// Element ID from the controller (e.g., "ch2_gain")
    pub element_id: String,
    /// Assignment target
    pub target: MidiAssignmentTarget,
}

/// User configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// Preferred language code (e.g., "en", "de")
    pub language: String,
    /// Last opened project path
    #[serde(default)]
    pub last_project: Option<String>,
    /// Recently opened files
    #[serde(default)]
    pub recent_files: Vec<String>,
    /// UI Theme settings
    #[serde(default)]
    pub theme: ThemeConfig,
    /// Target frame rate (FPS)
    #[serde(default)]
    pub target_fps: Option<f32>,
    /// Audio meter style
    #[serde(default)]
    pub meter_style: AudioMeterStyle,
    /// MIDI element assignments
    #[serde(default)]
    pub midi_assignments: Vec<MidiAssignment>,
    /// Selected audio input device name
    #[serde(default)]
    pub selected_audio_device: Option<String>,

    // === Window Geometry ===
    /// Window width in pixels
    #[serde(default)]
    pub window_width: Option<u32>,
    /// Window height in pixels
    #[serde(default)]
    pub window_height: Option<u32>,
    /// Window X position
    #[serde(default)]
    pub window_x: Option<i32>,
    /// Window Y position
    #[serde(default)]
    pub window_y: Option<i32>,
    /// Whether the window was maximized
    #[serde(default)]
    pub window_maximized: bool,

    // === Panel Visibility ===
    /// Show left sidebar
    #[serde(default = "default_true")]
    pub show_left_sidebar: bool,
    /// Show inspector panel
    #[serde(default = "default_true")]
    pub show_inspector: bool,
    /// Show timeline
    #[serde(default = "default_true")]
    pub show_timeline: bool,
    /// Show media browser
    #[serde(default = "default_true")]
    pub show_media_browser: bool,
    /// Show module canvas
    #[serde(default)]
    pub show_module_canvas: bool,
    /// Show controller overlay
    #[serde(default)]
    pub show_controller_overlay: bool,
}

fn default_true() -> bool {
    true
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            last_project: None,
            recent_files: Vec::new(),
            theme: ThemeConfig::default(),
            target_fps: Some(60.0),
            meter_style: AudioMeterStyle::default(),
            midi_assignments: Vec::new(),
            selected_audio_device: None,
            // Window geometry - None means use default
            window_width: None,
            window_height: None,
            window_x: None,
            window_y: None,
            window_maximized: false,
            // Panel visibility defaults
            show_left_sidebar: true,
            show_inspector: true,
            show_timeline: true,
            show_media_browser: true,
            show_module_canvas: false,
            show_controller_overlay: false,
        }
    }
}

impl UserConfig {
    /// Get the config file path
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("MapFlow");
            p.push("config.json");
            p
        })
    }

    /// Load configuration from disk
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| {
                if path.exists() {
                    fs::read_to_string(&path).ok()
                } else {
                    None
                }
            })
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(&path, content)?;
        }
        Ok(())
    }

    /// Update language and save
    pub fn set_language(&mut self, lang: &str) {
        self.language = lang.to_string();
        if let Err(e) = self.save() {
            tracing::error!("Failed to save config: {}", e);
        }
    }

    /// Add a file to recent files list
    pub fn add_recent_file(&mut self, path: &str) {
        // Remove if already exists
        self.recent_files.retain(|p| p != path);
        // Add to front
        self.recent_files.insert(0, path.to_string());
        // Keep max 10 recent files
        self.recent_files.truncate(10);
        if let Err(e) = self.save() {
            tracing::error!("Failed to save config: {}", e);
        }
    }

    /// Set or update a MIDI assignment
    pub fn set_midi_assignment(&mut self, element_id: &str, target: MidiAssignmentTarget) {
        // Remove existing assignment for this element
        self.midi_assignments.retain(|a| a.element_id != element_id);
        // Add new assignment
        self.midi_assignments.push(MidiAssignment {
            element_id: element_id.to_string(),
            target,
        });
        if let Err(e) = self.save() {
            tracing::error!("Failed to save config: {}", e);
        }
    }

    /// Remove a MIDI assignment
    pub fn remove_midi_assignment(&mut self, element_id: &str) {
        self.midi_assignments.retain(|a| a.element_id != element_id);
        if let Err(e) = self.save() {
            tracing::error!("Failed to save config: {}", e);
        }
    }

    /// Set and save the selected audio device
    pub fn set_audio_device(&mut self, device: Option<String>) {
        self.selected_audio_device = device;
        if let Err(e) = self.save() {
            tracing::error!("Failed to save config: {}", e);
        }
    }

    /// Get assignment for an element
    pub fn get_midi_assignment(&self, element_id: &str) -> Option<&MidiAssignment> {
        self.midi_assignments
            .iter()
            .find(|a| a.element_id == element_id)
    }

    /// Get all assignments for a specific target type
    pub fn get_assignments_by_type(
        &self,
    ) -> (
        Vec<&MidiAssignment>,
        Vec<&MidiAssignment>,
        Vec<&MidiAssignment>,
    ) {
        let mapflow: Vec<_> = self
            .midi_assignments
            .iter()
            .filter(|a| matches!(a.target, MidiAssignmentTarget::MapFlow(_)))
            .collect();
        let streamerbot: Vec<_> = self
            .midi_assignments
            .iter()
            .filter(|a| matches!(a.target, MidiAssignmentTarget::StreamerBot(_)))
            .collect();
        let mixxx: Vec<_> = self
            .midi_assignments
            .iter()
            .filter(|a| matches!(a.target, MidiAssignmentTarget::Mixxx(_)))
            .collect();
        (mapflow, streamerbot, mixxx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UserConfig::default();
        assert_eq!(config.language, "en");
        assert!(config.recent_files.is_empty());
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = UserConfig {
            language: "de".to_string(),
            last_project: Some("/path/to/project.MapFlow".to_string()),
            recent_files: vec!["file1.mp4".to_string(), "file2.mp4".to_string()],
            theme: ThemeConfig::default(),
            target_fps: Some(60.0),
            meter_style: AudioMeterStyle::Digital,
            midi_assignments: Vec::new(),
            selected_audio_device: None,
            window_width: Some(1920),
            window_height: Some(1080),
            window_x: Some(100),
            window_y: Some(50),
            window_maximized: false,
            show_left_sidebar: true,
            show_inspector: true,
            show_timeline: true,
            show_media_browser: true,
            show_module_canvas: false,
            show_controller_overlay: false,
        };

        let json = serde_json::to_string(&config).unwrap();
        let loaded: UserConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.language, "de");
        assert_eq!(loaded.recent_files.len(), 2);
        assert_eq!(loaded.meter_style, AudioMeterStyle::Digital);
    }
}
