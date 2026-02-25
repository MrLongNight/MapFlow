use super::types::{CanvasAction, MediaPlaybackCommand, MediaPlayerInfo, ModulePreset};
use crate::editors::mesh_editor::MeshEditor;
use egui::{Pos2, TextureHandle, Vec2};
use mapmap_core::{
    audio_reactive::AudioTriggerData,
    diagnostics::ModuleIssue,
    module::{
        AudioBand, AudioTriggerOutputConfig, EffectType, LayerType, MaskShape, MaskType,
        ModulePartId, ModulePartType, ModuleSocketType, ModulizerType, OutputType, SourceType,
        TriggerType,
    },
};
#[cfg(feature = "ndi")]
use mapmap_io::ndi::NdiSource;
#[cfg(feature = "ndi")]
use std::sync::mpsc;

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
    /// Connection being created: (from_part, from_socket_idx, is_output, socket_type, start_pos)
    pub creating_connection: Option<(ModulePartId, usize, bool, ModuleSocketType, Pos2)>,
    /// Part ID pending deletion (set when X button clicked)
    pub pending_delete: Option<ModulePartId>,
    /// Selected parts for multi-select and copy/paste
    pub selected_parts: Vec<ModulePartId>,
    /// Clipboard for copy/paste (stores part types and relative positions)
    pub clipboard: Vec<(ModulePartType, (f32, f32))>,
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
    pub plug_icons: std::collections::HashMap<String, egui::TextureHandle>,
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
    /// Pending NDI connection (part_id, source)
    #[cfg(feature = "ndi")]
    pub pending_ndi_connect: Option<(ModulePartId, NdiSource)>,
    /// Available outputs (id, name) for output node selection
    pub available_outputs: Vec<(u64, String)>,
    /// ID of the part being edited in a popup
    pub editing_part_id: Option<ModulePartId>,
    /// Video Texture Previews for Media Nodes ((Module ID, Part ID) -> Egui Texture)
    pub node_previews: std::collections::HashMap<(u64, u64), egui::TextureId>,
    /// Pending playback commands (Part ID, Command)
    pub pending_playback_commands: Vec<(ModulePartId, MediaPlaybackCommand)>,
    /// Last diagnostic check results
    pub diagnostic_issues: Vec<ModuleIssue>,
    /// Whether diagnostic popup is shown
    pub show_diagnostics: bool,
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
    pub fn ensure_icons_loaded(&mut self, ctx: &egui::Context) {
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

    fn load_svg_icon(path: &std::path::Path, ctx: &egui::Context) -> Option<TextureHandle> {
        let svg_data = std::fs::read(path).ok()?;
        let options = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(&svg_data, &options).ok()?;
        let size = tree.size();
        let width = size.width().round() as u32;
        let height = size.height().round() as u32;

        let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::default(),
            &mut pixmap.as_mut(),
        );

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
            source_size: egui::vec2(width as f32, height as f32),
            pixels,
        };

        Some(ctx.load_texture(
            path.file_name()?.to_string_lossy(),
            image,
            egui::TextureOptions::LINEAR,
        ))
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

    /// Create default presets/templates
    pub fn default_presets() -> Vec<ModulePreset> {
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
                            ndi_enabled: false,
                            ndi_stream_name: String::new(),
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![(0, 0, 1, 0), (1, 0, 2, 0)],
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
                        (350.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Modulizer(ModulizerType::Effect {
                            effect_type: EffectType::Blur,
                            params: std::collections::HashMap::new(),
                        }),
                        (650.0, 100.0),
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
                            ndi_enabled: false,
                            ndi_stream_name: String::new(),
                        }),
                        (950.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![(0, 0, 1, 0), (1, 0, 2, 0), (2, 0, 3, 0)],
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
                        (350.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Modulizer(ModulizerType::Effect {
                            effect_type: EffectType::Glitch,
                            params: std::collections::HashMap::new(),
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Layer(LayerType::All {
                            opacity: 1.0,
                            blend_mode: None,
                        }),
                        (950.0, 100.0),
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
                            ndi_enabled: false,
                            ndi_stream_name: String::new(),
                        }),
                        (1250.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![(0, 0, 1, 0), (1, 0, 2, 0), (2, 0, 3, 0), (3, 0, 4, 0)],
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
                        (350.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Mask(MaskType::Shape(MaskShape::Circle)),
                        (650.0, 100.0),
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
                            ndi_enabled: false,
                            ndi_stream_name: String::new(),
                        }),
                        (950.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![(0, 0, 1, 0), (1, 0, 2, 0), (2, 0, 3, 0)],
            },
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
                            ndi_enabled: false,
                            ndi_stream_name: String::new(),
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![(0, 0, 1, 0), (1, 0, 2, 0)],
            },
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
                connections: vec![(0, 0, 1, 0), (1, 0, 2, 0)],
            },
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
                            ndi_enabled: false,
                            ndi_stream_name: String::new(),
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![(0, 0, 1, 0), (1, 0, 2, 0)],
            },
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
                    (
                        ModulePartType::Output(OutputType::Spout {
                            name: "MapFlow Spout".to_string(),
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![(0, 0, 1, 0), (1, 0, 2, 0)],
            },
        ]
    }
}
