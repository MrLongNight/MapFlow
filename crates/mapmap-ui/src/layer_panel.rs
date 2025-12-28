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
        icon_manager: Option<&crate::icons::IconManager>,
    ) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-layers"))
            .open(&mut open)
            .default_size([380.0, 400.0])
            .show(ctx, |ui| {
                // --- Toolbar ---
                ui.horizontal(|ui| {
                    if ui.button(i18n.t("btn-add-layer")).clicked() {
                        actions.push(UIAction::AddLayer);
                    }
                    if ui
                        .add_enabled(
                            selected_layer_id.is_some(),
                            egui::Button::new(i18n.t("btn-remove")),
                        )
                        .on_hover_text(i18n.t("tooltip-remove-layer"))
                        .clicked()
                    {
                        if let Some(id) = *selected_layer_id {
                            actions.push(UIAction::RemoveLayer(id));
                            *selected_layer_id = None;
                        }
                    }
                    if ui
                        .add_enabled(
                            selected_layer_id.is_some(),
                            egui::Button::new(i18n.t("btn-duplicate")),
                        )
                        .on_hover_text(i18n.t("tooltip-duplicate-layer"))
                        .clicked()
                    {
                        if let Some(id) = *selected_layer_id {
                            actions.push(UIAction::DuplicateLayer(id));
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(i18n.t("btn-eject-all"))
                            .on_hover_text(i18n.t("tooltip-eject-all"))
                            .clicked()
                        {
                            actions.push(UIAction::EjectAllLayers);
                        }
                    });
                });
                ui.separator();

                // --- Layer List ---
                let mut move_layer = None;
                let id_source = "layer_dnd_source";
                let layer_ids: Vec<u64> = layer_manager.layers().iter().map(|l| l.id).collect();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let dragged_item_id =
                        ctx.memory(|mem| mem.data.get_temp::<u64>(Id::new(id_source)));

                    let mut drop_target_index = None;

                    for (i, layer_id) in layer_ids.iter().enumerate() {
                        // --- Render Drop Target ---
                        if dragged_item_id.is_some() && dragged_item_id != Some(*layer_id) {
                            let (_rect, response) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 4.0),
                                Sense::hover(),
                            );
                            if response.hovered() {
                                drop_target_index = Some(i);
                                ui.painter().rect_filled(
                                    response.rect,
                                    Rounding::same(2.0),
                                    ctx.style().visuals.selection.bg_fill,
                                );
                            }
                        }

                        // --- Render Layer Item ---
                        if let Some(layer) = layer_manager.get_layer_mut(*layer_id) {
                            let is_selected = *selected_layer_id == Some(*layer_id);

                            let frame = Frame::group(ui.style())
                                .inner_margin(Margin::same(4.0))
                                .fill(if is_selected {
                                    ui.style().visuals.selection.bg_fill
                                } else {
                                    Color32::TRANSPARENT
                                });

                            let response = frame
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Visibility Toggle
                                        let mut icon_processed = false;
                                        if let Some(mgr) = icon_manager {
                                            let icon = if layer.visible {
                                                crate::icons::AppIcon::Eye
                                            } else {
                                                crate::icons::AppIcon::EyeSlash
                                            };
                                            if let Some(img) = mgr.image(icon, 16.0) {
                                                let btn = egui::ImageButton::new(img);
                                                if ui
                                                    .add(btn)
                                                    .on_hover_text(
                                                        i18n.t("tooltip-toggle-visibility"),
                                                    )
                                                    .clicked()
                                                {
                                                    layer.visible = !layer.visible;
                                                }
                                                icon_processed = true;
                                            }
                                        }
                                        if !icon_processed {
                                            ui.checkbox(&mut layer.visible, "")
                                                .on_hover_text(i18n.t("tooltip-toggle-visibility"));
                                        }

                                        // Layer Name
                                        ui.label(&layer.name);

                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                // Opacity Slider
                                                let slider =
                                                    Slider::new(&mut layer.opacity, 0.0..=1.0)
                                                        .show_value(false)
                                                        .min_decimals(2)
                                                        .max_decimals(2);
                                                ui.add(slider).on_hover_text(
                                                    i18n.t("tooltip-layer-opacity"),
                                                );

                                                // Blend Mode
                                                egui::ComboBox::from_id_source(format!("blend_{}", layer.id))
                                                    .selected_text(format!("{:?}", layer.blend_mode))
                                                    .width(80.0)
                                                    .show_ui(ui, |ui| {
                                                        for mode in BlendMode::all() {
                                                            ui.selectable_value(
                                                                &mut layer.blend_mode,
                                                                *mode,
                                                                format!("{:?}", mode),
                                                            );
                                                        }
                                                    });

                                                // Solo Button
                                                if let Some(mgr) = icon_manager {
                                                    if let Some(img) =
                                                        mgr.image(crate::icons::AppIcon::Solo, 16.0)
                                                    {
                                                        if ui
                                                            .add(egui::ImageButton::new(img))
                                                            .on_hover_text(i18n.t("tooltip-solo"))
                                                            .clicked()
                                                        {
                                                            layer.solo = !layer.solo;
                                                        }
                                                    }
                                                }

                                                // Bypass Button
                                                if let Some(mgr) = icon_manager {
                                                    if let Some(img) = mgr
                                                        .image(crate::icons::AppIcon::Bypass, 16.0)
                                                    {
                                                        if ui
                                                            .add(egui::ImageButton::new(img))
                                                            .on_hover_text(i18n.t("tooltip-bypass"))
                                                            .clicked()
                                                        {
                                                            layer.bypass = !layer.bypass;
                                                        }
                                                    }
                                                }
                                            },
                                        );
                                    });
                                })
                                .response;

                            if response.clicked() {
                                *selected_layer_id = Some(*layer_id);
                            }

                            if response.drag_started() {
                                ctx.memory_mut(|mem| {
                                    mem.data.insert_temp(Id::new(id_source), *layer_id)
                                });
                            }
                        }
                    }

                    // --- Handle Drop ---
                    if ctx.input(|i| i.pointer.any_released()) {
                        if let Some(dragged_id) = dragged_item_id {
                            if let Some(target_idx) = drop_target_index {
                                if let Some(dragged_idx) =
                                    layer_ids.iter().position(|id| *id == dragged_id)
                                {
                                    // Adjust target index if moving an item downwards
                                    let final_target_idx = if dragged_idx < target_idx {
                                        target_idx - 1
                                    } else {
                                        target_idx
                                    };
                                    move_layer = Some((dragged_idx, final_target_idx));
                                }
                            }
                        }
                        ctx.memory_mut(|mem| mem.data.remove::<u64>(Id::new(id_source)));
                    }
                });

                if let Some((old_index, new_index)) = move_layer {
                    if let Some(layer_id) = layer_ids.get(old_index) {
                        layer_manager.move_layer_to(*layer_id, new_index);
                    }
                }
            });

        self.visible = open;
    }
}
