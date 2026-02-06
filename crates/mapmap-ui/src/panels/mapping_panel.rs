//! Egui-based Mapping Management Panel
use crate::i18n::LocaleManager;
use crate::widgets;
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
                widgets::render_panel_header(
                    ui,
                    &i18n.t_args(
                        "label-total-mappings",
                        &[("count", &mapping_manager.mappings().len().to_string())],
                    ),
                    |ui| {
                        if ui.button(i18n.t("btn-add-mapping")).clicked() {
                            actions.push(UIAction::AddMapping);
                        }
                    },
                );

                // Scrollable mapping list
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        // Collect IDs to avoid borrow issues
                        let mapping_ids: Vec<MappingId> =
                            mapping_manager.mappings().iter().map(|m| m.id).collect();

                        for mapping_id in mapping_ids {
                            if let Some(mapping) = mapping_manager.get_mapping_mut(mapping_id) {
                                ui.push_id(mapping.id, |ui| {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            // Visibility
                                            let _old_visible = mapping.visible;
                                            if ui.checkbox(&mut mapping.visible, "").changed() {
                                                actions.push(UIAction::ToggleMappingVisibility(
                                                    mapping.id,
                                                    mapping.visible,
                                                ));
                                            }

                                            // Name (Click to select)
                                            // We don't have a specific "selected_mapping_id" passed in yet,
                                            // but we can fire an action to select it.
                                            let label = format!(
                                                "{} (Paint #{})",
                                                mapping.name, mapping.paint_id
                                            );
                                            if ui.button(label).clicked() {
                                                actions.push(UIAction::SelectMapping(mapping.id));
                                            }

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui
                                                        .button(i18n.t("btn-remove"))
                                                        .on_hover_text(i18n.t("btn-remove"))
                                                        .clicked()
                                                    {
                                                        actions.push(UIAction::RemoveMapping(
                                                            mapping.id,
                                                        ));
                                                    }
                                                },
                                            );
                                        });

                                        // Indented properties
                                        ui.indent("mapping_props", |ui| {
                                            ui.horizontal(|ui| {
                                                ui.checkbox(
                                                    &mut mapping.solo,
                                                    i18n.t("check-solo"),
                                                );
                                                ui.checkbox(
                                                    &mut mapping.locked,
                                                    i18n.t("check-lock"),
                                                );
                                            });

                                            // Opacity
                                            ui.add(
                                                Slider::new(&mut mapping.opacity, 0.0..=1.0)
                                                    .text(i18n.t("label-master-opacity")),
                                            );
                                        });
                                    });
                                });
                            }
                        }
                    });

            });

        self.visible = open;
    }
}
