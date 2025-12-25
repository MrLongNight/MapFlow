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
pub mod icon_demo_panel;
pub mod icons;
pub mod inspector_panel;
pub mod layer_panel;
pub mod mapping_panel;
pub mod media_browser;
pub mod menu_bar;
pub mod mesh_editor;
pub mod module_canvas;
pub mod module_sidebar;
pub mod node_editor;
pub mod osc_panel;
pub mod oscillator_panel;
pub mod output_panel;
pub mod paint_panel;
pub mod shortcut_panel;
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
pub use dashboard::{Dashboard, DashboardAction};
pub use edge_blend_panel::{EdgeBlendAction, EdgeBlendPanel};
pub use effect_chain_panel::{
    EffectChainAction, EffectChainPanel, PresetEntry, UIEffect, UIEffectChain,
};

pub use inspector_panel::{InspectorAction, InspectorContext, InspectorPanel};
pub use layer_panel::{LayerPanel, LayerPanelAction};
pub use mapping_panel::MappingPanel;
pub use media_browser::{MediaBrowser, MediaBrowserAction, MediaEntry, MediaType};
pub use mesh_editor::{MeshEditor, MeshEditorAction};
pub use module_canvas::ModuleCanvas;
pub use module_sidebar::ModuleSidebar;
pub use node_editor::{Node, NodeEditor, NodeEditorAction, NodeType};
pub use oscillator_panel::OscillatorPanel;
pub use paint_panel::PaintPanel;
pub use shortcut_panel::{ShortcutAction, ShortcutPanel};
pub use theme::{Theme, ThemeConfig};
pub use transform_panel::{TransformAction, TransformPanel};
pub use undo_redo::{Command, CommandError, EditorState, UndoManager};

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
    ToggleModuleCanvas,

    // Audio actions
    SelectAudioDevice(String),
    UpdateAudioConfig(mapmap_core::audio::AudioConfig),
    ToggleAudioPanel,

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
    pub show_layers: bool,
    pub show_toolbar: bool,

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
    pub shortcut_panel: ShortcutPanel,
    pub icon_manager: Option<icons::IconManager>,
    pub icon_demo_panel: icon_demo_panel::IconDemoPanel,
    pub user_config: config::UserConfig,
    /// Show settings window
    pub show_settings: bool,
    pub show_media_browser: bool,
    pub media_browser: MediaBrowser,
    /// Inspector panel for context-sensitive properties
    pub inspector_panel: InspectorPanel,
    pub show_inspector: bool,
    pub module_sidebar: ModuleSidebar,
    pub show_module_sidebar: bool,
    pub module_canvas: ModuleCanvas,
    pub show_module_canvas: bool,
}

impl Default for AppUI {
    fn default() -> Self {
        Self {
            menu_bar: menu_bar::MenuBar::default(),
            dashboard: Dashboard::default(),
            paint_panel: PaintPanel::default(),
            show_osc_panel: false, // Hide by default - advanced feature
            selected_control_target: ControlTarget::Custom("".to_string()),
            osc_port_input: "8000".to_string(),
            osc_client_input: "127.0.0.1:9000".to_string(),
            show_controls: false, // Hide by default - use Dashboard instead
            show_stats: true,     // Keep performance overlay
            show_layers: true,
            layer_panel: LayerPanel { visible: true },
            show_mappings: false, // Hide by default - use Inspector when ready
            mapping_panel: MappingPanel { visible: false },
            show_transforms: false,     // Hide - will move to Inspector
            show_master_controls: true, // Keep visible
            show_outputs: false,        // Hide by default
            output_panel: output_panel::OutputPanel { visible: false },
            edge_blend_panel: EdgeBlendPanel::default(),
            oscillator_panel: OscillatorPanel { visible: false }, // Hide by default
            show_audio: false, // Hide by default - use Dashboard toggle
            audio_panel: AudioPanel::default(),
            show_cue_panel: false, // Hide by default
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
            show_timeline: true,      // Essential panel
            show_shader_graph: false, // Advanced - hide by default
            node_editor_panel: node_editor::NodeEditor::default(),
            transform_panel: TransformPanel::default(),
            shortcut_panel: ShortcutPanel::new(),
            show_toolbar: true,
            icon_manager: None, // Will be initialized with egui context
            icon_demo_panel: icon_demo_panel::IconDemoPanel::default(),
            user_config: config::UserConfig::load(),
            show_settings: false,
            show_media_browser: true, // Essential panel
            media_browser: MediaBrowser::new(std::env::current_dir().unwrap_or_default()),
            inspector_panel: InspectorPanel::default(),
            show_inspector: true, // Essential panel
            module_sidebar: ModuleSidebar::default(),
            show_module_sidebar: true, // Show when Module Canvas is active
            module_canvas: ModuleCanvas::default(),
            show_module_canvas: false,
        }
    }
}

impl AppUI {
    /// Take all pending actions and clear the list
    pub fn take_actions(&mut self) -> Vec<UIAction> {
        std::mem::take(&mut self.actions)
    }

    /// Initialize the icon manager with the egui context
    pub fn initialize_icons(&mut self, ctx: &egui::Context, assets_dir: &std::path::Path) {
        if self.icon_manager.is_none() {
            self.icon_manager = Some(icons::IconManager::new(ctx, assets_dir, 64));
        }
    }

    /// Render the icon demo panel
    pub fn render_icon_demo(&mut self, ctx: &egui::Context) {
        self.icon_demo_panel
            .ui(ctx, self.icon_manager.as_ref(), &self.i18n);
    }

    /// Toggle icon demo panel visibility
    pub fn toggle_icon_demo(&mut self) {
        self.icon_demo_panel.visible = !self.icon_demo_panel.visible;
    }

    /// Render the media browser as left side panel
    pub fn render_media_browser(&mut self, ctx: &egui::Context) {
        if !self.show_media_browser {
            return;
        }

        egui::SidePanel::left("media_browser_panel")
            .resizable(true)
            .default_width(280.0)
            .min_width(200.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(self.i18n.t("panel-media-browser"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            self.show_media_browser = false;
                        }
                    });
                });
                ui.separator();
                let _ = self
                    .media_browser
                    .ui(ui, &self.i18n, self.icon_manager.as_ref());
            });
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

    /// Render performance stats as top-right overlay (Phase 6 Migration)
    pub fn render_stats_overlay(&mut self, ctx: &egui::Context, fps: f32, frame_time_ms: f32) {
        if !self.show_stats {
            return;
        }

        // Use Area with anchor to position in top-right corner
        egui::Area::new(egui::Id::new("performance_overlay"))
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 50.0]) // Offset from menu bar
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 220))
                    .rounding(4.0)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 80)))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("FPS: {:.0}", fps))
                                    .color(egui::Color32::from_rgb(100, 200, 100))
                                    .strong(),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("{:.1}ms", frame_time_ms))
                                    .color(egui::Color32::from_rgb(150, 150, 200)),
                            );
                        });
                    });
            });
    }

    /// Legacy floating window version (deprecated)
    pub fn render_stats(&mut self, ctx: &egui::Context, fps: f32, frame_time_ms: f32) {
        // Redirect to overlay version
        self.render_stats_overlay(ctx, fps, frame_time_ms);
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
}
