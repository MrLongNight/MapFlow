use crate::i18n::LocaleManager;
use crate::theme::colors;
use crate::widgets;
use crate::widgets::icons::IconManager;
use crate::widgets::panel::{cyber_panel_frame, render_panel_header};
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
        icon_manager: Option<&IconManager>,
    ) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-layers"))
            .open(&mut open)
            .default_size([380.0, 400.0])
            .frame(cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                render_panel_header(ui, &i18n.t("panel-layers"), |_| {});

                ui.add_space(8.0);

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
                            icon_manager,
                            0,
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
    fn render_layer_row(
        ui: &mut egui::Ui,
        layer: &mapmap_core::Layer,
        layer_manager: &LayerManager,
        idx: usize,
        count: usize,
        depth: usize,
        prev_id: Option<u64>,
        next_id: Option<u64>,
        selected_layer_id: &mut Option<u64>,
        actions: &mut Vec<UIAction>,
        _i18n: &LocaleManager,
        icon_manager: Option<&IconManager>,
    ) {
        let is_selected = *selected_layer_id == Some(layer.id);
        let is_group = layer.is_group;
        let collapsed = layer.collapsed;

        // Zebra striping
        let bg_color = if is_selected {
            colors::CYAN_ACCENT.linear_multiply(0.2)
        } else if idx % 2 == 1 {
            colors::DARKER_GREY.linear_multiply(1.2)
        } else {
            colors::DARKER_GREY
        };

        // Selection stroke
        let stroke = if is_selected {
            Stroke::new(1.0, colors::CYAN_ACCENT)
        } else {
            Stroke::NONE
        };

        // Row container
        let row_response = egui::Frame::default()
            .fill(bg_color)
            .stroke(stroke)
            .corner_radius(0.0)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Manual Indentation
                    if depth > 0 {
                        ui.add_space(depth as f32 * 20.0);
                    }

                    // 1. Group Toggle / Spacer
                    if is_group {
                        // Use ArrowRight (collapsed) / ArrowDown (expanded) if available, else text
                        // We use ArrowRight for collapsed, and ArrowDown for expanded.
                        // Since we might not have ArrowDown, we can rotate ArrowRight?
                        // Or just use text for now to be safe.
                        let icon = if collapsed { "▶" } else { "▼" };
                        if ui
                            .add(
                                Button::new(icon)
                                    .frame(false)
                                    .min_size(Vec2::new(16.0, 16.0)),
                            )
                            .clicked()
                        {
                            actions.push(UIAction::ToggleGroupCollapsed(layer.id));
                        }
                    } else {
                        ui.add_space(16.0); // Align with group toggle
                    }

                    // 2. Visibility
                    let vis_icon = if layer.visible {
                        crate::widgets::icons::AppIcon::Eye
                    } else {
                        crate::widgets::icons::AppIcon::EyeSlash
                    };
                    if widgets::custom::icon_button_simple(
                        ui,
                        icon_manager,
                        vis_icon,
                        16.0,
                        colors::CYAN_ACCENT,
                    )
                    .clicked()
                    {
                        actions.push(UIAction::SetLayerVisibility(layer.id, !layer.visible));
                    }

                    // 3. Name (Selectable)
                    let name_text = if is_group {
                        RichText::new(&layer.name).strong().color(Color32::WHITE)
                    } else {
                        RichText::new(&layer.name).color(if is_selected {
                            Color32::WHITE
                        } else {
                            Color32::from_gray(200)
                        })
                    };

                    let label_resp = ui.add(Label::new(name_text).sense(Sense::click()));
                    if label_resp.clicked() {
                        *selected_layer_id = Some(layer.id);
                    }

                    // Spacer to push right-side controls
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), ui.available_height()),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            // Indent/Unindent
                            // Unindent (Left)
                            if layer.parent_id.is_some() {
                                if widgets::custom::icon_button_simple(
                                    ui,
                                    icon_manager,
                                    crate::widgets::icons::AppIcon::ArrowLeft,
                                    16.0,
                                    Color32::WHITE,
                                )
                                .clicked()
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
                            } else {
                                ui.add_space(20.0);
                            }

                            // Indent (Right)
                            if idx > 0 {
                                // Check if prev sibling is a group
                                let mut can_indent = false;
                                if let Some(pid) = prev_id {
                                    if let Some(prev) = layer_manager.get_layer(pid) {
                                        if prev.is_group {
                                            can_indent = true;
                                        }
                                    }
                                }

                                if can_indent {
                                    if widgets::custom::icon_button_simple(
                                        ui,
                                        icon_manager,
                                        crate::widgets::icons::AppIcon::ArrowRight,
                                        16.0,
                                        Color32::WHITE,
                                    )
                                    .clicked()
                                    {
                                        if let Some(pid) = prev_id {
                                            actions
                                                .push(UIAction::ReparentLayer(layer.id, Some(pid)));
                                        }
                                    }
                                } else {
                                    ui.add_space(20.0);
                                }
                            } else {
                                ui.add_space(20.0);
                            }

                            ui.separator();

                            // Delete
                            if widgets::custom::icon_button_simple(
                                ui,
                                icon_manager,
                                crate::widgets::icons::AppIcon::Remove,
                                16.0,
                                colors::ERROR_COLOR,
                            )
                            .clicked()
                            {
                                actions.push(UIAction::RemoveLayer(layer.id));
                            }

                            // Duplicate
                            if !is_group
                                && widgets::custom::icon_button_simple(
                                    ui,
                                    icon_manager,
                                    crate::widgets::icons::AppIcon::Duplicate,
                                    16.0,
                                    colors::CYAN_ACCENT,
                                )
                                .clicked()
                            {
                                actions.push(UIAction::DuplicateLayer(layer.id));
                            }

                            ui.add_space(4.0);

                            // Solo
                            let solo_color = if layer.solo {
                                colors::MINT_ACCENT
                            } else {
                                Color32::from_gray(100)
                            };
                            if widgets::custom::icon_button_simple(
                                ui,
                                icon_manager,
                                crate::widgets::icons::AppIcon::Solo,
                                16.0,
                                solo_color,
                            )
                            .clicked()
                            {
                                actions.push(UIAction::ToggleLayerSolo(layer.id));
                            }

                            // Bypass
                            let bypass_color = if layer.bypass {
                                colors::WARN_COLOR
                            } else {
                                Color32::from_gray(100)
                            };
                            if widgets::custom::icon_button_simple(
                                ui,
                                icon_manager,
                                crate::widgets::icons::AppIcon::Bypass,
                                16.0,
                                bypass_color,
                            )
                            .clicked()
                            {
                                actions.push(UIAction::ToggleLayerBypass(layer.id));
                            }

                            ui.separator();

                            // Reorder (Up/Down)
                            if idx < count - 1 {
                                if widgets::move_down_button(ui).clicked() {
                                    if let Some(nid) = next_id {
                                        actions.push(UIAction::SwapLayers(layer.id, nid));
                                    }
                                }
                            } else {
                                ui.add_space(24.0); // Placeholder size for button
                            }

                            if idx > 0 {
                                if widgets::move_up_button(ui).clicked() {
                                    if let Some(pid) = prev_id {
                                        actions.push(UIAction::SwapLayers(layer.id, pid));
                                    }
                                }
                            } else {
                                ui.add_space(24.0);
                            }
                        },
                    );
                });
            });

        // Hover effect for the whole row
        if row_response.response.hovered() {
            ui.painter().rect_stroke(
                row_response.response.rect,
                0.0,
                Stroke::new(1.0, colors::CYAN_ACCENT.linear_multiply(0.5)),
                egui::StrokeKind::Middle,
            );
        }
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
        icon_manager: Option<&IconManager>,
        depth: usize,
    ) {
        if let Some(children) = children_map.get(&parent_id) {
            let count = children.len();
            for (idx, &layer_id) in children.iter().enumerate() {
                if let Some(layer) = layer_manager.get_layer(layer_id) {
                    let prev_id = if idx > 0 {
                        Some(children[idx - 1])
                    } else {
                        None
                    };
                    let next_id = if idx < count - 1 {
                        Some(children[idx + 1])
                    } else {
                        None
                    };

                    Self::render_layer_row(
                        ui,
                        layer,
                        layer_manager,
                        idx,
                        count,
                        depth,
                        prev_id,
                        next_id,
                        selected_layer_id,
                        actions,
                        i18n,
                        icon_manager,
                    );

                    // Inline Properties
                    if *selected_layer_id == Some(layer.id) {
                        ui.horizontal(|ui| {
                            if depth > 0 {
                                ui.add_space(depth as f32 * 20.0 + 20.0);
                            } else {
                                ui.add_space(20.0);
                            }
                            ui.vertical(|ui| {
                                // Opacity
                                let mut opacity = layer.opacity;
                                if ui
                                    .add(
                                        Slider::new(&mut opacity, 0.0..=1.0)
                                            .text(i18n.t("label-master-opacity")),
                                    )
                                    .changed()
                                {
                                    actions.push(UIAction::SetLayerOpacity(layer.id, opacity));
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
                                    actions
                                        .push(UIAction::SetLayerBlendMode(layer.id, selected_mode));
                                }
                            });
                        });
                    }

                    // Recursion
                    if layer.is_group && !layer.collapsed {
                        Self::render_tree(
                            ui,
                            Some(layer.id),
                            children_map,
                            layer_manager,
                            selected_layer_id,
                            actions,
                            i18n,
                            icon_manager,
                            depth + 1,
                        );
                    }
                }
            }
        }
    }
}
