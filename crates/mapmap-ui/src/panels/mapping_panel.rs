//! Egui-based Mapping Management Panel
use crate::i18n::LocaleManager;
use crate::theme::colors;
use crate::widgets::{custom, panel};
use crate::UIAction;
use egui::*;
use mapmap_core::{MappingId, MappingManager};

#[derive(Debug, Default)]
pub struct MappingPanel {
    pub visible: bool,
}

impl MappingPanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        mapping_manager: &mut MappingManager,
        actions: &mut Vec<UIAction>,
        i18n: &LocaleManager,
    ) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-mappings"))
            .open(&mut open)
            .default_size([380.0, 400.0])
            .show(ctx, |ui| {
                // Header
                panel::render_panel_header(ui, &i18n.t("panel-mappings"), |ui| {
                    ui.label(i18n.t_args(
                        "label-total-mappings",
                        &[("count", &mapping_manager.mappings().len().to_string())],
                    ));
                });

                ui.add_space(4.0);

                // Scrollable mapping list
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        // Collect IDs to avoid borrow issues
                        let mapping_ids: Vec<MappingId> =
                            mapping_manager.mappings().iter().map(|m| m.id).collect();

                        if mapping_ids.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(20.0);
                                ui.label(RichText::new("No mappings created yet.").weak());
                                ui.add_space(20.0);
                            });
                        }

                        for (i, mapping_id) in mapping_ids.iter().enumerate() {
                            if let Some(mapping) = mapping_manager.get_mapping_mut(*mapping_id) {
                                ui.push_id(mapping.id, |ui| {
                                    // Zebra striping for Cyber Dark look
                                    let bg_color = if i % 2 == 1 {
                                        colors::DARKER_GREY.linear_multiply(0.5)
                                    } else {
                                        colors::DARK_GREY
                                    };

                                    egui::Frame::default()
                                        .fill(bg_color)
                                        .inner_margin(4.0)
                                        .corner_radius(0.0) // Sharp corners
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Visibility Checkbox
                                                if ui.checkbox(&mut mapping.visible, "").changed() {
                                                    actions.push(
                                                        UIAction::ToggleMappingVisibility(
                                                            mapping.id,
                                                            mapping.visible,
                                                        ),
                                                    );
                                                }

                                                // Name (Clickable Label for selection)
                                                let label = format!(
                                                    "{} (Paint #{})",
                                                    mapping.name, mapping.paint_id
                                                );
                                                // We don't have a "selected_mapping_id" passed in show() unfortunately,
                                                // so we can't highlight selection state perfectly here without changing signature.
                                                // But we can make it clickable.
                                                if ui.selectable_label(false, label).clicked() {
                                                    actions
                                                        .push(UIAction::SelectMapping(mapping.id));
                                                }

                                                // Right Aligned Actions
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        // Delete Button
                                                        if custom::delete_button(ui) {
                                                            actions.push(UIAction::RemoveMapping(
                                                                mapping.id,
                                                            ));
                                                        }

                                                        ui.add_space(4.0);

                                                        // Lock Button
                                                        if custom::lock_button(ui, mapping.locked)
                                                            .clicked()
                                                        {
                                                            mapping.locked = !mapping.locked;
                                                        }

                                                        ui.add_space(4.0);

                                                        // Solo Button
                                                        if custom::solo_button(ui, mapping.solo)
                                                            .clicked()
                                                        {
                                                            mapping.solo = !mapping.solo;
                                                        }
                                                    },
                                                );
                                            });

                                            // Second row: Opacity (Indented)
                                            // Only show if visible to reduce clutter? Or always?
                                            // Let's keep it always for quick access.
                                            ui.horizontal(|ui| {
                                                ui.add_space(24.0); // Indent to align with name text
                                                ui.label(
                                                    RichText::new(i18n.t("label-master-opacity"))
                                                        .size(10.0)
                                                        .weak(),
                                                );
                                                custom::styled_slider(
                                                    ui,
                                                    &mut mapping.opacity,
                                                    0.0..=1.0,
                                                    1.0,
                                                );
                                            });
                                        });
                                });
                                // Small spacing between items
                                ui.add_space(1.0);
                            }
                        }
                    });

                ui.separator();

                // Add Mapping Button Area
                ui.horizontal(|ui| {
                    if ui
                        .button(format!("➕ {}", i18n.t("btn-add-mapping")))
                        .clicked()
                    {
                        actions.push(UIAction::AddMapping);
                    }
                });
            });

        self.visible = open;
    }
}
