//! User configuration management
//!
//! Handles saving and loading user preferences including language settings.

use crate::theme::ThemeConfig;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;

/// Sichtbarkeitseinstellungen für das Hauptlayout.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LayoutVisibility {
    #[serde(default = "default_true")]
    pub show_toolbar: bool,
    #[serde(default = "default_true")]
    pub show_left_sidebar: bool,
    #[serde(default = "default_true")]
    pub show_inspector: bool,
    #[serde(default = "default_true")]
    pub show_timeline: bool,
    #[serde(default = "default_true")]
    pub show_media_browser: bool,
    #[serde(default)]
    pub show_module_canvas: bool,
}

impl Default for LayoutVisibility {
    fn default() -> Self {
        Self {
            show_toolbar: true,
            show_left_sidebar: true,
            show_inspector: true,
            show_timeline: true,
            show_media_browser: true,
            show_module_canvas: false,
        }
    }
}

/// Größenparameter des Hauptlayouts.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LayoutPanelSizes {
    #[serde(default = "default_sidebar_width")]
    pub left_sidebar_width: f32,
    #[serde(default = "default_inspector_width")]
    pub inspector_width: f32,
    #[serde(default = "default_timeline_height")]
    pub timeline_height: f32,
}

impl Default for LayoutPanelSizes {
    fn default() -> Self {
        Self {
            left_sidebar_width: default_sidebar_width(),
            inspector_width: default_inspector_width(),
            timeline_height: default_timeline_height(),
        }
    }
}

/// Persistentes Layout-Profil für die Arbeitsoberfläche.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutProfile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub visibility: LayoutVisibility,
    #[serde(default)]
    pub panel_sizes: LayoutPanelSizes,
    #[serde(default)]
    pub lock_layout: bool,
}

impl LayoutProfile {
    /// Standardprofil, das dem bisherigen Dock-Layout entspricht.
    pub fn default_profile() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default".to_string(),
            visibility: LayoutVisibility::default(),
            panel_sizes: LayoutPanelSizes::default(),
            lock_layout: false,
        }
    }
}

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

/// Vertical Synchronization Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VSyncMode {
    #[default]
    Auto,
    On,
    Off,
}

/// Toolbar metric disclosure behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ToolbarMetricMode {
    /// Metric is visible directly in toolbar
    #[default]
    Always,
    /// Metric is available via hover/popover only
    Hover,
}

/// Globales Animationsprofil für UI-Bewegungen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnimationProfile {
    /// Animationen deaktiviert.
    Off,
    /// Subtile Animationen (Standard).
    #[default]
    Subtle,
    /// Cinematische Animationen mit stärkerem Effekt.
    Cinematic,
}

impl fmt::Display for AnimationProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Subtle => write!(f, "Subtle"),
            Self::Cinematic => write!(f, "Cinematic"),
        }
    }
}

/// Per-metric visibility settings for toolbar telemetry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolbarMetricsConfig {
    #[serde(default = "default_true")]
    pub show_bpm: bool,
    #[serde(default = "default_true")]
    pub show_fps: bool,
    #[serde(default = "default_true")]
    pub show_frame_time: bool,
    #[serde(default = "default_true")]
    pub show_cpu: bool,
    #[serde(default = "default_true")]
    pub show_gpu: bool,
    #[serde(default = "default_true")]
    pub show_ram: bool,
    #[serde(default = "default_true")]
    pub show_overall_status: bool,
    #[serde(default)]
    pub mode: ToolbarMetricMode,
}

impl Default for ToolbarMetricsConfig {
    fn default() -> Self {
        Self {
            show_bpm: true,
            show_fps: true,
            show_frame_time: true,
            show_cpu: true,
            show_gpu: true,
            show_ram: true,
            show_overall_status: true,
            mode: ToolbarMetricMode::Always,
        }
    }
}

impl fmt::Display for VSyncMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "Auto"),
            Self::On => write!(f, "On (VSync)"),
            Self::Off => write!(f, "Off (No Limit)"),
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

/// Configuration for Philips Hue integration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct HueConfig {
    /// Bridge IP address
    #[serde(default)]
    pub bridge_ip: String,
    /// Whitelisted username (generated by bridge)
    #[serde(default)]
    pub username: String,
    /// DTLS Client Key (generated by bridge for Entertainment API)
    #[serde(default)]
    pub client_key: String,
    /// Selected Entertainment Area ID
    #[serde(default)]
    pub entertainment_area: String,
    /// Setup mode/auto-connect
    #[serde(default)]
    pub auto_connect: bool,
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
    /// Desired frame rate for playback or updates.
    pub target_fps: Option<f32>,
    /// Preferred GPU Adapter Name
    #[serde(default)]
    pub preferred_gpu: Option<String>,
    /// Vertical Sync Mode
    #[serde(default)]
    pub vsync_mode: VSyncMode,
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
    /// Show media manager window
    #[serde(default)]
    pub show_media_manager: bool,
    /// Show dashboard window
    #[serde(default = "default_true")]
    pub show_dashboard: bool,

    /// Enable NDI discovery
    #[serde(default = "default_true")]
    pub ndi_discovery: bool,

    /// Philips Hue Configuration
    #[serde(default)]
    pub hue_config: HueConfig,

    // === Global Output Settings ===
    /// Enable fullscreen for all projectors
    #[serde(default)]
    pub global_fullscreen: bool,

    /// Global UI font scale factor (0.8 - 1.4)
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,

    /// Toolbar telemetry visibility and disclosure settings
    #[serde(default)]
    pub toolbar_metrics: ToolbarMetricsConfig,

    /// Enable animated node visuals in module canvas
    #[serde(default = "default_true")]
    pub node_animations_enabled: bool,

    /// Enable startup intro animation.
    #[serde(default = "default_true")]
    pub startup_animation_enabled: bool,

    /// Video path for startup intro animation.
    #[serde(default = "default_startup_animation_path")]
    pub startup_animation_path: String,

    /// Reduziert Bewegungen/Animationen global für bessere Zugänglichkeit.
    #[serde(default)]
    pub reduce_motion_enabled: bool,

    /// Deaktiviert Sounds bei App-Start-Sequenzen.
    #[serde(default)]
    pub silent_startup_enabled: bool,

    /// Globales Profil für UI-Animationen.
    #[serde(default)]
    pub animation_profile: AnimationProfile,

    /// Enable short-circuit effect for invalid node connections
    #[serde(default = "default_true")]
    pub short_circuit_animation_enabled: bool,

    /// Verfügbare UI-Layoutprofile
    #[serde(default = "default_layout_profiles")]
    pub layouts: Vec<LayoutProfile>,
    /// Aktives Layoutprofil (id)
    #[serde(default = "default_active_layout_id")]
    pub active_layout_id: String,
}

fn default_true() -> bool {
    true
}

fn default_ui_scale() -> f32 {
    1.0
}

fn default_startup_animation_path() -> String {
    "resources/app_videos/MF-Mechanical_Cube_Logo_Splash_Animation.webm".to_string()
}

fn default_sidebar_width() -> f32 {
    300.0
}

fn default_inspector_width() -> f32 {
    360.0
}

fn default_timeline_height() -> f32 {
    200.0
}

fn default_layout_profiles() -> Vec<LayoutProfile> {
    vec![LayoutProfile::default_profile()]
}

fn default_active_layout_id() -> String {
    "default".to_string()
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            last_project: None,
            recent_files: Vec::new(),
            theme: ThemeConfig::default(),
            target_fps: Some(60.0),
            preferred_gpu: None,
            vsync_mode: VSyncMode::default(),
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
            show_media_manager: false,
            show_dashboard: true,
            ndi_discovery: true,
            hue_config: HueConfig::default(),
            global_fullscreen: false,
            ui_scale: 1.0,
            toolbar_metrics: ToolbarMetricsConfig::default(),
            node_animations_enabled: true,
            startup_animation_enabled: true,
            startup_animation_path: default_startup_animation_path(),
            reduce_motion_enabled: false,
            silent_startup_enabled: false,
            animation_profile: AnimationProfile::Subtle,
            short_circuit_animation_enabled: true,
            layouts: default_layout_profiles(),
            active_layout_id: default_active_layout_id(),
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
        let mut loaded: Self = Self::config_path()
            .and_then(|path| {
                if path.exists() {
                    fs::read_to_string(&path).ok()
                } else {
                    None
                }
            })
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default();

        loaded.ensure_layout_profiles();
        loaded
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

    /// Stellt sicher, dass mindestens ein valides Layoutprofil verfügbar ist.
    pub fn ensure_layout_profiles(&mut self) {
        if self.layouts.is_empty() {
            self.layouts = default_layout_profiles();
        }

        if !self.layouts.iter().any(|l| l.id == self.active_layout_id) {
            self.active_layout_id = self
                .layouts
                .first()
                .map(|l| l.id.clone())
                .unwrap_or_else(default_active_layout_id);
        }
    }

    /// Liefert das aktive Layoutprofil.
    pub fn active_layout(&self) -> Option<&LayoutProfile> {
        self.layouts.iter().find(|l| l.id == self.active_layout_id)
    }

    /// Liefert das aktive Layoutprofil als mutable Referenz.
    pub fn active_layout_mut(&mut self) -> Option<&mut LayoutProfile> {
        self.layouts
            .iter_mut()
            .find(|l| l.id == self.active_layout_id)
    }

    /// Wechselt das aktive Layoutprofil.
    pub fn set_active_layout(&mut self, layout_id: &str) -> bool {
        if self.layouts.iter().any(|l| l.id == layout_id) {
            self.active_layout_id = layout_id.to_string();
            true
        } else {
            false
        }
    }

    /// Erstellt ein neues Layoutprofil als Kopie der übergebenen Daten.
    pub fn add_layout_profile(&mut self, mut profile: LayoutProfile) {
        if profile.id.trim().is_empty() {
            profile.id = format!("layout-{}", self.layouts.len() + 1);
        }
        self.layouts.push(profile);
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
            preferred_gpu: None,
            vsync_mode: VSyncMode::default(),
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
            show_media_manager: false,
            show_dashboard: true,
            ndi_discovery: true,
            hue_config: HueConfig::default(),
            global_fullscreen: true,
            ui_scale: 1.2,
            toolbar_metrics: ToolbarMetricsConfig::default(),
            node_animations_enabled: true,
            startup_animation_enabled: true,
            startup_animation_path: default_startup_animation_path(),
            reduce_motion_enabled: false,
            silent_startup_enabled: false,
            animation_profile: AnimationProfile::Subtle,
            short_circuit_animation_enabled: true,
            layouts: default_layout_profiles(),
            active_layout_id: default_active_layout_id(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let loaded: UserConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.language, "de");
        assert_eq!(loaded.recent_files.len(), 2);
        assert_eq!(loaded.meter_style, AudioMeterStyle::Digital);
    }

    #[test]
    fn test_ensure_layout_profiles_repairs_empty_state() {
        let mut config = UserConfig {
            layouts: Vec::new(),
            active_layout_id: "missing".to_string(),
            ..UserConfig::default()
        };

        config.ensure_layout_profiles();

        assert!(!config.layouts.is_empty());
        assert_eq!(config.active_layout_id, "default");
    }

    #[test]
    fn test_set_active_layout() {
        let mut config = UserConfig::default();
        config.add_layout_profile(LayoutProfile {
            id: "live".to_string(),
            name: "Live".to_string(),
            visibility: LayoutVisibility::default(),
            panel_sizes: LayoutPanelSizes::default(),
            lock_layout: false,
        });

        assert!(config.set_active_layout("live"));
        assert_eq!(config.active_layout_id, "live");
        assert!(!config.set_active_layout("does-not-exist"));
    }
}
