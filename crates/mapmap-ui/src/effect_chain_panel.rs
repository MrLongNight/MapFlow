//! Effect Chain UI Panel
//!
//! egui-based panel for managing effect chains with drag & drop reordering,
//! parameter sliders, and preset browser.

use crate::i18n::LocaleManager;
use crate::icons::{AppIcon, IconManager};
use egui::{Color32, RichText, Ui};
use serde::{Deserialize, Serialize};

/// Available effect types (mirror of mapmap-render::EffectType)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    ColorAdjust,
    Blur,
    ChromaticAberration,
    EdgeDetect,
    Glow,
    Kaleidoscope,
    Invert,
    Pixelate,
    Vignette,
    FilmGrain,
    Wave,
    Glitch,
    RgbSplit,
    Mirror,
    HueShift,
    Custom,
}

impl EffectType {
    pub fn display_name(&self, locale: &LocaleManager) -> String {
        match self {
            EffectType::ColorAdjust => locale.t("effect-name-color-adjust"),
            EffectType::Blur => locale.t("effect-name-blur"),
            EffectType::ChromaticAberration => locale.t("effect-name-chromatic-aberration"),
            EffectType::EdgeDetect => locale.t("effect-name-edge-detect"),
            EffectType::Glow => locale.t("effect-name-glow"),
            EffectType::Kaleidoscope => locale.t("effect-name-kaleidoscope"),
            EffectType::Invert => locale.t("effect-name-invert"),
            EffectType::Pixelate => locale.t("effect-name-pixelate"),
            EffectType::Vignette => locale.t("effect-name-vignette"),
            EffectType::FilmGrain => locale.t("effect-name-film-grain"),
            EffectType::Wave => locale.t("effect-name-wave"),
            EffectType::Glitch => locale.t("effect-name-glitch"),
            EffectType::RgbSplit => locale.t("effect-name-rgb-split"),
            EffectType::Mirror => locale.t("effect-name-mirror"),
            EffectType::HueShift => locale.t("effect-name-hue-shift"),
            EffectType::Custom => locale.t("effect-name-custom"),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            EffectType::ColorAdjust => "🎨",
            EffectType::Blur => "🌫️",
            EffectType::ChromaticAberration => "🌈",
            EffectType::EdgeDetect => "📐",
            EffectType::Glow => "✨",
            EffectType::Kaleidoscope => "🔮",
            EffectType::Invert => "🔄",
            EffectType::Pixelate => "🟩",
            EffectType::Vignette => "🌑",
            EffectType::FilmGrain => "🎞️",
            EffectType::Wave => "🌊",
            EffectType::Glitch => "👾",
            EffectType::RgbSplit => "🌈",
            EffectType::Mirror => "🪞",
            EffectType::HueShift => "🎨",
            EffectType::Custom => "⚙️",
        }
    }

    pub fn app_icon(&self) -> AppIcon {
        match self {
            EffectType::ColorAdjust => AppIcon::MagicWand,
            EffectType::Blur => AppIcon::MagicWand,
            EffectType::ChromaticAberration => AppIcon::MagicWand,
            EffectType::EdgeDetect => AppIcon::Pencil,
            EffectType::Glow => AppIcon::MagicWand,
            EffectType::Kaleidoscope => AppIcon::MagicWand,
            EffectType::Invert => AppIcon::Repeat,
            EffectType::Pixelate => AppIcon::Screen,
            EffectType::Vignette => AppIcon::AppWindow,
            EffectType::FilmGrain => AppIcon::VideoFile,
            EffectType::Wave => AppIcon::MagicWand,
            EffectType::Glitch => AppIcon::Screen,
            EffectType::RgbSplit => AppIcon::MagicWand,
            EffectType::Mirror => AppIcon::Repeat,
            EffectType::HueShift => AppIcon::PaintBucket,
            EffectType::Custom => AppIcon::Cog,
        }
    }

    pub fn all() -> &'static [EffectType] {
        &[
            EffectType::ColorAdjust,
            EffectType::Blur,
            EffectType::ChromaticAberration,
            EffectType::EdgeDetect,
            EffectType::Glow,
            EffectType::Kaleidoscope,
            EffectType::Invert,
            EffectType::Pixelate,
            EffectType::Vignette,
            EffectType::FilmGrain,
            EffectType::Wave,
            EffectType::Glitch,
            EffectType::RgbSplit,
            EffectType::Mirror,
            EffectType::HueShift,
            EffectType::Custom,
        ]
    }
}

/// Effect instance for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIEffect {
    pub id: u64,
    pub effect_type: EffectType,
    pub enabled: bool,
    pub intensity: f32,
    pub expanded: bool,
    pub parameters: std::collections::HashMap<String, f32>,
}

impl UIEffect {
    pub fn new(id: u64, effect_type: EffectType) -> Self {
        let mut parameters = std::collections::HashMap::new();

        // Default parameters
        match effect_type {
            EffectType::ColorAdjust => {
                parameters.insert("brightness".to_string(), 0.0);
                parameters.insert("contrast".to_string(), 1.0);
                parameters.insert("saturation".to_string(), 1.0);
            }
            EffectType::Blur => {
                parameters.insert("radius".to_string(), 5.0);
            }
            EffectType::ChromaticAberration => {
                parameters.insert("amount".to_string(), 0.01);
            }
            EffectType::Glow => {
                parameters.insert("threshold".to_string(), 0.5);
                parameters.insert("radius".to_string(), 10.0);
            }
            EffectType::Kaleidoscope => {
                parameters.insert("segments".to_string(), 6.0);
                parameters.insert("rotation".to_string(), 0.0);
            }
            EffectType::Pixelate => {
                parameters.insert("pixel_size".to_string(), 8.0);
            }
            EffectType::Vignette => {
                parameters.insert("radius".to_string(), 0.5);
                parameters.insert("softness".to_string(), 0.5);
            }
            EffectType::FilmGrain => {
                parameters.insert("amount".to_string(), 0.1);
                parameters.insert("speed".to_string(), 1.0);
            }
            _ => {}
        }

        Self {
            id,
            effect_type,
            enabled: true,
            intensity: 1.0,
            expanded: true,
            parameters,
        }
    }

    pub fn get_param(&self, name: &str, default: f32) -> f32 {
        *self.parameters.get(name).unwrap_or(&default)
    }

    pub fn set_param(&mut self, name: &str, value: f32) {
        self.parameters.insert(name.to_string(), value);
    }
}

/// Effect chain for UI
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UIEffectChain {
    pub effects: Vec<UIEffect>,
    next_id: u64,
}

impl UIEffectChain {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_effect(&mut self, effect_type: EffectType) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.effects.push(UIEffect::new(id, effect_type));
        id
    }

    pub fn remove_effect(&mut self, id: u64) {
        self.effects.retain(|e| e.id != id);
    }

    pub fn move_up(&mut self, id: u64) {
        if let Some(pos) = self.effects.iter().position(|e| e.id == id) {
            if pos > 0 {
                self.effects.swap(pos, pos - 1);
            }
        }
    }

    pub fn move_down(&mut self, id: u64) {
        if let Some(pos) = self.effects.iter().position(|e| e.id == id) {
            if pos < self.effects.len() - 1 {
                self.effects.swap(pos, pos + 1);
            }
        }
    }

    pub fn get_effect_mut(&mut self, id: u64) -> Option<&mut UIEffect> {
        self.effects.iter_mut().find(|e| e.id == id)
    }
}

/// Actions from the effect chain panel
#[derive(Debug, Clone)]
pub enum EffectChainAction {
    /// Add a new effect of the given type
    AddEffect(EffectType),
    /// Add a new effect with specific parameters
    AddEffectWithParams(EffectType, std::collections::HashMap<String, f32>),
    /// Remove an effect by ID
    RemoveEffect(u64),
    /// Move effect up in chain
    MoveUp(u64),
    /// Move effect down in chain
    MoveDown(u64),
    /// Toggle effect enabled state
    ToggleEnabled(u64),
    /// Set effect intensity
    SetIntensity(u64, f32),
    /// Set effect parameter
    SetParameter(u64, String, f32),
    /// Load a preset by name
    LoadPreset(String),
    /// Save current chain as preset
    SavePreset(String),
    /// Clear all effects
    ClearAll,
}

/// Preset entry for the browser
#[derive(Debug, Clone)]
pub struct PresetEntry {
    pub name: String,
    pub category: String,
    pub path: String,
    pub is_favorite: bool,
}

/// Effect Chain Panel
#[derive(Default, Debug)]
pub struct EffectChainPanel {
    /// Current effect chain
    pub chain: UIEffectChain,

    /// Whether the panel is visible
    pub visible: bool,

    /// Show add effect menu
    show_add_menu: bool,

    /// Show preset browser
    show_preset_browser: bool,

    /// Preset search query
    preset_search: String,

    /// Available presets
    presets: Vec<PresetEntry>,

    /// Currently dragging effect ID
    #[allow(dead_code)]
    dragging_effect: Option<u64>,

    /// Save preset name input
    save_preset_name: String,

    /// Pending actions
    actions: Vec<EffectChainAction>,
}

impl EffectChainPanel {
    pub fn new() -> Self {
        Self {
            chain: UIEffectChain::new(),
            visible: true,
            show_add_menu: false,
            show_preset_browser: false,
            preset_search: String::new(),
            presets: Vec::new(),
            dragging_effect: None,
            save_preset_name: String::new(),
            actions: Vec::new(),
        }
    }

    /// Set available presets
    pub fn set_presets(&mut self, presets: Vec<PresetEntry>) {
        self.presets = presets;
    }

    /// Take all pending actions
    pub fn take_actions(&mut self) -> Vec<EffectChainAction> {
        std::mem::take(&mut self.actions)
    }

    /// Render the effect chain panel
    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        locale: &LocaleManager,
        icon_manager: Option<&IconManager>,
        mut recent_configs: Option<&mut mapmap_core::RecentEffectConfigs>,
    ) {
        if !self.visible {
            return;
        }

        egui::Window::new(locale.t("panel-effect-chain"))
            .default_size([320.0, 500.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_toolbar(ui, locale, icon_manager, &mut recent_configs);
                ui.separator();
                self.render_effect_list(ui, locale, icon_manager);
                ui.separator();
                self.render_footer(ui, locale, icon_manager);
            });

        // Render popups
        if self.show_preset_browser {
            self.render_preset_browser(ctx, locale, icon_manager);
        }
    }

    fn render_toolbar(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        icon_manager: Option<&IconManager>,
        recent_configs: &mut Option<&mut mapmap_core::RecentEffectConfigs>,
    ) {
        ui.horizontal(|ui| {
            // Add effect button
            if ui
                .button(locale.t("effect-add"))
                .on_hover_text(locale.t("effect-add"))
                .clicked()
            {
                self.show_add_menu = !self.show_add_menu;
            }

            // Preset buttons
            if ui
                .button(locale.t("effect-presets"))
                .on_hover_text(locale.t("effect-presets"))
                .clicked()
            {
                self.show_preset_browser = !self.show_preset_browser;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(mgr) = icon_manager {
                    if let Some(img) = mgr.image(AppIcon::Remove, 16.0) {
                        if ui
                            .add(egui::Button::image(img))
                            .on_hover_text(locale.t("effect-clear"))
                            .clicked()
                        {
                            self.actions.push(EffectChainAction::ClearAll);
                            self.chain.effects.clear();
                        }
                    }
                }
            });
        });

        // Add effect menu
        if self.show_add_menu {
            ui.group(|ui| {
                ui.label(locale.t("effect-select-type"));
                ui.horizontal_wrapped(|ui| {
                    for effect_type in EffectType::all() {
                        let label = effect_type.display_name(locale);
                        if let Some(mgr) = icon_manager {
                            if let Some(img) = mgr.image(effect_type.app_icon(), 16.0) {
                                let btn = ui.add(egui::Button::image_and_text(img, label));
                                if btn.clicked() {
                                    self.chain.add_effect(*effect_type);
                                    self.actions
                                        .push(EffectChainAction::AddEffect(*effect_type));
                                    self.show_add_menu = false;
                                }

                                // Show context menu for recent configs on right click
                                btn.context_menu(|ui| {
                                    ui.label("Recent Configurations:");
                                    if let Some(recent) = recent_configs {
                                        let type_name = format!("{:?}", effect_type);
                                        let configs = recent.get_recent(&type_name);

                                        if configs.is_empty() {
                                            ui.label("No recent configs");
                                        } else {
                                            for config in configs {
                                                if ui.button(config.name.to_string()).on_hover_text(format!("{:?}", config.params)).clicked() {
                                                     self.chain.add_effect(*effect_type);

                                                     let id = self.chain.effects.last().unwrap().id;
                                                     let effect = self.chain.get_effect_mut(id).unwrap();

                                                     let mut f32_params = std::collections::HashMap::new();
                                                     for (k, v) in &config.params {
                                                         if let mapmap_core::recent_effect_configs::EffectParamValue::Float(f) = v {
                                                             effect.set_param(k, *f);
                                                             f32_params.insert(k.clone(), *f);
                                                         }
                                                     }

                                                     self.actions.push(EffectChainAction::AddEffectWithParams(*effect_type, f32_params));
                                                     ui.close();
                                                     self.show_add_menu = false;
                                                }
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    }
                });
            });
        }
    }

    fn render_effect_list(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        icon_manager: Option<&IconManager>,
    ) {
        egui::ScrollArea::vertical()
            .max_height(350.0)
            .show(ui, |ui| {
                if self.chain.effects.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(
                            RichText::new(locale.t("effect-no-effects"))
                                .size(16.0)
                                .weak(),
                        );
                        ui.label(locale.t("effect-start-tip"));
                        ui.add_space(50.0);
                    });
                } else {
                    let mut effect_to_remove = None;
                    let mut effect_to_move_up = None;
                    let mut effect_to_move_down = None;

                    // Collect effect data to avoid borrow issues
                    let effect_count = self.chain.effects.len();

                    for idx in 0..effect_count {
                        let is_first = idx == 0;
                        let is_last = idx == effect_count - 1;

                        let effect = &mut self.chain.effects[idx];
                        let effect_id = effect.id;
                        let effect_type = effect.effect_type;
                        let enabled = effect.enabled;
                        let expanded = effect.expanded;
                        let intensity = effect.intensity;

                        let (
                            remove,
                            move_up,
                            move_down,
                            new_enabled,
                            new_expanded,
                            new_intensity,
                            param_changes,
                        ) = Self::render_effect_card_static(
                            ui,
                            effect_id,
                            effect_type,
                            enabled,
                            expanded,
                            intensity,
                            &effect.parameters,
                            is_first,
                            is_last,
                            locale,
                            icon_manager,
                        );

                        // Apply changes
                        let effect = &mut self.chain.effects[idx];
                        if new_enabled != enabled {
                            effect.enabled = new_enabled;
                            self.actions
                                .push(EffectChainAction::ToggleEnabled(effect_id));
                        }
                        if new_expanded != expanded {
                            effect.expanded = new_expanded;
                        }
                        if (new_intensity - intensity).abs() > 0.001 {
                            effect.intensity = new_intensity;
                            self.actions
                                .push(EffectChainAction::SetIntensity(effect_id, new_intensity));
                        }
                        for (name, value) in param_changes {
                            effect.set_param(&name, value);
                            self.actions
                                .push(EffectChainAction::SetParameter(effect_id, name, value));
                        }

                        if remove {
                            effect_to_remove = Some(effect_id);
                        }
                        if move_up {
                            effect_to_move_up = Some(effect_id);
                        }
                        if move_down {
                            effect_to_move_down = Some(effect_id);
                        }
                    }

                    // Apply pending operations
                    if let Some(id) = effect_to_remove {
                        self.chain.remove_effect(id);
                        self.actions.push(EffectChainAction::RemoveEffect(id));
                    }
                    if let Some(id) = effect_to_move_up {
                        self.chain.move_up(id);
                        self.actions.push(EffectChainAction::MoveUp(id));
                    }
                    if let Some(id) = effect_to_move_down {
                        self.chain.move_down(id);
                        self.actions.push(EffectChainAction::MoveDown(id));
                    }
                }
            });
    }

    /// Static rendering function that doesn't borrow self
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    fn render_effect_card_static(
        ui: &mut Ui,
        effect_id: u64,
        effect_type: EffectType,
        mut enabled: bool,
        mut expanded: bool,
        mut intensity: f32,
        parameters: &std::collections::HashMap<String, f32>,
        is_first: bool,
        is_last: bool,
        locale: &LocaleManager,
        icon_manager: Option<&IconManager>,
    ) -> (bool, bool, bool, bool, bool, f32, Vec<(String, f32)>) {
        let mut remove = false;
        let mut move_up = false;
        let mut move_down = false;
        let mut param_changes = Vec::new();

        let frame_color = if enabled {
            Color32::from_rgba_premultiplied(60, 80, 120, 200)
        } else {
            Color32::from_rgba_premultiplied(60, 60, 60, 150)
        };

        egui::Frame::NONE
            .fill(frame_color)
            .corner_radius(8.0)
            .inner_margin(8.0)
            .outer_margin(2.0)
            .show(ui, |ui| {
                // Header row
                ui.horizontal(|ui| {
                    // Enable toggle
                    ui.checkbox(&mut enabled, "");

                    // Effect name with icon
                    let header_text = effect_type.display_name(locale);
                    if let Some(mgr) = icon_manager {
                        if let Some(img) = mgr.image(effect_type.app_icon(), 16.0) {
                            if ui
                                .add(
                                    egui::Button::image_and_text(img, header_text)
                                        .selected(expanded),
                                )
                                .clicked()
                            {
                                expanded = !expanded;
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete button
                        if let Some(mgr) = icon_manager {
                            if let Some(img) = mgr.image(AppIcon::Remove, 16.0) {
                                if ui.add(egui::Button::image(img)).clicked() {
                                    remove = true;
                                }
                            }
                        }

                        // Move buttons
                        ui.add_enabled_ui(!is_last, |ui| {
                            if ui.small_button("▼").clicked() {
                                move_down = true;
                            }
                        });
                        ui.add_enabled_ui(!is_first, |ui| {
                            if ui.small_button("▲").clicked() {
                                move_up = true;
                            }
                        });
                    });
                });

                // Expanded content
                if expanded {
                    ui.separator();

                    // Intensity slider
                    ui.horizontal(|ui| {
                        ui.label(locale.t("effect-intensity"));
                        ui.add(egui::Slider::new(&mut intensity, 0.0..=1.0));
                    });

                    // Effect-specific parameters
                    Self::render_effect_parameters_static(
                        ui,
                        effect_type,
                        effect_id,
                        parameters,
                        &mut param_changes,
                        locale,
                    );
                }
            });

        (
            remove,
            move_up,
            move_down,
            enabled,
            expanded,
            intensity,
            param_changes,
        )
    }

    fn render_effect_parameters_static(
        ui: &mut Ui,
        effect_type: EffectType,
        effect_id: u64,
        parameters: &std::collections::HashMap<String, f32>,
        param_changes: &mut Vec<(String, f32)>,
        locale: &LocaleManager,
    ) {
        match effect_type {
            EffectType::ColorAdjust => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "brightness",
                    &locale.t("param-brightness"),
                    -1.0,
                    1.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "contrast",
                    &locale.t("param-contrast"),
                    0.0,
                    2.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "saturation",
                    &locale.t("param-saturation"),
                    0.0,
                    2.0,
                );
            }
            EffectType::Blur => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "radius",
                    &locale.t("param-radius"),
                    0.0,
                    20.0,
                );
            }
            EffectType::ChromaticAberration => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "amount",
                    &locale.t("param-amount"),
                    0.0,
                    0.1,
                );
            }
            EffectType::Glow => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "threshold",
                    &locale.t("param-threshold"),
                    0.0,
                    1.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "radius",
                    &locale.t("param-radius"),
                    0.0,
                    30.0,
                );
            }
            EffectType::Kaleidoscope => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "segments",
                    &locale.t("param-segments"),
                    2.0,
                    16.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "rotation",
                    &locale.t("param-rotation"),
                    0.0,
                    360.0,
                );
            }
            EffectType::Pixelate => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "pixel_size",
                    &locale.t("param-pixel-size"),
                    1.0,
                    64.0,
                );
            }
            EffectType::Vignette => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "radius",
                    &locale.t("param-radius"),
                    0.0,
                    1.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "softness",
                    &locale.t("param-softness"),
                    0.0,
                    1.0,
                );
            }
            EffectType::FilmGrain => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "amount",
                    &locale.t("param-amount"),
                    0.0,
                    0.5,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "speed",
                    &locale.t("param-speed"),
                    0.0,
                    5.0,
                );
            }
            EffectType::Wave => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "frequency",
                    &locale.t("param-frequency"),
                    0.0,
                    50.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "amplitude",
                    &locale.t("param-amplitude"),
                    0.0,
                    2.0,
                );
            }
            EffectType::Glitch => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "block_size",
                    &locale.t("param-block-size"),
                    1.0,
                    50.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "color_shift",
                    &locale.t("param-color-shift"),
                    0.0,
                    20.0,
                );
            }
            EffectType::RgbSplit => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "offset_x",
                    &locale.t("param-offset-x"),
                    -50.0,
                    50.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "offset_y",
                    &locale.t("param-offset-y"),
                    -50.0,
                    50.0,
                );
            }
            EffectType::Mirror => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "center",
                    &locale.t("param-center"),
                    0.0,
                    1.0,
                );
            }
            EffectType::HueShift => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "hue_shift",
                    &locale.t("param-hue-shift"),
                    0.0,
                    1.0,
                );
            }
            _ => {
                ui.label(locale.t("no-parameters")); // NOTE: Check if key exists or add it
            }
        }
        let _ = effect_id; // Silence unused warning
    }

    fn render_param_slider_static(
        ui: &mut Ui,
        parameters: &std::collections::HashMap<String, f32>,
        param_changes: &mut Vec<(String, f32)>,
        param_name: &str,
        label: &str,
        min: f32,
        max: f32,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            let old_value = *parameters.get(param_name).unwrap_or(&0.0);
            let mut value = old_value;
            ui.add(egui::Slider::new(&mut value, min..=max));
            if (value - old_value).abs() > 0.0001 {
                param_changes.push((param_name.to_string(), value));
            }
        });
    }

    fn render_footer(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        icon_manager: Option<&IconManager>,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!(
                "{} {}",
                self.chain.effects.len(),
                locale.t("panel-effect-chain")
            ));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Save preset button
                if let Some(mgr) = icon_manager {
                    if let Some(img) = mgr.image(AppIcon::FloppyDisk, 16.0) {
                        if ui
                            .add(egui::Button::image(img))
                            .on_hover_text(locale.t("effect-save"))
                            .clicked()
                        {
                            self.show_preset_browser = true;
                        }
                    }
                }
            });
        });
    }

    fn render_preset_browser(
        &mut self,
        ctx: &egui::Context,
        locale: &LocaleManager,
        icon_manager: Option<&IconManager>,
    ) {
        let mut close_browser = false;
        let mut load_preset_path: Option<String> = None;

        let mut open = self.show_preset_browser;
        egui::Window::new(locale.t("effect-presets-browser"))
            .default_size([400.0, 300.0])
            .resizable(true)
            .open(&mut open)
            .show(ctx, |ui| {
                // Search bar
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.preset_search)
                            .hint_text(locale.t("effect-search")),
                    );
                });

                ui.separator();

                // Preset list
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let search_lower = self.preset_search.to_lowercase();

                        for preset in &self.presets {
                            if !self.preset_search.is_empty()
                                && !preset.name.to_lowercase().contains(&search_lower)
                            {
                                continue;
                            }

                            ui.horizontal(|ui| {
                                let star = if preset.is_favorite { "⭐" } else { "☆" };
                                ui.label(star);

                                if ui.button(&preset.name).clicked() {
                                    load_preset_path = Some(preset.path.clone());
                                    close_browser = true;
                                }

                                ui.weak(&preset.category);
                            });
                        }

                        if self.presets.is_empty() {
                            ui.label(locale.t("effect-no-presets"));
                        }
                    });

                ui.separator();

                // Save new preset
                ui.horizontal(|ui| {
                    ui.label(locale.t("effect-save-as"));
                    ui.text_edit_singleline(&mut self.save_preset_name);
                    if let Some(mgr) = icon_manager {
                        if let Some(img) = mgr.image(AppIcon::FloppyDisk, 16.0) {
                            if ui
                                .add(egui::Button::image(img))
                                .on_hover_text(locale.t("effect-save"))
                                .clicked()
                                && !self.save_preset_name.is_empty()
                            {
                                self.actions.push(EffectChainAction::SavePreset(
                                    self.save_preset_name.clone(),
                                ));
                                self.save_preset_name.clear();
                            }
                        }
                    }
                });
            });

        self.show_preset_browser = open;

        if let Some(path) = load_preset_path {
            self.actions.push(EffectChainAction::LoadPreset(path));
        }
        if close_browser {
            self.show_preset_browser = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_effect_chain_creation() {
        let mut chain = UIEffectChain::new();

        let id1 = chain.add_effect(EffectType::Blur);
        let id2 = chain.add_effect(EffectType::ColorAdjust);

        assert_eq!(chain.effects.len(), 2);
        assert_eq!(chain.effects[0].id, id1);
        assert_eq!(chain.effects[1].id, id2);
    }

    #[test]
    fn test_ui_effect_chain_reorder() {
        let mut chain = UIEffectChain::new();

        let id1 = chain.add_effect(EffectType::Blur);
        let id2 = chain.add_effect(EffectType::ColorAdjust);

        chain.move_up(id2);

        assert_eq!(chain.effects[0].id, id2);
        assert_eq!(chain.effects[1].id, id1);
    }

    #[test]
    fn test_effect_panel_actions() {
        let mut panel = EffectChainPanel::new();

        panel.chain.add_effect(EffectType::Blur);
        panel
            .actions
            .push(EffectChainAction::AddEffect(EffectType::Blur));

        let actions = panel.take_actions();
        assert_eq!(actions.len(), 1);
        assert!(panel.actions.is_empty());
    }
}
