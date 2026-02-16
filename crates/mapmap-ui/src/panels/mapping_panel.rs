use crate::i18n::LocaleManager;
use crate::theme::colors;
use crate::widgets::custom;
use crate::widgets::icons::{AppIcon, IconManager};
use crate::widgets::panel::{cyber_panel_frame, render_panel_header};
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
        icon_manager: Option<&IconManager>,
    ) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-mappings"))
            .open(&mut open)
            .default_size([380.0, 400.0])
            .frame(cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                render_panel_header(
                    ui,
                    &i18n.t("panel-mappings"),
                    Some(AppIcon::Screen),
                    icon_manager,
                    |_| {},
                );

                ui.add_space(8.0);

                ui.horizontal(|ui| {
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

                        for (i, mapping_id) in mapping_ids.iter().enumerate() {
                            if let Some(mapping) = mapping_manager.get_mapping_mut(*mapping_id) {
                                ui.push_id(mapping.id, |ui| {
                                    // Zebra striping
                                    let bg_color = if i % 2 == 0 {
                                        colors::DARK_GREY
                                    } else {
                                        colors::DARKER_GREY
                                    };

                                    egui::Frame::new().fill(bg_color).inner_margin(4.0).show(
                                        ui,
                                        |ui| {
                                            ui.horizontal(|ui| {
                                                // Visibility
                                                if ui.checkbox(&mut mapping.visible, "").changed() {
                                                    actions.push(
                                                        UIAction::ToggleMappingVisibility(
                                                            mapping.id,
                                                            mapping.visible,
                                                        ),
                                                    );
                                                }

                                                // Name (Click to select)
                                                let label = format!(
                                                    "{} (Paint #{})",
                                                    mapping.name, mapping.paint_id
                                                );
                                                if ui
                                                    .add(
                                                        egui::Label::new(label)
                                                            .sense(Sense::click()),
                                                    )
                                                    .clicked()
                                                {
                                                    actions
                                                        .push(UIAction::SelectMapping(mapping.id));
                                                }

                                                // Spacer
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

                                                        ui.add_space(8.0);

                                                        // Solo Button
                                                        if custom::solo_button(ui, mapping.solo)
                                                            .clicked()
                                                        {
                                                            mapping.solo = !mapping.solo;
                                                        }

                                                        ui.add_space(8.0);

                                                        // Lock Button
                                                        ui.checkbox(
                                                            &mut mapping.locked,
                                                            i18n.t("check-lock"),
                                                        );
                                                    },
                                                );
                                            });

                                            // Second row: Opacity
                                            ui.horizontal(|ui| {
                                                ui.add_space(24.0); // Indent to align with name
                                                ui.label(i18n.t("label-master-opacity"));
                                                custom::styled_slider(
                                                    ui,
                                                    &mut mapping.opacity,
                                                    0.0..=1.0,
                                                    1.0,
                                                );
                                            });
                                        },
                                    );
                                });
                                ui.add_space(2.0);
                            }
                        }
                    });

                ui.separator();
                ui.add_space(4.0);

                // Add Mapping Button
                if ui.button(i18n.t("btn-add-mapping")).clicked() {
                    actions.push(UIAction::AddMapping);
                }
            });

        self.visible = open;
    }
}
