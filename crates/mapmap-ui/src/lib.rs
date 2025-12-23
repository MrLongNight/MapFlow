//! MapFlow UI - ImGui and egui Integration
//!
//! This crate provides the user interface layer using ImGui (legacy) and egui (Phase 6+), including:
//! - ImGui context setup (Phase 0-5)
//! - egui integration (Phase 6+)
//! - Window management
//! - Control panels
//! - Advanced authoring UI (Phase 6)
//! - Effect Chain Panel (Phase 3)

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
pub mod mapping_panel;
pub mod media_browser;
pub mod menu_bar;
pub mod mesh_editor;
pub mod node_editor;
pub mod osc_panel;
pub mod oscillator_panel;
pub mod output_panel;
pub mod paint_panel;
pub mod theme;
pub mod timeline_v2;
pub mod transform_panel;
pub mod undo_redo;

pub use i18n::LocaleManager;

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
pub use mapping_panel::MappingPanel;
pub use media_browser::{MediaBrowser, MediaBrowserAction, MediaEntry, MediaType};
pub use mesh_editor::{MeshEditor, MeshEditorAction};
pub use node_editor::{Node, NodeEditor, NodeEditorAction, NodeType};
pub use oscillator_panel::OscillatorPanel;
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
    pub mapping_panel: MappingPanel,
    pub show_transforms: bool,      // Phase 1
    pub show_master_controls: bool, // Phase 1
    pub show_outputs: bool,         // Phase 2
    pub output_panel: output_panel::OutputPanel,
    pub edge_blend_panel: EdgeBlendPanel,
    pub oscillator_panel: OscillatorPanel,
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
    pub timeline_panel: timeline_v2::TimelineV2,
    pub node_editor_panel: node_editor::NodeEditor,
    pub transform_panel: TransformPanel,
    pub user_config: config::UserConfig,
    /// Show settings window
    pub show_settings: bool,
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
            mapping_panel: MappingPanel { visible: true },
            show_transforms: true,
            show_master_controls: true,
            show_outputs: true,
            output_panel: output_panel::OutputPanel { visible: true },
            edge_blend_panel: EdgeBlendPanel::default(),
            oscillator_panel: OscillatorPanel { visible: true },
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
            timeline_panel: timeline_v2::TimelineV2::default(),
            node_editor_panel: node_editor::NodeEditor::default(),
            transform_panel: TransformPanel::default(),
            user_config: config::UserConfig::load(),
            show_settings: false,
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
                    &mut self.oscillator_panel.visible,
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
}
