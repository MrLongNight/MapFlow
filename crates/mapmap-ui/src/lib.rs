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
pub mod widgets;

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

use egui;
use sysinfo::{CpuExt, System, SystemExt};

/// UI actions that can betriggered by the user interface
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
    pub show_toolbar: bool,
    pub show_layers: bool,

    pub show_timeline: bool,
    pub show_shader_graph: bool,
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
    pub sys: System,
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
            show_toolbar: true,
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
            show_timeline: true,
            show_shader_graph: true,
            node_editor_panel: node_editor::NodeEditor::default(),
            transform_panel: TransformPanel::default(),
            user_config: config::UserConfig::load(),
            show_settings: false,
            sys: System::new_all(),
        }
    }
}

impl AppUI {
    /// Take all pending actions and clear the list
    pub fn take_actions(&mut self) -> Vec<UIAction> {
        std::mem::take(&mut self.actions)
    }

    /// Render the control panel
    pub fn render_controls(&mut self, ctx: &egui::Context) {
        if !self.show_controls {
            return;
        }

        egui::Window::new(self.i18n.t("panel-playback"))
            .default_size([320.0, 360.0])
            .show(ctx, |ui| {
                ui.heading(self.i18n.t("header-video-playback"));
                ui.separator();

                // Transport controls
                ui.horizontal(|ui| {
                    if ui.button(self.i18n.t("btn-play")).clicked() {
                        self.actions.push(UIAction::Play);
                    }
                    if ui.button(self.i18n.t("btn-pause")).clicked() {
                        self.actions.push(UIAction::Pause);
                    }
                    if ui.button(self.i18n.t("btn-stop")).clicked() {
                        self.actions.push(UIAction::Stop);
                    }
                });

                ui.separator();

                // Speed control
                let old_speed = self.playback_speed;
                ui.add(
                    egui::Slider::new(&mut self.playback_speed, 0.1..=2.0)
                        .text(self.i18n.t("label-speed")),
                );
                if (self.playback_speed - old_speed).abs() > 0.001 {
                    self.actions.push(UIAction::SetSpeed(self.playback_speed));
                }

                // Loop control
                ui.label(self.i18n.t("label-mode"));
                egui::ComboBox::from_label(self.i18n.t("label-mode"))
                    .selected_text(match self.loop_mode {
                        mapmap_media::LoopMode::Loop => self.i18n.t("mode-loop"),
                        mapmap_media::LoopMode::PlayOnce => self.i18n.t("mode-play-once"),
                    })
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(
                                &mut self.loop_mode,
                                mapmap_media::LoopMode::Loop,
                                self.i18n.t("mode-loop"),
                            )
                            .clicked()
                        {
                            self.actions
                                .push(UIAction::SetLoopMode(mapmap_media::LoopMode::Loop));
                        }
                        if ui
                            .selectable_value(
                                &mut self.loop_mode,
                                mapmap_media::LoopMode::PlayOnce,
                                self.i18n.t("mode-play-once"),
                            )
                            .clicked()
                        {
                            self.actions
                                .push(UIAction::SetLoopMode(mapmap_media::LoopMode::PlayOnce));
                        }
                    });
            });
    }

    /// Render performance stats (Phase 6 Migration)
    pub fn render_performance_panel(&mut self, ctx: &egui::Context, fps: f32, frame_time_ms: f32) {
        if !self.show_stats {
            return;
        }

        self.sys.refresh_all();

        egui::Window::new(self.i18n.t("panel-performance"))
            .default_size([250.0, 120.0])
            .show(ctx, |ui| {
                ui.label(format!("{}: {:.1}", self.i18n.t("label-fps"), fps));
                ui.label(format!(
                    "{}: {:.2} ms",
                    self.i18n.t("label-frame-time"),
                    frame_time_ms
                ));

                let total_memory = self.sys.total_memory() as f32 / 1024.0;
                let used_memory = self.sys.used_memory() as f32 / 1024.0;
                let memory_usage = used_memory / total_memory;

                ui.label(format!(
                    "RAM: {:.2} MB / {:.2} MB",
                    used_memory, total_memory
                ));
                widgets::colored_progress_bar(ui, memory_usage);

                let cpu_usage = self.sys.global_cpu_info().cpu_usage() / 100.0;
                ui.label(format!("CPU: {:.2}%", cpu_usage * 100.0));
                widgets::colored_progress_bar(ui, cpu_usage);
            });
    }

    /// Render master controls panel (Phase 6 Migration)
    pub fn render_master_controls(
        &mut self,
        ctx: &egui::Context,
        layer_manager: &mut mapmap_core::LayerManager,
    ) {
        if !self.show_master_controls {
            return;
        }

        egui::Window::new(self.i18n.t("panel-master"))
            .default_size([360.0, 300.0])
            .show(ctx, |ui| {
                ui.heading(self.i18n.t("header-master"));
                ui.separator();

                let composition = &mut layer_manager.composition;

                // Composition name
                ui.label(self.i18n.t("label-composition"));
                ui.label(&composition.name);
                ui.separator();

                // Master Opacity
                let old_master_opacity = composition.master_opacity;
                ui.add(
                    egui::Slider::new(&mut composition.master_opacity, 0.0..=1.0)
                        .text(self.i18n.t("label-master-opacity")),
                );
                if (composition.master_opacity - old_master_opacity).abs() > 0.001 {
                    self.actions
                        .push(UIAction::SetMasterOpacity(composition.master_opacity));
                }

                // Master Speed
                let old_master_speed = composition.master_speed;
                ui.add(
                    egui::Slider::new(&mut composition.master_speed, 0.1..=10.0)
                        .text(self.i18n.t("label-master-speed")),
                );
                if (composition.master_speed - old_master_speed).abs() > 0.001 {
                    self.actions
                        .push(UIAction::SetMasterSpeed(composition.master_speed));
                }

                ui.separator();
                ui.label(format!(
                    "{} {}x{}",
                    self.i18n.t("label-size"),
                    composition.size.0,
                    composition.size.1
                ));
                ui.label(format!(
                    "{} {:.1} fps",
                    self.i18n.t("label-frame-rate"),
                    composition.frame_rate
                ));

                ui.separator();
                ui.label(self.i18n.t("label-effective-multipliers"));
                ui.label(self.i18n.t("text-mult-opacity"));
                ui.label(self.i18n.t("text-mult-speed"));
            });
    }
    /// Render the main Docking Layout (Phase 2)
    pub fn render_docked_layout(
        &mut self,
        ctx: &egui::Context,
        layer_manager: &mut mapmap_core::LayerManager,
        _fps: f32,
        _frame_time_ms: f32,
    ) {
        // Apply Theme and Scaling
        let theme_config = crate::theme::ThemeConfig {
            theme: self.user_config.theme.unwrap_or(crate::theme::Theme::Dark),
            ui_scale: self.user_config.ui_scale.unwrap_or(1.0),
            ..Default::default()
        };
        theme_config.apply(ctx);

        // TOP: Menu
        // MenuBar render creates its own TopBottomPanel internally
        // TOP: Menu
        // Rendered externally in main.rs to avoid borrow checker issues with &mut self

        // LEFT: Monitors & Stats
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading("Monitors");
                ui.separator();
                ui.heading("Output Monitor");
                ui.label("Output Preview (Placeholder)");
            });

        // BOTTOM: Unified Inspector
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(350.0)
            .show(ctx, |ui| {
                self.render_unified_inspector(ui, layer_manager);
            });

        // CENTER: Grid / Layers
        egui::CentralPanel::default().show(ctx, |ui| {
            self.layer_panel.ui(
                ui,
                layer_manager,
                &mut self.selected_layer_id,
                &mut self.actions,
                &self.i18n,
            );
        });
    }

    fn render_unified_inspector(
        &mut self,
        ui: &mut egui::Ui,
        layer_manager: &mut mapmap_core::LayerManager,
    ) {
        // Dashboard
        widgets::render_header(ui, "Dashboard");
        ui.add_space(4.0);
        if let Some(action) = self.dashboard.ui_embedded(ui, &self.i18n) {
            if let crate::dashboard::DashboardAction::AudioDeviceChanged(device) = action {
                self.actions.push(UIAction::SelectAudioDevice(device));
            }
            // TODO: Map other dashboard actions
        }

        // Transport
        ui.separator();
        widgets::render_header(ui, &self.i18n.t("panel-playback"));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button(self.i18n.t("btn-play")).clicked() {
                self.actions.push(UIAction::Play);
            }
            if ui.button(self.i18n.t("btn-pause")).clicked() {
                self.actions.push(UIAction::Pause);
            }
            if ui.button(self.i18n.t("btn-stop")).clicked() {
                self.actions.push(UIAction::Stop);
            }

            ui.separator();
            ui.add(
                egui::Slider::new(&mut self.playback_speed, 0.1..=2.0)
                    .text(self.i18n.t("label-speed")),
            );
        });
        ui.add_space(4.0);

        // Composition
        ui.separator();
        widgets::render_header(ui, &self.i18n.t("header-master"));
        ui.add_space(4.0);
        let composition = &mut layer_manager.composition;
        ui.horizontal(|ui| {
            ui.label(self.i18n.t("label-master-opacity"));
            if widgets::styled_slider(ui, &mut composition.master_opacity, 0.0..=1.0).changed() {
                self.actions
                    .push(UIAction::SetMasterOpacity(composition.master_opacity));
            }
        });
        ui.add_space(4.0);

        // Audio
        ui.separator();
        widgets::render_header(ui, &self.i18n.t("panel-audio"));
        ui.add_space(4.0);
        ui.label("Audio Controls (Refactoring...)");
        // ToDo: Embed AudioPanel::ui here
        ui.add_space(4.0);

        // Effects
        ui.separator();
        widgets::render_header(ui, "Effects");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Effect 1");
            widgets::bypass_button(ui, true);
            widgets::param_button(ui);
            widgets::delete_button(ui);
        });
        ui.label("Effect Chain (Refactoring...)");
        // ToDo: Embed EffectChainPanel::ui here
        ui.add_space(4.0);

        // Transform
        ui.separator();
        widgets::render_header(ui, "Transform");
        ui.add_space(4.0);
        // ToDo: Embed TransformPanel::ui
        ui.label("Transform Controls");
        ui.add_space(4.0);
    }
}
