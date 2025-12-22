// crates/mapmap-ui/src/mapping_panel.rs

use crate::i18n::LocaleManager;
use egui::{ComboBox, Context, Slider, TextEdit};
use mapmap_core::{MappingId, MappingManager, MeshType, PaintManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingPanelAction {
    AddMapping,
    RemoveMapping(MappingId),
}

#[derive(Default)]
pub struct MappingPanel {
    pub visible: bool,
    action: Option<MappingPanelAction>,
}

impl MappingPanel {
    pub fn take_action(&mut self) -> Option<MappingPanelAction> {
        self.action.take()
    }

    pub fn render(
        &mut self,
        ctx: &Context,
        i18n: &LocaleManager,
        mapping_manager: &mut MappingManager,
        paint_manager: &PaintManager,
    ) {
        if !self.visible {
            return;
        }

        egui::Window::new(i18n.t("panel-mappings"))
            .open(&mut self.visible)
            .show(ctx, |ui| {
                ui.heading(i18n.t_args(
                    "label-total-mappings",
                    &[("count", &mapping_manager.mappings().len().to_string())],
                ));
                ui.separator();

                let mapping_ids: Vec<_> = mapping_manager.mappings().iter().map(|m| m.id).collect();
                let paint_sources: Vec<_> = paint_manager
                    .paints()
                    .iter()
                    .map(|p| (p.id, p.name.clone()))
                    .collect();

                for mapping_id in mapping_ids {
                    if let Some(mapping) = mapping_manager.get_mapping_mut(mapping_id) {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut mapping.visible, "");
                                ui.add(
                                    TextEdit::singleline(&mut mapping.name)
                                        .hint_text("Mapping Name"),
                                );
                            });

                            // Paint source selector
                            let selected_paint_name = paint_sources
                                .iter()
                                .find(|(id, _)| *id == mapping.paint_id)
                                .map(|(_, name)| name.clone())
                                .unwrap_or_else(|| "Invalid Paint".to_string());

                            ComboBox::from_label(i18n.t("paints-source"))
                                .selected_text(selected_paint_name)
                                .show_ui(ui, |ui| {
                                    for (paint_id, paint_name) in &paint_sources {
                                        ui.selectable_value(
                                            &mut mapping.paint_id,
                                            *paint_id,
                                            paint_name,
                                        );
                                    }
                                });

                            // Mesh type selector
                            let mesh_type_label = format!("{:?}", mapping.mesh.mesh_type);
                            ComboBox::from_label(i18n.t("label-mesh-type"))
                                .selected_text(mesh_type_label)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut mapping.mesh.mesh_type,
                                        MeshType::Quad,
                                        "Quad",
                                    );
                                    ui.selectable_value(
                                        &mut mapping.mesh.mesh_type,
                                        MeshType::Triangle,
                                        "Triangle",
                                    );
                                    ui.selectable_value(
                                        &mut mapping.mesh.mesh_type,
                                        MeshType::Custom,
                                        "Custom",
                                    );
                                });

                            ui.add(
                                Slider::new(&mut mapping.opacity, 0.0..=1.0)
                                    .text(i18n.t("label-master-opacity")),
                            );
                            ui.add(
                                Slider::new(&mut mapping.depth, -10.0..=10.0)
                                    .text(i18n.t("label-depth")),
                            );

                            ui.horizontal(|ui| {
                                ui.checkbox(&mut mapping.solo, i18n.t("check-solo"));
                                ui.checkbox(&mut mapping.locked, i18n.t("check-lock"));
                            });

                            if ui.button(i18n.t("btn-remove")).clicked() {
                                self.action = Some(MappingPanelAction::RemoveMapping(mapping.id));
                            }
                        });
                    }
                }

                ui.separator();

                if ui.button(i18n.t("btn-add-mapping")).clicked() {
                    self.action = Some(MappingPanelAction::AddMapping);
                }
            });
    }
}
