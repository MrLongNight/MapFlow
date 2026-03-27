use super::models::*;
use super::types::EffectType;
use crate::core::responsive::ResponsiveLayout;
use crate::i18n::LocaleManager;
use crate::icons::{AppIcon, IconManager};
use crate::theme::colors;
use crate::widgets::custom::icon_button_simple;
use crate::widgets::panel::{cyber_panel_frame, render_panel_header};
use egui::{CornerRadius, Ui};

/// Effect Chain Panel
#[derive(Default, Debug)]
pub struct EffectChainPanel {
    pub(crate) actions: Vec<EffectChainAction>,
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
    dragging_effect: Option<u64>,

    /// Save preset name input
    save_preset_name: String,
}

impl EffectChainPanel {
    /// Creates a new, uninitialized instance with default settings.
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
        mut recent_configs: Option<&mut vorce_core::RecentEffectConfigs>,
    ) {
        if !self.visible {
            return;
        }

        let layout = ResponsiveLayout::new(ctx);
        let window_size = layout.window_size(400.0, 600.0);

        egui::Window::new(locale.t("panel-effect-chain"))
            .default_size(window_size)
            .resizable(true)
            .scroll([false, true])
            .frame(cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                render_panel_header(ui, &locale.t("panel-effect-chain"), |_| {});

                ui.add_space(8.0);

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
        recent_configs: &mut Option<&mut vorce_core::RecentEffectConfigs>,
    ) {
        ui.horizontal(|ui| {
            // Add effect button
            if ui
                .button(locale.t("effect-add"))
                .clone()
                .on_hover_text(locale.t("effect-add"))
                .clicked()
            {
                self.show_add_menu = !self.show_add_menu;
            }

            // Preset buttons
            if ui
                .button(locale.t("effect-presets"))
                .clone()
                .on_hover_text(locale.t("effect-presets"))
                .clicked()
            {
                self.show_preset_browser = !self.show_preset_browser;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::widgets::custom::hold_to_action_icon(
                    ui,
                    icon_manager,
                    AppIcon::Remove,
                    16.0,
                    crate::theme::colors::WARN_COLOR,
                    &locale.t("effect-clear"),
                ) {
                    self.actions.push(EffectChainAction::ClearAll);
                    self.chain.effects.clear();
                }
            });
        });

        // Add effect menu
        if self.show_add_menu {
            // Replaced ui.group with egui::Frame for Cyber Dark Theme
            egui::Frame::default()
                .fill(colors::DARK_GREY)
                .stroke(egui::Stroke::new(1.0, colors::STROKE_GREY))
                .corner_radius(CornerRadius::ZERO)
                .inner_margin(8.0)
                .show(ui, |ui| {
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
                                                crate::widgets::custom::render_info_label(ui, "No recent configs");
                                            } else {
                                                for config in configs {
                                                    if ui.button(config.name.to_string()).clone().on_hover_text(format!("{:?}", config.params)).clicked() {
                                                         self.chain.add_effect(*effect_type);

                                                         let id = self.chain.effects.last().unwrap().id;
                                                         let effect = self.chain.get_effect_mut(id).unwrap();

                                                         let mut f32_params = std::collections::HashMap::new();
                                                         for (k, v) in &config.params {
                                                             if let vorce_core::recent_effect_configs::EffectParamValue::Float(f) = v {
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
                        crate::widgets::custom::render_info_label_with_size(
                            ui,
                            &locale.t("effect-no-effects"),
                            16.0,
                        );
                        ui.label(locale.t("effect-start-tip"));
                        ui.add_space(50.0);
                    });
                } else {
                    let mut effect_to_remove = None;
                    let mut effect_to_move_up = None;
                    let mut effect_to_move_down = None;
                    let mut drag_started_id = None;
                    let mut swap_request = None; // (from_id, to_idx)

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
                        let is_dragging = self.dragging_effect == Some(effect_id);

                        let (
                            remove,
                            reset,
                            move_up,
                            move_down,
                            dragged,
                            card_rect,
                            new_enabled,
                            new_expanded,
                            new_intensity,
                            param_changes,
                            new_lut_path,
                            new_error,
                        ) = Self::render_effect_card_static(
                            ui,
                            effect_id,
                            effect_type,
                            enabled,
                            expanded,
                            intensity,
                            &effect.lut_path,
                            effect.error.as_ref(),
                            &effect.parameters,
                            is_first,
                            is_last,
                            is_dragging,
                            idx,
                            locale,
                            icon_manager,
                        );

                        if dragged {
                            drag_started_id = Some(effect_id);
                        }

                        // Handle swap on hover
                        if let Some(dragging_id) = self.dragging_effect {
                            if dragging_id != effect_id
                                && card_rect.contains(
                                    ui.input(|i| i.pointer.interact_pos().unwrap_or_default()),
                                )
                            {
                                // Only swap if we are hovering over the middle of the card to prevent flickering
                                // or just simple containment for now
                                swap_request = Some((dragging_id, idx));
                            }
                        }

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

                        if let Some(path) = new_lut_path {
                            effect.lut_path = path.clone();
                            self.actions
                                .push(EffectChainAction::SetLUTPath(effect_id, path));
                        }

                        if let Some(err) = new_error {
                            effect.error = err;
                        }

                        if reset {
                            for (k, v) in effect_type.default_params() {
                                effect.set_param(&k, v);
                            }
                            self.actions.push(EffectChainAction::ResetEffect(effect_id));
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

                    // Handle drag state updates
                    if let Some(id) = drag_started_id {
                        self.dragging_effect = Some(id);
                    }

                    if ui.input(|i| i.pointer.any_released()) {
                        self.dragging_effect = None;
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
                    if let Some((from_id, to_idx)) = swap_request {
                        if let Some(from_idx) =
                            self.chain.effects.iter().position(|e| e.id == from_id)
                        {
                            if from_idx != to_idx {
                                self.chain.move_effect(from_id, to_idx);
                                self.actions
                                    .push(EffectChainAction::MoveEffect(from_id, to_idx));
                            }
                        }
                    }
                }
            });
    }

    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
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
                if icon_button_simple(
                    ui,
                    icon_manager,
                    AppIcon::FloppyDisk,
                    16.0,
                    &locale.t("effect-save"),
                )
                .clicked()
                {
                    self.show_preset_browser = true;
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
            .frame(cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                render_panel_header(ui, &locale.t("effect-presets-browser"), |_| {});
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.preset_search)
                            .hint_text(locale.t("effect-search")),
                    );
                });
                ui.separator();
                egui::Frame::default()
                    .fill(colors::DARKER_GREY)
                    .stroke(egui::Stroke::new(1.0, colors::STROKE_GREY))
                    .corner_radius(CornerRadius::ZERO)
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                let search_lower = self.preset_search.to_lowercase();
                                for preset in &self.presets {
                                    if !self.preset_search.is_empty()
                                        && !preset.name_lower.contains(&search_lower)
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
                                        crate::widgets::custom::render_info_label(
                                            ui,
                                            &preset.category,
                                        );
                                    });
                                }
                                if self.presets.is_empty() {
                                    crate::widgets::custom::render_info_label(
                                        ui,
                                        &locale.t("effect-no-presets"),
                                    );
                                }
                            });
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(locale.t("effect-save-as"));
                    ui.text_edit_singleline(&mut self.save_preset_name);
                    if icon_button_simple(
                        ui,
                        icon_manager,
                        AppIcon::FloppyDisk,
                        16.0,
                        &locale.t("effect-save"),
                    )
                    .clicked()
                        && !self.save_preset_name.is_empty()
                    {
                        self.actions
                            .push(EffectChainAction::SavePreset(self.save_preset_name.clone()));
                        self.save_preset_name.clear();
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
