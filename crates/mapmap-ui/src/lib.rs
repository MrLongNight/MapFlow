//! MapFlow UI - ImGui and egui Integration
//!
//! This crate provides the user interface layer using ImGui (legacy) and egui (Phase 6+), including:
//! - ImGui context setup (Phase 0-5)
//! - egui integration (Phase 6+)
//! - Window management
//! - Control panels
//! - Advanced authoring UI (Phase 6)
//! - Effect Chain Panel (Phase 3)

// Phase 3: Effects Pipeline UI (ImGui-based)
pub mod shader_graph_editor;
pub mod timeline;

// Phase 6: Advanced Authoring UI (egui-based)
pub mod asset_manager;
pub mod audio_panel;
pub mod config;
pub mod cue_panel;
pub mod dashboard;
pub mod edge_blend_panel;
pub mod effect_chain_panel;
pub mod i18n;
pub mod layer_panel;
pub mod media_browser;
pub mod menu_bar;
pub mod mesh_editor;
pub mod node_editor;
pub mod osc_panel;
pub mod paint_panel;
pub mod theme;
pub mod timeline_v2;
pub mod transform_panel;
pub mod undo_redo;

pub use i18n::LocaleManager;

pub use shader_graph_editor::{ShaderGraphAction, ShaderGraphEditor};
pub use timeline::{TimelineAction, TimelineEditor};

// Phase 6 exports
pub use asset_manager::{AssetManager, AssetManagerAction, EffectPreset, TransformPreset};
pub use audio_panel::AudioPanel;
pub use config::UserConfig;
pub use cue_panel::CuePanel;
pub use dashboard::{Dashboard, DashboardAction, DashboardWidget, WidgetType};
pub use edge_blend_panel::{EdgeBlendAction, EdgeBlendPanel};
pub use effect_chain_panel::{
    EffectChainAction, EffectChainPanel, PresetEntry, UIEffect, UIEffectChain,
};
pub use imgui::OwnedDrawData;
pub use layer_panel::{LayerPanel, LayerPanelAction};
pub use media_browser::{MediaBrowser, MediaBrowserAction, MediaEntry, MediaType};
pub use mesh_editor::{MeshEditor, MeshEditorAction};
pub use node_editor::{Node, NodeEditor, NodeEditorAction, NodeType};
pub use paint_panel::PaintPanel;
pub use theme::{Theme, ThemeConfig};
pub use timeline_v2::{InterpolationType, TimelineAction as TimelineV2Action, TimelineV2};
pub use transform_panel::{TransformAction, TransformPanel};
pub use undo_redo::{Command, CommandError, EditorState, UndoManager};

use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;

/// UI actions that can be triggered by the user interface
#[derive(Debug, Clone)]
pub enum UIAction {
    // Playback actions
    Play,
    Pause,
    Stop,
    SetSpeed(f32),
    SetLoopMode(mapmap_media::LoopMode),

    // File actions
    NewProject,
    LoadVideo(String),
    SaveProject(String),
    SaveProjectAs,
    LoadProject(String),
    LoadRecentProject(String),
    Export,
    OpenSettings,
    Exit,

    // Edit actions
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,

    // Mapping actions
    AddMapping,
    RemoveMapping(u64),
    ToggleMappingVisibility(u64, bool),
    SelectMapping(u64),

    // Paint actions
    AddPaint,
    RemovePaint(u64),

    // Layer actions (Phase 1)
    AddLayer,
    RemoveLayer(u64),
    DuplicateLayer(u64),
    RenameLayer(u64, String),
    ToggleLayerBypass(u64),
    ToggleLayerSolo(u64),
    SetLayerOpacity(u64, f32),
    EjectAllLayers,

    // Transform actions (Phase 1)
    SetLayerTransform(u64, mapmap_core::Transform),
    ApplyResizeMode(u64, mapmap_core::ResizeMode),

    // Master controls (Phase 1)
    SetMasterOpacity(f32),
    SetMasterSpeed(f32),
    SetCompositionName(String),

    // Phase 2: Output management
    AddOutput(String, mapmap_core::CanvasRegion, (u32, u32)),
    RemoveOutput(u64),
    ConfigureOutput(u64, mapmap_core::OutputConfig),
    SetOutputEdgeBlend(u64, mapmap_core::EdgeBlendConfig),
    SetOutputColorCalibration(u64, mapmap_core::ColorCalibration),
    CreateProjectorArray2x2((u32, u32), f32),

    // View actions
    ToggleFullscreen,
    ResetLayout,

    // Audio actions
    SelectAudioDevice(String),

    // Settings
    SetLanguage(String),

    // Help actions
    OpenDocs,
    OpenAbout,
    OpenLicense,
}

pub struct ImGuiContext {
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    last_frame: Instant,
    draw_data: Option<&'static imgui::DrawData>,
}

impl ImGuiContext {
    /// Create a new ImGui context
    pub fn new(
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);

        // Setup fonts
        imgui.io_mut().font_global_scale = 1.0;

        // Apply professional dark theme (matching egui style)
        theme::apply_imgui_theme(&mut imgui);

        // Create renderer
        let renderer_config = RendererConfig {
            texture_format: surface_format,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut imgui, device, queue, renderer_config);

        Self {
            imgui,
            platform,
            renderer,
            last_frame: Instant::now(),
            draw_data: None,
        }
    }

    /// Prepares the ImGui frame.
    pub fn prepare_frame<F>(&mut self, window: &winit::window::Window, build_ui: F)
    where
        F: FnOnce(&mut Ui),
    {
        // Update delta time
        let now = Instant::now();
        self.imgui.io_mut().update_delta_time(now - self.last_frame);
        self.last_frame = now;

        // Prepare frame
        self.platform
            .prepare_frame(self.imgui.io_mut(), window)
            .expect("Failed to prepare frame");

        // Begin frame and build UI
        let ui = self.imgui.frame();
        build_ui(ui);

        // End frame and prepare for rendering
        self.platform.prepare_render(ui, window);
        let draw_data = self.imgui.render();
        // SAFETY: We extend the lifetime of draw_data to 'static for imgui-wgpu renderer
        // The draw_data is only used within the same frame and cleared before next frame
        #[allow(clippy::missing_transmute_annotations)]
        {
            self.draw_data = Some(unsafe { std::mem::transmute(draw_data) });
        }
    }

    /// Renders the ImGui frame.
    pub fn render_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        if let Some(draw_data) = self.draw_data.take() {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ImGui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.renderer
                .render(draw_data, queue, device, &mut render_pass)
                .expect("Failed to render ImGui");
        }
    }

    /// Handle window events
    pub fn handle_event<T>(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<T>,
    ) {
        self.platform
            .handle_event(self.imgui.io_mut(), window, event);
    }
}

use mapmap_control::ControlTarget;

/// UI state for the application
pub struct AppUI {
    pub menu_bar: menu_bar::MenuBar,
    pub dashboard: Dashboard,
    pub paint_panel: PaintPanel,
    pub show_osc_panel: bool,
    pub selected_control_target: ControlTarget,
    pub osc_port_input: String,
    pub osc_client_input: String,
    pub show_controls: bool,
    pub show_stats: bool,
    pub show_layers: bool,
    pub layer_panel: LayerPanel,
    pub show_mappings: bool,
    pub show_transforms: bool,      // Phase 1
    pub show_master_controls: bool, // Phase 1
    pub show_outputs: bool,         // Phase 2
    pub edge_blend_panel: EdgeBlendPanel,
    pub show_oscillator: bool, // Oscillator distortion effect
    pub show_audio: bool,
    pub audio_panel: AudioPanel,
    pub show_cue_panel: bool,
    pub playback_speed: f32,
    pub loop_mode: mapmap_media::LoopMode,
    // Phase 1: Transform editing state
    pub selected_layer_id: Option<u64>,
    // Phase 2: Output configuration state
    pub selected_output_id: Option<u64>,
    pub audio_devices: Vec<String>,
    pub selected_audio_device: Option<String>,
    pub recent_files: Vec<String>,
    pub actions: Vec<UIAction>,
    pub i18n: LocaleManager,
    pub effect_chain_panel: EffectChainPanel,
    pub cue_panel: CuePanel,
    pub transform_panel: TransformPanel,
    pub user_config: config::UserConfig,
}

impl Default for AppUI {
    fn default() -> Self {
        Self {
            menu_bar: menu_bar::MenuBar::default(),
            dashboard: Dashboard::default(),
            paint_panel: PaintPanel::default(),
            show_osc_panel: true,
            selected_control_target: ControlTarget::Custom("".to_string()),
            osc_port_input: "8000".to_string(),
            osc_client_input: "127.0.0.1:9000".to_string(),
            show_controls: true,
            show_stats: true,
            show_layers: true,
            layer_panel: LayerPanel { visible: true },
            show_mappings: true,
            show_transforms: true,
            show_master_controls: true,
            show_outputs: true,
            edge_blend_panel: EdgeBlendPanel::default(),
            show_oscillator: true,
            show_audio: true,
            audio_panel: AudioPanel::default(),
            show_cue_panel: true,
            playback_speed: 1.0,
            loop_mode: mapmap_media::LoopMode::Loop,
            selected_layer_id: None,
            selected_output_id: None,
            audio_devices: vec!["None".to_string()],
            selected_audio_device: None,
            recent_files: {
                let config = config::UserConfig::load();
                config.recent_files.clone()
            },
            actions: Vec::new(),
            i18n: {
                let config = config::UserConfig::load();
                LocaleManager::new(&config.language)
            },
            effect_chain_panel: EffectChainPanel::default(),
            cue_panel: CuePanel::default(),
            transform_panel: TransformPanel::default(),
            user_config: config::UserConfig::load(),
        }
    }
}

impl AppUI {
    /// Take all pending actions and clear the list
    pub fn take_actions(&mut self) -> Vec<UIAction> {
        std::mem::take(&mut self.actions)
    }

    /// Render the cue panel
    pub fn render_cue_panel(&mut self, ui: &Ui) {
        if !self.show_cue_panel {
            return;
        }

        self.cue_panel.render(ui);
    }

    /// Render the control panel
    pub fn render_controls(&mut self, ui: &Ui) {
        if !self.show_controls {
            return;
        }

        ui.window(self.i18n.t("panel-playback"))
            .size([320.0, 360.0], Condition::FirstUseEver)
            .position([380.0, 10.0], Condition::FirstUseEver)
            .build(|| {
                ui.text(self.i18n.t("header-video-playback"));
                ui.separator();

                // Transport controls
                if ui.button(self.i18n.t("btn-play")) {
                    self.actions.push(UIAction::Play);
                }
                ui.same_line();
                if ui.button(self.i18n.t("btn-pause")) {
                    self.actions.push(UIAction::Pause);
                }
                ui.same_line();
                if ui.button(self.i18n.t("btn-stop")) {
                    self.actions.push(UIAction::Stop);
                }

                ui.separator();

                // Speed control
                let old_speed = self.playback_speed;
                ui.slider(
                    self.i18n.t("label-speed"),
                    0.1,
                    2.0,
                    &mut self.playback_speed,
                );
                if (self.playback_speed - old_speed).abs() > 0.001 {
                    self.actions.push(UIAction::SetSpeed(self.playback_speed));
                }

                // Loop control
                ui.text(self.i18n.t("label-mode"));
                let mode_names = [self.i18n.t("mode-loop"), self.i18n.t("mode-play-once")];
                let mut mode_idx = match self.loop_mode {
                    mapmap_media::LoopMode::Loop => 0,
                    mapmap_media::LoopMode::PlayOnce => 1,
                };

                if ui.combo(
                    self.i18n.t("label-mode"),
                    &mut mode_idx,
                    &mode_names,
                    |item| std::borrow::Cow::Borrowed(item),
                ) {
                    let new_mode = match mode_idx {
                        0 => mapmap_media::LoopMode::Loop,
                        1 => mapmap_media::LoopMode::PlayOnce,
                        _ => mapmap_media::LoopMode::Loop,
                    };
                    self.loop_mode = new_mode;
                    self.actions.push(UIAction::SetLoopMode(new_mode));
                }
            });
    }

    /// Render performance stats
    pub fn render_stats(&mut self, ui: &Ui, fps: f32, frame_time_ms: f32) {
        if !self.show_stats {
            return;
        }

        ui.window(self.i18n.t("panel-performance"))
            .size([250.0, 120.0], Condition::FirstUseEver)
            .position([10.0, 10.0], Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("{}: {:.1}", self.i18n.t("label-fps"), fps));
                ui.text(format!(
                    "{}: {:.2} ms",
                    self.i18n.t("label-frame-time"),
                    frame_time_ms
                ));
            });
    }

    /// Render main menu bar
    pub fn render_menu_bar(&mut self, ui: &Ui) {
        ui.main_menu_bar(|| {
            ui.menu(self.i18n.t("menu-file"), || {
                if ui.menu_item(self.i18n.t("menu-file-load-video")) {
                    // TODO: Open file dialog
                    self.actions.push(UIAction::LoadVideo(String::new()));
                }
                if ui.menu_item(self.i18n.t("menu-file-save-project")) {
                    self.actions.push(UIAction::SaveProject(String::new()));
                }
                if ui.menu_item(self.i18n.t("menu-file-load-project")) {
                    self.actions.push(UIAction::LoadProject(String::new()));
                }

                // Recent Files Submenu
                if !self.recent_files.is_empty() {
                    ui.menu(self.i18n.t("menu-file-open-recent"), || {
                        for path in &self.recent_files {
                            // Display only the filename if possible, otherwise full path
                            let label = std::path::Path::new(path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(path);

                            if ui.menu_item(label) {
                                self.actions.push(UIAction::LoadRecentProject(path.clone()));
                            }
                        }
                    });
                }

                ui.separator();
                if ui.menu_item(self.i18n.t("menu-file-exit")) {
                    self.actions.push(UIAction::Exit);
                }
            });

            ui.menu(self.i18n.t("menu-view"), || {
                ui.checkbox(self.i18n.t("check-show-osc"), &mut self.show_osc_panel);
                ui.checkbox(self.i18n.t("check-show-controls"), &mut self.show_controls);
                ui.checkbox(
                    self.i18n.t("check-show-layers"),
                    &mut self.layer_panel.visible,
                );
                ui.checkbox(
                    self.i18n.t("check-show-paints"),
                    &mut self.paint_panel.visible,
                );
                ui.checkbox(self.i18n.t("check-show-mappings"), &mut self.show_mappings);
                ui.checkbox(
                    self.i18n.t("check-show-transforms"),
                    &mut self.transform_panel.visible,
                );
                ui.checkbox(
                    self.i18n.t("check-show-master"),
                    &mut self.show_master_controls,
                );
                ui.checkbox(
                    self.i18n.t("panel-edge-blend-color"),
                    &mut self.edge_blend_panel.visible,
                );
                ui.checkbox(
                    self.i18n.t("check-show-oscillator"),
                    &mut self.show_oscillator,
                );
                ui.checkbox(
                    self.i18n.t("panel-effect-chain"),
                    &mut self.effect_chain_panel.visible,
                );
                ui.checkbox(self.i18n.t("check-show-audio"), &mut self.show_audio);
                ui.checkbox(self.i18n.t("check-show-cues"), &mut self.show_cue_panel);
                ui.checkbox(self.i18n.t("check-show-stats"), &mut self.show_stats);
                ui.separator();
                if ui.menu_item(self.i18n.t("btn-fullscreen")) {
                    self.actions.push(UIAction::ToggleFullscreen);
                }
            });

            ui.menu(self.i18n.t("menu-help"), || {
                if ui.menu_item(self.i18n.t("menu-help-about")) {
                    // Show about dialog
                }

                ui.separator();

                // Language Selection
                if ui.menu_item(self.i18n.t("menu-help-lang-en")) {
                    self.actions.push(UIAction::SetLanguage("en".to_string()));
                }
                if ui.menu_item(self.i18n.t("menu-help-lang-de")) {
                    self.actions.push(UIAction::SetLanguage("de".to_string()));
                }
            });
        });
    }

    /// Render layer management panel
    pub fn render_layer_panel(&mut self, ui: &Ui, layer_manager: &mut mapmap_core::LayerManager) {
        use mapmap_core::BlendMode;

        if !self.show_layers {
            return;
        }

        ui.window(self.i18n.t("panel-layers"))
            .size([380.0, 400.0], Condition::FirstUseEver)
            .position([1530.0, 10.0], Condition::FirstUseEver)
            .build(|| {
                ui.text(self.i18n.t_args(
                    "label-total-layers",
                    &[("count", &layer_manager.layers().len().to_string())],
                ));
                ui.separator();

                // Collect layer IDs to avoid borrow issues
                let layer_ids: Vec<u64> = layer_manager.layers().iter().map(|l| l.id).collect();

                // Layer list
                for layer_id in layer_ids {
                    if let Some(layer) = layer_manager.get_layer_mut(layer_id) {
                        let _id = ui.push_id_usize(layer.id as usize);

                        // Layer header with visibility toggle
                        let mut visible = layer.visible;
                        if ui.checkbox(format!("##visible_{}", layer.id), &mut visible) {
                            layer.visible = visible;
                        }
                        ui.same_line();

                        // Layer name (clickable to select)
                        if ui.small_button(&layer.name) {
                            self.selected_layer_id = Some(layer.id);
                        }

                        // Indent for layer properties
                        ui.indent();

                        // Phase 1: Bypass, Solo, Lock toggles
                        let mut bypass = layer.bypass;
                        if ui.checkbox(self.i18n.t("check-bypass"), &mut bypass) {
                            layer.bypass = bypass;
                            self.actions.push(UIAction::ToggleLayerBypass(layer.id));
                        }
                        ui.same_line();

                        let mut solo = layer.solo;
                        if ui.checkbox(self.i18n.t("check-solo"), &mut solo) {
                            layer.solo = solo;
                            self.actions.push(UIAction::ToggleLayerSolo(layer.id));
                        }

                        // Blend mode selector
                        let blend_modes = [
                            "Normal",
                            "Add",
                            "Subtract",
                            "Multiply",
                            "Screen",
                            "Overlay",
                            "Soft Light",
                            "Hard Light",
                            "Lighten",
                            "Darken",
                            "Color Dodge",
                            "Color Burn",
                            "Difference",
                            "Exclusion",
                        ];

                        let current_mode_idx = layer.blend_mode as usize;
                        let mut selected = current_mode_idx;

                        if ui.combo(
                            self.i18n.t("label-mode"),
                            &mut selected,
                            &blend_modes,
                            |item| std::borrow::Cow::Borrowed(item),
                        ) {
                            layer.blend_mode = match selected {
                                0 => BlendMode::Normal,
                                1 => BlendMode::Add,
                                2 => BlendMode::Subtract,
                                3 => BlendMode::Multiply,
                                4 => BlendMode::Screen,
                                5 => BlendMode::Overlay,
                                6 => BlendMode::SoftLight,
                                7 => BlendMode::HardLight,
                                8 => BlendMode::Lighten,
                                9 => BlendMode::Darken,
                                10 => BlendMode::ColorDodge,
                                11 => BlendMode::ColorBurn,
                                12 => BlendMode::Difference,
                                13 => BlendMode::Exclusion,
                                _ => BlendMode::Normal,
                            };
                        }

                        // Phase 1: Opacity slider (Video Fader)
                        let old_opacity = layer.opacity;
                        ui.slider(
                            self.i18n.t("label-master-opacity"),
                            0.0,
                            1.0,
                            &mut layer.opacity,
                        );
                        if (layer.opacity - old_opacity).abs() > 0.001 {
                            self.actions
                                .push(UIAction::SetLayerOpacity(layer.id, layer.opacity));
                        }

                        // Phase 1: Layer management buttons
                        if ui.button(self.i18n.t("btn-duplicate")) {
                            self.actions.push(UIAction::DuplicateLayer(layer.id));
                        }
                        ui.same_line();
                        if ui.button(self.i18n.t("btn-remove")) {
                            self.actions.push(UIAction::RemoveLayer(layer.id));
                        }

                        ui.unindent();
                        ui.separator();
                    }
                }

                ui.separator();

                // Layer management buttons
                if ui.button(self.i18n.t("btn-add-layer")) {
                    self.actions.push(UIAction::AddLayer);
                }
                ui.same_line();
                if ui.button(self.i18n.t("btn-eject-all")) {
                    self.actions.push(UIAction::EjectAllLayers);
                }
            });
    }

    /// Render mapping management panel
    pub fn render_mapping_panel(
        &mut self,
        ui: &Ui,
        mapping_manager: &mut mapmap_core::MappingManager,
    ) {
        if !self.show_mappings {
            return;
        }

        ui.window(self.i18n.t("panel-mappings"))
            .size([380.0, 300.0], Condition::FirstUseEver)
            .position([1530.0, 420.0], Condition::FirstUseEver)
            .build(|| {
                ui.text(self.i18n.t_args(
                    "label-total-mappings",
                    &[("count", &mapping_manager.mappings().len().to_string())],
                ));
                ui.separator();

                // Mapping list
                let mapping_ids: Vec<mapmap_core::MappingId> =
                    mapping_manager.mappings().iter().map(|m| m.id).collect();

                for mapping_id in mapping_ids {
                    if let Some(mapping) = mapping_manager.get_mapping_mut(mapping_id) {
                        let _id = ui.push_id_usize(mapping.id as usize);

                        // Mapping header with visibility
                        let old_visible = mapping.visible;
                        if ui.checkbox(format!("##visible_{}", mapping.id), &mut mapping.visible)
                            && mapping.visible != old_visible
                        {
                            self.actions.push(UIAction::ToggleMappingVisibility(
                                mapping.id,
                                mapping.visible,
                            ));
                        }
                        ui.same_line();

                        // Make the mapping name clickable to select it
                        if ui
                            .small_button(format!("{} (Paint #{})", mapping.name, mapping.paint_id))
                        {
                            self.actions.push(UIAction::SelectMapping(mapping.id));
                        }

                        // Indent for mapping properties
                        ui.indent();

                        // Solo and Lock toggles
                        ui.checkbox(self.i18n.t("check-solo"), &mut mapping.solo);
                        ui.same_line();
                        ui.checkbox(self.i18n.t("check-lock"), &mut mapping.locked);

                        // Opacity slider
                        ui.slider(
                            self.i18n.t("label-master-opacity"),
                            0.0,
                            1.0,
                            &mut mapping.opacity,
                        );

                        // Depth (Z-order)
                        ui.slider(
                            self.i18n.t("label-frame-time"),
                            -10.0,
                            10.0,
                            &mut mapping.depth,
                        );

                        // Mesh info
                        ui.text(self.i18n.t_args(
                            "label-mesh",
                            &[
                                ("type", &format!("{:?}", mapping.mesh.mesh_type)),
                                ("count", &mapping.mesh.vertex_count().to_string()),
                            ],
                        ));

                        // Remove button for this mapping
                        if ui.button(self.i18n.t("btn-remove-this")) {
                            self.actions.push(UIAction::RemoveMapping(mapping.id));
                        }

                        ui.unindent();
                        ui.separator();
                    }
                }

                ui.separator();

                // Mapping management buttons
                if ui.button(self.i18n.t("btn-add-quad")) {
                    self.actions.push(UIAction::AddMapping);
                }
            });
    }

    /// Render master controls panel (Phase 1)
    pub fn render_master_controls(
        &mut self,
        ui: &Ui,
        layer_manager: &mut mapmap_core::LayerManager,
    ) {
        if !self.show_master_controls {
            return;
        }

        ui.window(self.i18n.t("panel-master"))
            .size([360.0, 300.0], Condition::FirstUseEver)
            .position([10.0, 670.0], Condition::FirstUseEver)
            .build(|| {
                ui.text(self.i18n.t("header-master"));
                ui.separator();

                let composition = &mut layer_manager.composition;

                // Composition name (Phase 1, Month 5)
                ui.text(self.i18n.t("label-composition"));
                ui.text_wrapped(&composition.name);

                // Note: ImGui text input requires mutable String buffer
                // For now, just display the name
                ui.separator();

                // Master Opacity (Phase 1, Month 4)
                let old_master_opacity = composition.master_opacity;
                ui.slider(
                    self.i18n.t("label-master-opacity"),
                    0.0,
                    1.0,
                    &mut composition.master_opacity,
                );
                if (composition.master_opacity - old_master_opacity).abs() > 0.001 {
                    self.actions
                        .push(UIAction::SetMasterOpacity(composition.master_opacity));
                }

                // Master Speed (Phase 1, Month 5)
                let old_master_speed = composition.master_speed;
                ui.slider(
                    self.i18n.t("label-master-speed"),
                    0.1,
                    10.0,
                    &mut composition.master_speed,
                );
                if (composition.master_speed - old_master_speed).abs() > 0.001 {
                    self.actions
                        .push(UIAction::SetMasterSpeed(composition.master_speed));
                }

                ui.separator();
                ui.text(format!(
                    "{} {}x{}",
                    self.i18n.t("label-size"),
                    composition.size.0,
                    composition.size.1
                ));
                ui.text(format!(
                    "{} {:.1} fps",
                    self.i18n.t("label-frame-rate"),
                    composition.frame_rate
                ));

                ui.separator();
                ui.text(self.i18n.t("label-effective-multipliers"));
                ui.text(self.i18n.t("text-mult-opacity"));
                ui.text(self.i18n.t("text-mult-speed"));
            });
    }

    /// Phase 2: Render output configuration panel
    pub fn render_output_panel(
        &mut self,
        ui: &Ui,
        output_manager: &mut mapmap_core::OutputManager,
    ) {
        if !self.show_outputs {
            return;
        }

        ui.window(self.i18n.t("panel-outputs"))
            .size([420.0, 500.0], Condition::FirstUseEver)
            .position([380.0, 380.0], Condition::FirstUseEver)
            .build(|| {
                ui.text(self.i18n.t("header-outputs"));
                ui.separator();

                // Canvas size display
                let canvas_size = output_manager.canvas_size();
                ui.text(format!(
                    "{}: {}x{}",
                    self.i18n.t("label-canvas"),
                    canvas_size.0,
                    canvas_size.1
                ));
                ui.separator();

                // Output list
                ui.text(format!(
                    "{}: {}",
                    self.i18n.t("panel-outputs"),
                    output_manager.outputs().len()
                ));

                for output in output_manager.outputs() {
                    let _id = ui.push_id_usize(output.id as usize);

                    let is_selected = self.selected_output_id == Some(output.id);
                    if ui
                        .selectable_config(&output.name)
                        .selected(is_selected)
                        .build()
                    {
                        self.selected_output_id = Some(output.id);
                    }

                    // Show output info
                    ui.same_line();
                    ui.text_disabled(format!(
                        "{}x{} | {}",
                        output.resolution.0,
                        output.resolution.1,
                        if output.fullscreen { "FS" } else { "Win" }
                    ));
                }

                ui.separator();

                // Quick setup buttons
                if ui.button(self.i18n.t("btn-projector-array")) {
                    self.actions.push(UIAction::CreateProjectorArray2x2(
                        (1920, 1080),
                        0.1, // 10% overlap
                    ));
                }

                ui.same_line();
                if ui.button(self.i18n.t("btn-add-output")) {
                    // Add a single output covering full canvas
                    self.actions.push(UIAction::AddOutput(
                        "New Output".to_string(),
                        mapmap_core::CanvasRegion::new(0.0, 0.0, 1.0, 1.0),
                        (1920, 1080),
                    ));
                }

                ui.separator();

                // Edit selected output
                if let Some(output_id) = self.selected_output_id {
                    if let Some(output) =
                        output_manager.outputs().iter().find(|o| o.id == output_id)
                    {
                        ui.text(self.i18n.t("header-selected-output"));
                        ui.separator();

                        ui.text(format!("{}: {}", self.i18n.t("label-name"), output.name));
                        ui.text(format!(
                            "{}: {}x{}",
                            self.i18n.t("label-resolution"),
                            output.resolution.0,
                            output.resolution.1
                        ));

                        ui.separator();
                        ui.text(self.i18n.t("label-canvas-region"));
                        ui.text(format!(
                            "  {}: {:.2}, {}: {:.2}",
                            self.i18n.t("label-x"),
                            output.canvas_region.x,
                            self.i18n.t("label-y"),
                            output.canvas_region.y
                        ));
                        ui.text(format!(
                            "  {}: {:.2}, {}: {:.2}",
                            self.i18n.t("label-width"),
                            output.canvas_region.width,
                            self.i18n.t("label-height"),
                            output.canvas_region.height
                        ));

                        ui.separator();

                        // Edge blending status
                        let blend = &output.edge_blend;
                        ui.text(format!("{}:", self.i18n.t("panel-edge-blend")));
                        if blend.left.enabled {
                            ui.text(format!("  {}", self.i18n.t("check-left")));
                        }
                        if blend.right.enabled {
                            ui.text(format!("  {}", self.i18n.t("check-right")));
                        }
                        if blend.top.enabled {
                            ui.text(format!("  {}", self.i18n.t("check-top")));
                        }
                        if blend.bottom.enabled {
                            ui.text(format!("  {}", self.i18n.t("check-bottom")));
                        }
                        if !blend.left.enabled
                            && !blend.right.enabled
                            && !blend.top.enabled
                            && !blend.bottom.enabled
                        {
                            ui.text_disabled(format!("  {}", self.i18n.t("label-none")));
                        }

                        ui.separator();

                        // Color calibration status
                        let cal = &output.color_calibration;
                        ui.text(format!("{}:", self.i18n.t("panel-color-cal")));
                        if cal.brightness != 0.0 {
                            ui.text(format!(
                                "  {}: {:.2}",
                                self.i18n.t("label-brightness"),
                                cal.brightness
                            ));
                        }
                        if cal.contrast != 1.0 {
                            ui.text(format!(
                                "  {}: {:.2}",
                                self.i18n.t("label-contrast"),
                                cal.contrast
                            ));
                        }
                        if cal.saturation != 1.0 {
                            ui.text(format!(
                                "  {}: {:.2}",
                                self.i18n.t("label-saturation"),
                                cal.saturation
                            ));
                        }
                        if cal.brightness == 0.0 && cal.contrast == 1.0 && cal.saturation == 1.0 {
                            ui.text_disabled(format!("  ({})", self.i18n.t("label-none")));
                        }

                        ui.separator();

                        ui.text_colored(
                            [0.5, 0.8, 1.0, 1.0],
                            format!("{}:", self.i18n.t("output-tip")),
                        );
                        ui.text_wrapped(self.i18n.t("tip-panels-auto-open"));

                        ui.separator();

                        if ui.button(self.i18n.t("btn-remove-output")) {
                            self.actions.push(UIAction::RemoveOutput(output_id));
                            self.selected_output_id = None;
                        }
                    }
                }

                ui.separator();
                ui.text_colored([0.0, 1.0, 0.0, 1.0], self.i18n.t("msg-multi-window-active"));
                ui.text_disabled(self.i18n.t("msg-output-windows-tip"));
            });
    }

    /// Render oscillator distortion effect control panel
    pub fn render_oscillator_panel(&mut self, ui: &Ui, config: &mut mapmap_core::OscillatorConfig) {
        if !self.show_oscillator {
            return;
        }

        ui.window(self.i18n.t("panel-oscillator"))
            .size([450.0, 750.0], Condition::FirstUseEver)
            .position([1560.0, 520.0], Condition::FirstUseEver)
            .build(|| {
                // Master enable
                ui.checkbox(self.i18n.t("check-enable"), &mut config.enabled);
                ui.separator();

                // Preset buttons
                ui.text(format!("{}:", self.i18n.t("header-quick-presets")));
                if ui.button(self.i18n.t("btn-subtle")) {
                    *config = mapmap_core::OscillatorConfig::preset_subtle();
                }
                ui.same_line();
                if ui.button(self.i18n.t("btn-dramatic")) {
                    *config = mapmap_core::OscillatorConfig::preset_dramatic();
                }
                ui.same_line();
                if ui.button(self.i18n.t("btn-rings")) {
                    *config = mapmap_core::OscillatorConfig::preset_rings();
                }
                ui.same_line();
                if ui.button(self.i18n.t("btn-reset")) {
                    *config = mapmap_core::OscillatorConfig::default();
                }

                ui.separator();

                // Distortion parameters
                ui.text(self.i18n.t("header-distortion"));
                ui.slider(
                    self.i18n.t("label-amount"),
                    0.0,
                    1.0,
                    &mut config.distortion_amount,
                );
                if ui.is_item_hovered() {
                    ui.tooltip_text("Intensity of the distortion effect");
                }

                ui.slider(
                    self.i18n.t("label-dist-scale"),
                    0.0,
                    0.1,
                    &mut config.distortion_scale,
                );
                if ui.is_item_hovered() {
                    ui.tooltip_text("Spatial scale of distortion");
                }

                ui.slider(
                    self.i18n.t("label-dist-speed"),
                    0.0,
                    5.0,
                    &mut config.distortion_speed,
                );
                if ui.is_item_hovered() {
                    ui.tooltip_text("Animation speed");
                }

                ui.separator();

                // Visual overlay
                ui.text(self.i18n.t("header-visual-overlay"));
                ui.slider(
                    self.i18n.t("label-overlay-opacity"),
                    0.0,
                    1.0,
                    &mut config.overlay_opacity,
                );

                // Color mode combo
                let color_modes = ["Off", "Rainbow", "Black & White", "Complementary"];
                let mut color_idx = match config.color_mode {
                    mapmap_core::ColorMode::Off => 0,
                    mapmap_core::ColorMode::Rainbow => 1,
                    mapmap_core::ColorMode::BlackWhite => 2,
                    mapmap_core::ColorMode::Complementary => 3,
                };

                if ui.combo(
                    self.i18n.t("label-color-mode"),
                    &mut color_idx,
                    &color_modes,
                    |item| std::borrow::Cow::Borrowed(item),
                ) {
                    config.color_mode = match color_idx {
                        0 => mapmap_core::ColorMode::Off,
                        1 => mapmap_core::ColorMode::Rainbow,
                        2 => mapmap_core::ColorMode::BlackWhite,
                        3 => mapmap_core::ColorMode::Complementary,
                        _ => mapmap_core::ColorMode::Off,
                    };
                }

                ui.separator();

                // Simulation parameters
                ui.text(self.i18n.t("header-simulation"));

                // Resolution combo
                let res_names = ["Low (128x128)", "Medium (256x256)", "High (512x512)"];
                let mut res_idx = match config.simulation_resolution {
                    mapmap_core::SimulationResolution::Low => 0,
                    mapmap_core::SimulationResolution::Medium => 1,
                    mapmap_core::SimulationResolution::High => 2,
                };

                if ui.combo(
                    self.i18n.t("label-resolution"),
                    &mut res_idx,
                    &res_names,
                    |item| std::borrow::Cow::Borrowed(item),
                ) {
                    config.simulation_resolution = match res_idx {
                        0 => mapmap_core::SimulationResolution::Low,
                        1 => mapmap_core::SimulationResolution::Medium,
                        2 => mapmap_core::SimulationResolution::High,
                        _ => mapmap_core::SimulationResolution::Medium,
                    };
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text("Higher resolution = more detail but slower");
                }

                ui.slider(
                    self.i18n.t("label-kernel-radius"),
                    1.0,
                    64.0,
                    &mut config.kernel_radius,
                );
                if ui.is_item_hovered() {
                    ui.tooltip_text("Coupling interaction distance");
                }

                ui.slider(
                    self.i18n.t("label-noise-amount"),
                    0.0,
                    1.0,
                    &mut config.noise_amount,
                );
                if ui.is_item_hovered() {
                    ui.tooltip_text("Random variation in oscillation");
                }

                ui.slider(
                    self.i18n.t("label-freq-min"),
                    0.0,
                    10.0,
                    &mut config.frequency_min,
                );
                ui.slider(
                    self.i18n.t("label-freq-max"),
                    0.0,
                    10.0,
                    &mut config.frequency_max,
                );

                ui.separator();

                // Coordinate mode
                let coord_modes = ["Cartesian", "Log-Polar"];
                let mut coord_idx = match config.coordinate_mode {
                    mapmap_core::CoordinateMode::Cartesian => 0,
                    mapmap_core::CoordinateMode::LogPolar => 1,
                };

                if ui.combo(
                    self.i18n.t("label-coordinate-mode"),
                    &mut coord_idx,
                    &coord_modes,
                    |item| std::borrow::Cow::Borrowed(item),
                ) {
                    config.coordinate_mode = match coord_idx {
                        0 => mapmap_core::CoordinateMode::Cartesian,
                        1 => mapmap_core::CoordinateMode::LogPolar,
                        _ => mapmap_core::CoordinateMode::Cartesian,
                    };
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text("Log-Polar creates radial/spiral patterns");
                }

                // Phase initialization mode
                let phase_modes = ["Random", "Uniform", "Plane H", "Plane V", "Diagonal"];
                let mut phase_idx = match config.phase_init_mode {
                    mapmap_core::PhaseInitMode::Random => 0,
                    mapmap_core::PhaseInitMode::Uniform => 1,
                    mapmap_core::PhaseInitMode::PlaneHorizontal => 2,
                    mapmap_core::PhaseInitMode::PlaneVertical => 3,
                    mapmap_core::PhaseInitMode::PlaneDiagonal => 4,
                };

                if ui.combo(
                    self.i18n.t("label-phase-init"),
                    &mut phase_idx,
                    &phase_modes,
                    |item| std::borrow::Cow::Borrowed(item),
                ) {
                    config.phase_init_mode = match phase_idx {
                        0 => mapmap_core::PhaseInitMode::Random,
                        1 => mapmap_core::PhaseInitMode::Uniform,
                        2 => mapmap_core::PhaseInitMode::PlaneHorizontal,
                        3 => mapmap_core::PhaseInitMode::PlaneVertical,
                        4 => mapmap_core::PhaseInitMode::PlaneDiagonal,
                        _ => mapmap_core::PhaseInitMode::Random,
                    };
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text("Initial phase pattern for oscillators");
                }

                ui.separator();

                // Coupling rings
                if ui.collapsing_header(self.i18n.t("header-coupling"), TreeNodeFlags::empty()) {
                    for i in 0..4 {
                        let _id = ui.push_id_usize(i);

                        let is_active = config.rings[i].distance > 0.0
                            || config.rings[i].width > 0.0
                            || config.rings[i].coupling.abs() > 0.01;

                        if let Some(_token) = ui
                            .tree_node_config(format!("Ring {}", i + 1))
                            .default_open(is_active)
                            .build(|| {
                                ui.slider(
                                    format!("{}##ring", self.i18n.t("label-dist-scale")),
                                    0.0,
                                    1.0,
                                    &mut config.rings[i].distance,
                                );
                                if ui.is_item_hovered() {
                                    ui.tooltip_text("Distance from center (0-1)");
                                }

                                ui.slider(
                                    format!("{}##ring", self.i18n.t("label-width")),
                                    0.0,
                                    1.0,
                                    &mut config.rings[i].width,
                                );
                                if ui.is_item_hovered() {
                                    ui.tooltip_text("Ring width (0-1)");
                                }

                                ui.slider(
                                    self.i18n.t("label-diff-coupling"),
                                    -5.0,
                                    5.0,
                                    &mut config.rings[i].coupling,
                                );
                                if ui.is_item_hovered() {
                                    ui.tooltip_text("Negative = anti-sync, Positive = sync");
                                }

                                if ui.button(self.i18n.t("btn-reset-ring")) {
                                    config.rings[i] = mapmap_core::RingParams::default();
                                }
                                ui.same_line();
                                if ui.button(self.i18n.t("btn-clear-ring")) {
                                    config.rings[i] = mapmap_core::RingParams {
                                        distance: 0.0,
                                        width: 0.0,
                                        coupling: 0.0,
                                    };
                                }
                            })
                        {}
                    }
                }
            });
    }
}
