//! Egui-based Layer Management Panel
use crate::i18n::LocaleManager;
use crate::UIAction;
use egui::*;
use mapmap_core::{BlendMode, LayerManager};

#[derive(Debug, Clone)]
pub enum LayerPanelAction {
    AddLayer,
    RemoveLayer(u64),
    DuplicateLayer(u64),
    RenameLayer(u64, String),
    ToggleLayerBypass(u64),
    ToggleLayerSolo(u64),
    SetLayerOpacity(u64, f32),
    EjectAllLayers,
    // Note: Reordering is handled directly on LayerManager or via custom action if needed
    // MoveLayerUp(u64),
    // MoveLayerDown(u64),
}

#[derive(Debug, Default)]
pub struct LayerPanel {
    pub visible: bool,
    // selected_layer_id is managed by AppUI but we accept it as a param to sync
}

impl LayerPanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        layer_manager: &mut LayerManager,
        selected_layer_id: &mut Option<u64>,
        actions: &mut Vec<UIAction>,
        i18n: &LocaleManager,
    ) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-layers"))
            .open(&mut open)
            .default_size([380.0, 400.0])
            .show(ctx, |ui| {
                self.ui(ui, layer_manager, selected_layer_id, actions, i18n);
            });
        self.visible = open;
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        layer_manager: &mut LayerManager,
        selected_layer_id: &mut Option<u64>,
        actions: &mut Vec<UIAction>,
        i18n: &LocaleManager,
    ) {
        ui.horizontal(|ui| {
            ui.label(i18n.t_args(
                "label-total-layers",
                &[("count", &layer_manager.layers().len().to_string())],
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(i18n.t("btn-eject-all")).clicked() {
                    actions.push(UIAction::EjectAllLayers);
                }
            });
        });
        ui.separator();

        // Layer list area
        let mut move_up_id = None;
        let mut move_down_id = None;

        egui::ScrollArea::vertical()
            .max_height(300.0) // Limit height to leave room for bottom buttons
            .show(ui, |ui| {
                // Iterate over layer IDs to avoid borrow issues while mutating
                // We need indices to determine if move up/down is possible
                let layer_ids: Vec<u64> = layer_manager.layers().iter().map(|l| l.id).collect();
                let total_layers = layer_ids.len();

                for (index, layer_id) in layer_ids.iter().enumerate() {
                    let is_first = index == 0;
                    let is_last = index == total_layers - 1;

                    if let Some(layer) = layer_manager.get_layer_mut(*layer_id) {
                        ui.push_id(layer.id, |ui| {
                            // Layer Row
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    // Reorder buttons
                                    ui.vertical(|ui| {
                                        if ui
                                            .add_enabled(!is_first, egui::Button::new("⬆"))
                                            .clicked()
                                        {
                                            move_up_id = Some(layer.id);
                                        }
                                        if ui
                                            .add_enabled(!is_last, egui::Button::new("⬇"))
                                            .clicked()
                                        {
                                            move_down_id = Some(layer.id);
                                        }
                                    });

                                    // Visibility
                                    let mut visible = layer.visible;
                                    if ui.checkbox(&mut visible, "").changed() {
                                        layer.visible = visible;
                                    }

                                    // Name and Selection
                                    let is_selected = *selected_layer_id == Some(layer.id);
                                    if ui.selectable_label(is_selected, &layer.name).clicked() {
                                        *selected_layer_id = Some(layer.id);
                                    }

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // Phase 1: Layer management buttons (Duplicate, Remove)
                                            if ui
                                                .small_button("🗑")
                                                .on_hover_text(i18n.t("btn-remove"))
                                                .clicked()
                                            {
                                                actions.push(UIAction::RemoveLayer(layer.id));
                                            }
                                            if ui
                                                .small_button("📄")
                                                .on_hover_text(i18n.t("btn-duplicate"))
                                                .clicked()
                                            {
                                                actions.push(UIAction::DuplicateLayer(layer.id));
                                            }
                                        },
                                    );
                                });

                                // Indented properties
                                ui.indent("layer_props", |ui| {
                                    ui.horizontal(|ui| {
                                        // Phase 1: Bypass, Solo
                                        let mut bypass = layer.bypass;
                                        if ui
                                            .checkbox(&mut bypass, i18n.t("check-bypass"))
                                            .changed()
                                        {
                                            // Direct modification plus action for undo/consistency if needed
                                            // But UIAction handles ToggleLayerBypass which might toggle it back if we modify it here?
                                            // Standard pattern in ImGui code was: direct modify AND push action.
                                            // UIAction processing logic in main.rs toggles it. So if we set it here, then toggle it, it reverts.
                                            // We should only push action OR modify.
                                            // ImGui code: `if ui.checkbox(...) { layer.bypass = bypass; actions.push(...) }`
                                            // Main.rs logic: `ToggleLayerBypass(id) => if let Some(l) = ... { l.toggle_bypass() }`
                                            // If we set `layer.bypass = bypass` (new value) here, and then push `Toggle`, `Toggle` will flip it back to old value.
                                            // So we should NOT modify here if we push action. Or push `SetBypass` (if it existed).
                                            // `ToggleLayerBypass` implies we just fire the action.
                                            // But Checkbox expects a `&mut bool`.
                                            // If we pass `&mut layer.bypass`, it updates immediately. Then we push Toggle -> it flips back.
                                            // FIX: Don't push action for simple bool toggles if we have direct mutable access, OR don't update mutable access.
                                            // But egui Checkbox requires `&mut bool`.
                                            // So we update it. We should NOT push `ToggleLayerBypass` if we already updated it.
                                            // UNLESS the action is for other systems (Undo).
                                            // Let's check `lib.rs` ImGui implementation again.
                                            /*
                                            if ui.checkbox(self.i18n.t("check-bypass"), &mut bypass) {
                                                layer.bypass = bypass;
                                                self.actions.push(UIAction::ToggleLayerBypass(layer.id));
                                            }
                                            */
                                            // This ImGui code seems buggy if ToggleLayerBypass flips it.
                                            // Let's assume we just modify state directly here for now as we have mutable access.
                                            layer.bypass = bypass;
                                        }

                                        let mut solo = layer.solo;
                                        if ui.checkbox(&mut solo, i18n.t("check-solo")).changed() {
                                            layer.solo = solo;
                                        }
                                    });

                                    // Opacity
                                    let mut opacity = layer.opacity;
                                    if ui
                                        .add(
                                            Slider::new(&mut opacity, 0.0..=1.0)
                                                .text(i18n.t("label-master-opacity")),
                                        )
                                        .changed()
                                    {
                                        layer.opacity = opacity;
                                        // For sliders, we might want to push action only on release, but for now direct update is fine.
                                        // If we need to record for Undo, we'd need a "drag ended" event.
                                    }

                                    // Blend Mode
                                    let blend_modes = BlendMode::all();
                                    let current_mode = layer.blend_mode;
                                    let mut selected_mode = current_mode;

                                    egui::ComboBox::from_id_source(format!("blend_{}", layer.id))
                                        .selected_text(format!("{:?}", current_mode))
                                        .show_ui(ui, |ui| {
                                            for mode in blend_modes {
                                                ui.selectable_value(
                                                    &mut selected_mode,
                                                    *mode,
                                                    format!("{:?}", mode),
                                                );
                                            }
                                        });

                                    if selected_mode != current_mode {
                                        layer.blend_mode = selected_mode;
                                    }
                                });
                            });
                        });
                    }
                }
            });

        // Apply reordering
        if let Some(id) = move_up_id {
            layer_manager.move_layer_up(id);
        }
        if let Some(id) = move_down_id {
            layer_manager.move_layer_down(id);
        }

        ui.separator();

        // Add Layer Button
        if ui.button(i18n.t("btn-add-layer")).clicked() {
            actions.push(UIAction::AddLayer);
        }
    }
}
