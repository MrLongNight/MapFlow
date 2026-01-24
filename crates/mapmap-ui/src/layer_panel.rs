//! Egui-based Layer Management Panel
use crate::i18n::LocaleManager;
use crate::widgets;
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
                        let layer_ids: Vec<u64> =
                            layer_manager.layers().iter().map(|l| l.id).collect();
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
                                            if ui
                                                .selectable_label(is_selected, &layer.name)
                                                .clicked()
                                            {
                                                *selected_layer_id = Some(layer.id);
                                            }

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    // Phase 1: Layer management buttons (Delete, Duplicate, Solo, Bypass)
                                                    // Right-aligned, so order is reversed: Delete, Duplicate, Solo, Bypass

                                                    if widgets::delete_button(ui).clicked() {
                                                        actions
                                                            .push(UIAction::RemoveLayer(layer.id));
                                                    }

                                                    ui.add_space(4.0);

                                                    if widgets::duplicate_button(ui).clicked() {
                                                        actions.push(UIAction::DuplicateLayer(
                                                            layer.id,
                                                        ));
                                                    }

                                                    ui.add_space(4.0);

                                                    // Use newly added styled widgets for state toggles
                                                    if widgets::solo_button(ui, layer.solo)
                                                        .clicked()
                                                    {
                                                        layer.solo = !layer.solo;
                                                    }

                                                    ui.add_space(4.0);

                                                    if widgets::bypass_button(ui, layer.bypass)
                                                        .clicked()
                                                    {
                                                        layer.bypass = !layer.bypass;
                                                    }
                                                },
                                            );
                                        });

                                        // Indented properties
                                        ui.indent("layer_props", |ui| {
                                            // Moved Bypass/Solo to header

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

                                            egui::ComboBox::from_id_salt(format!(
                                                "blend_{}",
                                                layer.id
                                            ))
                                            .selected_text(format!("{:?}", current_mode))
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    for mode in blend_modes {
                                                        ui.selectable_value(
                                                            &mut selected_mode,
                                                            *mode,
                                                            format!("{:?}", mode),
                                                        );
                                                    }
                                                },
                                            );

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
            });

        self.visible = open;
    }
}
