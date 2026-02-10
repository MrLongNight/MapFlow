//! Egui-based Layer Management Panel
use crate::i18n::LocaleManager;
use crate::theme::colors;
use crate::widgets;
use crate::UIAction;
use egui::*;
use mapmap_core::{BlendMode, LayerManager};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum LayerPanelAction {
    AddLayer,
    CreateGroup,
    RemoveLayer(u64),
    DuplicateLayer(u64),
    ReparentLayer(u64, Option<u64>),
    SwapLayers(u64, u64),
    ToggleGroupCollapsed(u64),
    RenameLayer(u64, String),
    ToggleLayerBypass(u64),
    ToggleLayerSolo(u64),
    SetLayerOpacity(u64, f32),
    EjectAllLayers,
}

#[derive(Debug, Default)]
pub struct LayerPanel {
    pub visible: bool,
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
                // We build a tree structure (parent_id -> list of child IDs)
                // The order in the list is preserved from the main layer list, so reordering works.
                let mut children_map: HashMap<Option<u64>, Vec<u64>> = HashMap::new();
                for layer in layer_manager.layers() {
                    children_map
                        .entry(layer.parent_id)
                        .or_default()
                        .push(layer.id);
                }

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        Self::render_tree(
                            ui,
                            None,
                            &children_map,
                            layer_manager,
                            selected_layer_id,
                            actions,
                            i18n,
                        );
                    });

                ui.separator();

                // Add Layer / Group Buttons
                ui.horizontal(|ui| {
                    if ui.button(i18n.t("btn-add-layer")).clicked() {
                        actions.push(UIAction::AddLayer);
                    }
                    if ui.button("+ Group").clicked() {
                        actions.push(UIAction::CreateGroup);
                    }
                });
            });

        self.visible = open;
    }

    #[allow(clippy::too_many_arguments)]
    fn render_tree(
        ui: &mut egui::Ui,
        parent_id: Option<u64>,
        children_map: &HashMap<Option<u64>, Vec<u64>>,
        layer_manager: &LayerManager,
        selected_layer_id: &mut Option<u64>,
        actions: &mut Vec<UIAction>,
        i18n: &LocaleManager,
    ) {
        if let Some(children) = children_map.get(&parent_id) {
            let count = children.len();
            for (idx, &layer_id) in children.iter().enumerate() {
                if let Some(layer) = layer_manager.get_layer(layer_id) {
                    ui.push_id(layer.id, |ui| {
                        let is_selected = *selected_layer_id == Some(layer.id);
                        let is_group = layer.is_group;
                        let collapsed = layer.collapsed;

                        // Zebra striping for better list readability
                        let bg_color = if is_selected {
                            colors::CYAN_ACCENT.linear_multiply(0.2)
                        } else if idx % 2 == 1 {
                            colors::DARKER_GREY.linear_multiply(0.5) // Subtle alternating background
                        } else {
                            Color32::TRANSPARENT
                        };

                        // Selection stroke or subtle border
                        let stroke = if is_selected {
                            Stroke::new(1.0, colors::CYAN_ACCENT)
                        } else {
                            Stroke::new(1.0, colors::STROKE_GREY.linear_multiply(0.5))
                        };

                        egui::Frame::NONE
                            .fill(bg_color)
                            .stroke(stroke)
                            .corner_radius(0) // Sharp corners for Cyber/Resolume style
                            .inner_margin(4.0)
                            .show(ui, |ui: &mut egui::Ui| {
                                ui.horizontal(|ui| {
                                    // Reorder Buttons (Move Up/Down)
                                    ui.vertical(|ui| {
                                        // Move Up
                                        ui.add_enabled_ui(idx > 0, |ui| {
                                            if widgets::move_up_button(ui).clicked() && idx > 0 {
                                                let prev_id = children[idx - 1];
                                                actions
                                                    .push(UIAction::SwapLayers(layer.id, prev_id));
                                            }
                                        });
                                        // Move Down
                                        ui.add_enabled_ui(idx < count - 1, |ui| {
                                            if widgets::move_down_button(ui).clicked()
                                                && idx < count - 1
                                            {
                                                let next_id = children[idx + 1];
                                                actions
                                                    .push(UIAction::SwapLayers(layer.id, next_id));
                                            }
                                        });
                                    });

                                    // Indent/Unindent
                                    ui.vertical(|ui| {
                                        // Unindent (Left)
                                        if layer.parent_id.is_some()
                                            && ui.button("⬅").on_hover_text("Unindent").clicked()
                                        {
                                            if let Some(pid) = layer.parent_id {
                                                if let Some(parent) = layer_manager.get_layer(pid) {
                                                    actions.push(UIAction::ReparentLayer(
                                                        layer.id,
                                                        parent.parent_id,
                                                    ));
                                                }
                                            }
                                        }

                                        // Indent (Right)
                                        if idx > 0
                                            && ui.button("➡").on_hover_text("Indent").clicked()
                                        {
                                            let prev_sibling_id = children[idx - 1];
                                            if let Some(prev) =
                                                layer_manager.get_layer(prev_sibling_id)
                                            {
                                                if prev.is_group {
                                                    actions.push(UIAction::ReparentLayer(
                                                        layer.id,
                                                        Some(prev.id),
                                                    ));
                                                }
                                            }
                                        }
                                    });

                                    // Group Expander
                                    if is_group {
                                        let icon = if collapsed { "▶" } else { "▼" };
                                        if ui.add(Button::new(icon).frame(false)).clicked() {
                                            actions.push(UIAction::ToggleGroupCollapsed(layer.id));
                                        }
                                    } else {
                                        ui.add_space(16.0);
                                    }

                                    // Visibility
                                    let mut visible = layer.visible;
                                    if ui.checkbox(&mut visible, "").changed() {
                                        actions
                                            .push(UIAction::SetLayerVisibility(layer.id, visible));
                                    }

                                    // Name
                                    let name_label = if is_group {
                                        RichText::new(&layer.name).strong()
                                    } else {
                                        RichText::new(&layer.name)
                                    };

                                    if ui.selectable_label(is_selected, name_label).clicked() {
                                        *selected_layer_id = Some(layer.id);
                                    }

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if widgets::delete_button(ui) {
                                                actions.push(UIAction::RemoveLayer(layer.id));
                                            }

                                            if !is_group && widgets::duplicate_button(ui).clicked()
                                            {
                                                actions.push(UIAction::DuplicateLayer(layer.id));
                                            }

                                            ui.add_space(4.0);
                                            if widgets::solo_button(ui, layer.solo).clicked() {
                                                actions.push(UIAction::ToggleLayerSolo(layer.id));
                                            }

                                            ui.add_space(4.0);
                                            if widgets::bypass_button(ui, layer.bypass).clicked() {
                                                actions.push(UIAction::ToggleLayerBypass(layer.id));
                                            }
                                        },
                                    );
                                });

                                // Inline Properties for Selected Layer
                                if is_selected {
                                    ui.indent("props", |ui| {
                                        // Opacity
                                        let mut opacity = layer.opacity;
                                        if ui
                                            .add(
                                                Slider::new(&mut opacity, 0.0..=1.0)
                                                    .text(i18n.t("label-master-opacity")),
                                            )
                                            .changed()
                                        {
                                            actions
                                                .push(UIAction::SetLayerOpacity(layer.id, opacity));
                                        }

                                        // Blend Mode
                                        let blend_modes = BlendMode::all();
                                        let current_mode = layer.blend_mode;
                                        let mut selected_mode = current_mode;
                                        egui::ComboBox::from_id_salt(format!("blend_{}", layer.id))
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
                                            actions.push(UIAction::SetLayerBlendMode(
                                                layer.id,
                                                selected_mode,
                                            ));
                                        }
                                    });
                                }
                            });

                        // Children
                        if is_group && !collapsed {
                            ui.indent(format!("group_{}", layer.id), |ui| {
                                Self::render_tree(
                                    ui,
                                    Some(layer.id),
                                    children_map,
                                    layer_manager,
                                    selected_layer_id,
                                    actions,
                                    i18n,
                                );
                            });
                        }
                    });
                }
            }
        }
    }
}
