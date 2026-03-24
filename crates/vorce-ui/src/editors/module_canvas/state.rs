use crate::editors::mesh_editor::MeshEditor;
use egui::{Pos2, TextureHandle, Vec2};
#[cfg(feature = "ndi")]
use std::sync::mpsc;
use vorce_core::{
    audio_reactive::AudioTriggerData,
    module::{ModulePartId, ModuleSocketType},
};
#[cfg(feature = "ndi")]
use vorce_io::ndi::NdiSource;

use super::types::*;
use super::utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerInspectorViewMode {
    Preview,
    MeshEditor,
}

#[allow(dead_code)]
pub struct ModuleCanvas {
    /// The ID of the currently active/edited module
    pub active_module_id: Option<u64>,
    /// Canvas pan offset
    pub pan_offset: Vec2,
    /// Canvas zoom level
    pub zoom: f32,
    /// Part being dragged
    pub dragging_part: Option<(ModulePartId, Vec2)>,
    /// Part being resized: (part_id, original_size)
    pub resizing_part: Option<(ModulePartId, (f32, f32))>,
    /// Box selection start position (screen coords)
    pub box_select_start: Option<Pos2>,
    /// Connection being created: (from_part, from_socket_id, is_output, socket_type, start_pos)
    pub creating_connection: Option<(ModulePartId, String, bool, ModuleSocketType, Pos2)>,
    /// Part ID pending deletion (set when X button clicked)
    pub pending_delete: Option<ModulePartId>,
    /// Selected parts for multi-select and copy/paste
    pub selected_parts: Vec<ModulePartId>,
    /// Clipboard for copy/paste (stores part types and relative positions)
    pub clipboard: Vec<(vorce_core::module::ModulePartType, (f32, f32))>,
    /// Search filter text
    pub search_filter: String,
    /// Whether search popup is visible
    pub show_search: bool,
    /// Undo history stack
    pub undo_stack: Vec<CanvasAction>,
    /// Redo history stack
    pub redo_stack: Vec<CanvasAction>,
    /// Saved module presets
    pub presets: Vec<ModulePreset>,
    /// Whether preset panel is visible
    pub show_presets: bool,
    /// New preset name input
    pub new_preset_name: String,
    /// Context menu position
    pub context_menu_pos: Option<Pos2>,
    /// Context menu target (connection index or None)
    pub context_menu_connection: Option<usize>,
    /// Context menu target (part ID or None)
    pub context_menu_part: Option<ModulePartId>,
    /// MIDI Learn mode - which part is waiting for MIDI input
    pub midi_learn_part_id: Option<ModulePartId>,
    /// Whether we are currently panning the canvas (started on empty area)
    pub panning_canvas: bool,
    /// Cached textures for plug icons
    pub plug_icons: std::collections::HashMap<String, TextureHandle>,
    /// Learned MIDI mapping: (part_id, channel, cc_or_note, is_note)
    pub learned_midi: Option<(ModulePartId, u8, u8, bool)>,
    /// Live audio trigger data from AudioAnalyzerV2
    pub audio_trigger_data: AudioTriggerData,

    /// Discovered NDI sources
    #[cfg(feature = "ndi")]
    pub ndi_sources: Vec<NdiSource>,
    /// Channel to receive discovered NDI sources from async task
    #[cfg(feature = "ndi")]
    pub ndi_discovery_rx: Option<mpsc::Receiver<Vec<NdiSource>>>,
    /// Available outputs (id, name) for output node selection
    pub available_outputs: Vec<(u64, String)>,
    /// ID of the part being edited in a popup
    pub editing_part_id: Option<ModulePartId>,
    /// Video Texture Previews for Media Nodes ((Module ID, Part ID) -> Egui Texture)
    pub node_previews: std::collections::HashMap<(u64, u64), egui::TextureId>,
    /// Output previews mirrored from the app layer for inspector usage (Output ID -> Egui Texture)
    pub output_previews: std::collections::HashMap<u64, egui::TextureId>,
    /// Pending playback commands (Part ID, Command)
    pub pending_playback_commands: Vec<(ModulePartId, MediaPlaybackCommand)>,
    /// Last diagnostic check results
    pub diagnostic_issues: Vec<vorce_core::diagnostics::ModuleIssue>,
    /// Whether diagnostic popup is shown
    pub show_diagnostics: bool,
    /// Media player info for timeline display (Part ID -> Info)
    pub player_info: std::collections::HashMap<ModulePartId, MediaPlayerInfo>,

    // Hue Integration
    /// Discovered Hue bridges
    pub hue_bridges: Vec<vorce_control::hue::api::discovery::DiscoveredBridge>,
    /// Channel for Hue discovery results
    pub hue_discovery_rx: Option<
        std::sync::mpsc::Receiver<
            Result<Vec<vorce_control::hue::api::discovery::DiscoveredBridge>, String>,
        >,
    >,
    /// Channel for Hue pairing results
    pub hue_pairing_rx:
        Option<std::sync::mpsc::Receiver<Result<vorce_control::hue::models::HueConfig, String>>>,
    /// Status message for Hue operations
    pub hue_status_message: Option<String>,
    /// Last known trigger values for visualization (Part ID -> Value 0.0-1.0)
    pub last_trigger_values: std::collections::HashMap<ModulePartId, f32>,
    /// Whether inspector previews should be shown where available.
    pub show_inspector_previews: bool,
    /// Shared workspace mode for layer nodes inside the inspector.
    pub layer_inspector_view_mode: LayerInspectorViewMode,

    /// Advanced Mesh Editor instance
    pub mesh_editor: MeshEditor,
    /// Whether to show the mesh editor in a separate window
    pub show_mesh_editor: bool,
    /// ID of the part currently being edited in the mesh editor (to detect selection changes)
    pub last_mesh_edit_id: Option<ModulePartId>,

    // Quick Create State
    /// Whether the quick create popup is visible
    pub show_quick_create: bool,
    /// Filter text for quick create
    pub quick_create_filter: String,
    /// Screen position for the quick create popup
    pub quick_create_pos: Pos2,
    /// Index of the currently selected item in the quick create list
    pub quick_create_selected_index: usize,

    /// Snapshot of a part before editing, used to create Undo/Redo commands when an edit finishes.
    pub edit_snapshot: Option<vorce_core::module::ModulePart>,
}

impl Default for ModuleCanvas {
    fn default() -> Self {
        Self {
            active_module_id: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            dragging_part: None,
            resizing_part: None,
            box_select_start: None,
            creating_connection: None,
            pending_delete: None,
            selected_parts: Vec::new(),
            clipboard: Vec::new(),
            search_filter: String::new(),
            show_search: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            presets: utils::default_presets(),
            show_presets: false,
            new_preset_name: String::new(),
            context_menu_pos: None,
            context_menu_connection: None,
            context_menu_part: None,
            midi_learn_part_id: None,
            panning_canvas: false,
            plug_icons: std::collections::HashMap::new(),
            learned_midi: None,
            audio_trigger_data: AudioTriggerData::default(),
            #[cfg(feature = "ndi")]
            ndi_sources: Vec::new(),
            #[cfg(feature = "ndi")]
            ndi_discovery_rx: None,
            available_outputs: Vec::new(),
            editing_part_id: None,
            node_previews: std::collections::HashMap::new(),
            output_previews: std::collections::HashMap::new(),
            pending_playback_commands: Vec::new(),
            diagnostic_issues: Vec::new(),
            show_diagnostics: false,
            player_info: std::collections::HashMap::new(),
            hue_bridges: Vec::new(),
            hue_discovery_rx: None,
            hue_pairing_rx: None,
            hue_status_message: None,
            last_trigger_values: std::collections::HashMap::new(),
            show_inspector_previews: true,
            layer_inspector_view_mode: LayerInspectorViewMode::MeshEditor,
            mesh_editor: MeshEditor::new(),
            show_mesh_editor: false,
            last_mesh_edit_id: None,
            show_quick_create: false,
            quick_create_filter: String::new(),
            quick_create_pos: Pos2::ZERO,
            quick_create_selected_index: 0,
            edit_snapshot: None,
        }
    }
}
