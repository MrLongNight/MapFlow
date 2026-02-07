use crate::editors::mesh_editor::MeshEditor;
use crate::i18n::LocaleManager;
use crate::theme::colors;
use crate::UIAction;
use egui::epaint::CubicBezierShape;
use egui::{Color32, Pos2, Rect, Sense, Shadow, Stroke, TextureHandle, Ui, Vec2};
use mapmap_core::{
    audio_reactive::AudioTriggerData,
    module::{
        BlendModeType, EffectType as ModuleEffectType, HueNodeType, LayerType, MapFlowModule,
        MaskType, ModuleId, ModuleManager, ModulePart, ModulePartId, ModulePartType,
        ModuleSocketType, ModulizerType, NodeLinkData, SourceType, TriggerType,
    },
};

use egui_node_editor::*;
use std::borrow::Cow;

// --- Node Editor Types ---

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum MyDataType {
    Trigger,
    Media,
    Effect,
    Layer,
    Output,
    Link,
}

#[derive(Clone, Debug)]
pub struct MyNodeData {
    pub title: String,
    pub part_type: mapmap_core::module::ModulePartType,
    pub original_part_id: ModulePartId,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Default)]
pub struct MyValueType;

#[derive(Clone, Debug)]
pub struct MyNodeTemplate {
    pub label: String,
    pub part_type_variant: String,
}

#[derive(Clone, Debug, Default)]
pub struct MyUserState {
    pub trigger_values: std::collections::HashMap<ModulePartId, f32>,
}

impl DataTypeTrait<MyUserState> for MyDataType {
    fn data_type_color(&self, _user_state: &mut MyUserState) -> Color32 {
        match self {
            MyDataType::Trigger => Color32::from_rgb(180, 100, 220),
            MyDataType::Media => Color32::from_rgb(100, 180, 220),
            MyDataType::Effect => Color32::from_rgb(220, 180, 100),
            MyDataType::Layer => Color32::from_rgb(100, 220, 140),
            MyDataType::Output => Color32::from_rgb(220, 100, 100),
            MyDataType::Link => Color32::from_rgb(200, 200, 200),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            MyDataType::Trigger => Cow::Borrowed("Trigger"),
            MyDataType::Media => Cow::Borrowed("Media"),
            MyDataType::Effect => Cow::Borrowed("Effect"),
            MyDataType::Layer => Cow::Borrowed("Layer"),
            MyDataType::Output => Cow::Borrowed("Output"),
            MyDataType::Link => Cow::Borrowed("Link"),
        }
    }
}

impl NodeTemplateTrait for MyNodeTemplate {
    type NodeData = MyNodeData;
    type DataType = MyDataType;
    type ValueType = MyValueType;
    type UserState = MyUserState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        Cow::Borrowed(&self.label)
    }

    fn node_graph_label(&self, _user_state: &mut Self::UserState) -> String {
        self.label.clone()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        MyNodeData {
            title: self.label.clone(),
            part_type: mapmap_core::module::ModulePartType::Trigger(TriggerType::Beat), // Mock
            original_part_id: 0,
        }
    }

    fn build_node(
        &self,
        _graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        _node_id: NodeId,
    ) {
        // Mock
    }
}

impl NodeDataTrait for MyNodeData {
    type Response = MyResponse;
    type UserState = MyUserState;
    type DataType = MyDataType;
    type ValueType = MyValueType;

    fn can_delete(
        &self,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> bool {
        true
    }

    fn bottom_ui(
        &self,
        ui: &mut Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, Self>>
    where
        Self::Response: UserResponseTrait,
    {
        ui.label(format!("Type: {:?}", self.part_type));
        Vec::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MyResponse {
    Connect(NodeId, usize, NodeId, usize),
    Delete(NodeId),
}

impl UserResponseTrait for MyResponse {}

#[cfg(feature = "ndi")]
use mapmap_io::ndi::NdiSource;
#[cfg(feature = "ndi")]
use std::sync::mpsc;

/// Playback commands for media players
#[derive(Debug, Clone, PartialEq)]
pub enum MediaPlaybackCommand {
    Play,
    Pause,
    Stop,
    /// Reload the media from disk (used when path changes)
    Reload,
    /// Set playback speed (1.0 = normal)
    SetSpeed(f32),
    /// Set loop mode
    SetLoop(bool),
    /// Seek to position (seconds from start)
    Seek(f64),
    /// Set reverse playback
    SetReverse(bool),
}

/// Information about a media player's current state
#[derive(Debug, Clone, Default)]
pub struct MediaPlayerInfo {
    /// Current playback position in seconds
    pub current_time: f64,
    /// Total duration in seconds
    pub duration: f64,
    /// Whether the player is currently playing
    pub is_playing: bool,
}

/// Information about a socket position for hit detection
#[derive(Clone)]
struct SocketInfo {
    part_id: ModulePartId,
    socket_idx: usize,
    is_output: bool,
    socket_type: ModuleSocketType,
    position: Pos2,
}

#[allow(dead_code)]
pub struct ModuleCanvas {
    /// The ID of the currently active/edited module
    pub active_module_id: Option<u64>,
    /// Canvas pan offset
    pan_offset: Vec2,
    /// Canvas zoom level
    zoom: f32,
    /// Part being dragged
    dragging_part: Option<(ModulePartId, Vec2)>,
    /// Part being resized: (part_id, original_size)
    resizing_part: Option<(ModulePartId, (f32, f32))>,
    /// Box selection start position (screen coords)
    box_select_start: Option<Pos2>,
    /// Connection being created: (from_part, from_socket_idx, is_output, socket_type, start_pos)
    creating_connection: Option<(ModulePartId, usize, bool, ModuleSocketType, Pos2)>,
    /// Part ID pending deletion (set when X button clicked)
    pending_delete: Option<ModulePartId>,
    /// Selected parts for multi-select and copy/paste
    selected_parts: Vec<ModulePartId>,
    /// Clipboard for copy/paste (stores part types and relative positions)
    clipboard: Vec<(mapmap_core::module::ModulePartType, (f32, f32))>,
    /// Search filter text
    search_filter: String,
    /// Whether search popup is visible
    show_search: bool,
    /// Undo history stack
    undo_stack: Vec<CanvasAction>,
    /// Redo history stack
    redo_stack: Vec<CanvasAction>,
    /// Saved module presets
    presets: Vec<ModulePreset>,
    /// Whether preset panel is visible
    show_presets: bool,
    /// New preset name input
    new_preset_name: String,
    /// Context menu position
    context_menu_pos: Option<Pos2>,
    /// Context menu target (connection index or None)
    context_menu_connection: Option<usize>,
    /// Context menu target (part ID or None)
    context_menu_part: Option<ModulePartId>,
    /// MIDI Learn mode - which part is waiting for MIDI input
    midi_learn_part_id: Option<ModulePartId>,
    /// Whether we are currently panning the canvas (started on empty area)
    panning_canvas: bool,
    /// Cached textures for plug icons
    plug_icons: std::collections::HashMap<String, egui::TextureHandle>,
    /// Learned MIDI mapping: (part_id, channel, cc_or_note, is_note)
    learned_midi: Option<(ModulePartId, u8, u8, bool)>,
    /// Live audio trigger data from AudioAnalyzerV2
    audio_trigger_data: AudioTriggerData,

    /// Discovered NDI sources
    #[cfg(feature = "ndi")]
    ndi_sources: Vec<NdiSource>,
    /// Channel to receive discovered NDI sources from async task
    #[cfg(feature = "ndi")]
    ndi_discovery_rx: Option<mpsc::Receiver<Vec<NdiSource>>>,
    /// Pending NDI connection (part_id, source)
    #[cfg(feature = "ndi")]
    pending_ndi_connect: Option<(ModulePartId, NdiSource)>,
    /// Available outputs (id, name) for output node selection
    pub available_outputs: Vec<(u64, String)>,
    /// ID of the part being edited in a popup
    pub editing_part_id: Option<ModulePartId>,
    /// Video Texture Previews for Media Nodes ((Module ID, Part ID) -> Egui Texture)
    pub node_previews: std::collections::HashMap<(u64, u64), egui::TextureId>,
    /// Pending playback commands (Part ID, Command)
    pub pending_playback_commands: Vec<(ModulePartId, MediaPlaybackCommand)>,
    /// Last diagnostic check results
    pub diagnostic_issues: Vec<mapmap_core::diagnostics::ModuleIssue>,
    /// Whether diagnostic popup is shown
    show_diagnostics: bool,
    /// Media player info for timeline display (Part ID -> Info)
    pub player_info: std::collections::HashMap<ModulePartId, MediaPlayerInfo>,

    // Hue Integration
    /// Discovered Hue bridges
    pub hue_bridges: Vec<mapmap_control::hue::api::discovery::DiscoveredBridge>,
    /// Channel for Hue discovery results
    pub hue_discovery_rx: Option<
        std::sync::mpsc::Receiver<
            Result<Vec<mapmap_control::hue::api::discovery::DiscoveredBridge>, String>,
        >,
    >,
    /// Status message for Hue operations
    pub hue_status_message: Option<String>,
    /// Last known trigger values for visualization (Part ID -> Value 0.0-1.0)
    pub last_trigger_values: std::collections::HashMap<ModulePartId, f32>,

    /// Advanced Mesh Editor instance
    pub mesh_editor: MeshEditor,
    /// ID of the part currently being edited in the mesh editor (to detect selection changes)
    pub last_mesh_edit_id: Option<ModulePartId>,
}

pub type PresetPart = (
    mapmap_core::module::ModulePartType,
    (f32, f32),
    Option<(f32, f32)>,
);
pub type PresetConnection = (usize, usize, usize, usize); // from_idx, from_socket, to_idx, to_socket

/// A saved module preset/template
#[derive(Debug, Clone)]
pub struct ModulePreset {
    pub name: String,
    pub parts: Vec<PresetPart>,
    pub connections: Vec<PresetConnection>,
}

/// Actions that can be undone/redone
#[derive(Debug, Clone)]
pub enum CanvasAction {
    AddPart {
        part_id: ModulePartId,
        part_data: mapmap_core::module::ModulePart,
    },
    DeletePart {
        part_data: mapmap_core::module::ModulePart,
    },
    MovePart {
        part_id: ModulePartId,
        old_pos: (f32, f32),
        new_pos: (f32, f32),
    },
    AddConnection {
        connection: mapmap_core::module::ModuleConnection,
    },
    DeleteConnection {
        connection: mapmap_core::module::ModuleConnection,
    },
    Batch(Vec<CanvasAction>),
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
            presets: Self::default_presets(),
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
            #[cfg(feature = "ndi")]
            pending_ndi_connect: None,
            available_outputs: Vec::new(),
            editing_part_id: None,
            node_previews: std::collections::HashMap::new(),
            pending_playback_commands: Vec::new(),
            diagnostic_issues: Vec::new(),
            show_diagnostics: false,
            player_info: std::collections::HashMap::new(),
            hue_bridges: Vec::new(),
            hue_discovery_rx: None,
            hue_status_message: None,
            last_trigger_values: std::collections::HashMap::new(),
            mesh_editor: MeshEditor::new(),
            last_mesh_edit_id: None,
        }
    }
}

impl ModuleCanvas {
    fn ensure_icons_loaded(&mut self, ctx: &egui::Context) {
        if !self.plug_icons.is_empty() {
            return;
        }

        let paths = [
            "resources/stecker_icons",
            "../resources/stecker_icons",
            r"C:\Users\Vinyl\Desktop\VJMapper\VjMapper\resources\stecker_icons",
        ];

        let files = [
            "audio-jack.svg",
            "audio-jack_2.svg",
            "plug.svg",
            "power-plug.svg",
            "usb-cable.svg",
        ];

        for path_str in paths {
            let base_path = std::path::Path::new(path_str);
            if base_path.exists() {
                for file in files {
                    let path = base_path.join(file);
                    if let Some(texture) = Self::load_svg_icon(&path, ctx) {
                        self.plug_icons.insert(file.to_string(), texture);
                    }
                }
                if !self.plug_icons.is_empty() {
                    break;
                }
            }
        }
    }

    /// Sync the mesh editor with the current selection's mesh
    pub fn sync_mesh_editor_to_current_selection(
        &mut self,
        part: &mapmap_core::module::ModulePart,
    ) {
        use mapmap_core::module::{LayerType, MeshType, ModulePartType};

        // Extract MeshType from part
        let mesh = match &part.part_type {
            ModulePartType::Layer(LayerType::Single { mesh, .. }) => mesh,
            ModulePartType::Layer(LayerType::Group { mesh, .. }) => mesh,
            ModulePartType::Mesh(mesh) => mesh,
            _ => return, // Not a mesh-capable part
        };

        // Only reset if it's a different part
        if self.last_mesh_edit_id == Some(part.id) {
            return;
        }

        self.last_mesh_edit_id = Some(part.id);
        self.mesh_editor.mode = crate::editors::mesh_editor::EditMode::Select;

        // Visual scale for editor (0-1 -> 0-200)
        let scale = 200.0;

        match mesh {
            MeshType::Quad { tl, tr, br, bl } => {
                self.mesh_editor.set_from_quad(
                    egui::Pos2::new(tl.0 * scale, tl.1 * scale),
                    egui::Pos2::new(tr.0 * scale, tr.1 * scale),
                    egui::Pos2::new(br.0 * scale, br.1 * scale),
                    egui::Pos2::new(bl.0 * scale, bl.1 * scale),
                );
            }
            MeshType::BezierSurface { control_points } => {
                // Deserialize scaled points
                let points: Vec<(f32, f32)> = control_points
                    .iter()
                    .map(|(x, y)| (x * scale, y * scale))
                    .collect();
                self.mesh_editor.set_from_bezier_points(&points);
            }
            // Fallback for unsupported types - reset to default quad for now
            _ => {
                self.mesh_editor
                    .create_quad(egui::Pos2::new(100.0, 100.0), 200.0);
            }
        }
    }

    /// Apply mesh editor changes back to the selection
    pub fn apply_mesh_editor_to_selection(&mut self, part: &mut mapmap_core::module::ModulePart) {
        use mapmap_core::module::{LayerType, MeshType, ModulePartType};

        // Get mutable reference to mesh
        let mesh = match &mut part.part_type {
            ModulePartType::Layer(LayerType::Single { mesh, .. }) => mesh,
            ModulePartType::Layer(LayerType::Group { mesh, .. }) => mesh,
            ModulePartType::Mesh(mesh) => mesh,
            _ => return,
        };

        let scale = 200.0;

        // Try to update current mesh type
        match mesh {
            MeshType::Quad { tl, tr, br, bl } => {
                if let Some((p_tl, p_tr, p_br, p_bl)) = self.mesh_editor.get_quad_corners() {
                    *tl = (p_tl.x / scale, p_tl.y / scale);
                    *tr = (p_tr.x / scale, p_tr.y / scale);
                    *br = (p_br.x / scale, p_br.y / scale);
                    *bl = (p_bl.x / scale, p_bl.y / scale);
                }
            }
            MeshType::BezierSurface { control_points } => {
                let points = self.mesh_editor.get_bezier_points();
                *control_points = points.iter().map(|(x, y)| (x / scale, y / scale)).collect();
            }
            _ => {
                // Other types not yet supported for write-back
            }
        }
    }

    /// Render the unified mesh editor UI for a given mesh
    pub fn render_mesh_editor_ui(
        &mut self,
        ui: &mut Ui,
        mesh: &mut mapmap_core::module::MeshType,
        part_id: mapmap_core::module::ModulePartId,
        id_salt: u64,
    ) {
        use mapmap_core::module::MeshType;

        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("🕸️ Mesh/Geometry").strong());
            ui.separator();

            egui::ComboBox::from_id_salt(format!("mesh_type_{}", id_salt))
                .selected_text(match mesh {
                    MeshType::Quad { .. } => "Quad",
                    MeshType::Grid { .. } => "Grid",
                    MeshType::BezierSurface { .. } => "Bezier",
                    MeshType::Polygon { .. } => "Polygon",
                    MeshType::TriMesh => "Triangle",
                    MeshType::Circle { .. } => "Circle",
                    MeshType::Cylinder { .. } => "Cylinder",
                    MeshType::Sphere { .. } => "Sphere",
                    MeshType::Custom { .. } => "Custom",
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(matches!(mesh, MeshType::Quad { .. }), "Quad")
                        .clicked()
                    {
                        *mesh = MeshType::Quad {
                            tl: (0.0, 0.0),
                            tr: (1.0, 0.0),
                            br: (1.0, 1.0),
                            bl: (0.0, 1.0),
                        };
                        self.last_mesh_edit_id = None; // Trigger resync
                    }
                    if ui
                        .selectable_label(matches!(mesh, MeshType::Grid { .. }), "Grid")
                        .clicked()
                    {
                        *mesh = MeshType::Grid { rows: 4, cols: 4 };
                        self.last_mesh_edit_id = None; // Trigger resync
                    }
                    if ui
                        .selectable_label(matches!(mesh, MeshType::BezierSurface { .. }), "Bezier")
                        .clicked()
                    {
                        // Default bezier
                        *mesh = MeshType::BezierSurface {
                            control_points: vec![],
                        };
                        self.last_mesh_edit_id = None;
                    }
                });

            // Resync logic if type changed (handled by caller passing part, but here we just have mesh)
            if self.last_mesh_edit_id.is_none() {
                let scale = 200.0;
                match mesh {
                    MeshType::Quad { tl, tr, br, bl } => {
                        self.mesh_editor.set_from_quad(
                            egui::Pos2::new(tl.0 * scale, tl.1 * scale),
                            egui::Pos2::new(tr.0 * scale, tr.1 * scale),
                            egui::Pos2::new(br.0 * scale, br.1 * scale),
                            egui::Pos2::new(bl.0 * scale, bl.1 * scale),
                        );
                        self.last_mesh_edit_id = Some(part_id);
                    }
                    MeshType::BezierSurface { control_points } => {
                        // Deserialize scaled points
                        let points: Vec<(f32, f32)> = control_points
                            .iter()
                            .map(|(x, y)| (x * scale, y * scale))
                            .collect();
                        self.mesh_editor.set_from_bezier_points(&points);
                        self.last_mesh_edit_id = Some(part_id);
                    }
                    _ => {
                        // Fallback
                        self.mesh_editor
                            .create_quad(egui::Pos2::new(100.0, 100.0), 200.0);
                        self.last_mesh_edit_id = Some(part_id);
                    }
                }
            }

            ui.separator();
            ui.label("Visual Editor:");

            if let Some(_action) = self.mesh_editor.ui(ui) {
                // Sync back
                let scale = 200.0;
                match mesh {
                    MeshType::Quad { tl, tr, br, bl } => {
                        if let Some((p_tl, p_tr, p_br, p_bl)) = self.mesh_editor.get_quad_corners()
                        {
                            *tl = (p_tl.x / scale, p_tl.y / scale);
                            *tr = (p_tr.x / scale, p_tr.y / scale);
                            *br = (p_br.x / scale, p_br.y / scale);
                            *bl = (p_bl.x / scale, p_bl.y / scale);
                        }
                    }
                    MeshType::BezierSurface { control_points } => {
                        let points = self.mesh_editor.get_bezier_points();
                        *control_points =
                            points.iter().map(|(x, y)| (x / scale, y / scale)).collect();
                    }
                    _ => {}
                }
            }
        });
    }

    /// Takes all pending playback commands and clears the internal buffer.
    pub fn take_playback_commands(&mut self) -> Vec<(ModulePartId, MediaPlaybackCommand)> {
        std::mem::take(&mut self.pending_playback_commands)
    }

    /// Renders the property editor popup for the currently selected node.
    /// Get the ID of the selected part
    pub fn get_selected_part_id(&self) -> Option<ModulePartId> {
        self.selected_parts.last().copied()
    }

    /// Sets default parameters for a given effect type
    pub fn set_default_effect_params(
        effect_type: ModuleEffectType,
        params: &mut std::collections::HashMap<String, f32>,
    ) {
        use mapmap_core::module::EffectType;
        params.clear();
        match effect_type {
            EffectType::Blur => {
                params.insert("radius".to_string(), 5.0);
                params.insert("samples".to_string(), 9.0);
            }
            EffectType::Pixelate => {
                params.insert("pixel_size".to_string(), 8.0);
            }
            EffectType::FilmGrain => {
                params.insert("amount".to_string(), 0.1);
                params.insert("speed".to_string(), 1.0);
            }
            EffectType::Vignette => {
                params.insert("radius".to_string(), 0.5);
                params.insert("softness".to_string(), 0.5);
            }
            EffectType::ChromaticAberration => {
                params.insert("amount".to_string(), 0.01);
            }
            EffectType::EdgeDetect => {
                // Usually no params, or threshold?
            }
            EffectType::Brightness | EffectType::Contrast | EffectType::Saturation => {
                params.insert("brightness".to_string(), 0.0);
                params.insert("contrast".to_string(), 1.0);
                params.insert("saturation".to_string(), 1.0);
            }
            _ => {}
        }
    }

    pub fn render_inspector_for_part(
        &mut self,
        ui: &mut Ui,
        part: &mut mapmap_core::module::ModulePart,
        actions: &mut Vec<UIAction>,
        module_id: mapmap_core::module::ModuleId,
        shared_media_ids: &[String],
    ) {
        // Sync mesh editor state if needed
        self.sync_mesh_editor_to_current_selection(part);

        use mapmap_core::module::*;
        let part_id = part.id;
        let mut changed_part_id: Option<ModulePartId> = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // --- Input Configuration ---
                self.render_trigger_config_ui(ui, part);
                                ui.separator();

                                match &mut part.part_type {
                                    ModulePartType::Trigger(trigger) => {
                                        ui.label("Trigger Type:");
                                        match trigger {
                                            TriggerType::Beat => {
                                                ui.label("🥁 Beat Sync");
                                                ui.label("Triggers on BPM beat.");
                                            }
                                            TriggerType::AudioFFT { band: _band, threshold, output_config } => {
                                                ui.label("🔊 Audio FFT");
                                                ui.label("Outputs 9 frequency bands, plus volume and beat.");
                                                ui.add(
                                                    egui::Slider::new(threshold, 0.0..=1.0)
                                                        .text("Threshold"),
                                                );

                                                ui.separator();
                                                ui.label("📤 Output Configuration:");
                                                ui.checkbox(&mut output_config.beat_output, "🥁 Beat Detection");
                                                ui.checkbox(&mut output_config.bpm_output, "⏱️ BPM");
                                                ui.checkbox(&mut output_config.volume_outputs, "📊 Volume (RMS, Peak)");
                                                ui.checkbox(&mut output_config.frequency_bands, "🎵 Frequency Bands (9)");

                                                ui.separator();
                                                ui.collapsing("🔄 Invert Signals (NOT Logic)", |ui| {
                                                    ui.label("Select signals to invert (Active = 0.0):");

                                                    let mut toggle_invert = |ui: &mut Ui, name: &str, label: &str| {
                                                        let name_string = name.to_string();
                                                        let mut invert = output_config.inverted_outputs.contains(&name_string);
                                                        if ui.checkbox(&mut invert, label).changed() {
                                                            if invert {
                                                                output_config.inverted_outputs.insert(name_string);
                                                            } else {
                                                                output_config.inverted_outputs.remove(&name_string);
                                                            }
                                                        }
                                                    };

                                                    if output_config.beat_output {
                                                        toggle_invert(ui, "Beat Out", "🥁 Beat Out");
                                                    }
                                                    if output_config.bpm_output {
                                                        toggle_invert(ui, "BPM Out", "⏱️ BPM Out");
                                                    }
                                                    if output_config.volume_outputs {
                                                        toggle_invert(ui, "RMS Volume", "📊 RMS Volume");
                                                        toggle_invert(ui, "Peak Volume", "📊 Peak Volume");
                                                    }
                                                    if output_config.frequency_bands {
                                                        ui.label("Bands:");
                                                        toggle_invert(ui, "SubBass Out", "SubBass (20-60Hz)");
                                                        toggle_invert(ui, "Bass Out", "Bass (60-250Hz)");
                                                        toggle_invert(ui, "LowMid Out", "LowMid (250-500Hz)");
                                                        toggle_invert(ui, "Mid Out", "Mid (500-1kHz)");
                                                        toggle_invert(ui, "HighMid Out", "HighMid (1-2kHz)");
                                                        toggle_invert(ui, "UpperMid Out", "UpperMid (2-4kHz)");
                                                        toggle_invert(ui, "Presence Out", "Presence (4-6kHz)");
                                                        toggle_invert(ui, "Brilliance Out", "Brilliance (6-12kHz)");
                                                        toggle_invert(ui, "Air Out", "Air (12-20kHz)");
                                                    }
                                                });

                                                // Note: Changing output config requires regenerating sockets
                                                // This will be handled when the part is updated
                                                ui.label(
                                                    "Threshold is used for the node's visual glow effect.",
                                                );
                                            }
                                            TriggerType::Random {
                                                min_interval_ms,
                                                max_interval_ms,
                                                probability,
                                            } => {
                                                ui.label("🎲 Random");
                                                ui.add(
                                                    egui::Slider::new(min_interval_ms, 50..=5000)
                                                        .text("Min (ms)"),
                                                );
                                                ui.add(
                                                    egui::Slider::new(max_interval_ms, 100..=10000)
                                                        .text("Max (ms)"),
                                                );
                                                ui.add(
                                                    egui::Slider::new(probability, 0.0..=1.0)
                                                        .text("Probability"),
                                                );
                                            }
                                            TriggerType::Fixed {
                                                interval_ms,
                                                offset_ms,
                                                ..
                                            } => {
                                                ui.label("⏱️ Fixed Timer");
                                                ui.add(
                                                    egui::Slider::new(interval_ms, 16..=10000)
                                                        .text("Interval (ms)"),
                                                );
                                                ui.add(
                                                    egui::Slider::new(offset_ms, 0..=5000)
                                                        .text("Offset (ms)"),
                                                );
                                            }
                                            TriggerType::Midi { channel, note, device: _ } => {
                                                ui.label("🎹 MIDI Trigger");

                                                // Available MIDI ports dropdown
                                                ui.horizontal(|ui| {
                                                    ui.label("Device:");
                                                    #[cfg(feature = "midi")]
                                                    {
                                                        if let Ok(ports) =
                                                            mapmap_control::midi::MidiInputHandler::list_ports()
                                                        {
                                                            if ports.is_empty() {
                                                                ui.label("No MIDI devices");
                                                            } else {
                                                                egui::ComboBox::from_id_salt(
                                                                    "midi_device",
                                                                )
                                                                .selected_text(
                                                                    ports.first().cloned().unwrap_or_default(),
                                                                )
                                                                .show_ui(ui, |ui| {
                                                                    for port in &ports {
                                                                        let _ = ui.selectable_label(false, port);
                                                                    }
                                                                });
                                                            }
                                                        } else {
                                                            ui.label("MIDI unavailable");
                                                        }
                                                    }
                                                    #[cfg(not(feature = "midi"))]
                                                    {
                                                        ui.label("(MIDI disabled)");
                                                    }
                                                });

                                                ui.add(
                                                    egui::Slider::new(channel, 1..=16)
                                                        .text("Channel"),
                                                );
                                                ui.add(
                                                    egui::Slider::new(note, 0..=127).text("Note"),
                                                );

                                                // MIDI Learn button
                                                let is_learning =
                                                    self.midi_learn_part_id == Some(part_id);
                                                let learn_text = if is_learning {
                                                    "⏳ Waiting for MIDI..."
                                                } else {
                                                    "🎯 MIDI Learn"
                                                };
                                                if ui.button(learn_text).clicked() {
                                                    if is_learning {
                                                        self.midi_learn_part_id = None;
                                                    } else {
                                                        self.midi_learn_part_id = Some(part_id);
                                                    }
                                                }
                                                if is_learning {
                                                    ui.label("Press any MIDI key/knob...");
                                                }
                                            }
                                            TriggerType::Osc { address } => {
                                                ui.label("📡 OSC Trigger");
                                                ui.horizontal(|ui| {
                                                    ui.label("Address:");
                                                    ui.add(
                                                        egui::TextEdit::singleline(address)
                                                            .desired_width(150.0),
                                                    );
                                                });
                                                ui.label("Format: /path/to/trigger");
                                                ui.label("Default port: 8000");
                                            }
                                            TriggerType::Shortcut {
                                                key_code,
                                                modifiers,
                                            } => {
                                                ui.label("⌨️ Shortcut");
                                                ui.horizontal(|ui| {
                                                    ui.label("Key:");
                                                    ui.text_edit_singleline(key_code);
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Mods:");
                                                    ui.label(format!(
                                                        "Ctrl={} Shift={} Alt={}",
                                                        *modifiers & 1 != 0,
                                                        *modifiers & 2 != 0,
                                                        *modifiers & 4 != 0
                                                    ));
                                                });
                                            }
                                        }
                                    }
                                    ModulePartType::Source(source) => {
                                        ui.horizontal(|ui| {
                                            ui.label("Source Type:");
                                            let current_mode = match source {
                                                SourceType::MediaFile { .. } => "📹 Media File",
                                                SourceType::VideoUni { .. } => "📹 Video (Uni)",
                                                SourceType::ImageUni { .. } => "🖼 Image (Uni)",
                                                SourceType::VideoMulti { .. } => "🔗 Video (Multi)",
                                                SourceType::ImageMulti { .. } => "🔗 Image (Multi)",
                                                SourceType::Shader { .. } => "🎨 Shader",
                                                SourceType::LiveInput { .. } => "📹 Live Input",
                                                SourceType::NdiInput { .. } => "📡 NDI Input",
                                                #[cfg(target_os = "windows")]
                                                SourceType::SpoutInput { .. } => "🚰 Spout Input",
                                                SourceType::Bevy => "🎮 Bevy Scene",
                                                SourceType::BevyAtmosphere { .. } => "☁️ Atmosphere",
                                                SourceType::BevyHexGrid { .. } => "🛑 Hex Grid",
                                                SourceType::BevyParticles { .. } => "✨ Particles",
                                            };

                                            let mut next_type = None;
                                            egui::ComboBox::from_id_salt(format!("{}_source_type_picker", part_id))
                                                .selected_text(current_mode)
                                                .show_ui(ui, |ui| {
                                                    ui.label("--- File Based ---");
                                                    if ui.selectable_label(matches!(source, SourceType::MediaFile { .. }), "📹 Media File").clicked() { next_type = Some("MediaFile"); }
                                                    if ui.selectable_label(matches!(source, SourceType::VideoUni { .. }), "📹 Video (Uni)").clicked() { next_type = Some("VideoUni"); }
                                                    if ui.selectable_label(matches!(source, SourceType::ImageUni { .. }), "🖼 Image (Uni)").clicked() { next_type = Some("ImageUni"); }

                                                    ui.label("--- Shared ---");
                                                    if ui.selectable_label(matches!(source, SourceType::VideoMulti { .. }), "🔗 Video (Multi)").clicked() { next_type = Some("VideoMulti"); }
                                                    if ui.selectable_label(matches!(source, SourceType::ImageMulti { .. }), "🔗 Image (Multi)").clicked() { next_type = Some("ImageMulti"); }
                                                });

                                            if let Some(t) = next_type {
                                                let path = match source {
                                                    SourceType::MediaFile { path, .. } => path.clone(),
                                                    SourceType::VideoUni { path, .. } => path.clone(),
                                                    SourceType::ImageUni { path, .. } => path.clone(),
                                                    _ => String::new(),
                                                };
                                                let shared_id = match source {
                                                    SourceType::VideoMulti { shared_id, .. } => shared_id.clone(),
                                                    SourceType::ImageMulti { shared_id, .. } => shared_id.clone(),
                                                    _ => String::new(),
                                                };

                                                *source = match t {
                                                    "MediaFile" => SourceType::new_media_file(if path.is_empty() { shared_id } else { path }),
                                                    "VideoUni" => SourceType::VideoUni {
                                                        path: if path.is_empty() { shared_id } else { path },
                                                        speed: 1.0, loop_enabled: true, start_time: 0.0, end_time: 0.0,
                                                        opacity: 1.0, blend_mode: None, brightness: 0.0, contrast: 1.0, saturation: 1.0, hue_shift: 0.0,
                                                        scale_x: 1.0, scale_y: 1.0, rotation: 0.0, offset_x: 0.0, offset_y: 0.0,
                                                        target_width: None, target_height: None, target_fps: None,
                                                        flip_horizontal: false, flip_vertical: false, reverse_playback: false,
                                                    },
                                                    "ImageUni" => SourceType::ImageUni {
                                                        path: if path.is_empty() { shared_id } else { path },
                                                        opacity: 1.0, blend_mode: None, brightness: 0.0, contrast: 1.0, saturation: 1.0, hue_shift: 0.0,
                                                        scale_x: 1.0, scale_y: 1.0, rotation: 0.0, offset_x: 0.0, offset_y: 0.0,
                                                        target_width: None, target_height: None,
                                                        flip_horizontal: false, flip_vertical: false,
                                                    },
                                                    "VideoMulti" => SourceType::VideoMulti {
                                                        shared_id: if shared_id.is_empty() { path } else { shared_id },
                                                        opacity: 1.0, blend_mode: None, brightness: 0.0, contrast: 1.0, saturation: 1.0, hue_shift: 0.0,
                                                        scale_x: 1.0, scale_y: 1.0, rotation: 0.0, offset_x: 0.0, offset_y: 0.0,
                                                        flip_horizontal: false, flip_vertical: false,
                                                    },
                                                    "ImageMulti" => SourceType::ImageMulti {
                                                        shared_id: if shared_id.is_empty() { path } else { shared_id },
                                                        opacity: 1.0, blend_mode: None, brightness: 0.0, contrast: 1.0, saturation: 1.0, hue_shift: 0.0,
                                                        scale_x: 1.0, scale_y: 1.0, rotation: 0.0, offset_x: 0.0, offset_y: 0.0,
                                                        flip_horizontal: false, flip_vertical: false,
                                                    },
                                                    _ => source.clone(),
                                                };
                                            }
                                        });

                                        ui.separator();

                                        match source {
                                            SourceType::MediaFile {
                                                path, speed, loop_enabled, start_time, end_time, opacity, blend_mode,
                                                brightness, contrast, saturation, hue_shift, scale_x, scale_y, rotation,
                                                offset_x, offset_y, flip_horizontal, flip_vertical, reverse_playback, ..
                                            } | SourceType::VideoUni {
                                                path, speed, loop_enabled, start_time, end_time, opacity, blend_mode,
                                                brightness, contrast, saturation, hue_shift, scale_x, scale_y, rotation,
                                                offset_x, offset_y, flip_horizontal, flip_vertical, reverse_playback, ..
                                            } => {
                                                 // Media Picker (common for file-based video)
                                                if path.is_empty() {
                                                    ui.vertical_centered(|ui| {
                                                        ui.add_space(10.0);
                                                        if ui.add(egui::Button::new("📂 Select Media File").min_size(egui::vec2(150.0, 30.0))).clicked() {
                                                            actions.push(crate::UIAction::PickMediaFile(module_id, part_id, "".to_string()));
                                                        }
                                                        ui.label(egui::RichText::new("No media loaded").weak());
                                                        ui.add_space(10.0);
                                                    });
                                                } else {
                                                    ui.collapsing("📁 File Info", |ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.label("Path:");
                                                            ui.add(egui::TextEdit::singleline(path).desired_width(160.0));
                                                            if ui.button("📂").on_hover_text("Select Media File").clicked() {
                                                                actions.push(crate::UIAction::PickMediaFile(module_id, part_id, "".to_string()));
                                                            }
                                                        });
                                                    });
                                                }

                                                // Playback Info
                                                let player_info = self.player_info.get(&part_id).cloned().unwrap_or_default();
                                                let video_duration = player_info.duration.max(1.0) as f32;
                                                let current_pos = player_info.current_time as f32;
                                                let is_playing = player_info.is_playing;

                                                // Timecode
                                                let current_min = (current_pos / 60.0) as u32;
                                                let current_sec = (current_pos % 60.0) as u32;
                                                let current_frac = ((current_pos * 100.0) % 100.0) as u32;
                                                let duration_min = (video_duration / 60.0) as u32;
                                                let duration_sec = (video_duration % 60.0) as u32;
                                                let duration_frac = ((video_duration * 100.0) % 100.0) as u32;

                                                ui.add_space(5.0);
                                                ui.vertical_centered(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "{:02}:{:02}.{:02} / {:02}:{:02}.{:02}",
                                                            current_min, current_sec, current_frac,
                                                            duration_min, duration_sec, duration_frac
                                                        ))
                                                        .monospace().size(22.0).strong()
                                                        .color(if is_playing { Color32::from_rgb(100, 255, 150) } else { Color32::from_rgb(200, 200, 200) })
                                                    );
                                                });
                                                ui.add_space(10.0);

                                                self.render_transport_controls(ui, part_id, is_playing, current_pos, loop_enabled, reverse_playback);

                                                ui.add_space(10.0);

                                                // Preview
                                                if let Some(tex_id) = self.node_previews.get(&(module_id, part_id)) {
                                                    let size = Vec2::new(ui.available_width(), ui.available_width() * 9.0 / 16.0);
                                                    ui.image((*tex_id, size));
                                                }
                                                ui.add_space(4.0);

                                                self.render_timeline(ui, part_id, video_duration, current_pos, start_time, end_time);

                                                // Safe Reset Clip (Mary StyleUX)
                                                ui.vertical_centered(|ui| {
                                                    ui.add_space(4.0);
                                                    if crate::widgets::hold_to_action_button(
                                                        ui,
                                                        "⟲ Reset Clip",
                                                        colors::WARN_COLOR,
                                                    ) {
                                                        *start_time = 0.0;
                                                        *end_time = 0.0;
                                                    }
                                                });

                                                ui.add_space(8.0);
                                                ui.horizontal(|ui| {
                                                    ui.label("Playback Speed:");
                                                    let speed_slider = ui.add(egui::Slider::new(speed, 0.1..=4.0).suffix("x").show_value(true));
                                                    if speed_slider.changed() {
                                                        actions.push(UIAction::MediaCommand(part_id, MediaPlaybackCommand::SetSpeed(*speed)));
                                                    }
                                                });
                                                ui.separator();

                                                // === VIDEO OPTIONS ===
                                                ui.collapsing("🎬 Video Options", |ui| {
                                                    let mut reverse = *reverse_playback;
                                                    if ui.checkbox(&mut reverse, "⏪ Reverse Playback").changed() {
                                                        actions.push(crate::UIAction::MediaCommand(part_id, MediaPlaybackCommand::SetReverse(reverse)));
                                                    }

                                                    ui.separator();
                                                    ui.label("Seek Position:");
                                                    // Note: Actual seek requires video duration from player
                                                    // For now, just show the control - needs integration with player state
                                                    let mut seek_pos: f64 = 0.0;
                                                    let seek_slider = ui.add(
                                                        egui::Slider::new(&mut seek_pos, 0.0..=100.0)
                                                            .text("Position")
                                                            .suffix("%")
                                                            .show_value(true)
                                                    );
                                                    if seek_slider.drag_stopped() && seek_slider.changed() {
                                                        // Convert percentage to duration-based seek
                                                        // This will need actual video duration from player
                                                        self.pending_playback_commands.push((part_id, MediaPlaybackCommand::Seek(seek_pos / 100.0 * 300.0)));
                                                    }
                                                });
                                                ui.separator();

                                                Self::render_common_controls(
                                                    ui, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                                                    scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical
                                                );
                                            }
                                            SourceType::ImageUni {
                                                path, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                                                scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical, ..
                                            } => {
                                                // Image Picker
                                                if path.is_empty() {
                                                    ui.vertical_centered(|ui| {
                                                        ui.add_space(10.0);
                                                        if ui.add(egui::Button::new("📂 Select Image File").min_size(egui::vec2(150.0, 30.0))).clicked() {
                                                            actions.push(crate::UIAction::PickMediaFile(module_id, part_id, "".to_string()));
                                                        }
                                                        ui.label(egui::RichText::new("No image loaded").weak());
                                                        ui.add_space(10.0);
                                                    });
                                                } else {
                                                    ui.collapsing("📁 File Info", |ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.label("Path:");
                                                            ui.add(egui::TextEdit::singleline(path).desired_width(160.0));
                                                            if ui.button("📂").on_hover_text("Select Image File").clicked() {
                                                                actions.push(crate::UIAction::PickMediaFile(module_id, part_id, "".to_string()));
                                                            }
                                                        });
                                                    });
                                                }

                                                ui.separator();
                                                Self::render_common_controls(
                                                    ui, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                                                    scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical
                                                );

                                            }
                                            SourceType::VideoMulti {
                                                shared_id, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                                                scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical, ..
                                            } => {
                                                ui.label("🔗 Shared Video Source");
                                                ui.horizontal(|ui| {
                                                    ui.label("Shared ID:");
                                                    ui.add(egui::TextEdit::singleline(shared_id).hint_text("Enter ID...").desired_width(140.0));

                                                    egui::ComboBox::from_id_salt("shared_media_video")
                                                        .selected_text("Select Existing")
                                                        .show_ui(ui, |ui| {
                                                            for id in shared_media_ids {
                                                                if ui.selectable_label(shared_id == id, id).clicked() {
                                                                    *shared_id = id.clone();
                                                                }
                                                            }
                                                        });
                                                });
                                                ui.label(egui::RichText::new("Use the same ID to sync multiple nodes.").weak().small());

                                                ui.separator();
                                                Self::render_common_controls(
                                                    ui, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                                                    scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical
                                                );
                                            }
                                            SourceType::ImageMulti {
                                                shared_id, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                                                scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical, ..
                                            } => {
                                                 ui.label("🔗 Shared Image Source");
                                                ui.horizontal(|ui| {
                                                    ui.label("Shared ID:");
                                                    ui.add(egui::TextEdit::singleline(shared_id).hint_text("Enter ID...").desired_width(140.0));

                                                    egui::ComboBox::from_id_salt("shared_media_image")
                                                        .selected_text("Select Existing")
                                                        .show_ui(ui, |ui| {
                                                            for id in shared_media_ids {
                                                                if ui.selectable_label(shared_id == id, id).clicked() {
                                                                    *shared_id = id.clone();
                                                                }
                                                            }
                                                        });
                                                });
                                                ui.label(egui::RichText::new("Use the same ID to sync multiple nodes.").weak().small());

                                                ui.separator();
                                                Self::render_common_controls(
                                                    ui, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                                                    scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical
                                                );
                                            }                                            SourceType::Shader { name, params: _ } => {
                                                ui.label("🎨 Shader");
                                                egui::Grid::new("shader_grid")
                                                    .num_columns(2)
                                                    .spacing([10.0, 8.0])
                                                    .show(ui, |ui| {
                                                        ui.label("Name:");
                                                        ui.text_edit_singleline(name);
                                                        ui.end_row();
                                                    });
                                            }
                                            SourceType::LiveInput { device_id } => {
                                                ui.label("📹 Live Input");
                                                egui::Grid::new("live_input_grid")
                                                    .num_columns(2)
                                                    .spacing([10.0, 8.0])
                                                    .show(ui, |ui| {
                                                        ui.label("Device ID:");
                                                        ui.add(egui::Slider::new(device_id, 0..=10));
                                                        ui.end_row();
                                                    });
                                            }
                                            #[cfg(feature = "ndi")]
                                            SourceType::NdiInput { source_name } => {
                                                ui.label("📡 NDI Input");

                                                // Smart Empty State
                                                if source_name.is_none()
                                                    && self.ndi_sources.is_empty()
                                                    && self.ndi_discovery_rx.is_none()
                                                {
                                                    ui.vertical_centered(|ui| {
                                                        ui.add_space(10.0);
                                                        if ui
                                                            .add(
                                                                egui::Button::new(
                                                                    "🔍 Discover Sources",
                                                                )
                                                                .min_size(egui::vec2(150.0, 30.0)),
                                                            )
                                                            .clicked()
                                                        {
                                                            // Start async discovery
                                                            let (tx, rx) =
                                                                std::sync::mpsc::channel();
                                                            self.ndi_discovery_rx = Some(rx);
                                                            mapmap_io::ndi::NdiReceiver::discover_sources_async(tx);
                                                            self.ndi_sources.clear();
                                                            ui.ctx().request_repaint();
                                                        }
                                                        ui.label(
                                                            egui::RichText::new(
                                                                "No NDI source selected",
                                                            )
                                                            .weak(),
                                                        );
                                                        ui.add_space(10.0);
                                                    });
                                                } else {
                                                    // Display current source
                                                    let display_name = source_name
                                                        .clone()
                                                        .unwrap_or_else(|| {
                                                            "Not Connected".to_string()
                                                        });
                                                    ui.label(format!("Current: {}", display_name));

                                                    // Discover button
                                                    ui.horizontal(|ui| {
                                                        if ui
                                                            .button("🔍 Discover Sources")
                                                            .clicked()
                                                        {
                                                            // Start async discovery
                                                            let (tx, rx) =
                                                                std::sync::mpsc::channel();
                                                            self.ndi_discovery_rx = Some(rx);
                                                            mapmap_io::ndi::NdiReceiver::discover_sources_async(tx);
                                                            self.ndi_sources.clear();
                                                            ui.ctx().request_repaint();
                                                        }

                                                        // Check for discovery results
                                                        if let Some(rx) = &self.ndi_discovery_rx {
                                                            if let Ok(sources) = rx.try_recv() {
                                                                self.ndi_sources = sources;
                                                                self.ndi_discovery_rx = None;
                                                            }
                                                        }

                                                        // Show spinner if discovering
                                                        if self.ndi_discovery_rx.is_some() {
                                                            ui.spinner();
                                                            ui.label("Searching...");
                                                        }
                                                    });

                                                    // Source selection dropdown
                                                    if !self.ndi_sources.is_empty() {
                                                        ui.separator();
                                                        ui.label("Available Sources:");

                                                        egui::ComboBox::from_id_salt(
                                                            "ndi_source_select",
                                                        )
                                                        .selected_text(display_name.clone())
                                                        .show_ui(ui, |ui| {
                                                            // Option to disconnect
                                                            if ui
                                                                .selectable_label(
                                                                    source_name.is_none(),
                                                                    "❌ None (Disconnect)",
                                                                )
                                                                .clicked()
                                                            {
                                                                *source_name = None;
                                                            }

                                                            // Available sources
                                                            for ndi_source in &self.ndi_sources {
                                                                let selected = source_name.as_ref()
                                                                    == Some(&ndi_source.name);
                                                                if ui
                                                                    .selectable_label(
                                                                        selected,
                                                                        &ndi_source.name,
                                                                    )
                                                                    .clicked()
                                                                {
                                                                    *source_name = Some(
                                                                        ndi_source.name.clone(),
                                                                    );

                                                                    // Trigger connection action
                                                                    self.pending_ndi_connect =
                                                                        Some((
                                                                            part_id,
                                                                            ndi_source.clone(),
                                                                        ));
                                                                }
                                                            }
                                                        });

                                                        ui.label(format!(
                                                            "Found {} source(s)",
                                                            self.ndi_sources.len()
                                                        ));
                                                    } else if self.ndi_discovery_rx.is_none() {
                                                        ui.label(
                                                            "Click 'Discover' to find NDI sources",
                                                        );
                                                    }
                                                }
                                            }
                                            #[cfg(not(feature = "ndi"))]
                                            SourceType::NdiInput { .. } => {
                                                ui.label("📡 NDI Input (Feature Disabled)");
                                            }
                                            #[cfg(target_os = "windows")]
                                            SourceType::SpoutInput { sender_name } => {
                                                ui.label("🚰 Spout Input");
                                                ui.horizontal(|ui| {
                                                    ui.label("Sender:");
                                                    ui.text_edit_singleline(sender_name);
                                                });
                                            }
                                            SourceType::BevyAtmosphere { .. } | SourceType::BevyHexGrid { .. } | SourceType::BevyParticles { .. } => {
                                                ui.label("Controls for this Bevy node are not yet implemented in UI.");
                                            }
                                            SourceType::Bevy => {
                                                ui.label("🎮 Bevy Scene");
                                                ui.label(egui::RichText::new("Rendering Internal 3D Scene").weak());
                                                ui.small("The scene is rendered internally and available as 'bevy_output'");
                                            }

                                        }
                                    }
                                    ModulePartType::Mask(mask) => {
                                        ui.label("Mask Type:");
                                        match mask {
                                            MaskType::File { path } => {
                                                ui.label("📁 Mask File");
                                                if path.is_empty() {
                                                    ui.vertical_centered(|ui| {
                                                        ui.add_space(10.0);
                                                        if ui.add(egui::Button::new("📂 Select Mask File")
                                                            .min_size(egui::vec2(150.0, 30.0)))
                                                            .clicked()
                                                        {
                                                            if let Some(picked) = rfd::FileDialog::new()
                                                                .add_filter(
                                                                    "Image",
                                                                    &[
                                                                        "png", "jpg", "jpeg", "webp",
                                                                        "bmp",
                                                                    ],
                                                                )
                                                                .pick_file()
                                                            {
                                                                *path = picked.display().to_string();
                                                            }
                                                        }
                                                        ui.label(egui::RichText::new("No mask loaded").weak());
                                                        ui.add_space(10.0);
                                                    });
                                                } else {
                                                    ui.horizontal(|ui| {
                                                        ui.add(
                                                            egui::TextEdit::singleline(path)
                                                                .desired_width(120.0),
                                                        );
                                                        if ui.button("📂").on_hover_text("Select Mask File").clicked() {
                                                            if let Some(picked) = rfd::FileDialog::new()
                                                                .add_filter(
                                                                    "Image",
                                                                    &[
                                                                        "png", "jpg", "jpeg", "webp",
                                                                        "bmp",
                                                                    ],
                                                                )
                                                                .pick_file()
                                                            {
                                                                *path = picked.display().to_string();
                                                            }
                                                        }
                                                    });
                                                }
                                            }
                                            MaskType::Shape(shape) => {
                                                ui.label("🔷 Shape Mask");
                                                egui::ComboBox::from_id_salt("mask_shape")
                                                    .selected_text(format!("{:?}", shape))
                                                    .show_ui(ui, |ui| {
                                                        if ui
                                                            .selectable_label(
                                                                matches!(shape, MaskShape::Circle),
                                                                "Circle",
                                                            )
                                                            .clicked()
                                                        {
                                                            *shape = MaskShape::Circle;
                                                        }
                                                        if ui
                                                            .selectable_label(
                                                                matches!(
                                                                    shape,
                                                                    MaskShape::Rectangle
                                                                ),
                                                                "Rectangle",
                                                            )
                                                            .clicked()
                                                        {
                                                            *shape = MaskShape::Rectangle;
                                                        }
                                                        if ui
                                                            .selectable_label(
                                                                matches!(
                                                                    shape,
                                                                    MaskShape::Triangle
                                                                ),
                                                                "Triangle",
                                                            )
                                                            .clicked()
                                                        {
                                                            *shape = MaskShape::Triangle;
                                                        }
                                                        if ui
                                                            .selectable_label(
                                                                matches!(shape, MaskShape::Star),
                                                                "Star",
                                                            )
                                                            .clicked()
                                                        {
                                                            *shape = MaskShape::Star;
                                                        }
                                                        if ui
                                                            .selectable_label(
                                                                matches!(shape, MaskShape::Ellipse),
                                                                "Ellipse",
                                                            )
                                                            .clicked()
                                                        {
                                                            *shape = MaskShape::Ellipse;
                                                        }
                                                    });
                                            }
                                            MaskType::Gradient { angle, softness } => {
                                                ui.label("🌈 Gradient Mask");
                                                ui.add(
                                                    egui::Slider::new(angle, 0.0..=360.0)
                                                        .text("Angle °"),
                                                );
                                                ui.add(
                                                    egui::Slider::new(softness, 0.0..=1.0)
                                                        .text("Softness"),
                                                );
                                            }
                                        }
                                    }
                                    ModulePartType::Modulizer(mod_type) => {
                                        ui.label("Modulator:");
                                        match mod_type {
                                            ModulizerType::Effect { effect_type: effect, params } => {
                                                // === LIVE HEADER ===
                                                ui.add_space(5.0);

                                                // 1. Big Title
                                                ui.vertical_centered(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(effect.name())
                                                            .size(22.0)
                                                            .color(Color32::from_rgb(100, 200, 255))
                                                            .strong(),
                                                    );
                                                });
                                                ui.add_space(10.0);

                                                // 2. Safe Reset Button (Prominent)
                                                ui.vertical_centered(|ui| {
                                                    if crate::widgets::hold_to_action_button(
                                                        ui,
                                                        "⟲ Safe Reset",
                                                        Color32::from_rgb(255, 180, 0),
                                                    ) {
                                                        Self::set_default_effect_params(
                                                            *effect, params,
                                                        );
                                                    }
                                                });

                                                ui.add_space(10.0);
                                                ui.separator();

                                                let mut changed_type = None;

                                                egui::ComboBox::from_id_salt(format!("{}_effect", part_id))
                                                    .selected_text(effect.name())
                                                    .show_ui(ui, |ui| {
                                                        ui.label("--- Basic ---");
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Blur), "Blur").clicked() { changed_type = Some(ModuleEffectType::Blur); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Invert), "Invert").clicked() { changed_type = Some(ModuleEffectType::Invert); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Sharpen), "Sharpen").clicked() { changed_type = Some(ModuleEffectType::Sharpen); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Threshold), "Threshold").clicked() { changed_type = Some(ModuleEffectType::Threshold); }

                                                        ui.label("--- Color ---");
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Brightness), "Brightness").clicked() { changed_type = Some(ModuleEffectType::Brightness); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Contrast), "Contrast").clicked() { changed_type = Some(ModuleEffectType::Contrast); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Saturation), "Saturation").clicked() { changed_type = Some(ModuleEffectType::Saturation); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::HueShift), "Hue Shift").clicked() { changed_type = Some(ModuleEffectType::HueShift); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Colorize), "Colorize").clicked() { changed_type = Some(ModuleEffectType::Colorize); }

                                                        ui.label("--- Distortion ---");
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Wave), "Wave").clicked() { changed_type = Some(ModuleEffectType::Wave); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Spiral), "Spiral").clicked() { changed_type = Some(ModuleEffectType::Spiral); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Kaleidoscope), "Kaleidoscope").clicked() { changed_type = Some(ModuleEffectType::Kaleidoscope); }

                                                        ui.label("--- Stylize ---");
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Pixelate), "Pixelate").clicked() { changed_type = Some(ModuleEffectType::Pixelate); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::EdgeDetect), "Edge Detect").clicked() { changed_type = Some(ModuleEffectType::EdgeDetect); }

                                                        ui.label("--- Composite ---");
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::RgbSplit), "RGB Split").clicked() { changed_type = Some(ModuleEffectType::RgbSplit); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::ChromaticAberration), "Chromatic").clicked() { changed_type = Some(ModuleEffectType::ChromaticAberration); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::FilmGrain), "Film Grain").clicked() { changed_type = Some(ModuleEffectType::FilmGrain); }
                                                        if ui.selectable_label(matches!(effect, ModuleEffectType::Vignette), "Vignette").clicked() { changed_type = Some(ModuleEffectType::Vignette); }
                                                    });

                                                if let Some(new_type) = changed_type {
                                                    *effect = new_type;
                                                    Self::set_default_effect_params(new_type, params);
                                                }

                                                ui.separator();
                                                match effect {
                                                    ModuleEffectType::Blur => {
                                                        let val = params.entry("radius".to_string()).or_insert(5.0);
                                                        ui.add(egui::Slider::new(val, 0.0..=50.0).text("Radius"));
                                                        let samples = params.entry("samples".to_string()).or_insert(9.0);
                                                        ui.add(egui::Slider::new(samples, 1.0..=20.0).text("Samples"));
                                                    }
                                                    ModuleEffectType::Pixelate => {
                                                        let val = params.entry("pixel_size".to_string()).or_insert(8.0);
                                                        ui.add(egui::Slider::new(val, 1.0..=100.0).text("Pixel Size"));
                                                    }
                                                    ModuleEffectType::FilmGrain => {
                                                        let amt = params.entry("amount".to_string()).or_insert(0.1);
                                                        ui.add(egui::Slider::new(amt, 0.0..=1.0).text("Amount"));
                                                        let spd = params.entry("speed".to_string()).or_insert(1.0);
                                                        ui.add(egui::Slider::new(spd, 0.0..=5.0).text("Speed"));
                                                    }
                                                    ModuleEffectType::Vignette => {
                                                        let rad = params.entry("radius".to_string()).or_insert(0.5);
                                                        ui.add(egui::Slider::new(rad, 0.0..=1.0).text("Radius"));
                                                        let soft = params.entry("softness".to_string()).or_insert(0.5);
                                                        ui.add(egui::Slider::new(soft, 0.0..=1.0).text("Softness"));
                                                    }
                                                    ModuleEffectType::ChromaticAberration => {
                                                        let amt = params.entry("amount".to_string()).or_insert(0.01);
                                                        ui.add(egui::Slider::new(amt, 0.0..=0.1).text("Amount"));
                                                    }
                                                    ModuleEffectType::Brightness | ModuleEffectType::Contrast | ModuleEffectType::Saturation => {
                                                        let bri = params.entry("brightness".to_string()).or_insert(0.0);
                                                        ui.add(egui::Slider::new(bri, -1.0..=1.0).text("Brightness"));
                                                        let con = params.entry("contrast".to_string()).or_insert(1.0);
                                                        ui.add(egui::Slider::new(con, 0.0..=2.0).text("Contrast"));
                                                        let sat = params.entry("saturation".to_string()).or_insert(1.0);
                                                        ui.add(egui::Slider::new(sat, 0.0..=2.0).text("Saturation"));
                                                    }
                                                    _ => {
                                                        ui.label("No configurable parameters");
                                                    }
                                                }
                                            }
                                            ModulizerType::BlendMode(blend) => {
                                                ui.label("🎨 Blend Mode");
                                                egui::ComboBox::from_id_salt("blend_mode")
                                                    .selected_text(format!("{:?}", blend))
                                                    .show_ui(ui, |ui| {
                                                        if ui.selectable_label(matches!(blend, BlendModeType::Normal), "Normal").clicked() { *blend = BlendModeType::Normal; }
                                                        if ui.selectable_label(matches!(blend, BlendModeType::Add), "Add").clicked() { *blend = BlendModeType::Add; }
                                                        if ui.selectable_label(matches!(blend, BlendModeType::Multiply), "Multiply").clicked() { *blend = BlendModeType::Multiply; }
                                                        if ui.selectable_label(matches!(blend, BlendModeType::Screen), "Screen").clicked() { *blend = BlendModeType::Screen; }
                                                        if ui.selectable_label(matches!(blend, BlendModeType::Overlay), "Overlay").clicked() { *blend = BlendModeType::Overlay; }
                                                        if ui.selectable_label(matches!(blend, BlendModeType::Difference), "Difference").clicked() { *blend = BlendModeType::Difference; }
                                                        if ui.selectable_label(matches!(blend, BlendModeType::Exclusion), "Exclusion").clicked() { *blend = BlendModeType::Exclusion; }
                                                    });
                                                ui.add(
                                                    egui::Slider::new(&mut 1.0_f32, 0.0..=1.0)
                                                        .text("Opacity"),
                                                );
                                            }
                                            ModulizerType::AudioReactive { source } => {
                                                ui.label("🔊 Audio Reactive");
                                                ui.horizontal(|ui| {
                                                    ui.label("Source:");
                                                    egui::ComboBox::from_id_salt("audio_source")
                                                        .selected_text(source.as_str())
                                                        .show_ui(ui, |ui| {
                                                            if ui.selectable_label(source == "SubBass", "SubBass").clicked() { *source = "SubBass".to_string(); }
                                                            if ui.selectable_label(source == "Bass", "Bass").clicked() { *source = "Bass".to_string(); }
                                                            if ui.selectable_label(source == "LowMid", "LowMid").clicked() { *source = "LowMid".to_string(); }
                                                            if ui.selectable_label(source == "Mid", "Mid").clicked() { *source = "Mid".to_string(); }
                                                            if ui.selectable_label(source == "HighMid", "HighMid").clicked() { *source = "HighMid".to_string(); }
                                                            if ui.selectable_label(source == "Presence", "Presence").clicked() { *source = "Presence".to_string(); }
                                                            if ui.selectable_label(source == "Brilliance", "Brilliance").clicked() { *source = "Brilliance".to_string(); }
                                                            if ui.selectable_label(source == "RMS", "RMS Volume").clicked() { *source = "RMS".to_string(); }
                                                            if ui.selectable_label(source == "Peak", "Peak").clicked() { *source = "Peak".to_string(); }
                                                            if ui.selectable_label(source == "BPM", "BPM").clicked() { *source = "BPM".to_string(); }
                                                        });
                                                });
                                                ui.add(
                                                    egui::Slider::new(&mut 0.1_f32, 0.0..=1.0)
                                                        .text("Smoothing"),
                                                );
                                            }
                                        }
                                    }
                                    ModulePartType::Layer(layer) => {
                                        ui.label("📋 Layer:");

                                        // Helper to render mesh UI
                                        let mut render_mesh_ui = |ui: &mut Ui, mesh: &mut MeshType, id_salt: u64| {
                                            self.render_mesh_editor_ui(ui, mesh, part_id, id_salt);
                                        };

                                        match layer {
                                            LayerType::Single { id, name, opacity, blend_mode, mesh, mapping_mode } => {
                                                ui.label("🔲 Single Layer");
                                                ui.horizontal(|ui| { ui.label("ID:"); ui.add(egui::DragValue::new(id)); });
                                                ui.text_edit_singleline(name);
                                                ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));

                                                // Blend mode
                                                let blend_text = blend_mode.as_ref().map(|b| format!("{:?}", b)).unwrap_or_else(|| "None".to_string());
                                                egui::ComboBox::from_id_salt("layer_blend").selected_text(blend_text).show_ui(ui, |ui| {
                                                    if ui.selectable_label(blend_mode.is_none(), "None").clicked() { *blend_mode = None; }
                                                    if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Normal)), "Normal").clicked() { *blend_mode = Some(BlendModeType::Normal); }
                                                    if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Add)), "Add").clicked() { *blend_mode = Some(BlendModeType::Add); }
                                                    if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Multiply)), "Multiply").clicked() { *blend_mode = Some(BlendModeType::Multiply); }
                                                });

                                                ui.checkbox(mapping_mode, "Mapping Mode (Grid)");

                                                render_mesh_ui(ui, mesh, *id);
                                            }
                                            LayerType::Group { name, opacity, mesh, mapping_mode, .. } => {
                                                ui.label("📂 Group");
                                                ui.text_edit_singleline(name);
                                                ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
                                                ui.checkbox(mapping_mode, "Mapping Mode (Grid)");
                                                render_mesh_ui(ui, mesh, 9999); // Dummy ID
                                            }
                                            LayerType::All { opacity, .. } => {
                                                ui.label("🎚️ Master");
                                                ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
                                            }
                                        }
                                    }
                                    ModulePartType::Mesh(mesh) => {
                                        ui.label("🕸️ Mesh Node");
                                        ui.separator();

                                        self.render_mesh_editor_ui(ui, mesh, part_id, part_id);
                                    }
                                    ModulePartType::Output(output) => {
                                        ui.label("Output:");
                                        match output {
                                            OutputType::Projector {
                                                id,
                                                name,
                                                hide_cursor,
                                                target_screen,
                                                show_in_preview_panel,
                                                extra_preview_window,
                                                ..
                                            } => {
                                                ui.label("📽️ Projector Output");

                                                // Output ID selection
                                                ui.horizontal(|ui| {
                                                    ui.label("Output #:");
                                                    ui.add(egui::DragValue::new(id).range(1..=8));
                                                });

                                                ui.horizontal(|ui| {
                                                    ui.label("Name:");
                                                    ui.text_edit_singleline(name);
                                                });

                                                ui.separator();
                                                ui.label("🖥️ Window Settings:");

                                                // Target screen selection
                                                ui.horizontal(|ui| {
                                                    ui.label("Target Screen:");
                                                    egui::ComboBox::from_id_salt("target_screen_select")
                                                        .selected_text(format!("Monitor {}", target_screen))
                                                        .show_ui(ui, |ui| {
                                                            for i in 0..=3u8 {
                                                                let label = if i == 0 { "Primary".to_string() } else { format!("Monitor {}", i) };
                                                                if ui.selectable_label(*target_screen == i, &label).clicked() {
                                                                    *target_screen = i;
                                                                }
                                                            }
                                                        });
                                                });

                                                ui.checkbox(hide_cursor, "🖱️ Hide Mouse Cursor");

                                                ui.separator();
                                                ui.label("👁️ Preview:");
                                                ui.checkbox(show_in_preview_panel, "Show in Preview Panel");
                                                ui.checkbox(extra_preview_window, "Extra Preview Window");
                                            }
                                            #[cfg(feature = "ndi")]
                                            OutputType::NdiOutput { name } => {
                                                ui.label("📡 NDI Output");
                                                ui.horizontal(|ui| {
                                                    ui.label("Stream Name:");
                                                    ui.text_edit_singleline(name);
                                                });
                                            }
                                            #[cfg(not(feature = "ndi"))]
                                            OutputType::NdiOutput { .. } => {
                                                ui.label("📡 NDI Output (Feature Disabled)");
                                            }
                                            #[cfg(target_os = "windows")]
                                            OutputType::Spout { name } => {
                                                ui.label("🚰 Spout Output");
                                                ui.horizontal(|ui| {
                                                    ui.label("Stream Name:");
                                                    ui.text_edit_singleline(name);
                                                });
                                            }
                                            OutputType::Hue {
                                                bridge_ip,
                                                username,
                                                client_key: _client_key,
                                                entertainment_area,
                                                lamp_positions,
                                                mapping_mode,
                                            } => {
                                                ui.label("💡 Philips Hue Entertainment");
                                                ui.separator();

                                                // --- Tabs for Hue configuration ---
                                                ui.collapsing("⚙️ Setup (Bridge & Pairing)", |ui| {
                                                    // Discovery status
                                                    if let Some(msg) = &self.hue_status_message {
                                                        ui.label(format!("Status: {}", msg));
                                                    }

                                                    // Handle discovery results
                                                    if let Some(rx) = &self.hue_discovery_rx {
                                                        if let Ok(result) = rx.try_recv() {
                                                            self.hue_discovery_rx = None;
                                                            // Explicit type annotation for the result to help inference
                                                            let result: Result<Vec<mapmap_control::hue::api::discovery::DiscoveredBridge>, _> = result;
                                                            match result {
                                                                Ok(bridges) => {
                                                                    self.hue_bridges = bridges;
                                                                    self.hue_status_message = Some(format!("Found {} bridges", self.hue_bridges.len()));
                                                                }
                                                                Err(e) => {
                                                                    self.hue_status_message = Some(format!("Discovery failed: {}", e));
                                                                }
                                                            }
                                                        } else {
                                                            ui.horizontal(|ui| {
                                                                ui.spinner();
                                                                ui.label("Searching for bridges...");
                                                            });
                                                        }
                                                    }

                                                    // Using a block to allow attributes on expression
                                                    // Hue Discovery Logic extracted to method to satisfy rustfmt
                                                    self.render_hue_bridge_discovery(ui, bridge_ip);

                                                    ui.separator();
                                                    ui.label("Manual IP:");
                                                    ui.text_edit_singleline(bridge_ip);

                                                    // Pairing (Requires bridge button press)
                                                    if ui.button("🔗 Pair with Bridge").on_hover_text("Press button on Bridge then click this").clicked() {
                                                        // TODO: Implement pairing logic
                                                        // This requires async call to `register_user`
                                                        // Similar pattern to discovery
                                                    }

                                                    if !username.is_empty() {
                                                        ui.label("✅ Paired");
                                                        // ui.label(format!("User: {}", username)); // Keep secret?
                                                    } else {
                                                        ui.label("❌ Not Paired");
                                                    }
                                                });

                                                ui.collapsing("🎭 Area & Mode", |ui| {
                                                     ui.label("Entertainment Area:");
                                                     ui.text_edit_singleline(entertainment_area);
                                                     // TODO: Fetch areas from bridge if paired

                                                     ui.separator();
                                                     ui.label("Mapping Mode:");
                                                     ui.radio_value(mapping_mode, HueMappingMode::Ambient, "Ambient (Average Color)");
                                                     ui.radio_value(mapping_mode, HueMappingMode::Spatial, "Spatial (2D Map)");
                                                     ui.radio_value(mapping_mode, HueMappingMode::Trigger, "Trigger (Strobe/Pulse)");
                                                });

                                                if *mapping_mode == HueMappingMode::Spatial {
                                                    ui.collapsing("🗺️ Spatial Editor", |ui| {
                                                        ui.label("Position lamps in the virtual room:");
                                                        // Render 2D room editor
                                                        self.render_hue_spatial_editor(ui, lamp_positions);
                                                    });
                                                }
                                            }
                                        }
                                    }
                                     ModulePartType::Hue(hue_node) => {
                                        ui.label("💡 Hue Node");
                                        ui.separator();

                                        // Helper to render common Hue controls (duplicate of the one in render_node_inspector for now)
                                        let draw_hue_controls = |ui: &mut Ui, brightness: &mut f32, color: &mut [f32; 3], effect: &mut Option<String>, effect_active: &mut bool| {
                                            ui.add_space(8.0);
                                            ui.group(|ui| {
                                                ui.label("Light Control");
                                                ui.horizontal(|ui| {
                                                    ui.label("Brightness:");
                                                    ui.add(egui::Slider::new(brightness, 0.0..=1.0).text("%"));
                                                });

                                                ui.horizontal(|ui| {
                                                    ui.label("Color:");
                                                    ui.color_edit_button_rgb(color);
                                                });

                                                ui.horizontal(|ui| {
                                                    ui.label("Effect:");
                                                    let current_effect = effect.as_deref().unwrap_or("None");
                                                    egui::ComboBox::from_id_salt("hue_effect_popup")
                                                        .selected_text(current_effect)
                                                        .show_ui(ui, |ui| {
                                                            if ui.selectable_label(effect.is_none(), "None").clicked() {
                                                                *effect = None;
                                                            }
                                                            if ui.selectable_label(effect.as_deref() == Some("colorloop"), "Colorloop").clicked() {
                                                                *effect = Some("colorloop".to_string());
                                                            }
                                                        });
                                                });

                                                if effect.is_some() {
                                                    let btn_text = if *effect_active { "Stop Effect" } else { "Start Effect" };
                                                    if ui.button(btn_text).clicked() {
                                                        *effect_active = !*effect_active;
                                                    }
                                                }
                                            });
                                        };

                                        match hue_node {
                                            HueNodeType::SingleLamp { id, name, brightness, color, effect, effect_active } => {
                                                ui.horizontal(|ui| {
                                                    ui.label("Name:");
                                                    ui.text_edit_singleline(name);
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Lamp ID:");
                                                    ui.text_edit_singleline(id);
                                                });
                                                draw_hue_controls(ui, brightness, color, effect, effect_active);
                                            }
                                            HueNodeType::MultiLamp { ids, name, brightness, color, effect, effect_active } => {
                                                ui.horizontal(|ui| {
                                                    ui.label("Name:");
                                                    ui.text_edit_singleline(name);
                                                });
                                                ui.label("Lamp IDs (comma separated):");
                                                let mut ids_str = ids.join(", ");
                                                if ui.text_edit_singleline(&mut ids_str).changed() {
                                                    *ids = ids_str.split(',')
                                                        .map(|s| s.trim().to_string())
                                                        .filter(|s| !s.is_empty())
                                                        .collect();
                                                }
                                                draw_hue_controls(ui, brightness, color, effect, effect_active);
                                            }
                                            HueNodeType::EntertainmentGroup { name, brightness, color, effect, effect_active } => {
                                                ui.horizontal(|ui| {
                                                    ui.label("Name:");
                                                    ui.text_edit_singleline(name);
                                                });
                                                draw_hue_controls(ui, brightness, color, effect, effect_active);
                                            }
                                        }
                                    }
                                    // All part types handled above
                                }

                                // Link System UI
                                {
                                    use mapmap_core::module::*;
                                    let supports_link_system = matches!(part.part_type,
                                        ModulePartType::Mask(_) |
                                        ModulePartType::Modulizer(_) |
                                        ModulePartType::Layer(_) |
                                        ModulePartType::Mesh(_)
                                    );

                                    if supports_link_system {
                                        ui.separator();
                                        ui.collapsing("🔗 Link System", |ui| {
                                            let mut changed = false;
                                            let link_data = &mut part.link_data;

                                            ui.horizontal(|ui| {
                                                ui.label("Link Mode:");
                                                egui::ComboBox::from_id_salt(format!("link_mode_{}", part_id))
                                                    .selected_text(format!("{:?}", link_data.mode))
                                                    .show_ui(ui, |ui| {
                                                        if ui.selectable_label(link_data.mode == LinkMode::Off, "Off").clicked() {
                                                            link_data.mode = LinkMode::Off;
                                                            changed = true;
                                                        }
                                                        if ui.selectable_label(link_data.mode == LinkMode::Master, "Master").clicked() {
                                                            link_data.mode = LinkMode::Master;
                                                            changed = true;
                                                        }
                                                        if ui.selectable_label(link_data.mode == LinkMode::Slave, "Slave").clicked() {
                                                            link_data.mode = LinkMode::Slave;
                                                            changed = true;
                                                        }
                                                    });
                                            });

                                            if link_data.mode == LinkMode::Slave {
                                                ui.horizontal(|ui| {
                                                    ui.label("Behavior:");
                                                    egui::ComboBox::from_id_salt(format!("link_behavior_{}", part_id))
                                                        .selected_text(format!("{:?}", link_data.behavior))
                                                        .show_ui(ui, |ui| {
                                                            if ui.selectable_label(link_data.behavior == LinkBehavior::SameAsMaster, "Same as Master").clicked() {
                                                                link_data.behavior = LinkBehavior::SameAsMaster;
                                                            }
                                                            if ui.selectable_label(link_data.behavior == LinkBehavior::Inverted, "Inverted").clicked() {
                                                                link_data.behavior = LinkBehavior::Inverted;
                                                            }
                                                        });
                                                });
                                                ui.label("ℹ️ Visibility controlled by Link Input");
                                            } else if ui.checkbox(&mut link_data.trigger_input_enabled, "Enable Trigger Input (Visibility Control)").changed() {
                                                changed = true;
                                            }

                                            if changed {
                                                changed_part_id = Some(part_id);
                                            }
                                        });
                                    }
                                }

                                ui.add_space(16.0);
                                ui.separator();

                                // Node position info
                                ui.label(format!(
                                    "Position: ({:.0}, {:.0})",
                                    part.position.0, part.position.1
                                ));
                                if let Some((w, h)) = part.size {
                                    ui.label(format!("Size: {:.0} × {:.0}", w, h));
                                }
                                ui.label(format!("Inputs: {}", part.inputs.len()));
                                ui.label(format!("Outputs: {}", part.outputs.len()));
                            });
    }

    fn load_svg_icon(path: &std::path::Path, ctx: &egui::Context) -> Option<TextureHandle> {
        let svg_data = std::fs::read(path).ok()?;
        let options = usvg::Options::default();
        let tree = usvg::Tree::from_data(&svg_data, &options).ok()?;
        let size = tree.size();
        let width = size.width().round() as u32;
        let height = size.height().round() as u32;

        let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

        let mut pixels = Vec::with_capacity((width * height) as usize);
        for pixel in pixmap.pixels() {
            // Preserve original RGBA from SVG
            pixels.push(egui::Color32::from_rgba_premultiplied(
                pixel.red(),
                pixel.green(),
                pixel.blue(),
                pixel.alpha(),
            ));
        }

        let image = egui::ColorImage {
            size: [width as usize, height as usize],
            pixels,
            source_size: egui::Vec2::new(width as f32, height as f32),
        };

        Some(ctx.load_texture(
            path.file_name()?.to_string_lossy(),
            image,
            egui::TextureOptions::LINEAR,
        ))
    }

    /// Set the active module ID
    pub fn set_active_module(&mut self, module_id: Option<u64>) {
        self.active_module_id = module_id;
    }

    /// Get the active module ID
    pub fn active_module_id(&self) -> Option<u64> {
        self.active_module_id
    }

    /// Update live audio data for trigger nodes
    pub fn set_audio_data(&mut self, data: AudioTriggerData) {
        self.audio_trigger_data = data;
    }

    /// Get a reference to the live audio data.
    pub fn get_audio_trigger_data(&self) -> Option<&AudioTriggerData> {
        Some(&self.audio_trigger_data)
    }

    /// Get the live value of a specific output socket on a part.
    /// This is used to draw live data visualizations on the nodes.
    fn get_socket_live_value(&self, part: &ModulePart, socket_idx: usize) -> Option<f32> {
        if let ModulePartType::Trigger(TriggerType::AudioFFT { .. }) = &part.part_type {
            // The 9 frequency bands are the first 9 outputs
            if socket_idx < 9 {
                return Some(self.audio_trigger_data.band_energies[socket_idx]);
            }
            // After the bands, we have RMS, Peak, Beat, BPM
            match socket_idx {
                9 => return Some(self.audio_trigger_data.rms_volume),
                10 => return Some(self.audio_trigger_data.peak_volume),
                11 => return Some(self.audio_trigger_data.beat_strength),
                12 => return self.audio_trigger_data.bpm,
                _ => return None,
            }
        }
        None
    }

    /// Get current RMS volume
    pub fn get_rms_volume(&self) -> f32 {
        self.audio_trigger_data.rms_volume
    }

    /// Get beat detection status
    pub fn is_beat_detected(&self) -> bool {
        self.audio_trigger_data.beat_detected
    }

    /// Get audio trigger state for a part type
    /// Returns (is_audio_trigger, current_value, threshold, is_active)
    fn get_audio_trigger_state(
        &self,
        part_type: &mapmap_core::module::ModulePartType,
    ) -> (bool, f32, f32, bool) {
        use mapmap_core::module::{ModulePartType, TriggerType};

        match part_type {
            ModulePartType::Trigger(TriggerType::AudioFFT {
                band, threshold, ..
            }) => {
                let value = match band {
                    mapmap_core::module::AudioBand::SubBass => self
                        .audio_trigger_data
                        .band_energies
                        .first()
                        .copied()
                        .unwrap_or(0.0),
                    mapmap_core::module::AudioBand::Bass => self
                        .audio_trigger_data
                        .band_energies
                        .get(1)
                        .copied()
                        .unwrap_or(0.0),
                    mapmap_core::module::AudioBand::LowMid => self
                        .audio_trigger_data
                        .band_energies
                        .get(2)
                        .copied()
                        .unwrap_or(0.0),
                    mapmap_core::module::AudioBand::Mid => self
                        .audio_trigger_data
                        .band_energies
                        .get(3)
                        .copied()
                        .unwrap_or(0.0),
                    mapmap_core::module::AudioBand::HighMid => self
                        .audio_trigger_data
                        .band_energies
                        .get(4)
                        .copied()
                        .unwrap_or(0.0),
                    mapmap_core::module::AudioBand::UpperMid => self
                        .audio_trigger_data
                        .band_energies
                        .get(5)
                        .copied()
                        .unwrap_or(0.0),
                    mapmap_core::module::AudioBand::Presence => self
                        .audio_trigger_data
                        .band_energies
                        .get(6)
                        .copied()
                        .unwrap_or(0.0),
                    mapmap_core::module::AudioBand::Brilliance => self
                        .audio_trigger_data
                        .band_energies
                        .get(7)
                        .copied()
                        .unwrap_or(0.0),
                    mapmap_core::module::AudioBand::Air => self
                        .audio_trigger_data
                        .band_energies
                        .get(8)
                        .copied()
                        .unwrap_or(0.0),
                    mapmap_core::module::AudioBand::Peak => self.audio_trigger_data.peak_volume,
                    mapmap_core::module::AudioBand::BPM => {
                        self.audio_trigger_data.bpm.unwrap_or(0.0) / 200.0
                    }
                };
                let is_active = value > *threshold;
                (true, value, *threshold, is_active)
            }
            ModulePartType::Trigger(TriggerType::Beat) => {
                let is_active = self.audio_trigger_data.beat_detected;
                let value = self.audio_trigger_data.beat_strength;
                (true, value, 0.5, is_active)
            }
            _ => (false, 0.0, 0.0, false),
        }
    }

    /// Process incoming MIDI message for MIDI Learn
    #[cfg(feature = "midi")]
    pub fn process_midi_message(&mut self, message: mapmap_control::midi::MidiMessage) {
        // Check if we're in learn mode for any part
        if let Some(part_id) = self.midi_learn_part_id {
            // We received a MIDI message while in learn mode
            // Store the learned values in a pending result
            // The actual module update will happen in the show() method
            // For now, we log it and clear learn mode
            match message {
                mapmap_control::midi::MidiMessage::ControlChange {
                    channel,
                    controller,
                    ..
                } => {
                    tracing::info!(
                        "MIDI Learn: Part {:?} assigned to CC {} on channel {}",
                        part_id,
                        controller,
                        channel
                    );
                    // Store learned values - will be applied in UI
                    self.learned_midi = Some((part_id, channel, controller, false));
                    self.midi_learn_part_id = None;
                }
                mapmap_control::midi::MidiMessage::NoteOn { channel, note, .. } => {
                    tracing::info!(
                        "MIDI Learn: Part {:?} assigned to Note {} on channel {}",
                        part_id,
                        note,
                        channel
                    );
                    // Store learned values - will be applied in UI
                    self.learned_midi = Some((part_id, channel, note, true));
                    self.midi_learn_part_id = None;
                }
                _ => {
                    // Ignore other message types during learn
                }
            }
        }
    }

    /// Process incoming MIDI message (no-op without midi feature)
    #[cfg(not(feature = "midi"))]
    pub fn process_midi_message(&mut self, _message: ()) {}

    /// Add a Trigger node with specified type
    fn add_trigger_node(
        &mut self,
        manager: &mut ModuleManager,
        trigger_type: TriggerType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((100.0, 100.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module.add_part_with_type(
                    mapmap_core::module::ModulePartType::Trigger(trigger_type),
                    pos,
                );
            }
        }
    }

    /// Add a Source node with specified type
    fn add_source_node(
        &mut self,
        manager: &mut ModuleManager,
        source_type: SourceType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((200.0, 100.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module.add_part_with_type(
                    mapmap_core::module::ModulePartType::Source(source_type),
                    pos,
                );
            }
        }
    }

    /// Add a Mask node with specified type
    fn add_mask_node(
        &mut self,
        manager: &mut ModuleManager,
        mask_type: MaskType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((300.0, 100.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module
                    .add_part_with_type(mapmap_core::module::ModulePartType::Mask(mask_type), pos);
            }
        }
    }

    /// Add a Modulator node with specified type
    fn add_modulator_node(
        &mut self,
        manager: &mut ModuleManager,
        mod_type: ModulizerType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((400.0, 100.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module.add_part_with_type(
                    mapmap_core::module::ModulePartType::Modulizer(mod_type),
                    pos,
                );
            }
        }
    }

    /// Add a Hue node with specified type
    fn add_hue_node(
        &mut self,
        manager: &mut ModuleManager,
        hue_type: HueNodeType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((500.0, 100.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module.add_part_with_type(mapmap_core::module::ModulePartType::Hue(hue_type), pos);
            }
        }
    }

    /// Render Hue bridge discovery UI
    #[rustfmt::skip]
    fn render_hue_bridge_discovery(&mut self, ui: &mut egui::Ui, current_ip: &mut String) {
        if ui.button("🔍 Discover Bridges").clicked() {
            let (tx, rx) = std::sync::mpsc::channel();
            self.hue_discovery_rx = Some(rx);
            // Spawn async task
            #[cfg(feature = "tokio")]
            {
                self.hue_status_message = Some("Searching...".to_string());
                let task = async move {
                    let result = mapmap_control::hue::api::discovery::discover_bridges().await
                        .map_err(|e| e.to_string());
                    let _ = tx.send(result);
                };
                tokio::spawn(task);
            }
            #[cfg(not(feature = "tokio"))]
            {
                let _ = tx;
                self.hue_status_message = Some("Async runtime not available".to_string());
            }
        }

        if !self.hue_bridges.is_empty() {
            ui.separator();
            ui.label("Select Bridge:");
            for bridge in &self.hue_bridges {
                if ui
                    .button(format!("{} ({})", bridge.id, bridge.ip))
                    .clicked()
                {
                    *current_ip = bridge.ip.clone();
                }
            }
        }
    }

    /// Content for the Sources menu
    #[rustfmt::skip]
    fn render_sources_menu_content(
        &mut self,
        ui: &mut egui::Ui,
        manager: &mut ModuleManager,
        pos_override: Option<(f32, f32)>,
    ) {
        ui.label("--- 📁 File Based ---");
        if ui.button("📹 Media File").clicked() {
            self.add_source_node(
                manager,
                SourceType::new_media_file(String::new()),
                pos_override,
            );
            ui.close();
        }
        if ui.button("📹 Video (Uni)").clicked() {
            self.add_source_node(
                manager,
                SourceType::VideoUni {
                    path: String::new(),
                    speed: 1.0,
                    loop_enabled: true,
                    start_time: 0.0,
                    end_time: 0.0,
                    opacity: 1.0,
                    blend_mode: None,
                    brightness: 0.0,
                    contrast: 1.0,
                    saturation: 1.0,
                    hue_shift: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotation: 0.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                    target_width: None,
                    target_height: None,
                    target_fps: None,
                    flip_horizontal: false,
                    flip_vertical: false,
                    reverse_playback: false,
                },
                pos_override,
            );
            ui.close();
        }
        if ui.button("🖼 Image (Uni)").clicked() {
            self.add_source_node(
                manager,
                SourceType::ImageUni {
                    path: String::new(),
                    opacity: 1.0,
                    blend_mode: None,
                    brightness: 0.0,
                    contrast: 1.0,
                    saturation: 1.0,
                    hue_shift: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotation: 0.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                    target_width: None,
                    target_height: None,
                    flip_horizontal: false,
                    flip_vertical: false,
                },
                pos_override,
            );
            ui.close();
        }

        ui.add_space(4.0);
        ui.label("--- 🔗 Shared (Multi) ---");
        if ui.button("📹 Video (Multi)").clicked() {
            self.add_source_node(
                manager,
                SourceType::VideoMulti {
                    shared_id: String::new(),
                    opacity: 1.0,
                    blend_mode: None,
                    brightness: 0.0,
                    contrast: 1.0,
                    saturation: 1.0,
                    hue_shift: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotation: 0.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                    flip_horizontal: false,
                    flip_vertical: false,
                },
                pos_override,
            );
            ui.close();
        }
        if ui.button("🖼 Image (Multi)").clicked() {
            self.add_source_node(
                manager,
                SourceType::ImageMulti {
                    shared_id: String::new(),
                    opacity: 1.0,
                    blend_mode: None,
                    brightness: 0.0,
                    contrast: 1.0,
                    saturation: 1.0,
                    hue_shift: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotation: 0.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                    flip_horizontal: false,
                    flip_vertical: false,
                },
                pos_override,
            );
            ui.close();
        }

        ui.add_space(4.0);
        ui.label("--- 📡 Hardware & Network ---");
        if ui.button("📹 Live Input").clicked() {
            self.add_source_node(
                manager,
                SourceType::LiveInput { device_id: 0 },
                pos_override,
            );
            ui.close();
        }
        if ui.button("📡 NDI Input").clicked() {
            self.add_source_node(
                manager,
                SourceType::NdiInput { source_name: None },
                pos_override,
            );
            ui.close();
        }
        #[cfg(target_os = "windows")]
        if ui.button("🚰 Spout Input").clicked() {
            self.add_source_node(
                manager,
                SourceType::SpoutInput {
                    sender_name: String::new(),
                },
                pos_override,
            );
            ui.close();
        }

        ui.add_space(4.0);
        ui.label("--- 🎨 Procedural & Misc ---");
        if ui.button("🎨 Shader").clicked() {
            self.add_source_node(
                manager,
                SourceType::Shader {
                    name: "New Shader".to_string(),
                    params: vec![],
                },
                pos_override,
            );
            ui.close();
        }
        if ui.button("🎮 Bevy Scene").clicked() {
            self.add_source_node(manager, SourceType::Bevy, pos_override);
            ui.close();
        }
    }

    /// Content for the Add Node menu (used by both toolbar and context menu)
    fn render_add_node_menu_content(
        &mut self,
        ui: &mut egui::Ui,
        manager: &mut ModuleManager,
        pos_override: Option<(f32, f32)>,
    ) {
        ui.set_min_width(150.0);

        ui.menu_button("🎬 Sources", |ui| {
            self.render_sources_menu_content(ui, manager, pos_override);
        });

        ui.menu_button("⚡ Triggers", |ui| {
            if ui.button("🎵 Audio FFT").clicked() {
                self.add_trigger_node(
                    manager,
                    TriggerType::AudioFFT {
                        band: mapmap_core::module::AudioBand::Bass,
                        threshold: 0.5,
                        output_config: mapmap_core::module::AudioTriggerOutputConfig::default(),
                    },
                    pos_override,
                );
                ui.close();
            }
            if ui.button("🎲 Random").clicked() {
                self.add_trigger_node(
                    manager,
                    TriggerType::Random {
                        min_interval_ms: 500,
                        max_interval_ms: 2000,
                        probability: 0.5,
                    },
                    pos_override,
                );
                ui.close();
            }
            if ui.button("⏱ Fixed").clicked() {
                self.add_trigger_node(
                    manager,
                    TriggerType::Fixed {
                        interval_ms: 1000,
                        offset_ms: 0,
                    },
                    pos_override,
                );
                ui.close();
            }
            if ui.button("🎹 MIDI").clicked() {
                self.add_trigger_node(
                    manager,
                    TriggerType::Midi {
                        device: "Default".to_string(),
                        channel: 1,
                        note: 60,
                    },
                    pos_override,
                );
                ui.close();
            }
        });

        ui.menu_button("🎭 Masks", |ui| {
            if ui.button("⭕ Shape").clicked() {
                self.add_mask_node(
                    manager,
                    MaskType::Shape(mapmap_core::module::MaskShape::Circle),
                    pos_override,
                );
                ui.close();
            }
            if ui.button("🌈 Gradient").clicked() {
                self.add_mask_node(
                    manager,
                    MaskType::Gradient {
                        angle: 0.0,
                        softness: 0.5,
                    },
                    pos_override,
                );
                ui.close();
            }
        });

        ui.menu_button("🎛 Modulators", |ui| {
            if ui.button("🎚 Blend Mode").clicked() {
                self.add_modulator_node(
                    manager,
                    ModulizerType::BlendMode(mapmap_core::module::BlendModeType::Normal),
                    pos_override,
                );
                ui.close();
            }
        });

        ui.menu_button("💡 Philips Hue", |ui| {
            if ui.button("💡 Single Lamp").clicked() {
                self.add_hue_node(
                    manager,
                    HueNodeType::SingleLamp {
                        id: String::new(),
                        name: "New Lamp".to_string(),
                        brightness: 1.0,
                        color: [1.0, 1.0, 1.0],
                        effect: None,
                        effect_active: false,
                    },
                    pos_override,
                );
                ui.close();
            }
        });

        ui.separator();

        if ui.button("🖼 Output").clicked() {
            if let Some(id) = self.active_module_id {
                if let Some(module) = manager.get_module_mut(id) {
                    let preferred_pos = pos_override.unwrap_or((600.0, 100.0));
                    let pos = Self::find_free_position(&module.parts, preferred_pos);
                    module.add_part_with_type(
                        mapmap_core::module::ModulePartType::Output(
                            mapmap_core::module::OutputType::Projector {
                                id: 1,
                                name: "Projector 1".to_string(),
                                hide_cursor: false,
                                target_screen: 0,
                                show_in_preview_panel: true,
                                extra_preview_window: false,
                                output_width: 0,
                                output_height: 0,
                                output_fps: 60.0,
                            },
                        ),
                        pos,
                    );
                }
            }
            ui.close();
        }
    }

    /// Renders the menu to add new nodes to the canvas
    fn render_add_node_menu(&mut self, ui: &mut egui::Ui, manager: &mut ModuleManager) {
        ui.menu_button("➕ Add Node", |ui| {
            self.render_add_node_menu_content(ui, manager, None);
        });
    }

    fn apply_undo_action(module: &mut MapFlowModule, action: &CanvasAction) {
        match action {
            CanvasAction::AddPart { part_id, .. } => {
                // Undo add = delete
                module.parts.retain(|p| p.id != *part_id);
            }
            CanvasAction::DeletePart { part_data } => {
                // Undo delete = restore
                module.parts.push(part_data.clone());
            }
            CanvasAction::MovePart {
                part_id, old_pos, ..
            } => {
                // Undo move = restore old position
                if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                    part.position = *old_pos;
                }
            }
            CanvasAction::AddConnection { connection } => {
                // Undo add connection = delete
                module.connections.retain(|c| {
                    !(c.from_part == connection.from_part
                        && c.to_part == connection.to_part
                        && c.from_socket == connection.from_socket
                        && c.to_socket == connection.to_socket)
                });
            }
            CanvasAction::DeleteConnection { connection } => {
                // Undo delete connection = restore
                module.connections.push(connection.clone());
            }
            CanvasAction::Batch(actions) => {
                for action in actions.iter().rev() {
                    Self::apply_undo_action(module, action);
                }
            }
        }
    }

    fn apply_redo_action(module: &mut MapFlowModule, action: &CanvasAction) {
        match action {
            CanvasAction::AddPart { part_data, .. } => {
                // Redo add = add again
                module.parts.push(part_data.clone());
            }
            CanvasAction::DeletePart { part_data } => {
                // Redo delete = delete again
                module.parts.retain(|p| p.id != part_data.id);
            }
            CanvasAction::MovePart {
                part_id, new_pos, ..
            } => {
                // Redo move = apply new position
                if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                    part.position = *new_pos;
                }
            }
            CanvasAction::AddConnection { connection } => {
                // Redo add connection = add again
                module.connections.push(connection.clone());
            }
            CanvasAction::DeleteConnection { connection } => {
                // Redo delete connection = delete again
                module.connections.retain(|c| {
                    !(c.from_part == connection.from_part
                        && c.to_part == connection.to_part
                        && c.from_socket == connection.from_socket
                        && c.to_socket == connection.to_socket)
                });
            }
            CanvasAction::Batch(actions) => {
                for action in actions.iter() {
                    Self::apply_redo_action(module, action);
                }
            }
        }
    }

    fn safe_delete_selection(&mut self, module: &mut MapFlowModule) {
        if self.selected_parts.is_empty() {
            return;
        }

        let mut actions = Vec::new();

        // 1. Identify all parts to delete
        let parts_to_delete: Vec<ModulePartId> = self.selected_parts.clone();

        // 2. Identify all connections to delete (connected to any selected part)
        // We need to capture the connection data for undo
        let mut connections_to_delete = Vec::new();

        for conn in module.connections.iter() {
            if parts_to_delete.contains(&conn.from_part) || parts_to_delete.contains(&conn.to_part)
            {
                connections_to_delete.push(conn.clone());
            }
        }

        // Add DeleteConnection actions
        for conn in connections_to_delete {
            actions.push(CanvasAction::DeleteConnection { connection: conn });
        }

        // 3. Capture part data for undo
        for part_id in &parts_to_delete {
            if let Some(part) = module.parts.iter().find(|p| p.id == *part_id) {
                actions.push(CanvasAction::DeletePart {
                    part_data: part.clone(),
                });
            }
        }

        // 4. Create Batch Action
        let batch_action = CanvasAction::Batch(actions);

        // 5. Execute Deletions (Modify Module)
        // Remove connections first
        module.connections.retain(|c| {
            !parts_to_delete.contains(&c.from_part) && !parts_to_delete.contains(&c.to_part)
        });

        // Remove parts
        module.parts.retain(|p| !parts_to_delete.contains(&p.id));

        // 6. Update Stacks
        self.undo_stack.push(batch_action);
        self.redo_stack.clear();
        self.selected_parts.clear();
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        manager: &mut ModuleManager,
        locale: &LocaleManager,
        actions: &mut Vec<crate::UIAction>,
    ) {
        // === KEYBOARD SHORTCUTS ===
        if !self.selected_parts.is_empty()
            && !ui.memory(|m| m.focused().is_some())
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Space))
        {
            if let Some(module_id) = self.active_module_id {
                if let Some(module) = manager.get_module_mut(module_id) {
                    for part_id in &self.selected_parts {
                        if let Some(part) = module.parts.iter().find(|p| p.id == *part_id) {
                            if let mapmap_core::module::ModulePartType::Source(
                                mapmap_core::module::SourceType::MediaFile { .. },
                            ) = &part.part_type
                            {
                                // Toggle playback
                                let is_playing = self
                                    .player_info
                                    .get(part_id)
                                    .map(|info| info.is_playing)
                                    .unwrap_or(false);

                                let command = if is_playing {
                                    MediaPlaybackCommand::Pause
                                } else {
                                    MediaPlaybackCommand::Play
                                };
                                self.pending_playback_commands.push((*part_id, command));
                            }
                        }
                    }
                }
            }
        }

        // === APPLY LEARNED MIDI VALUES ===
        if let Some((part_id, channel, cc_or_note, is_note)) = self.learned_midi.take() {
            if let Some(module_id) = self.active_module_id {
                if let Some(module) = manager.get_module_mut(module_id) {
                    if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                        if let mapmap_core::module::ModulePartType::Trigger(TriggerType::Midi {
                            channel: ref mut ch,
                            note: ref mut n,
                            ..
                        }) = part.part_type
                        {
                            *ch = channel;
                            *n = cc_or_note;
                            tracing::info!(
                                "Applied MIDI Learn: Channel={}, {}={}",
                                channel,
                                if is_note { "Note" } else { "CC" },
                                cc_or_note
                            );
                        }
                    }
                }
            }
        }

        // === CANVAS TOOLBAR ===
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(8, 6))
            .fill(ui.visuals().panel_fill)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // --- ROW 1: Module Context & Adding Nodes ---
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;

                        // LEFT: Module Selector & Info
                        ui.push_id("module_context", |ui| {
                            let mut module_names: Vec<(u64, String)> = manager
                                .list_modules()
                                .iter()
                                .map(|m| (m.id, m.name.clone()))
                                .collect();
                            module_names
                                .sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));

                            let current_name = self
                                .active_module_id
                                .and_then(|id| manager.get_module(id))
                                .map(|m| m.name.clone())
                                .unwrap_or_else(|| "— Select Module —".to_string());

                            egui::ComboBox::from_id_salt("module_selector")
                                .selected_text(current_name)
                                .width(160.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.active_module_id,
                                        None,
                                        "— None —",
                                    );
                                    ui.separator();
                                    for (id, name) in &module_names {
                                        ui.selectable_value(
                                            &mut self.active_module_id,
                                            Some(*id),
                                            name,
                                        );
                                    }
                                });

                            if ui
                                .button("➕ New")
                                .on_hover_text("Create a new module")
                                .clicked()
                            {
                                let new_id = manager
                                    .create_module(manager.get_next_available_name("New Module"));
                                self.active_module_id = Some(new_id);
                            }

                            if let Some(module_id) = self.active_module_id {
                                if let Some(module) = manager.get_module_mut(module_id) {
                                    ui.separator();
                                    ui.add(
                                        egui::TextEdit::singleline(&mut module.name)
                                            .desired_width(120.0)
                                            .hint_text("Name"),
                                    );

                                    let mut color_f32 = module.color;
                                    if ui
                                        .color_edit_button_rgba_unmultiplied(&mut color_f32)
                                        .clicked()
                                    {
                                        module.color = color_f32;
                                    }

                                    if ui.button("🗑").on_hover_text("Delete Module").clicked() {
                                        manager.delete_module(module_id);
                                        self.active_module_id = None;
                                    }
                                }
                            }
                        });

                        ui.separator();

                        // CENTER/RIGHT (Top Row): Add Node Menu
                        let has_module = self.active_module_id.is_some();
                        ui.add_enabled_ui(has_module, |ui| {
                            self.render_add_node_menu(ui, manager);
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("Canvas Editor")
                                    .strong()
                                    .color(ui.visuals().strong_text_color()),
                            );
                        });
                    });

                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);

                    // --- ROW 2: View Controls & Utilities ---
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        // Utility Buttons
                        if self.active_module_id.is_some() {
                            if ui.button("📋 Presets").clicked() {
                                self.show_presets = !self.show_presets;
                            }
                            if ui.button("⊞ Auto Layout").clicked() {
                                if let Some(id) = self.active_module_id {
                                    if let Some(m) = manager.get_module_mut(id) {
                                        Self::auto_layout_parts(&mut m.parts);
                                    }
                                }
                            }
                            if ui.button("🔍 Search").clicked() {
                                self.show_search = !self.show_search;
                            }

                            let check_label = if self.diagnostic_issues.is_empty() {
                                "✓"
                            } else {
                                "⚠"
                            };
                            if ui
                                .button(check_label)
                                .on_hover_text("Check Integrity")
                                .clicked()
                            {
                                if let Some(id) = self.active_module_id {
                                    if let Some(m) = manager.get_module(id) {
                                        self.diagnostic_issues =
                                            mapmap_core::diagnostics::check_module_integrity(m);
                                        self.show_diagnostics = true;
                                    }
                                }
                            }
                        }

                        // Right Aligned View Controls
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("⊡").on_hover_text("Reset View").clicked() {
                                self.zoom = 1.0;
                                self.pan_offset = Vec2::ZERO;
                            }
                            ui.label(format!("{:.0}%", self.zoom * 100.0));
                            if ui.button("+").on_hover_text("Zoom In").clicked() {
                                self.zoom = (self.zoom + 0.1).clamp(0.2, 3.0);
                            }
                            ui.add(
                                egui::Slider::new(&mut self.zoom, 0.2..=3.0)
                                    .show_value(false)
                                    .trailing_fill(true),
                            );
                            if ui.button("−").on_hover_text("Zoom Out").clicked() {
                                self.zoom = (self.zoom - 0.1).clamp(0.2, 3.0);
                            }
                            ui.label("Zoom:");
                        });
                    });
                });
            });

        ui.add_space(1.0);
        ui.separator();

        if let Some(module_id) = self.active_module_id {
            // Render the canvas taking up the full available space
            self.render_canvas(ui, manager, module_id, locale, actions);
            // Properties popup removed - moved to docked inspector
        } else {
            // Show a message if no module is selected
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("🔧 Module Canvas");
                    ui.add_space(10.0);
                    ui.label("Click '➕ New Module' to create a module.");
                    ui.label("Or select an existing module from the dropdown above.");
                });
            });
        }
    }

    fn render_canvas(
        &mut self,
        ui: &mut Ui,
        manager: &mut ModuleManager,
        module_id: ModuleId,
        _locale: &LocaleManager,
        actions: &mut Vec<crate::UIAction>,
    ) {
        let module = if let Some(m) = manager.get_module_mut(module_id) {
            m
        } else {
            return;
        };
        self.ensure_icons_loaded(ui.ctx());
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let canvas_rect = response.rect;

        // Store drag_started state before we check parts
        let drag_started_on_empty = response.drag_started() && self.dragging_part.is_none();

        // Handle canvas pan (only when not dragging a part and not creating connection)
        // We also need middle mouse button for panning to avoid conflicts
        let middle_button = ui.input(|i| i.pointer.middle_down());
        if response.dragged() && self.dragging_part.is_none() && self.creating_connection.is_none()
        {
            // Only pan with middle mouse or when not over a part
            if middle_button || self.panning_canvas {
                self.pan_offset += response.drag_delta();
            }
        }

        // Track if we started panning (for continuing the pan)
        if drag_started_on_empty && !middle_button {
            // Will be set to true if click was on empty canvas
            self.panning_canvas = false;
        }
        // Handle keyboard shortcuts
        let ctrl_held = ui.input(|i| i.modifiers.ctrl);

        // Handle context menu triggering
        if response.secondary_clicked()
            && self.dragging_part.is_none()
            && self.creating_connection.is_none()
        {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                self.context_menu_pos = Some(pointer_pos);
                // Clear specific targets, they will be set if a part or connection was clicked
                self.context_menu_part = None;
                self.context_menu_connection = None;
            }
        }
        let shift_held = ui.input(|i| i.modifiers.shift);

        // Keyboard Navigation (Arrow Keys)
        if !ui.memory(|m| m.focused().is_some()) && !self.show_search {
            let mut direction = Vec2::ZERO;
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                direction = Vec2::new(0.0, -1.0);
            } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                direction = Vec2::new(0.0, 1.0);
            } else if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                direction = Vec2::new(-1.0, 0.0);
            } else if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                direction = Vec2::new(1.0, 0.0);
            }

            if direction != Vec2::ZERO {
                if let Some(current_id) = self.get_selected_part_id() {
                    if let Some(current_part) = module.parts.iter().find(|p| p.id == current_id) {
                        let current_pos =
                            Vec2::new(current_part.position.0, current_part.position.1);
                        let mut best_candidate = None;
                        let mut best_score = f32::MAX;

                        for part in &module.parts {
                            if part.id == current_id {
                                continue;
                            }

                            let part_pos = Vec2::new(part.position.0, part.position.1);
                            let delta = part_pos - current_pos;
                            let distance = delta.length();

                            if distance < 1.0 {
                                continue;
                            }

                            let dir_to_part = delta.normalized();
                            let alignment = direction.dot(dir_to_part);

                            // Directional cone check
                            if alignment > 0.5 {
                                // Score prefers closer distance and better alignment
                                let score = distance * (2.0 - alignment);
                                if score < best_score {
                                    best_score = score;
                                    best_candidate = Some(part.id);
                                }
                            }
                        }

                        if let Some(target_id) = best_candidate {
                            self.selected_parts.clear();
                            self.selected_parts.push(target_id);
                        }
                    }
                } else if !module.parts.is_empty() {
                    // Select first part if nothing selected
                    self.selected_parts.push(module.parts[0].id);
                }
            }
        }

        // Ctrl+C: Copy selected parts
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::C)) && !self.selected_parts.is_empty()
        {
            self.clipboard.clear();
            // Find center of selection for relative positioning
            let center = if !self.selected_parts.is_empty() {
                let sum: (f32, f32) = module
                    .parts
                    .iter()
                    .filter(|p| self.selected_parts.contains(&p.id))
                    .map(|p| p.position)
                    .fold((0.0, 0.0), |acc, pos| (acc.0 + pos.0, acc.1 + pos.1));
                let count = self.selected_parts.len() as f32;
                (sum.0 / count, sum.1 / count)
            } else {
                (0.0, 0.0)
            };

            for part in module
                .parts
                .iter()
                .filter(|p| self.selected_parts.contains(&p.id))
            {
                let relative_pos = (part.position.0 - center.0, part.position.1 - center.1);
                self.clipboard.push((part.part_type.clone(), relative_pos));
            }
        }

        // Ctrl+V: Paste from clipboard
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::V)) && !self.clipboard.is_empty() {
            let paste_offset = (50.0, 50.0); // Offset from original position
            self.selected_parts.clear();

            for (part_type, rel_pos) in self.clipboard.clone() {
                let new_pos = (
                    rel_pos.0 + paste_offset.0 + 100.0,
                    rel_pos.1 + paste_offset.1 + 100.0,
                );
                let part_type_variant = Self::part_type_from_module_part_type(&part_type);
                let new_id = module.add_part(part_type_variant, new_pos);
                self.selected_parts.push(new_id);
            }
        }

        // Ctrl+A: Select all
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::A)) {
            self.selected_parts = module.parts.iter().map(|p| p.id).collect();
        }

        // Delete: Delete selected parts
        if !ui.memory(|m| m.focused().is_some())
            && (ui.input(|i| i.key_pressed(egui::Key::Delete))
                || ui.input(|i| i.key_pressed(egui::Key::Backspace)))
            && !self.selected_parts.is_empty()
        {
            self.safe_delete_selection(module);
        }

        // Escape: Deselect all or close search
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.show_search {
                self.show_search = false;
            } else {
                self.selected_parts.clear();
            }
        }

        // Ctrl+F: Toggle search popup
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::F)) {
            self.show_search = !self.show_search;
            if self.show_search {
                self.search_filter.clear();
            }
        }

        // Ctrl+Z: Undo
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::Z)) && !self.undo_stack.is_empty() {
            if let Some(action) = self.undo_stack.pop() {
                Self::apply_undo_action(module, &action);
                self.redo_stack.push(action);
            }
        }

        // Ctrl+Y: Redo
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::Y)) && !self.redo_stack.is_empty() {
            if let Some(action) = self.redo_stack.pop() {
                Self::apply_redo_action(module, &action);
                self.undo_stack.push(action);
            }
        }

        // For shift_held - used in click handling below
        let _ = shift_held;

        // Handle zoom
        if response.hovered() || ui.input(|i| i.modifiers.ctrl) {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                let speed = if ui.input(|i| i.modifiers.ctrl) {
                    0.004
                } else {
                    0.001
                };
                self.zoom *= 1.0 + scroll * speed;
                self.zoom = self.zoom.clamp(0.1, 5.0);
            }
        }

        let pan_offset = self.pan_offset;
        let zoom = self.zoom;
        let to_screen =
            move |pos: Pos2| -> Pos2 { canvas_rect.min + (pos.to_vec2() + pan_offset) * zoom };

        let from_screen = move |screen_pos: Pos2| -> Pos2 {
            let canvas_pos = (screen_pos.to_vec2() - pan_offset) / zoom;
            Pos2::new(canvas_pos.x, canvas_pos.y)
        };

        // Draw grid
        self.draw_grid(&painter, canvas_rect);

        // Draw connections first (behind nodes)
        self.draw_connections(ui, &painter, module, &to_screen);

        // Collect socket positions for hit detection
        let mut all_sockets: Vec<SocketInfo> = Vec::new();

        // Collect part info and socket positions
        let part_rects: Vec<_> = module
            .parts
            .iter()
            .map(|part| {
                let part_screen_pos = to_screen(Pos2::new(part.position.0, part.position.1));
                let part_height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                let part_size = Vec2::new(200.0, part_height);
                let rect = Rect::from_min_size(part_screen_pos, part_size * self.zoom);

                // Calculate socket positions
                let title_height = 28.0 * self.zoom;
                let socket_start_y = rect.min.y + title_height + 10.0 * self.zoom;

                // Input sockets (left side)
                for (i, socket) in part.inputs.iter().enumerate() {
                    let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
                    all_sockets.push(SocketInfo {
                        part_id: part.id,
                        socket_idx: i,
                        is_output: false,
                        socket_type: socket.socket_type,
                        position: Pos2::new(rect.min.x, socket_y),
                    });
                }

                // Output sockets (right side)
                for (i, socket) in part.outputs.iter().enumerate() {
                    let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
                    all_sockets.push(SocketInfo {
                        part_id: part.id,
                        socket_idx: i,
                        is_output: true,
                        socket_type: socket.socket_type,
                        position: Pos2::new(rect.max.x, socket_y),
                    });
                }

                (part.id, rect)
            })
            .collect();

        // Handle socket clicks for creating connections
        let socket_radius = 8.0 * self.zoom;
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let primary_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
        let primary_released = ui.input(|i| i.pointer.any_released());
        let clicked = ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary));
        let released = ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary));

        // Start connection on mouse down over socket
        if let Some(pos) = pointer_pos {
            if primary_down && self.creating_connection.is_none() && self.dragging_part.is_none() {
                for socket in &all_sockets {
                    if socket.position.distance(pos) < socket_radius {
                        // Start creating a connection
                        self.creating_connection = Some((
                            socket.part_id,
                            socket.socket_idx,
                            socket.is_output,
                            socket.socket_type,
                            socket.position,
                        ));
                        break;
                    }
                }
            }

            // Complete connection on release over compatible socket
            if primary_released && self.creating_connection.is_some() {
                if let Some((from_part, from_socket, from_is_output, ref _from_type, _)) =
                    self.creating_connection
                {
                    for socket in &all_sockets {
                        if socket.position.distance(pos) < socket_radius * 1.5 {
                            // Validate connection: must be different parts, opposite directions
                            // Type check relaxed for now - allow any connection for testing
                            if socket.part_id != from_part && socket.is_output != from_is_output {
                                // Create connection (from output to input)
                                if from_is_output {
                                    module.add_connection(
                                        from_part,
                                        from_socket,
                                        socket.part_id,
                                        socket.socket_idx,
                                    );
                                } else {
                                    module.add_connection(
                                        socket.part_id,
                                        socket.socket_idx,
                                        from_part,
                                        from_socket,
                                    );
                                }
                            }
                            break;
                        }
                    }
                }
                self.creating_connection = None;
            }
        }

        // Clear connection if mouse released without hitting a socket
        if primary_released && self.creating_connection.is_some() {
            self.creating_connection = None;
        }

        // Draw wire preview while dragging (visual feedback)
        if let Some((_, _, is_output, ref socket_type, start_pos)) = self.creating_connection {
            if let Some(mouse_pos) = pointer_pos {
                // Draw bezier curve from start to mouse
                let wire_color = Self::get_socket_color(socket_type);
                let control_offset = 50.0 * self.zoom;

                // Calculate control points for smooth curve
                let (ctrl1, ctrl2) = if is_output {
                    // Dragging from output (right side) - curve goes right then to mouse
                    (
                        Pos2::new(start_pos.x + control_offset, start_pos.y),
                        Pos2::new(mouse_pos.x - control_offset, mouse_pos.y),
                    )
                } else {
                    // Dragging from input (left side) - curve goes left then to mouse
                    (
                        Pos2::new(start_pos.x - control_offset, start_pos.y),
                        Pos2::new(mouse_pos.x + control_offset, mouse_pos.y),
                    )
                };

                // Draw bezier path
                painter.add(CubicBezierShape::from_points_stroke(
                    [start_pos, ctrl1, ctrl2, mouse_pos],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(3.0 * self.zoom, wire_color),
                ));

                // Draw endpoint circle at mouse
                painter.circle_filled(mouse_pos, 6.0 * self.zoom, wire_color);
            }
        }

        // Handle box selection start (on empty canvas)
        if clicked && self.creating_connection.is_none() && self.dragging_part.is_none() {
            if let Some(pos) = pointer_pos {
                // Check if not clicking on any part
                let on_part = part_rects.iter().any(|(_, rect)| rect.contains(pos));
                if !on_part && canvas_rect.contains(pos) {
                    self.box_select_start = Some(pos);
                }
            }
        }

        // Handle box selection drag
        if let Some(start_pos) = self.box_select_start {
            if let Some(current_pos) = pointer_pos {
                // Draw selection rectangle
                let select_rect = Rect::from_two_pos(start_pos, current_pos);
                painter.rect_stroke(
                    select_rect,
                    0.0,
                    Stroke::new(2.0, Color32::from_rgb(100, 200, 255)),
                    egui::StrokeKind::Inside,
                );
                painter.rect_filled(
                    select_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(100, 200, 255, 30),
                );
            }

            if released {
                // Select all parts within the box
                if let Some(current_pos) = pointer_pos {
                    let select_rect = Rect::from_two_pos(start_pos, current_pos);
                    if !shift_held {
                        self.selected_parts.clear();
                    }
                    for (part_id, part_rect) in &part_rects {
                        if select_rect.intersects(*part_rect)
                            && !self.selected_parts.contains(part_id)
                        {
                            self.selected_parts.push(*part_id);
                        }
                    }
                }
                self.box_select_start = None;
            }
        }

        // Handle part dragging and delete buttons
        let mut delete_part_id: Option<ModulePartId> = None;

        for (part_id, rect) in &part_rects {
            let part_response =
                ui.interact(*rect, egui::Id::new(*part_id), Sense::click_and_drag());

            // Handle double-click to open property editor popup
            if part_response.double_clicked() {
                self.editing_part_id = Some(*part_id);
            }

            // Handle right-click to open context menu
            if part_response.secondary_clicked() {
                self.context_menu_part = Some(*part_id);
                self.context_menu_pos =
                    Some(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()));
            }

            // Handle click for selection
            if part_response.clicked() && self.creating_connection.is_none() {
                if shift_held {
                    // Shift+Click: toggle selection
                    if self.selected_parts.contains(part_id) {
                        self.selected_parts.retain(|id| id != part_id);
                    } else {
                        self.selected_parts.push(*part_id);
                    }
                } else {
                    // Normal click: replace selection
                    self.selected_parts.clear();
                    self.selected_parts.push(*part_id);
                }
            }

            if part_response.drag_started() && self.creating_connection.is_none() {
                self.dragging_part = Some((*part_id, Vec2::ZERO));
                // If dragging a non-selected part, select only it
                if !self.selected_parts.contains(part_id) {
                    self.selected_parts.clear();
                    self.selected_parts.push(*part_id);
                }
            }

            if part_response.dragged() {
                if let Some((dragged_id, _)) = self.dragging_part {
                    if dragged_id == *part_id {
                        let delta = part_response.drag_delta() / self.zoom;

                        // Calculate new position
                        if let Some(part) = module.parts.iter().find(|p| p.id == *part_id) {
                            let new_x = part.position.0 + delta.x;
                            let new_y = part.position.1 + delta.y;
                            let part_height =
                                80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                            let new_rect = Rect::from_min_size(
                                Pos2::new(new_x, new_y),
                                Vec2::new(200.0, part_height),
                            );

                            // Check collision with other parts
                            let has_collision = module.parts.iter().any(|other| {
                                if other.id == *part_id {
                                    return false;
                                }
                                let other_height = 80.0
                                    + (other.inputs.len().max(other.outputs.len()) as f32) * 20.0;
                                let other_rect = Rect::from_min_size(
                                    Pos2::new(other.position.0, other.position.1),
                                    Vec2::new(200.0, other_height),
                                );
                                new_rect.intersects(other_rect)
                            });

                            // Only move if no collision
                            if !has_collision {
                                if let Some(part_mut) =
                                    module.parts.iter_mut().find(|p| p.id == *part_id)
                                {
                                    part_mut.position.0 = new_x;
                                    part_mut.position.1 = new_y;
                                }
                            }
                        }
                    }
                }
            }

            if part_response.drag_stopped() {
                self.dragging_part = None;
            }

            // Check for delete button click (× in top-right corner of title bar)
            let delete_button_rect = self.get_delete_button_rect(*rect);
            let delete_id = egui::Id::new((*part_id, "delete"));
            let delete_response = ui
                .interact(delete_button_rect, delete_id, Sense::click())
                .on_hover_text("Hold to delete (Mouse or Space/Enter)");

            // Mary StyleUX: Hold-to-Confirm for Node Deletion (Safety)
            if delete_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            let is_holding_delete = delete_response.is_pointer_button_down_on()
                || (delete_response.has_focus()
                    && ui.input(|i| i.key_down(egui::Key::Space) || i.key_down(egui::Key::Enter)));

            // Use shared helper
            let hold_duration = 0.6;
            let (triggered, progress) = crate::widgets::check_hold_state(
                ui,
                delete_id,
                is_holding_delete,
                hold_duration,
            );

            // Store progress for visualization in draw_part_with_delete
            ui.data_mut(|d| d.insert_temp(delete_id.with("progress"), progress));

            if triggered {
                delete_part_id = Some(*part_id);
                ui.data_mut(|d| d.remove_temp::<f32>(delete_id.with("progress")));
            }
        }

        // Process pending deletion
        if let Some(part_id) = delete_part_id {
            // Remove all connections involving this part
            module
                .connections
                .retain(|c| c.from_part != part_id && c.to_part != part_id);
            // Remove the part
            module.parts.retain(|p| p.id != part_id);
        }

        // Resize operations to apply after the loop
        let mut resize_ops = Vec::new();

        // Draw parts (nodes) with delete buttons and selection highlight
        for part in &module.parts {
            let part_screen_pos = to_screen(Pos2::new(part.position.0, part.position.1));

            // Use custom size or calculate default
            let (part_width, part_height) = part.size.unwrap_or_else(|| {
                let default_height =
                    80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                (200.0, default_height)
            });
            let part_size = Vec2::new(part_width, part_height);
            let part_screen_rect = Rect::from_min_size(part_screen_pos, part_size * self.zoom);

            // Draw selection highlight if selected
            if self.selected_parts.contains(&part.id) {
                let highlight_rect = part_screen_rect.expand(4.0 * self.zoom);
                // "Cyber" selection: Neon Cyan, Sharp Corners
                painter.rect_stroke(
                    highlight_rect,
                    0, // Sharp corners
                    Stroke::new(2.0 * self.zoom, Color32::from_rgb(0, 229, 255)),
                    egui::StrokeKind::Inside,
                );

                // Draw resize handle at bottom-right corner
                let handle_size = 12.0 * self.zoom;
                let handle_rect = Rect::from_min_size(
                    Pos2::new(
                        part_screen_rect.max.x - handle_size,
                        part_screen_rect.max.y - handle_size,
                    ),
                    Vec2::splat(handle_size),
                );
                // Cyan resize handle, sharp
                painter.rect_filled(handle_rect, 0, Color32::from_rgb(0, 229, 255));
                // Draw diagonal lines for resize indicator
                painter.line_segment(
                    [
                        handle_rect.min + Vec2::new(3.0, handle_size - 3.0),
                        handle_rect.min + Vec2::new(handle_size - 3.0, 3.0),
                    ],
                    Stroke::new(1.5, Color32::from_gray(40)),
                );

                // Handle resize drag interaction
                let resize_response = ui.interact(
                    handle_rect,
                    egui::Id::new((part.id, "resize")),
                    Sense::drag(),
                );

                if resize_response.drag_started() {
                    self.resizing_part = Some((part.id, (part_width, part_height)));
                }

                if resize_response.dragged() {
                    if let Some((id, _original_size)) = self.resizing_part {
                        if id == part.id {
                            let delta = resize_response.drag_delta() / self.zoom;
                            resize_ops.push((part.id, delta));
                        }
                    }
                }

                if resize_response.drag_stopped() {
                    self.resizing_part = None;
                }
            }

            self.draw_part_with_delete(ui, &painter, part, part_screen_rect, actions, module.id);
        }

        // Apply resize operations
        for (part_id, delta) in resize_ops {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                // Initialize size if None
                let current_size = part.size.unwrap_or_else(|| {
                    let h = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                    (200.0, h)
                });
                let new_w = (current_size.0 + delta.x).max(100.0);
                let new_h = (current_size.1 + delta.y).max(50.0);
                part.size = Some((new_w, new_h));
            }
        }

        // Draw connection being created with visual feedback
        if let Some((from_part_id, _from_socket_idx, from_is_output, ref from_type, start_pos)) =
            self.creating_connection
        {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                // Check if hovering over a compatible socket
                let socket_radius = 8.0 * self.zoom;
                let mut is_valid_target = false;
                let mut near_socket = false;

                for socket in &all_sockets {
                    if socket.position.distance(pointer_pos) < socket_radius * 2.0 {
                        near_socket = true;
                        // Valid if: different part, opposite direction, same type
                        if socket.part_id != from_part_id
                            && socket.is_output != from_is_output
                            && socket.socket_type == *from_type
                        {
                            is_valid_target = true;
                        }
                        break;
                    }
                }

                // Color based on validity
                let color = if near_socket {
                    if is_valid_target {
                        Color32::from_rgb(50, 255, 100) // Green = valid
                    } else {
                        Color32::from_rgb(255, 80, 80) // Red = invalid
                    }
                } else {
                    Self::get_socket_color(from_type) // Default socket color
                };

                // Draw the connection line
                painter.line_segment([start_pos, pointer_pos], Stroke::new(3.0, color));

                // Draw a circle at the end point
                painter.circle_filled(pointer_pos, 5.0, color);
            }
        }

        // Draw mini-map in bottom-right corner
        self.draw_mini_map(&painter, canvas_rect, module);

        // Draw search popup if visible
        if self.show_search {
            self.draw_search_popup(ui, canvas_rect, module);
        }

        // Draw presets popup if visible
        if self.show_presets {
            self.draw_presets_popup(ui, canvas_rect, module);
        }

        // Draw diagnostics popup if visible
        self.render_diagnostics_popup(ui);

        // Draw context menu for parts
        if let (Some(part_id), Some(pos)) = (self.context_menu_part, self.context_menu_pos) {
            let menu_width = 150.0;
            let menu_height = 80.0;
            let menu_rect = Rect::from_min_size(pos, Vec2::new(menu_width, menu_height));

            // Draw menu background
            let painter = ui.painter();
            painter.rect_filled(
                menu_rect,
                0.0,
                Color32::from_rgba_unmultiplied(40, 40, 50, 250),
            );
            painter.rect_stroke(
                menu_rect,
                0.0,
                Stroke::new(1.0, Color32::from_rgb(80, 80, 100)),
                egui::StrokeKind::Inside,
            );

            // Menu items
            let inner_rect = menu_rect.shrink(4.0);
            ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                ui.vertical(|ui| {
                    if ui.button("⚙ Open Properties").clicked() {
                        // Select the part to show it in the inspector
                        self.selected_parts.clear();
                        self.selected_parts.push(part_id);
                        self.context_menu_part = None;
                        self.context_menu_pos = None;
                    }
                    if ui.button("🗑 Delete").clicked() {
                        // Remove connections and part
                        module
                            .connections
                            .retain(|c| c.from_part != part_id && c.to_part != part_id);
                        module.parts.retain(|p| p.id != part_id);
                        self.context_menu_part = None;
                        self.context_menu_pos = None;
                    }
                });
            });

            // Close menu on click outside
            if ui.input(|i| i.pointer.any_click())
                && !menu_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
            {
                self.context_menu_part = None;
                self.context_menu_pos = None;
            }
        }

        // Draw context menu for connections
        if let (Some(conn_idx), Some(pos)) = (self.context_menu_connection, self.context_menu_pos) {
            let menu_width = 150.0;
            let menu_height = 40.0;
            let menu_rect = Rect::from_min_size(pos, Vec2::new(menu_width, menu_height));

            // Draw menu background
            let painter = ui.painter();
            painter.rect_filled(
                menu_rect,
                0.0,
                Color32::from_rgba_unmultiplied(40, 40, 50, 250),
            );
            painter.rect_stroke(
                menu_rect,
                0.0,
                Stroke::new(1.0, Color32::from_rgb(80, 80, 100)),
                egui::StrokeKind::Inside,
            );

            // Menu items
            let inner_rect = menu_rect.shrink(4.0);
            ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                ui.vertical(|ui| {
                    if ui.button("🗑 Delete Connection").clicked() {
                        if conn_idx < module.connections.len() {
                            module.connections.remove(conn_idx);
                        }
                        self.context_menu_connection = None;
                        self.context_menu_pos = None;
                    }
                });
            });

            // Close menu on click outside
            if ui.input(|i| i.pointer.any_click())
                && !menu_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
            {
                self.context_menu_connection = None;
                self.context_menu_pos = None;
            }
        }

        // Draw context menu for adding nodes (canvas level)
        if self.context_menu_part.is_none() && self.context_menu_connection.is_none() {
            if let Some(pos) = self.context_menu_pos {
                let menu_width = 180.0;
                let menu_height = 250.0; // Estimate or let it be dynamic
                let menu_rect = Rect::from_min_size(pos, Vec2::new(menu_width, menu_height));

                // Draw menu background
                let painter = ui.painter();
                painter.rect_filled(
                    menu_rect,
                    4.0,
                    Color32::from_rgba_unmultiplied(30, 30, 40, 245),
                );
                painter.rect_stroke(
                    menu_rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgb(80, 100, 150)),
                    egui::StrokeKind::Inside,
                );

                // Menu items
                let inner_rect = menu_rect.shrink(8.0);
                ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                    ui.vertical(|ui| {
                        ui.heading("➕ Add Node");
                        ui.separator();

                        // Convert screen position to canvas position for node placement
                        let canvas_pos = from_screen(pos);
                        let pos_tuple = (canvas_pos.x, canvas_pos.y);

                        self.render_add_node_menu_content(ui, manager, Some(pos_tuple));
                    });
                });

                // Close menu on click outside
                if ui.input(|i| i.pointer.any_click())
                    && !menu_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
                {
                    self.context_menu_pos = None;
                }
            }
        }
    }

    fn draw_search_popup(&mut self, ui: &mut Ui, canvas_rect: Rect, module: &mut MapFlowModule) {
        // Search popup in top-center
        let popup_width = 300.0;
        let popup_height = 200.0;
        let popup_rect = Rect::from_min_size(
            Pos2::new(
                canvas_rect.center().x - popup_width / 2.0,
                canvas_rect.min.y + 50.0,
            ),
            Vec2::new(popup_width, popup_height),
        );

        // Draw popup background
        let painter = ui.painter();
        painter.rect_filled(
            popup_rect,
            0.0,
            Color32::from_rgba_unmultiplied(30, 30, 40, 240),
        );
        painter.rect_stroke(
            popup_rect,
            0.0,
            Stroke::new(2.0, Color32::from_rgb(80, 120, 200)),
            egui::StrokeKind::Inside,
        );

        // Popup content
        let inner_rect = popup_rect.shrink(10.0);
        ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.search_filter);
                });
                ui.add_space(8.0);

                // Filter and show matching nodes
                let filter_lower = self.search_filter.to_lowercase();
                let matching_parts: Vec<_> = module
                    .parts
                    .iter()
                    .filter(|p| {
                        if filter_lower.is_empty() {
                            return true;
                        }
                        let name = Self::get_part_property_text(&p.part_type).to_lowercase();
                        let (_, _, _, type_name) = Self::get_part_style(&p.part_type);
                        name.contains(&filter_lower)
                            || type_name.to_lowercase().contains(&filter_lower)
                    })
                    .take(6)
                    .collect();

                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for part in matching_parts {
                            let (_, _, icon, type_name) = Self::get_part_style(&part.part_type);
                            let label = format!(
                                "{} {} - {}",
                                icon,
                                type_name,
                                Self::get_part_property_text(&part.part_type)
                            );
                            if ui
                                .selectable_label(self.selected_parts.contains(&part.id), &label)
                                .clicked()
                            {
                                self.selected_parts.clear();
                                self.selected_parts.push(part.id);
                                // Center view on selected node
                                self.pan_offset =
                                    Vec2::new(-part.position.0 + 200.0, -part.position.1 + 150.0);
                                self.show_search = false;
                            }
                        }
                    });
            });
        });
    }

    fn draw_presets_popup(&mut self, ui: &mut Ui, canvas_rect: Rect, module: &mut MapFlowModule) {
        // Presets popup in top-center
        let popup_width = 280.0;
        let popup_height = 220.0;
        let popup_rect = Rect::from_min_size(
            Pos2::new(
                canvas_rect.center().x - popup_width / 2.0,
                canvas_rect.min.y + 50.0,
            ),
            Vec2::new(popup_width, popup_height),
        );

        // Draw popup background
        let painter = ui.painter();
        painter.rect_filled(
            popup_rect,
            0.0,
            Color32::from_rgba_unmultiplied(30, 35, 45, 245),
        );
        painter.rect_stroke(
            popup_rect,
            0.0,
            Stroke::new(2.0, Color32::from_rgb(100, 180, 80)),
            egui::StrokeKind::Inside,
        );

        // Popup content
        let inner_rect = popup_rect.shrink(12.0);
        ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
            ui.vertical(|ui| {
                ui.heading("📋 Presets / Templates");
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        let presets = self.presets.clone();
                        for preset in &presets {
                            ui.horizontal(|ui| {
                                if ui.button(&preset.name).clicked() {
                                    // Clear current and load preset
                                    module.parts.clear();
                                    module.connections.clear();

                                    // Add parts from preset
                                    let mut part_ids = Vec::new();
                                    let mut next_id =
                                        module.parts.iter().map(|p| p.id).max().unwrap_or(0) + 1;
                                    for (part_type, position, size) in &preset.parts {
                                        let id = next_id;
                                        next_id += 1;

                                        let (inputs, outputs) =
                                            Self::get_sockets_for_part_type(part_type);

                                        module.parts.push(mapmap_core::module::ModulePart {
                                            id,
                                            part_type: part_type.clone(),
                                            position: *position,
                                            size: *size,
                                            inputs,
                                            outputs,
                                            link_data: NodeLinkData::default(),
                                            trigger_targets: std::collections::HashMap::new(),
                                        });
                                        part_ids.push(id);
                                    }

                                    // Add connections
                                    for (from_idx, from_socket, to_idx, to_socket) in
                                        &preset.connections
                                    {
                                        if *from_idx < part_ids.len() && *to_idx < part_ids.len() {
                                            module.connections.push(
                                                mapmap_core::module::ModuleConnection {
                                                    from_part: part_ids[*from_idx],
                                                    from_socket: *from_socket,
                                                    to_part: part_ids[*to_idx],
                                                    to_socket: *to_socket,
                                                },
                                            );
                                        }
                                    }

                                    self.show_presets = false;
                                }
                                ui.label(format!("({} nodes)", preset.parts.len()));
                            });
                        }
                    });

                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    self.show_presets = false;
                }
            });
        });
    }

    /// Render the 2D Spatial Editor for Hue lamps
    fn render_hue_spatial_editor(
        &self,
        ui: &mut Ui,
        lamp_positions: &mut std::collections::HashMap<String, (f32, f32)>,
    ) {
        let editor_size = Vec2::new(300.0, 300.0);
        let (response, painter) = ui.allocate_painter(editor_size, Sense::click_and_drag());
        let rect = response.rect;

        // Draw background (Room representation)
        painter.rect_filled(rect, 4.0, Color32::from_gray(30));
        painter.rect_stroke(
            rect,
            4,
            Stroke::new(1.0, Color32::GRAY),
            egui::StrokeKind::Inside,
        );

        // Draw grid
        let grid_steps = 5;
        for i in 1..grid_steps {
            let t = i as f32 / grid_steps as f32;
            let x = rect.min.x + t * rect.width();
            let y = rect.min.y + t * rect.height();

            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(1.0, Color32::from_white_alpha(20)),
            );
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0, Color32::from_white_alpha(20)),
            );
        }

        // Labels
        painter.text(
            rect.center_top() + Vec2::new(0.0, 10.0),
            egui::Align2::CENTER_TOP,
            "Front (TV/Screen)",
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );

        // If empty, add dummy lamps for visualization/testing
        if lamp_positions.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No Lamps Mapped",
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
            // Typically we would populate this from the Entertainment Area config
            if ui.button("Add Test Lamps").clicked() {
                lamp_positions.insert("1".to_string(), (0.2, 0.2)); // Front Left
                lamp_positions.insert("2".to_string(), (0.8, 0.2)); // Front Right
                lamp_positions.insert("3".to_string(), (0.2, 0.8)); // Rear Left
                lamp_positions.insert("4".to_string(), (0.8, 0.8)); // Rear Right
            }
            return;
        }

        let to_screen = |x: f32, y: f32| -> Pos2 {
            Pos2::new(
                rect.min.x + x.clamp(0.0, 1.0) * rect.width(),
                rect.min.y + y.clamp(0.0, 1.0) * rect.height(),
            )
        };

        // Handle lamp dragging
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let _is_dragging = ui.input(|i| i.pointer.primary_down());

        let mut dragged_lamp = None;

        // If dragging, find closest lamp
        if response.dragged() {
            if let Some(pos) = pointer_pos {
                // Find closest lamp within radius
                let mut min_dist = f32::MAX;
                let mut closest_id = None;

                for (id, (lx, ly)) in lamp_positions.iter() {
                    let lamp_pos = to_screen(*lx, *ly);
                    let dist = lamp_pos.distance(pos);
                    if dist < 20.0 && dist < min_dist {
                        min_dist = dist;
                        closest_id = Some(id.clone());
                    }
                }

                if let Some(id) = closest_id {
                    dragged_lamp = Some(id);
                }
            }
        }

        if let Some(id) = dragged_lamp {
            if let Some(pos) = pointer_pos {
                // Update position
                let nx = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                let ny = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                lamp_positions.insert(id, (nx, ny));
            }
        }

        // Draw Lamps
        for (id, (lx, ly)) in lamp_positions.iter() {
            let pos = to_screen(*lx, *ly);

            // Draw lamp body
            painter.circle_filled(pos, 8.0, Color32::from_rgb(255, 200, 100));
            painter.circle_stroke(pos, 8.0, Stroke::new(2.0, Color32::WHITE));

            // Draw Label
            painter.text(
                pos + Vec2::new(0.0, 12.0),
                egui::Align2::CENTER_TOP,
                id,
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );
        }
    }

    /// Get default sockets for a part type
    fn get_sockets_for_part_type(
        part_type: &mapmap_core::module::ModulePartType,
    ) -> (
        Vec<mapmap_core::module::ModuleSocket>,
        Vec<mapmap_core::module::ModuleSocket>,
    ) {
        use mapmap_core::module::{ModulePartType, ModuleSocket, ModuleSocketType};

        match part_type {
            ModulePartType::Trigger(_) => (
                vec![],
                vec![ModuleSocket {
                    name: "Trigger Out".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
            ),
            ModulePartType::Source(_) => (
                vec![ModuleSocket {
                    name: "Trigger In".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Mask(_) => (
                vec![
                    ModuleSocket {
                        name: "Media In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Mask In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                ],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Modulizer(_) => (
                vec![
                    ModuleSocket {
                        name: "Media In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Trigger In".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Mesh(_) => (vec![], vec![]),
            ModulePartType::Layer(_) => (
                vec![ModuleSocket {
                    name: "Media In".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
                vec![ModuleSocket {
                    name: "Layer Out".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
            ),
            ModulePartType::Output(_) => (
                vec![ModuleSocket {
                    name: "Layer In".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
                vec![],
            ),
            ModulePartType::Hue(_) => (
                vec![
                    ModuleSocket {
                        name: "Brightness".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                    ModuleSocket {
                        name: "Color (RGB)".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Strobe".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![],
            ),
        }
    }

    fn draw_mini_map(&self, painter: &egui::Painter, canvas_rect: Rect, module: &MapFlowModule) {
        if module.parts.is_empty() {
            return;
        }

        // Mini-map size and position
        let map_size = Vec2::new(150.0, 100.0);
        let map_margin = 10.0;
        let map_rect = Rect::from_min_size(
            Pos2::new(
                canvas_rect.max.x - map_size.x - map_margin,
                canvas_rect.max.y - map_size.y - map_margin,
            ),
            map_size,
        );

        // Background
        painter.rect_filled(
            map_rect,
            0,
            Color32::from_rgba_unmultiplied(30, 30, 40, 200),
        );
        painter.rect_stroke(
            map_rect,
            0,
            Stroke::new(1.0, Color32::from_gray(80)),
            egui::StrokeKind::Inside,
        );

        // Calculate bounds of all parts
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for part in &module.parts {
            let height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
            min_x = min_x.min(part.position.0);
            min_y = min_y.min(part.position.1);
            max_x = max_x.max(part.position.0 + 200.0);
            max_y = max_y.max(part.position.1 + height);
        }

        // Add padding
        let padding = 50.0;
        min_x -= padding;
        min_y -= padding;
        max_x += padding;
        max_y += padding;

        let world_width = (max_x - min_x).max(1.0);
        let world_height = (max_y - min_y).max(1.0);

        // Scale to fit in mini-map
        let scale_x = (map_size.x - 8.0) / world_width;
        let scale_y = (map_size.y - 8.0) / world_height;
        let scale = scale_x.min(scale_y);

        let to_map = |pos: Pos2| -> Pos2 {
            Pos2::new(
                map_rect.min.x + 4.0 + (pos.x - min_x) * scale,
                map_rect.min.y + 4.0 + (pos.y - min_y) * scale,
            )
        };

        // Draw parts as small rectangles
        for part in &module.parts {
            let height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
            let part_min = to_map(Pos2::new(part.position.0, part.position.1));
            let part_max = to_map(Pos2::new(part.position.0 + 200.0, part.position.1 + height));
            let part_rect = Rect::from_min_max(part_min, part_max);

            let (_, title_color, _, _) = Self::get_part_style(&part.part_type);
            painter.rect_filled(part_rect, 1.0, title_color);
        }

        // Draw viewport rectangle
        let viewport_min = to_map(Pos2::new(
            -self.pan_offset.x / self.zoom,
            -self.pan_offset.y / self.zoom,
        ));
        let viewport_max = to_map(Pos2::new(
            (-self.pan_offset.x + canvas_rect.width()) / self.zoom,
            (-self.pan_offset.y + canvas_rect.height()) / self.zoom,
        ));
        let viewport_rect = Rect::from_min_max(viewport_min, viewport_max).intersect(map_rect);
        painter.rect_stroke(
            viewport_rect,
            0,
            Stroke::new(1.5, Color32::WHITE),
            egui::StrokeKind::Inside,
        );
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_size = 20.0 * self.zoom;
        let color = Color32::from_rgb(40, 40, 40);
        let mut x = rect.left() - self.pan_offset.x % grid_size;
        while x < rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, color),
            );
            x += grid_size;
        }
        let mut y = rect.top() - self.pan_offset.y % grid_size;
        while y < rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, color),
            );
            y += grid_size;
        }
    }

    fn draw_connections<F>(
        &mut self,
        ui: &Ui,
        painter: &egui::Painter,
        module: &MapFlowModule,
        to_screen: &F,
    ) where
        F: Fn(Pos2) -> Pos2,
    {
        let node_width = 200.0;
        let title_height = 28.0;
        let socket_offset_y = 10.0;
        let socket_spacing = 22.0;
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let secondary_clicked = ui.input(|i| i.pointer.secondary_clicked());

        for (conn_idx, conn) in module.connections.iter().enumerate() {
            // Find source and target parts
            let from_part = module.parts.iter().find(|p| p.id == conn.from_part);
            let to_part = module.parts.iter().find(|p| p.id == conn.to_part);

            if let (Some(from), Some(to)) = (from_part, to_part) {
                // Determine cable color based on socket type
                let socket_type = if let Some(socket) = from.outputs.get(conn.from_socket) {
                    &socket.socket_type
                } else if let Some(socket) = to.inputs.get(conn.to_socket) {
                    &socket.socket_type
                } else {
                    &mapmap_core::module::ModuleSocketType::Media // Fallback
                };
                let cable_color = Self::get_socket_color(socket_type);

                // Calculate WORLD positions
                // Output: Right side + center of socket height
                let from_local_y = title_height
                    + socket_offset_y
                    + conn.from_socket as f32 * socket_spacing
                    + socket_spacing / 2.0;
                let from_socket_world =
                    Pos2::new(from.position.0 + node_width, from.position.1 + from_local_y);

                // Input: Left side + center of socket height
                let to_local_y = title_height
                    + socket_offset_y
                    + conn.to_socket as f32 * socket_spacing
                    + socket_spacing / 2.0;
                let to_socket_world = Pos2::new(to.position.0, to.position.1 + to_local_y);

                // Convert to SCREEN positions
                let start_pos = to_screen(from_socket_world);
                let end_pos = to_screen(to_socket_world);

                // Draw Plugs - plugs should point INTO the nodes
                let plug_size = 20.0 * self.zoom;

                let icon_name = match socket_type {
                    mapmap_core::module::ModuleSocketType::Trigger => "audio-jack.svg",
                    mapmap_core::module::ModuleSocketType::Media => "plug.svg",
                    mapmap_core::module::ModuleSocketType::Effect => "usb-cable.svg",
                    mapmap_core::module::ModuleSocketType::Layer => "power-plug.svg",
                    mapmap_core::module::ModuleSocketType::Output => "power-plug.svg",
                    mapmap_core::module::ModuleSocketType::Link => "power-plug.svg",
                };

                // Draw Cable (Bezier)
                let cable_start = start_pos;
                let cable_end = end_pos;

                let control_offset = (cable_end.x - cable_start.x).abs() * 0.4;
                let control_offset = control_offset.max(40.0 * self.zoom);

                let ctrl1 = Pos2::new(cable_start.x + control_offset, cable_start.y);
                let ctrl2 = Pos2::new(cable_end.x - control_offset, cable_end.y);

                // Hit Detection (Approximate Bezier with segments)
                let mut is_hovered = false;
                if let Some(pos) = pointer_pos {
                    let steps = 20;
                    let threshold = 5.0 * self.zoom.max(1.0); // Adjust hit area with zoom

                    // OPTIMIZATION: Broad-phase AABB Check
                    let min_x =
                        cable_start.x.min(cable_end.x).min(ctrl1.x).min(ctrl2.x) - threshold;
                    let max_x =
                        cable_start.x.max(cable_end.x).max(ctrl1.x).max(ctrl2.x) + threshold;
                    let min_y =
                        cable_start.y.min(cable_end.y).min(ctrl1.y).min(ctrl2.y) - threshold;
                    let max_y =
                        cable_start.y.max(cable_end.y).max(ctrl1.y).max(ctrl2.y) + threshold;

                    let in_aabb =
                        pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y;

                    if in_aabb {
                        // Iterative Bezier calculation (De Casteljau's algorithm logic unrolled/simplified)
                        let mut prev_p = cable_start;
                        for i in 1..=steps {
                            let t = i as f32 / steps as f32;
                            let l1 = cable_start.lerp(ctrl1, t);
                            let l2 = ctrl1.lerp(ctrl2, t);
                            let l3 = ctrl2.lerp(cable_end, t);
                            let q1 = l1.lerp(l2, t);
                            let q2 = l2.lerp(l3, t);
                            let p = q1.lerp(q2, t);

                            // Distance to segment
                            let segment = p - prev_p;
                            let len_sq = segment.length_sq();
                            if len_sq > 0.0 {
                                let t_proj = ((pos - prev_p).dot(segment) / len_sq).clamp(0.0, 1.0);
                                let closest = prev_p + segment * t_proj;
                                if pos.distance(closest) < threshold {
                                    is_hovered = true;
                                    break;
                                }
                            }
                            prev_p = p;
                        }
                    }
                }

                // Handle Interaction
                if is_hovered && secondary_clicked {
                    self.context_menu_connection = Some(conn_idx);
                    self.context_menu_pos = pointer_pos;
                    self.context_menu_part = None;
                }

                // Visual Style
                let (stroke_width, stroke_color, glow_width) = if is_hovered {
                    (3.0 * self.zoom, Color32::WHITE, 8.0 * self.zoom)
                } else {
                    (2.0 * self.zoom, cable_color, 6.0 * self.zoom)
                };

                // Glow (Behind)
                let glow_stroke = Stroke::new(glow_width, cable_color.linear_multiply(0.3));
                painter.add(CubicBezierShape::from_points_stroke(
                    [cable_start, ctrl1, ctrl2, cable_end],
                    false,
                    Color32::TRANSPARENT,
                    glow_stroke,
                ));

                // Core Cable (Front)
                let cable_stroke = Stroke::new(stroke_width, stroke_color);
                painter.add(CubicBezierShape::from_points_stroke(
                    [cable_start, ctrl1, ctrl2, cable_end],
                    false,
                    Color32::TRANSPARENT,
                    cable_stroke,
                ));

                // Add flow animation
                if self.zoom > 0.6 {
                    let time = ui.input(|i| i.time);
                    let flow_t = (time * 1.5).fract() as f32;
                    let l1 = cable_start.lerp(ctrl1, flow_t);
                    let l2 = ctrl1.lerp(ctrl2, flow_t);
                    let l3 = ctrl2.lerp(cable_end, flow_t);
                    let q1 = l1.lerp(l2, flow_t);
                    let q2 = l2.lerp(l3, flow_t);
                    let flow_pos = q1.lerp(q2, flow_t);

                    painter.circle_filled(
                        flow_pos,
                        3.0 * self.zoom,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 150),
                    );
                }
                // Draw Plugs on top of cable
                if let Some(texture) = self.plug_icons.get(icon_name) {
                    // Source Plug at OUTPUT socket - pointing LEFT (into node)
                    let start_rect = Rect::from_center_size(start_pos, Vec2::splat(plug_size));
                    // Flip horizontally so plug points left (into node)
                    painter.image(
                        texture.id(),
                        start_rect,
                        Rect::from_min_max(Pos2::new(1.0, 0.0), Pos2::new(0.0, 1.0)),
                        Color32::WHITE,
                    );

                    // Target Plug at INPUT socket - pointing RIGHT (into node)
                    let end_rect = Rect::from_center_size(end_pos, Vec2::splat(plug_size));
                    // Normal orientation (pointing right into node)
                    painter.image(
                        texture.id(),
                        end_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    // Fallback circles
                    painter.circle_filled(start_pos, 6.0 * self.zoom, cable_color);
                    painter.circle_filled(end_pos, 6.0 * self.zoom, cable_color);
                }
            }
        }
    }

    fn get_delete_button_rect(&self, part_rect: Rect) -> Rect {
        let title_height = 28.0 * self.zoom;
        Rect::from_center_size(
            Pos2::new(
                part_rect.max.x - 10.0 * self.zoom,
                part_rect.min.y + title_height * 0.5,
            ),
            Vec2::splat(20.0 * self.zoom),
        )
    }

    fn draw_part_with_delete(
        &self,
        ui: &Ui,
        painter: &egui::Painter,
        part: &ModulePart,
        rect: Rect,
        actions: &mut Vec<UIAction>,
        module_id: mapmap_core::module::ModuleId,
    ) {
        // Get part color and name based on type
        let (_bg_color, title_color, icon, name) = Self::get_part_style(&part.part_type);
        let category = Self::get_part_category(&part.part_type);

        // Check if this is an audio trigger and if it's active
        let (is_audio_trigger, audio_trigger_value, threshold, is_audio_active) =
            self.get_audio_trigger_state(&part.part_type);

        // Check generic trigger value from evaluator
        let generic_trigger_value = self
            .last_trigger_values
            .get(&part.id)
            .copied()
            .unwrap_or(0.0);
        let is_generic_active = generic_trigger_value > 0.1;

        // Combine
        let trigger_value = if is_generic_active {
            generic_trigger_value
        } else {
            audio_trigger_value
        };
        let is_active = is_audio_active || is_generic_active;

        // Draw glow effect if active
        if is_active {
            let glow_intensity = (trigger_value * 2.0).min(1.0);
            let base_color =
                Color32::from_rgba_unmultiplied(255, (160.0 * glow_intensity) as u8, 0, 255);

            // Cyber-Glow: Multi-layered sharp strokes
            for i in 1..=4 {
                let expansion = i as f32 * 1.5 * self.zoom;
                let alpha = (100.0 / (i as f32)).min(255.0) as u8;
                let color = base_color
                    .linear_multiply(glow_intensity)
                    .gamma_multiply(alpha as f32 / 255.0);

                painter.rect_stroke(
                    rect.expand(expansion),
                    0.0,
                    Stroke::new(1.0 * self.zoom, color),
                    egui::StrokeKind::Outside,
                );
            }

            // Inner "Light" border
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(
                    2.0 * self.zoom,
                    Color32::WHITE.gamma_multiply(180.0 * glow_intensity / 255.0),
                ),
                egui::StrokeKind::Inside,
            );
        }

        // MIDI Learn Highlight
        let is_midi_learn = self.midi_learn_part_id == Some(part.id);
        if is_midi_learn {
            let time = ui.input(|i| i.time);
            let pulse = (time * 8.0).sin().abs() as f32;
            let learn_color = Color32::from_rgb(0, 200, 255).linear_multiply(pulse);

            painter.rect_stroke(
                rect.expand(4.0 * self.zoom),
                0.0,
                Stroke::new(2.0 * self.zoom, learn_color),
                egui::StrokeKind::Outside,
            );

            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "WAITING FOR MIDI...",
                egui::FontId::proportional(12.0 * self.zoom),
                Color32::WHITE.gamma_multiply(200.0 * pulse / 255.0),
            );
        }

        // Draw shadow behind node
        let _shadow = Shadow {
            offset: [(2.0 * self.zoom) as i8, (4.0 * self.zoom) as i8],
            blur: (12.0 * self.zoom).min(255.0) as u8,
            spread: 0,
            color: Color32::from_black_alpha(100),
        };
        // TODO: Shadow::tessellate was removed in egui 0.33
        // painter.add(shadow.tessellate(rect, (6.0 * self.zoom) as u8));

        // Draw background (Dark Neutral for high contrast)
        // We use a very dark grey/black to make the content pop
        let neutral_bg = colors::DARK_GREY;
        // Sharp corners for "Cyber" look
        painter.rect_filled(rect, 0, neutral_bg);

        // Handle drag and drop for Media Files
        if let mapmap_core::module::ModulePartType::Source(
            mapmap_core::module::SourceType::MediaFile { .. },
        ) = &part.part_type
        {
            if ui.rect_contains_pointer(rect) {
                if let Some(dropped_path) = ui
                    .ctx()
                    .data(|d| d.get_temp::<std::path::PathBuf>(egui::Id::new("media_path")))
                {
                    painter.rect_stroke(
                        rect,
                        0,
                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                        egui::StrokeKind::Outside,
                    );

                    if ui.input(|i| i.pointer.any_released()) {
                        actions.push(UIAction::SetMediaFile(
                            module_id,
                            part.id,
                            dropped_path.to_string_lossy().to_string(),
                        ));
                    }
                }
            }
        }

        // Node border - colored by type for quick identification
        // This replaces the generic gray border
        painter.rect_stroke(
            rect,
            0, // Sharp corners
            Stroke::new(1.5 * self.zoom, title_color.linear_multiply(0.8)),
            egui::StrokeKind::Inside,
        );

        // Title bar
        let title_height = 28.0 * self.zoom;
        let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), title_height));

        // Title bar background (Dark)
        painter.rect_filled(
            title_rect,
            0, // Sharp corners
            colors::LIGHTER_GREY,
        );

        // Title bar Top Accent Stripe (Type Identifier)
        let stripe_height = 3.0 * self.zoom;
        let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), stripe_height));
        painter.rect_filled(stripe_rect, 0, title_color);

        // Title separator line - make it sharper
        painter.line_segment(
            [
                Pos2::new(rect.min.x, rect.min.y + title_height),
                Pos2::new(rect.max.x, rect.min.y + title_height),
            ],
            Stroke::new(1.0, colors::STROKE_GREY),
        );

        // Enhanced Title Rendering (Icon | Category | Name)
        let mut cursor_x = rect.min.x + 8.0 * self.zoom;
        let center_y = title_rect.center().y;

        // 1. Icon
        let icon_galley = ui.painter().layout_no_wrap(
            icon.to_string(),
            egui::FontId::proportional(16.0 * self.zoom),
            Color32::WHITE,
        );
        painter.galley(
            Pos2::new(cursor_x, center_y - icon_galley.size().y / 2.0),
            icon_galley.clone(),
            Color32::WHITE,
        );
        cursor_x += icon_galley.size().x + 6.0 * self.zoom;

        // 2. Category (Small Caps style, Dimmed)
        let category_text = category.to_uppercase();
        let category_color = Color32::from_white_alpha(160);
        let category_galley = ui.painter().layout_no_wrap(
            category_text,
            egui::FontId::proportional(10.0 * self.zoom),
            category_color,
        );
        painter.galley(
            Pos2::new(cursor_x, center_y - category_galley.size().y / 2.0),
            category_galley.clone(),
            category_color,
        );
        cursor_x += category_galley.size().x + 6.0 * self.zoom;

        // 3. Name (Bold/Bright)
        let name_galley = ui.painter().layout_no_wrap(
            name.to_string(),
            egui::FontId::proportional(14.0 * self.zoom),
            Color32::WHITE,
        );
        painter.galley(
            Pos2::new(cursor_x, center_y - name_galley.size().y / 2.0),
            name_galley,
            Color32::WHITE,
        );

        // Delete button (× in top-right corner)
        let delete_button_rect = self.get_delete_button_rect(rect);

        // Retrieve hold progress for visualization (Mary StyleUX)
        let delete_id = egui::Id::new((part.id, "delete"));
        let progress = ui
            .ctx()
            .data(|d| d.get_temp::<f32>(delete_id.with("progress")))
            .unwrap_or(0.0);

        if progress > 0.0 {
            // Draw loading ring (Mary StyleUX Unified Pattern)
            use std::f32::consts::TAU;
            let center = delete_button_rect.center();
            let max_radius = (delete_button_rect.width() / 2.0) - 2.0 * self.zoom;
            let stroke_width = 2.5 * self.zoom;
            let arc_radius = max_radius - stroke_width / 2.0;

            let start_angle = -std::f32::consts::FRAC_PI_2;
            let end_angle = start_angle + progress * TAU;

            let mut points = Vec::new();
            let steps = 32;
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let angle = egui::lerp(start_angle..=end_angle, t);
                points.push(center + Vec2::new(angle.cos(), angle.sin()) * arc_radius);
            }

            painter.add(egui::Shape::line(
                points,
                Stroke::new(stroke_width, Color32::from_rgb(255, 50, 50)),
            ));
        }

        painter.text(
            delete_button_rect.center(),
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::proportional(16.0 * self.zoom),
            Color32::from_rgba_unmultiplied(255, 100, 100, 200),
        );

        // Draw property display based on part type
        let property_text = Self::get_part_property_text(&part.part_type);
        let has_property_text = !property_text.is_empty();

        if has_property_text {
            // Position at the bottom of the node to avoid overlapping sockets
            let property_y = rect.max.y - 10.0 * self.zoom;
            painter.text(
                Pos2::new(rect.center().x, property_y),
                egui::Align2::CENTER_CENTER,
                property_text,
                egui::FontId::proportional(10.0 * self.zoom),
                Color32::from_gray(180), // Slightly brighter for readability
            );
        }

        // Draw Media Playback Progress Bar
        if let mapmap_core::module::ModulePartType::Source(
            mapmap_core::module::SourceType::MediaFile { .. },
        ) = &part.part_type
        {
            if let Some(info) = self.player_info.get(&part.id) {
                let duration = info.duration.max(0.001);
                let progress = (info.current_time / duration).clamp(0.0, 1.0) as f32;
                let is_playing = info.is_playing;

                let offset_from_bottom = if has_property_text { 28.0 } else { 12.0 };
                let bar_height = 4.0 * self.zoom;
                let bar_y = rect.max.y - (offset_from_bottom * self.zoom) - bar_height;
                let bar_width = rect.width() - 20.0 * self.zoom;
                let bar_x = rect.min.x + 10.0 * self.zoom;

                // Background
                let bar_bg =
                    Rect::from_min_size(Pos2::new(bar_x, bar_y), Vec2::new(bar_width, bar_height));
                painter.rect_filled(bar_bg, 2.0 * self.zoom, Color32::from_gray(30));

                // Progress
                let progress_width = (progress * bar_width).max(2.0 * self.zoom);
                let progress_rect = Rect::from_min_size(
                    Pos2::new(bar_x, bar_y),
                    Vec2::new(progress_width, bar_height),
                );

                let color = if is_playing {
                    Color32::from_rgb(100, 255, 100) // Green
                } else {
                    Color32::from_rgb(255, 200, 50) // Yellow/Orange
                };

                painter.rect_filled(progress_rect, 2.0 * self.zoom, color);

                // Interaction (Seek)
                let interact_rect = bar_bg.expand(6.0 * self.zoom);
                let bar_response = ui.interact(
                    interact_rect,
                    ui.id().with(("seek", part.id)),
                    Sense::click_and_drag(),
                );

                if bar_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if bar_response.clicked() || bar_response.dragged() {
                    if let Some(pos) = bar_response.interact_pointer_pos() {
                        let seek_norm = ((pos.x - bar_x) / bar_width).clamp(0.0, 1.0);
                        let seek_s = seek_norm as f64 * duration;
                        actions.push(UIAction::MediaCommand(
                            part.id,
                            MediaPlaybackCommand::Seek(seek_s),
                        ));
                    }
                }
            }
        }

        // Draw audio trigger VU meter and live value display
        if is_audio_trigger {
            let offset_from_bottom = if has_property_text { 28.0 } else { 12.0 };
            let meter_height = 4.0 * self.zoom; // Thinner meter
            let meter_y = rect.max.y - (offset_from_bottom * self.zoom) - meter_height;
            let meter_width = rect.width() - 20.0 * self.zoom;
            let meter_x = rect.min.x + 10.0 * self.zoom;

            // Background bar
            let meter_bg = Rect::from_min_size(
                Pos2::new(meter_x, meter_y),
                Vec2::new(meter_width, meter_height),
            );
            painter.rect_filled(meter_bg, 2.0, Color32::from_gray(20));

            // Value bar with Hardware-Segments
            let num_segments = 20;
            let segment_spacing = 1.0 * self.zoom;
            let segment_width =
                (meter_width - (num_segments as f32 - 1.0) * segment_spacing) / num_segments as f32;

            for i in 0..num_segments {
                let t = i as f32 / num_segments as f32;
                if t > trigger_value {
                    break;
                }

                let seg_x = meter_x + i as f32 * (segment_width + segment_spacing);
                let seg_rect = Rect::from_min_size(
                    Pos2::new(seg_x, meter_y),
                    Vec2::new(segment_width, meter_height),
                );

                let seg_color = if t < 0.6 {
                    Color32::from_rgb(0, 255, 100) // Green
                } else if t < 0.85 {
                    Color32::from_rgb(255, 180, 0) // Orange
                } else {
                    Color32::from_rgb(255, 50, 50) // Red
                };

                painter.rect_filled(seg_rect, 1.0, seg_color);
            }

            // Threshold line
            let threshold_x = meter_x + threshold * meter_width;
            painter.line_segment(
                [
                    Pos2::new(threshold_x, meter_y - 2.0),
                    Pos2::new(threshold_x, meter_y + meter_height + 2.0),
                ],
                Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 50, 50, 200)),
            );
        }

        // Draw input sockets (left side)
        let socket_start_y = rect.min.y + title_height + 10.0 * self.zoom;
        for (i, socket) in part.inputs.iter().enumerate() {
            let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
            let socket_pos = Pos2::new(rect.min.x, socket_y);
            let socket_radius = 7.0 * self.zoom;

            // Socket "Port" style (dark hole with colored ring)
            let socket_color = Self::get_socket_color(&socket.socket_type);

            // Check hover
            let is_hovered = if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                socket_pos.distance(pointer_pos) < socket_radius * 1.5
            } else {
                false
            };

            // Outer ring (Socket Color)
            let ring_stroke = if is_hovered {
                let pulse = (ui.input(|i| i.time) * 10.0).sin() as f32 * 0.2 + 0.8;
                Stroke::new(3.0 * self.zoom, Color32::WHITE.linear_multiply(pulse))
            } else {
                Stroke::new(2.0 * self.zoom, socket_color)
            };
            painter.circle_stroke(socket_pos, socket_radius, ring_stroke);
            // Inner hole (Dark)
            painter.circle_filled(
                socket_pos,
                socket_radius - 2.0 * self.zoom,
                Color32::from_gray(20),
            );
            // Inner dot (Connector contact)
            painter.circle_filled(
                socket_pos,
                2.0 * self.zoom,
                if is_hovered {
                    socket_color
                } else {
                    Color32::from_gray(100)
                },
            );

            // Socket label
            painter.text(
                Pos2::new(rect.min.x + 14.0 * self.zoom, socket_y),
                egui::Align2::LEFT_CENTER,
                &socket.name,
                egui::FontId::proportional(11.0 * self.zoom),
                Color32::from_gray(230), // Brighter text
            );
        }

        // Draw output sockets (right side)
        for (i, socket) in part.outputs.iter().enumerate() {
            let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
            let socket_pos = Pos2::new(rect.max.x, socket_y);
            let socket_radius = 7.0 * self.zoom;

            // Socket "Port" style
            let socket_color = Self::get_socket_color(&socket.socket_type);

            // Check hover
            let is_hovered = if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                socket_pos.distance(pointer_pos) < socket_radius * 1.5
            } else {
                false
            };

            // Outer ring (Socket Color)
            let ring_stroke = if is_hovered {
                let pulse = (ui.input(|i| i.time) * 10.0).sin() as f32 * 0.2 + 0.8;
                Stroke::new(3.0 * self.zoom, Color32::WHITE.linear_multiply(pulse))
            } else {
                Stroke::new(2.0 * self.zoom, socket_color)
            };
            painter.circle_stroke(socket_pos, socket_radius, ring_stroke);
            // Inner hole (Dark)
            painter.circle_filled(
                socket_pos,
                socket_radius - 2.0 * self.zoom,
                Color32::from_gray(20),
            );
            // Inner dot (Connector contact)
            painter.circle_filled(
                socket_pos,
                2.0 * self.zoom,
                if is_hovered {
                    socket_color
                } else {
                    Color32::from_gray(100)
                },
            );

            // Socket label
            painter.text(
                Pos2::new(rect.max.x - 14.0 * self.zoom, socket_y),
                egui::Align2::RIGHT_CENTER,
                &socket.name,
                egui::FontId::proportional(11.0 * self.zoom),
                Color32::from_gray(230), // Brighter text
            );

            // Draw live value meter for output sockets
            if let Some(value) = self.get_socket_live_value(part, i) {
                let meter_width = 30.0 * self.zoom;
                let meter_height = 8.0 * self.zoom;
                let meter_x = rect.max.x - 12.0 * self.zoom - meter_width;

                let meter_bg = Rect::from_min_size(
                    Pos2::new(meter_x, socket_y - meter_height / 2.0),
                    Vec2::new(meter_width, meter_height),
                );
                painter.rect_filled(meter_bg, 2.0, Color32::from_gray(40));

                let value_width = (value.clamp(0.0, 1.0) * meter_width).max(1.0);
                let value_bar = Rect::from_min_size(
                    Pos2::new(meter_x, socket_y - meter_height / 2.0),
                    Vec2::new(value_width, meter_height),
                );
                painter.rect_filled(value_bar, 2.0, Color32::from_rgb(100, 180, 220));
            }
        }
    }

    fn get_part_style(
        part_type: &mapmap_core::module::ModulePartType,
    ) -> (Color32, Color32, &'static str, &'static str) {
        use mapmap_core::module::{
            BlendModeType, EffectType, MaskShape, MaskType, ModulePartType, ModulizerType,
            OutputType, SourceType, TriggerType,
        };
        match part_type {
            ModulePartType::Trigger(trigger) => {
                let name = match trigger {
                    TriggerType::AudioFFT { .. } => "Audio FFT",
                    TriggerType::Beat => "Beat",
                    TriggerType::Midi { .. } => "MIDI",
                    TriggerType::Osc { .. } => "OSC",
                    TriggerType::Shortcut { .. } => "Shortcut",
                    TriggerType::Random { .. } => "Random",
                    TriggerType::Fixed { .. } => "Fixed Timer",
                };
                (
                    Color32::from_rgb(60, 50, 70),
                    Color32::from_rgb(130, 80, 180),
                    "⚡",
                    name,
                )
            }
            ModulePartType::Source(SourceType::BevyAtmosphere { .. }) => (
                Color32::from_rgb(40, 60, 80),
                Color32::from_rgb(100, 180, 220),
                "☁️",
                "Atmosphere",
            ),
            ModulePartType::Source(SourceType::BevyHexGrid { .. }) => (
                Color32::from_rgb(40, 60, 80),
                Color32::from_rgb(100, 180, 220),
                "🛑",
                "Hex Grid",
            ),
            ModulePartType::Source(SourceType::BevyParticles { .. }) => (
                Color32::from_rgb(40, 60, 80),
                Color32::from_rgb(100, 180, 220),
                "✨",
                "Particles",
            ),
            ModulePartType::Source(source) => {
                let name = match source {
                    SourceType::MediaFile { .. } => "Media File",
                    SourceType::Shader { .. } => "Shader",
                    SourceType::LiveInput { .. } => "Live Input",
                    SourceType::NdiInput { .. } => "NDI Input",
                    #[cfg(target_os = "windows")]
                    SourceType::SpoutInput { .. } => "Spout Input",
                    SourceType::VideoUni { .. } => "Video (Uni)",
                    SourceType::ImageUni { .. } => "Image (Uni)",
                    SourceType::VideoMulti { .. } => "Video (Multi)",
                    SourceType::ImageMulti { .. } => "Image (Multi)",
                    SourceType::Bevy => "Bevy Scene",
                    SourceType::BevyAtmosphere { .. } => "Atmosphere",
                    SourceType::BevyHexGrid { .. } => "Hex Grid",
                    SourceType::BevyParticles { .. } => "Particles",
                };
                (
                    Color32::from_rgb(50, 60, 70),
                    Color32::from_rgb(80, 140, 180),
                    "🎬",
                    name,
                )
            }

            ModulePartType::Mask(mask) => {
                let name = match mask {
                    MaskType::File { .. } => "File Mask",
                    MaskType::Shape(shape) => match shape {
                        MaskShape::Circle => "Circle",
                        MaskShape::Rectangle => "Rectangle",
                        MaskShape::Triangle => "Triangle",
                        MaskShape::Star => "Star",
                        MaskShape::Ellipse => "Ellipse",
                    },
                    MaskType::Gradient { .. } => "Gradient",
                };
                (
                    Color32::from_rgb(60, 55, 70),
                    Color32::from_rgb(160, 100, 180),
                    "🎭",
                    name,
                )
            }
            ModulePartType::Modulizer(mod_type) => {
                let name = match mod_type {
                    ModulizerType::Effect {
                        effect_type: effect,
                        ..
                    } => match effect {
                        EffectType::Blur => "Blur",
                        EffectType::Sharpen => "Sharpen",
                        EffectType::Invert => "Invert",
                        EffectType::Threshold => "Threshold",
                        EffectType::Brightness => "Brightness",
                        EffectType::Contrast => "Contrast",
                        EffectType::Saturation => "Saturation",
                        EffectType::HueShift => "Hue Shift",
                        EffectType::Colorize => "Colorize",
                        EffectType::Wave => "Wave",
                        EffectType::Spiral => "Spiral",
                        EffectType::Pinch => "Pinch",
                        EffectType::Mirror => "Mirror",
                        EffectType::Kaleidoscope => "Kaleidoscope",
                        EffectType::Pixelate => "Pixelate",
                        EffectType::Halftone => "Halftone",
                        EffectType::EdgeDetect => "Edge Detect",
                        EffectType::Posterize => "Posterize",
                        EffectType::Glitch => "Glitch",
                        EffectType::RgbSplit => "RGB Split",
                        EffectType::ChromaticAberration => "Chromatic",
                        EffectType::VHS => "VHS",
                        EffectType::FilmGrain => "Film Grain",
                        EffectType::Vignette => "Vignette",
                        EffectType::ShaderGraph(_) => "Custom Graph",
                    },
                    ModulizerType::BlendMode(blend) => match blend {
                        BlendModeType::Normal => "Normal",
                        BlendModeType::Add => "Add",
                        BlendModeType::Multiply => "Multiply",
                        BlendModeType::Screen => "Screen",
                        BlendModeType::Overlay => "Overlay",
                        BlendModeType::Difference => "Difference",
                        BlendModeType::Exclusion => "Exclusion",
                    },
                    ModulizerType::AudioReactive { .. } => "Audio Reactive",
                };
                (
                    egui::Color32::from_rgb(60, 60, 50),
                    egui::Color32::from_rgb(180, 140, 60),
                    "〰️",
                    name,
                )
            }
            ModulePartType::Mesh(_) => (
                egui::Color32::from_rgb(60, 60, 80),
                egui::Color32::from_rgb(100, 100, 200),
                "🕸️",
                "Mesh",
            ),
            ModulePartType::Layer(layer) => {
                let name = match layer {
                    LayerType::Single { .. } => "Single Layer",
                    LayerType::Group { .. } => "Layer Group",
                    LayerType::All { .. } => "All Layers",
                };
                (
                    Color32::from_rgb(50, 70, 60),
                    Color32::from_rgb(80, 180, 120),
                    "📑",
                    name,
                )
            }
            ModulePartType::Output(output) => {
                let name = match output {
                    OutputType::Projector { .. } => "Projector",
                    OutputType::NdiOutput { .. } => "NDI Output",
                    #[cfg(target_os = "windows")]
                    OutputType::Spout { .. } => "Spout Output",
                    OutputType::Hue { .. } => "Philips Hue",
                };
                (
                    Color32::from_rgb(70, 50, 50),
                    Color32::from_rgb(180, 80, 80),
                    "📺",
                    name,
                )
            }
            ModulePartType::Hue(hue) => {
                let name = match hue {
                    mapmap_core::module::HueNodeType::SingleLamp { .. } => "Single Lamp",
                    mapmap_core::module::HueNodeType::MultiLamp { .. } => "Multi Lamp",
                    mapmap_core::module::HueNodeType::EntertainmentGroup { .. } => {
                        "Entertainment Group"
                    }
                };
                (
                    Color32::from_rgb(60, 60, 40),
                    Color32::from_rgb(200, 200, 100),
                    "💡",
                    name,
                )
            }
        }
    }

    /// Returns the category name for a module part type
    fn get_part_category(part_type: &mapmap_core::module::ModulePartType) -> &'static str {
        use mapmap_core::module::ModulePartType;
        match part_type {
            ModulePartType::Trigger(_) => "Trigger",
            ModulePartType::Source(_) => "Source",
            ModulePartType::Mask(_) => "Mask",
            ModulePartType::Modulizer(_) => "Modulator",
            ModulePartType::Mesh(_) => "Mesh",
            ModulePartType::Layer(_) => "Layer",
            ModulePartType::Output(_) => "Output",
            ModulePartType::Hue(_) => "Hue",
        }
    }

    fn get_socket_color(socket_type: &mapmap_core::module::ModuleSocketType) -> Color32 {
        use mapmap_core::module::ModuleSocketType;
        match socket_type {
            ModuleSocketType::Trigger => Color32::from_rgb(180, 100, 220),
            ModuleSocketType::Media => Color32::from_rgb(100, 180, 220),
            ModuleSocketType::Effect => Color32::from_rgb(220, 180, 100),
            ModuleSocketType::Layer => Color32::from_rgb(100, 220, 140),
            ModuleSocketType::Output => Color32::from_rgb(220, 100, 100),
            ModuleSocketType::Link => Color32::from_rgb(200, 200, 200),
        }
    }

    fn get_part_property_text(part_type: &mapmap_core::module::ModulePartType) -> String {
        use mapmap_core::module::{
            MaskType, ModulePartType, ModulizerType, OutputType, SourceType, TriggerType,
        };
        match part_type {
            ModulePartType::Trigger(trigger_type) => match trigger_type {
                TriggerType::AudioFFT { band, .. } => format!("🔊 Audio: {:?}", band),
                TriggerType::Random { .. } => "🎲 Random".to_string(),
                TriggerType::Fixed { interval_ms, .. } => format!("⏱️ {}ms", interval_ms),
                TriggerType::Midi { channel, note, .. } => format!("🎹 Ch{} N{}", channel, note),
                TriggerType::Osc { address } => format!("📡 {}", address),
                TriggerType::Shortcut { key_code, .. } => format!("⌨️ {}", key_code),
                TriggerType::Beat => "🥁 Beat".to_string(),
            },
            ModulePartType::Source(source_type) => match source_type {
                SourceType::MediaFile { path, .. } => {
                    if path.is_empty() {
                        "📁 Select file...".to_string()
                    } else {
                        format!("📁 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                    }
                }
                SourceType::Shader { name, .. } => format!("🎨 {}", name),
                SourceType::LiveInput { device_id } => format!("📹 Device {}", device_id),
                SourceType::NdiInput { source_name } => {
                    format!("📡 {}", source_name.as_deref().unwrap_or("None"))
                }
                SourceType::Bevy => "🎮 Bevy Scene".to_string(),
                #[cfg(target_os = "windows")]
                SourceType::SpoutInput { sender_name } => format!("🚰 {}", sender_name),
                SourceType::VideoUni { path, .. } => {
                    if path.is_empty() {
                        "📁 Select video...".to_string()
                    } else {
                        format!("📹 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                    }
                }
                SourceType::ImageUni { path, .. } => {
                    if path.is_empty() {
                        "🖼 Select image...".to_string()
                    } else {
                        format!("🖼 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                    }
                }
                SourceType::VideoMulti { shared_id, .. } => format!("📹 Shared: {}", shared_id),
                SourceType::ImageMulti { shared_id, .. } => format!("🖼 Shared: {}", shared_id),
                SourceType::BevyAtmosphere { .. } => "☁️ Atmosphere".to_string(),
                SourceType::BevyHexGrid { .. } => "🛑 Hex Grid".to_string(),
                SourceType::BevyParticles { .. } => "✨ Particles".to_string(),
            },
            ModulePartType::Mask(mask_type) => match mask_type {
                MaskType::File { path } => {
                    if path.is_empty() {
                        "📁 Select mask...".to_string()
                    } else {
                        format!("📁 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                    }
                }
                MaskType::Shape(shape) => format!("🔷 {:?}", shape),
                MaskType::Gradient { angle, .. } => format!("🌈 Gradient {}°", *angle as i32),
            },
            ModulePartType::Modulizer(modulizer_type) => match modulizer_type {
                ModulizerType::Effect {
                    effect_type: effect,
                    ..
                } => format!("✨ {}", effect.name()),
                ModulizerType::BlendMode(blend) => format!("🔀 {}", blend.name()),
                ModulizerType::AudioReactive { source } => format!("🔊 {}", source),
            },
            ModulePartType::Mesh(_) => "🕸️ Mesh".to_string(),
            ModulePartType::Layer(layer_type) => {
                use mapmap_core::module::LayerType;
                match layer_type {
                    LayerType::Single { name, .. } => format!("📑 {}", name),
                    LayerType::Group { name, .. } => format!("📁 {}", name),
                    LayerType::All { .. } => "📑 All Layers".to_string(),
                }
            }
            ModulePartType::Output(output_type) => match output_type {
                OutputType::Projector { name, .. } => format!("📺 {}", name),
                OutputType::NdiOutput { name } => format!("📡 {}", name),
                #[cfg(target_os = "windows")]
                OutputType::Spout { name } => format!("🚰 {}", name),
                OutputType::Hue { bridge_ip, .. } => {
                    if bridge_ip.is_empty() {
                        "💡 Not Connected".to_string()
                    } else {
                        format!("💡 {}", bridge_ip)
                    }
                }
            },
            ModulePartType::Hue(hue) => match hue {
                mapmap_core::module::HueNodeType::SingleLamp { name, .. } => format!("💡 {}", name),
                mapmap_core::module::HueNodeType::MultiLamp { name, .. } => {
                    format!("💡💡 {}", name)
                }
                mapmap_core::module::HueNodeType::EntertainmentGroup { name, .. } => {
                    format!("🎭 {}", name)
                }
            },
        }
    }

    /// Render the diagnostics popup window
    fn render_diagnostics_popup(&mut self, ui: &mut Ui) {
        if !self.show_diagnostics {
            return;
        }

        let popup_size = Vec2::new(350.0, 250.0);
        let available = ui.available_rect_before_wrap();
        let popup_pos = Pos2::new(
            (available.min.x + available.max.x - popup_size.x) / 2.0,
            (available.min.y + available.max.y - popup_size.y) / 2.0,
        );
        let popup_rect = egui::Rect::from_min_size(popup_pos, popup_size);

        // Background
        let painter = ui.painter();
        painter.rect_filled(
            popup_rect,
            0.0,
            Color32::from_rgba_unmultiplied(30, 35, 45, 245),
        );
        painter.rect_stroke(
            popup_rect,
            0,
            Stroke::new(2.0, Color32::from_rgb(180, 100, 80)),
            egui::StrokeKind::Inside,
        );

        let inner_rect = popup_rect.shrink(12.0);
        ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
            ui.vertical(|ui| {
                ui.heading(if self.diagnostic_issues.is_empty() {
                    "✓ Module Check: OK"
                } else {
                    "⚠ Module Check: Issues Found"
                });
                ui.add_space(8.0);

                if self.diagnostic_issues.is_empty() {
                    ui.label("No issues found. Your module looks good!");
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for issue in &self.diagnostic_issues {
                                let (icon, color) = match issue.severity {
                                    mapmap_core::diagnostics::IssueSeverity::Error => {
                                        ("❌", Color32::RED)
                                    }
                                    mapmap_core::diagnostics::IssueSeverity::Warning => {
                                        ("⚠", Color32::YELLOW)
                                    }
                                    mapmap_core::diagnostics::IssueSeverity::Info => {
                                        ("ℹ", Color32::LIGHT_BLUE)
                                    }
                                };
                                ui.horizontal(|ui| {
                                    ui.colored_label(color, icon);
                                    ui.label(&issue.message);
                                });
                            }
                        });
                }

                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    self.show_diagnostics = false;
                }
            });
        });
    }

    /// Convert ModulePartType back to PartType for add_part
    fn part_type_from_module_part_type(
        mpt: &mapmap_core::module::ModulePartType,
    ) -> mapmap_core::module::PartType {
        use mapmap_core::module::{ModulePartType, PartType};
        match mpt {
            ModulePartType::Trigger(_) => PartType::Trigger,
            ModulePartType::Source(_) => PartType::Source,
            ModulePartType::Mask(_) => PartType::Mask,
            ModulePartType::Modulizer(_) => PartType::Modulator,
            ModulePartType::Mesh(_) => PartType::Mesh,
            ModulePartType::Layer(_) => PartType::Layer,
            ModulePartType::Output(_) => PartType::Output,
            ModulePartType::Hue(_) => PartType::Hue,
        }
    }

    /// Auto-layout parts in a grid by type (left to right: Trigger → Source → Mask → Modulator → Layer → Output)
    fn auto_layout_parts(parts: &mut [mapmap_core::module::ModulePart]) {
        use mapmap_core::module::ModulePartType;

        // Sort parts by type category for left-to-right flow
        let type_order = |pt: &ModulePartType| -> usize {
            match pt {
                ModulePartType::Trigger(_) => 0,
                ModulePartType::Source(_) => 1,
                ModulePartType::Mask(_) => 2,
                ModulePartType::Modulizer(_) => 3,
                ModulePartType::Mesh(_) => 4,
                ModulePartType::Layer(_) => 5,
                ModulePartType::Output(_) => 6,
                ModulePartType::Hue(_) => 7,
            }
        };

        // Group parts by type
        let mut columns: [Vec<usize>; 8] = Default::default();
        for (i, part) in parts.iter().enumerate() {
            let col = type_order(&part.part_type);
            columns[col].push(i);
        }

        // Layout parameters - increased spacing for better visibility
        let node_width = 200.0;
        let node_height = 120.0;
        let h_spacing = 100.0; // Increased from 50
        let v_spacing = 60.0; // Increased from 30
        let start_x = 50.0;
        let start_y = 50.0;

        // Position each column
        let mut x = start_x;
        for col in &columns {
            if col.is_empty() {
                continue;
            }

            let mut y = start_y;
            for &part_idx in col {
                parts[part_idx].position = (x, y);
                y += node_height + v_spacing;
            }

            x += node_width + h_spacing;
        }
    }

    /// Find a free position for a new node, avoiding overlaps with existing nodes
    fn find_free_position(
        parts: &[mapmap_core::module::ModulePart],
        preferred: (f32, f32),
    ) -> (f32, f32) {
        let node_width = 200.0;
        let node_height = 130.0;
        let grid_step = 30.0;

        let mut pos = preferred;
        let mut attempts = 0;

        loop {
            let new_rect =
                Rect::from_min_size(Pos2::new(pos.0, pos.1), Vec2::new(node_width, node_height));

            let has_collision = parts.iter().any(|part| {
                let part_height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                let part_rect = Rect::from_min_size(
                    Pos2::new(part.position.0, part.position.1),
                    Vec2::new(node_width, part_height),
                );
                new_rect.intersects(part_rect)
            });

            if !has_collision {
                return pos;
            }

            // Try different positions in a spiral pattern
            attempts += 1;
            if attempts > 100 {
                // Give up after 100 attempts, just offset significantly
                return (preferred.0, preferred.1 + (parts.len() as f32) * 150.0);
            }

            // Move down first, then right
            pos.1 += grid_step;
            if pos.1 > preferred.1 + 500.0 {
                pos.1 = preferred.1;
                pos.0 += node_width + 20.0;
            }
        }
    }

    /// Create default presets/templates
    fn default_presets() -> Vec<ModulePreset> {
        use mapmap_core::module::*;

        vec![
            ModulePreset {
                name: "Simple Media Chain".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::Beat),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::new_media_file(String::new())),
                        (350.0, 100.0), // Increased from 250 to 350
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::Projector {
                            id: 1,
                            name: "Projector 1".to_string(),

                            hide_cursor: true,
                            target_screen: 0,
                            show_in_preview_panel: true,
                            extra_preview_window: false,
                            output_width: 0,
                            output_height: 0,
                            output_fps: 60.0,
                        }),
                        (650.0, 100.0), // Increased from 450 to 650
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Trigger -> Source
                    (1, 0, 2, 0), // Source -> Output (NEW - was missing!)
                ],
            },
            ModulePreset {
                name: "Effect Chain".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::Beat),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::new_media_file(String::new())),
                        (350.0, 100.0), // Increased spacing
                        None,
                    ),
                    (
                        ModulePartType::Modulizer(ModulizerType::Effect {
                            effect_type: EffectType::Blur,
                            params: std::collections::HashMap::new(),
                        }),
                        (650.0, 100.0), // Increased spacing
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::Projector {
                            id: 1,
                            name: "Projector 1".to_string(),

                            hide_cursor: true,
                            target_screen: 0,
                            show_in_preview_panel: true,
                            extra_preview_window: false,
                            output_width: 0,
                            output_height: 0,
                            output_fps: 60.0,
                        }),
                        (950.0, 100.0), // Increased spacing
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Trigger -> Source
                    (1, 0, 2, 0), // Source -> Effect
                    (2, 0, 3, 0), // Effect -> Output (NEW - was missing!)
                ],
            },
            ModulePreset {
                name: "Audio Reactive".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::AudioFFT {
                            band: AudioBand::Bass,
                            threshold: 0.5,
                            output_config: AudioTriggerOutputConfig::default(),
                        }),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::new_media_file(String::new())),
                        (350.0, 100.0), // Increased spacing
                        None,
                    ),
                    (
                        ModulePartType::Modulizer(ModulizerType::Effect {
                            effect_type: EffectType::Glitch,
                            params: std::collections::HashMap::new(),
                        }),
                        (650.0, 100.0), // Increased spacing
                        None,
                    ),
                    (
                        ModulePartType::Layer(LayerType::All {
                            opacity: 1.0,
                            blend_mode: None,
                        }),
                        (950.0, 100.0), // Increased spacing
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::Projector {
                            id: 1,
                            name: "Projector 1".to_string(),

                            hide_cursor: true,
                            target_screen: 0,
                            show_in_preview_panel: true,
                            extra_preview_window: false,
                            output_width: 0,
                            output_height: 0,
                            output_fps: 60.0,
                        }),
                        (1250.0, 100.0), // Increased spacing
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Audio -> Source
                    (1, 0, 2, 0), // Source -> Effect
                    (2, 0, 3, 0), // Effect -> Layer
                    (3, 0, 4, 0), // Layer -> Output (NEW - was missing!)
                ],
            },
            ModulePreset {
                name: "Masked Media".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::Beat),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::new_media_file(String::new())),
                        (350.0, 100.0), // Increased spacing
                        None,
                    ),
                    (
                        ModulePartType::Mask(MaskType::Shape(MaskShape::Circle)),
                        (650.0, 100.0), // Increased spacing
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::Projector {
                            id: 1,
                            name: "Projector 1".to_string(),

                            hide_cursor: true,
                            target_screen: 0,
                            show_in_preview_panel: true,
                            extra_preview_window: false,
                            output_width: 0,
                            output_height: 0,
                            output_fps: 60.0,
                        }),
                        (950.0, 100.0), // Increased spacing
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Trigger -> Source
                    (1, 0, 2, 0), // Source -> Mask
                    (2, 0, 3, 0), // Mask -> Output (NEW - was missing!)
                ],
            },
            // NDI Source Preset
            ModulePreset {
                name: "NDI Source".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::Beat),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::NdiInput { source_name: None }),
                        (350.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::Projector {
                            id: 1,
                            name: "Projector 1".to_string(),

                            hide_cursor: true,
                            target_screen: 0,
                            show_in_preview_panel: true,
                            extra_preview_window: false,
                            output_width: 0,
                            output_height: 0,
                            output_fps: 60.0,
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Trigger -> NDI Source
                    (1, 0, 2, 0), // NDI Source -> Output
                ],
            },
            // NDI Output Preset
            ModulePreset {
                name: "NDI Output".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::Beat),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::new_media_file(String::new())),
                        (350.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::NdiOutput {
                            name: "MapFlow NDI".to_string(),
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Trigger -> Source
                    (1, 0, 2, 0), // Source -> NDI Output
                ],
            },
            // Spout Source (Windows only)
            #[cfg(target_os = "windows")]
            ModulePreset {
                name: "Spout Source".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::Beat),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::SpoutInput {
                            sender_name: String::new(),
                        }),
                        (350.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::Projector {
                            id: 1,
                            name: "Projector 1".to_string(),

                            hide_cursor: true,
                            target_screen: 0,
                            show_in_preview_panel: true,
                            extra_preview_window: false,
                            output_width: 0,
                            output_height: 0,
                            output_fps: 60.0,
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Trigger -> Spout Source
                    (1, 0, 2, 0), // Spout Source -> Output
                ],
            },
            // Spout Output (Windows only)
            #[cfg(target_os = "windows")]
            ModulePreset {
                name: "Spout Output".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::Beat),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::new_media_file(String::new())),
                        (350.0, 100.0),
                        None,
                    ),
                    #[cfg(target_os = "windows")]
                    (
                        ModulePartType::Output(OutputType::Spout {
                            name: "MapFlow Spout".to_string(),
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Trigger -> Source
                    (1, 0, 2, 0), // Source -> Spout Output
                ],
            },
        ]
    }
}

impl ModuleCanvas {
    fn render_trigger_config_ui(
        &mut self,
        ui: &mut egui::Ui,
        part: &mut mapmap_core::module::ModulePart,
    ) {
        // Only show for parts with input sockets
        if part.inputs.is_empty() {
            return;
        }

        ui.add_space(5.0);
        egui::CollapsingHeader::new("⚡ Trigger & Automation")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("MIDI Assignment:");
                    let is_learning = self.midi_learn_part_id == Some(part.id);
                    let btn_text = if is_learning {
                        "🛑 Stop Learning"
                    } else {
                        "🎹 MIDI Learn"
                    };
                    if ui.selectable_label(is_learning, btn_text).clicked() {
                        if is_learning {
                            self.midi_learn_part_id = None;
                        } else {
                            self.midi_learn_part_id = Some(part.id);
                        }
                    }
                });

                ui.separator();

                // Iterate over inputs
                for (idx, socket) in part.inputs.iter().enumerate() {
                    ui.push_id(idx, |ui| {
                        ui.separator();
                        ui.label(format!("Input {}: {}", idx, socket.name));

                        // Get config
                        let mut config = part.trigger_targets.entry(idx).or_default().clone();
                        let original_config = config.clone();

                        // Target Selector
                        egui::ComboBox::from_id_salt("target")
                            .selected_text(format!("{:?}", config.target))
                            .show_ui(ui, |ui| {
                                use mapmap_core::module::TriggerTarget;
                                ui.selectable_value(
                                    &mut config.target,
                                    TriggerTarget::None,
                                    "None",
                                );
                                ui.selectable_value(
                                    &mut config.target,
                                    TriggerTarget::Opacity,
                                    "Opacity",
                                );
                                ui.selectable_value(
                                    &mut config.target,
                                    TriggerTarget::Brightness,
                                    "Brightness",
                                );
                                ui.selectable_value(
                                    &mut config.target,
                                    TriggerTarget::Contrast,
                                    "Contrast",
                                );
                                ui.selectable_value(
                                    &mut config.target,
                                    TriggerTarget::Saturation,
                                    "Saturation",
                                );
                                ui.selectable_value(
                                    &mut config.target,
                                    TriggerTarget::HueShift,
                                    "Hue Shift",
                                );
                                ui.selectable_value(
                                    &mut config.target,
                                    TriggerTarget::ScaleX,
                                    "Scale X",
                                );
                                ui.selectable_value(
                                    &mut config.target,
                                    TriggerTarget::ScaleY,
                                    "Scale Y",
                                );
                                ui.selectable_value(
                                    &mut config.target,
                                    TriggerTarget::Rotation,
                                    "Rotation",
                                );
                            });

                        // Only show options if target is not None
                        if config.target != mapmap_core::module::TriggerTarget::None {
                            // Mode Selector
                            ui.horizontal(|ui| {
                                ui.label("Mode:");
                                // Helper to display mode name without fields
                                let mode_name = match config.mode {
                                    mapmap_core::module::TriggerMappingMode::Direct => "Direct",
                                    mapmap_core::module::TriggerMappingMode::Fixed => "Fixed",
                                    mapmap_core::module::TriggerMappingMode::RandomInRange => {
                                        "Random"
                                    }
                                    mapmap_core::module::TriggerMappingMode::Smoothed {
                                        ..
                                    } => "Smoothed",
                                };

                                egui::ComboBox::from_id_salt("mode")
                                    .selected_text(mode_name)
                                    .show_ui(ui, |ui| {
                                        use mapmap_core::module::TriggerMappingMode;
                                        ui.selectable_value(
                                            &mut config.mode,
                                            TriggerMappingMode::Direct,
                                            "Direct",
                                        );
                                        ui.selectable_value(
                                            &mut config.mode,
                                            TriggerMappingMode::Fixed,
                                            "Fixed",
                                        );
                                        ui.selectable_value(
                                            &mut config.mode,
                                            TriggerMappingMode::RandomInRange,
                                            "Random",
                                        );
                                        // For smoothed, we preserve existing params if already smoothed, else default
                                        let default_smoothed = TriggerMappingMode::Smoothed {
                                            attack: 0.1,
                                            release: 0.1,
                                        };
                                        ui.selectable_value(
                                            &mut config.mode,
                                            default_smoothed,
                                            "Smoothed",
                                        );
                                    });
                            });

                            // Params based on Mode
                            match &mut config.mode {
                                mapmap_core::module::TriggerMappingMode::Fixed => {
                                    ui.horizontal(|ui| {
                                        ui.label("Threshold:");
                                        ui.add(egui::Slider::new(&mut config.threshold, 0.0..=1.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Off:");
                                        ui.add(egui::Slider::new(
                                            &mut config.min_value,
                                            -5.0..=5.0,
                                        ));
                                        ui.label("On:");
                                        ui.add(egui::Slider::new(
                                            &mut config.max_value,
                                            -5.0..=5.0,
                                        ));
                                    });
                                }
                                mapmap_core::module::TriggerMappingMode::RandomInRange => {
                                    ui.horizontal(|ui| {
                                        ui.label("Range:");
                                        ui.add(
                                            egui::Slider::new(&mut config.min_value, -5.0..=5.0)
                                                .text("Min"),
                                        );
                                        ui.add(
                                            egui::Slider::new(&mut config.max_value, -5.0..=5.0)
                                                .text("Max"),
                                        );
                                    });
                                }
                                mapmap_core::module::TriggerMappingMode::Smoothed {
                                    attack,
                                    release,
                                } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Range:");
                                        ui.add(
                                            egui::Slider::new(&mut config.min_value, -5.0..=5.0)
                                                .text("Min"),
                                        );
                                        ui.add(
                                            egui::Slider::new(&mut config.max_value, -5.0..=5.0)
                                                .text("Max"),
                                        );
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Attack:");
                                        ui.add(egui::Slider::new(attack, 0.0..=2.0).text("s"));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Release:");
                                        ui.add(egui::Slider::new(release, 0.0..=2.0).text("s"));
                                    });
                                }
                                _ => {
                                    // Direct
                                    ui.horizontal(|ui| {
                                        ui.label("Range:");
                                        ui.add(
                                            egui::Slider::new(&mut config.min_value, -5.0..=5.0)
                                                .text("Min"),
                                        );
                                        ui.add(
                                            egui::Slider::new(&mut config.max_value, -5.0..=5.0)
                                                .text("Max"),
                                        );
                                    });
                                }
                            }

                            ui.checkbox(&mut config.invert, "Invert Input");
                        }

                        // Save back if changed
                        if config != original_config {
                            part.trigger_targets.insert(idx, config);
                        }
                    });
                }
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn render_common_controls(
        ui: &mut Ui,
        opacity: &mut f32,
        blend_mode: &mut Option<BlendModeType>,
        brightness: &mut f32,
        contrast: &mut f32,
        saturation: &mut f32,
        hue_shift: &mut f32,
        scale_x: &mut f32,
        scale_y: &mut f32,
        rotation: &mut f32,
        offset_x: &mut f32,
        offset_y: &mut f32,
        flip_horizontal: &mut bool,
        flip_vertical: &mut bool,
    ) {
        // === APPEARANCE ===
        ui.collapsing("🎨 Appearance", |ui| {
            egui::Grid::new("appearance_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Opacity:");
                    ui.add(egui::Slider::new(opacity, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Blend Mode:");
                    egui::ComboBox::from_id_salt("blend_mode_selector")
                        .selected_text(match blend_mode {
                            Some(BlendModeType::Normal) => "Normal",
                            Some(BlendModeType::Add) => "Add",
                            Some(BlendModeType::Multiply) => "Multiply",
                            Some(BlendModeType::Screen) => "Screen",
                            Some(BlendModeType::Overlay) => "Overlay",
                            Some(BlendModeType::Difference) => "Difference",
                            Some(BlendModeType::Exclusion) => "Exclusion",
                            None => "Normal",
                        })
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(blend_mode.is_none(), "Normal")
                                .clicked()
                            {
                                *blend_mode = None;
                            }
                            if ui
                                .selectable_label(*blend_mode == Some(BlendModeType::Add), "Add")
                                .clicked()
                            {
                                *blend_mode = Some(BlendModeType::Add);
                            }
                            if ui
                                .selectable_label(
                                    *blend_mode == Some(BlendModeType::Multiply),
                                    "Multiply",
                                )
                                .clicked()
                            {
                                *blend_mode = Some(BlendModeType::Multiply);
                            }
                            if ui
                                .selectable_label(
                                    *blend_mode == Some(BlendModeType::Screen),
                                    "Screen",
                                )
                                .clicked()
                            {
                                *blend_mode = Some(BlendModeType::Screen);
                            }
                            if ui
                                .selectable_label(
                                    *blend_mode == Some(BlendModeType::Overlay),
                                    "Overlay",
                                )
                                .clicked()
                            {
                                *blend_mode = Some(BlendModeType::Overlay);
                            }
                            if ui
                                .selectable_label(
                                    *blend_mode == Some(BlendModeType::Difference),
                                    "Difference",
                                )
                                .clicked()
                            {
                                *blend_mode = Some(BlendModeType::Difference);
                            }
                            if ui
                                .selectable_label(
                                    *blend_mode == Some(BlendModeType::Exclusion),
                                    "Exclusion",
                                )
                                .clicked()
                            {
                                *blend_mode = Some(BlendModeType::Exclusion);
                            }
                        });
                    ui.end_row();
                });
        });

        // === COLOR CORRECTION ===
        if crate::widgets::collapsing_header_with_reset(ui, "🌈 Color Correction", false, |ui| {
            egui::Grid::new("color_correction_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Brightness:");
                    ui.add(egui::Slider::new(brightness, -1.0..=1.0));
                    ui.end_row();

                    ui.label("Contrast:");
                    ui.add(egui::Slider::new(contrast, 0.0..=2.0));
                    ui.end_row();

                    ui.label("Saturation:");
                    ui.add(egui::Slider::new(saturation, 0.0..=2.0));
                    ui.end_row();

                    ui.label("Hue Shift:");
                    ui.add(egui::Slider::new(hue_shift, -180.0..=180.0).suffix("°"));
                    ui.end_row();
                });
        }) {
            *brightness = 0.0;
            *contrast = 1.0;
            *saturation = 1.0;
            *hue_shift = 0.0;
        }

        // === TRANSFORM ===
        if crate::widgets::collapsing_header_with_reset(ui, "📐 Transform", false, |ui| {
            egui::Grid::new("transform_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Scale:");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(scale_x).speed(0.01).prefix("X: "));
                        ui.add(egui::DragValue::new(scale_y).speed(0.01).prefix("Y: "));
                    });
                    ui.end_row();

                    ui.label("Offset:");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(offset_x).speed(1.0).prefix("X: "));
                        ui.add(egui::DragValue::new(offset_y).speed(1.0).prefix("Y: "));
                    });
                    ui.end_row();

                    ui.label("Rotation:");
                    ui.add(egui::Slider::new(rotation, -180.0..=180.0).suffix("°"));
                    ui.end_row();

                    ui.label("Mirror:");
                    ui.horizontal(|ui| {
                        ui.checkbox(flip_horizontal, "X");
                        ui.checkbox(flip_vertical, "Y");
                    });
                    ui.end_row();
                });
        }) {
            *scale_x = 1.0;
            *scale_y = 1.0;
            *rotation = 0.0;
            *offset_x = 0.0;
            *offset_y = 0.0;
            *flip_horizontal = false;
            *flip_vertical = false;
        }
    }

    fn render_transport_controls(
        &mut self,
        ui: &mut Ui,
        part_id: ModulePartId,
        is_playing: bool,
        current_pos: f32,
        loop_enabled: &mut bool,
        reverse_playback: &mut bool,
    ) {
        // 2. CONSOLIDATED TRANSPORT BAR (UX Improved)
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing.x = 8.0;
            let button_height = 42.0;
            let big_btn_size = Vec2::new(70.0, button_height);
            let small_btn_size = Vec2::new(40.0, button_height);

            // PLAY (Primary Action - Green)
            let play_btn = egui::Button::new(egui::RichText::new("▶").size(24.0))
                .min_size(big_btn_size)
                .fill(if is_playing {
                    Color32::from_rgb(40, 180, 60)
                } else {
                    Color32::from_gray(50)
                });
            if ui.add(play_btn).on_hover_text("Play").clicked() {
                self.pending_playback_commands
                    .push((part_id, MediaPlaybackCommand::Play));
            }

            // PAUSE (Secondary Action - Yellow)
            let pause_btn = egui::Button::new(egui::RichText::new("⏸").size(24.0))
                .min_size(big_btn_size)
                .fill(if !is_playing && current_pos > 0.1 {
                    Color32::from_rgb(200, 160, 40)
                } else {
                    Color32::from_gray(50)
                });
            if ui.add(pause_btn).on_hover_text("Pause").clicked() {
                self.pending_playback_commands
                    .push((part_id, MediaPlaybackCommand::Pause));
            }

            // Safety Spacer
            ui.add_space(24.0);
            ui.separator();
            ui.add_space(8.0);

            // STOP (Destructive Action - Separated)
            // Mary StyleUX: Use hold-to-confirm for safety
            if crate::widgets::hold_to_action_icon(ui, "⏹", Color32::from_rgb(255, 80, 80)) {
                self.pending_playback_commands
                    .push((part_id, MediaPlaybackCommand::Stop));
            }

            // LOOP
            let loop_color = if *loop_enabled {
                Color32::from_rgb(80, 150, 255)
            } else {
                Color32::from_gray(45)
            };
            if ui
                .add(
                    egui::Button::new(egui::RichText::new("🔁").size(18.0))
                        .min_size(small_btn_size)
                        .fill(loop_color),
                )
                .on_hover_text("Toggle Loop")
                .clicked()
            {
                *loop_enabled = !*loop_enabled;
                self.pending_playback_commands
                    .push((part_id, MediaPlaybackCommand::SetLoop(*loop_enabled)));
            }

            // REVERSE
            let rev_color = if *reverse_playback {
                Color32::from_rgb(200, 80, 80)
            } else {
                Color32::from_gray(45)
            };
            if ui
                .add(
                    egui::Button::new(egui::RichText::new("⏪").size(18.0))
                        .min_size(small_btn_size)
                        .fill(rev_color),
                )
                .on_hover_text("Toggle Reverse Playback")
                .clicked()
            {
                *reverse_playback = !*reverse_playback;
            }
        });
    }

    fn render_timeline(
        &mut self,
        ui: &mut Ui,
        part_id: ModulePartId,
        video_duration: f32,
        current_pos: f32,
        start_time: &mut f32,
        end_time: &mut f32,
    ) {
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), 32.0),
            Sense::click_and_drag(),
        );
        let rect = response.rect;

        // Background (Full Track)
        painter.rect_filled(rect, 0.0, Color32::from_gray(30));
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0 * self.zoom, Color32::from_gray(60)),
            egui::StrokeKind::Inside,
        );

        // Data normalization
        let effective_end = if *end_time > 0.0 {
            *end_time
        } else {
            video_duration
        };
        let start_x = rect.min.x + (*start_time / video_duration).clamp(0.0, 1.0) * rect.width();
        let end_x = rect.min.x + (effective_end / video_duration).clamp(0.0, 1.0) * rect.width();

        // Active Region Highlight
        let region_rect =
            Rect::from_min_max(Pos2::new(start_x, rect.min.y), Pos2::new(end_x, rect.max.y));
        painter.rect_filled(
            region_rect,
            0.0,
            Color32::from_rgba_unmultiplied(60, 180, 100, 80),
        );
        painter.rect_stroke(
            region_rect,
            0.0,
            Stroke::new(1.0, Color32::from_rgb(60, 180, 100)),
            egui::StrokeKind::Inside,
        );

        // INTERACTION LOGIC
        let mut handled = false;

        // 1. Handles (Prioritize resizing)
        let handle_width = 8.0;
        let start_handle_rect = Rect::from_center_size(
            Pos2::new(start_x, rect.center().y),
            Vec2::new(handle_width, rect.height()),
        );
        let end_handle_rect = Rect::from_center_size(
            Pos2::new(end_x, rect.center().y),
            Vec2::new(handle_width, rect.height()),
        );

        let start_resp = ui.interact(start_handle_rect, response.id.with("start"), Sense::drag());
        let end_resp = ui.interact(end_handle_rect, response.id.with("end"), Sense::drag());

        if start_resp.hovered() || end_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }

        if start_resp.dragged() {
            let delta_s = (start_resp.drag_delta().x / rect.width()) * video_duration;
            *start_time = (*start_time + delta_s).clamp(0.0, effective_end - 0.1);
            handled = true;
        } else if end_resp.dragged() {
            let delta_s = (end_resp.drag_delta().x / rect.width()) * video_duration;
            let mut new_end = (effective_end + delta_s).clamp(*start_time + 0.1, video_duration);
            // Snap to end (0.0) if close
            if (video_duration - new_end).abs() < 0.1 {
                new_end = 0.0;
            }
            *end_time = new_end;
            handled = true;
        }

        // 2. Body Interaction (Slide or Seek)
        if !handled && response.hovered() {
            if ui.input(|i| i.modifiers.shift)
                && region_rect.contains(response.hover_pos().unwrap_or_default())
            {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            } else {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }

        if !handled && response.dragged() {
            if ui.input(|i| i.modifiers.shift) {
                // Slide Region
                let delta_s = (response.drag_delta().x / rect.width()) * video_duration;
                let duration_s = effective_end - *start_time;

                let new_start = (*start_time + delta_s).clamp(0.0, video_duration - duration_s);
                let new_end = new_start + duration_s;

                *start_time = new_start;
                *end_time = if (video_duration - new_end).abs() < 0.1 {
                    0.0
                } else {
                    new_end
                };
            } else {
                // Seek
                if let Some(pos) = response.interact_pointer_pos() {
                    let seek_norm = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                    let seek_s = seek_norm * video_duration;
                    self.pending_playback_commands
                        .push((part_id, MediaPlaybackCommand::Seek(seek_s as f64)));
                }
            }
        }

        // Draw Handles
        painter.rect_filled(start_handle_rect.shrink(2.0), 2.0, Color32::WHITE);
        painter.rect_filled(end_handle_rect.shrink(2.0), 2.0, Color32::WHITE);

        // Draw Playhead
        let cursor_norm = (current_pos / video_duration).clamp(0.0, 1.0);
        let cursor_x = rect.min.x + cursor_norm * rect.width();
        painter.line_segment(
            [
                Pos2::new(cursor_x, rect.min.y),
                Pos2::new(cursor_x, rect.max.y),
            ],
            Stroke::new(2.0, Color32::from_rgb(255, 200, 50)),
        );
        // Playhead triangle top
        let tri_size = 6.0;
        painter.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(cursor_x - tri_size, rect.min.y),
                Pos2::new(cursor_x + tri_size, rect.min.y),
                Pos2::new(cursor_x, rect.min.y + tri_size * 1.5),
            ],
            Color32::from_rgb(255, 200, 50),
            Stroke::NONE,
        ));
    }
}
