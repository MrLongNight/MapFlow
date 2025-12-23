//! Egui-based Mapping Management Panel
use crate::i18n::LocaleManager;
use crate::UIAction;
use egui::*;
use mapmap_core::{MappingId, MappingManager, Mesh, MeshType};

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
                ui.horizontal(|ui| {
                    ui.label(i18n.t_args(
                        "label-total-mappings",
                        &[("count", &mapping_manager.mappings().len().to_string())],
                    ));
                });
                ui.separator();

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
                                            let label =
                                                format!("{} (Paint #{})", mapping.name, mapping.paint_id);
                                            if ui.button(label).clicked() {
                                                actions.push(UIAction::SelectMapping(mapping.id));
                                            }

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.small_button("🗑").on_hover_text(i18n.t("btn-remove")).clicked() {
                                                        actions.push(UIAction::RemoveMapping(mapping.id));
                                                    }
                                                },
                                            );
                                        });

                                        // Indented properties
                                        ui.indent("mapping_props", |ui| {
                                            ui.horizontal(|ui| {
                                                ui.checkbox(&mut mapping.solo, i18n.t("check-solo"));
                                                ui.checkbox(&mut mapping.locked, i18n.t("check-lock"));
                                            });

                                            // Opacity
                                            ui.add(
                                                Slider::new(&mut mapping.opacity, 0.0..=1.0)
                                                    .text(i18n.t("label-master-opacity")),
                                            );

                                            // Depth (Transform Z)
                                            ui.add(
                                                Slider::new(&mut mapping.depth, -10.0..=10.0)
                                                    .text(i18n.t("transform-depth")),
                                            );

                                            ui.separator();

                                            // Mesh Selection
                                            ui.horizontal(|ui| {
                                                ui.label(i18n.t("label-mesh-type"));

                                                let current_type = mapping.mesh.mesh_type;
                                                let mut selected_type = current_type;

                                                egui::ComboBox::from_id_source("mesh_type")
                                                    .selected_text(format!("{:?}", current_type))
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(&mut selected_type, MeshType::Quad, "Quad");
                                                        ui.selectable_value(&mut selected_type, MeshType::Triangle, "Triangle");
                                                        ui.selectable_value(&mut selected_type, MeshType::Ellipse, "Ellipse");
                                                    });

                                                if selected_type != current_type {
                                                    // Replace mesh with new type
                                                    mapping.mesh = match selected_type {
                                                        MeshType::Quad => Mesh::quad(),
                                                        MeshType::Triangle => Mesh::triangle(),
                                                        MeshType::Ellipse => Mesh::ellipse(
                                                            glam::Vec2::new(0.5, 0.5),
                                                            0.5, 0.5, 32
                                                        ),
                                                        _ => Mesh::quad(), // Fallback
                                                    };
                                                }
                                            });

                                            ui.label(i18n.t_args(
                                                "label-mesh-info",
                                                &[("count", &mapping.mesh.vertex_count().to_string())],
                                            ));

                                            // Transform Controls (Pseudo-transform by modifying mesh vertices)
                                            ui.collapsing(i18n.t("header-transform"), |ui| {
                                                // Position (Translation)
                                                // Since we don't store position, we just offer to move vertices
                                                ui.label(i18n.t("transform-move-mesh"));
                                                ui.horizontal(|ui| {
                                                    if ui.button("⬅").clicked() {
                                                        translate_mesh(&mut mapping.mesh, glam::Vec2::new(-0.01, 0.0));
                                                    }
                                                    if ui.button("➡").clicked() {
                                                        translate_mesh(&mut mapping.mesh, glam::Vec2::new(0.01, 0.0));
                                                    }
                                                    if ui.button("⬆").clicked() {
                                                        translate_mesh(&mut mapping.mesh, glam::Vec2::new(0.0, -0.01));
                                                    }
                                                    if ui.button("⬇").clicked() {
                                                        translate_mesh(&mut mapping.mesh, glam::Vec2::new(0.0, 0.01));
                                                    }
                                                });
                                            });
                                        });
                                    });
                                });
                            }
                        }
                    });

                ui.separator();

                // Add Mapping Button
                if ui.button(i18n.t("btn-add-quad")).clicked() {
                    actions.push(UIAction::AddMapping);
                }
            });

        self.visible = open;
    }
}

/// Helper to translate all vertices of a mesh
fn translate_mesh(mesh: &mut Mesh, delta: glam::Vec2) {
    for vertex in &mut mesh.vertices {
        vertex.position += delta;
    }
}
