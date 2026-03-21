use super::InspectorAction;
use crate::editors::mesh_editor::ui::MeshEditorUi;
use crate::i18n::LocaleManager;
use egui::Ui;
use mapmap_core::{Layer, Transform};

#[allow(clippy::too_many_arguments)]
pub fn render_layer_inspector(
    mesh_editor: &mut crate::editors::mesh_editor::MeshEditor,
    last_mesh_edit_id: &mut Option<u64>,
    ui: &mut Ui,
    layer: &Layer,
    transform: &Transform,
    _index: usize,
    first_mapping: Option<&mapmap_core::mapping::Mapping>,
    i18n: &LocaleManager,
) -> Option<InspectorAction> {
    let mut action = None;

    ui.vertical(|ui| {
        // Name & Type info
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&layer.name).strong().size(16.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("LAYER").weak().italics());
            });
        });

        ui.add_space(10.0);
        ui.separator();

        // --- TRANSFORM SECTION ---
        egui::CollapsingHeader::new(i18n.t("inspector-transform"))
            .default_open(true)
            .show(ui, |ui| {
                let mut new_transform = *transform;
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Position:");
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut new_transform.position.x)
                                .speed(1.0)
                                .prefix("X: "),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut new_transform.position.y)
                                .speed(1.0)
                                .prefix("Y: "),
                        )
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut new_transform.scale.x)
                                .speed(0.01)
                                .prefix("X: "),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut new_transform.scale.y)
                                .speed(0.01)
                                .prefix("Y: "),
                        )
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut new_transform.rotation.x)
                                .speed(1.0)
                                .prefix("X: ")
                                .suffix("°"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut new_transform.rotation.y)
                                .speed(1.0)
                                .prefix("Y: ")
                                .suffix("°"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut new_transform.rotation.z)
                                .speed(1.0)
                                .prefix("Z: ")
                                .suffix("°"),
                        )
                        .changed();
                });

                if changed {
                    action = Some(InspectorAction::UpdateTransform(layer.id, new_transform));
                }
            });

        ui.add_space(10.0);

        // --- APPEARANCE SECTION ---
        egui::CollapsingHeader::new(i18n.t("inspector-appearance"))
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Opacity:");
                    let mut opacity = layer.opacity;
                    if ui.add(egui::Slider::new(&mut opacity, 0.0..=1.0)).changed() {
                        action = Some(InspectorAction::UpdateOpacity(layer.id, opacity));
                    }
                });
            });

        ui.add_space(10.0);

        // --- MESH EDITOR SECTION ---
        egui::CollapsingHeader::new("Mesh Warp Editor")
            .default_open(false)
            .show(ui, |ui| {
                if let Some(mapping) = first_mapping {
                    ui.label(format!("Editing Mesh for Mapping: {}", mapping.name));

                    let scale = 200.0;

                    // Sync logic
                    if *last_mesh_edit_id != Some(mapping.id) {
                        *last_mesh_edit_id = Some(mapping.id);
                        mesh_editor.mode = crate::editors::mesh_editor::EditMode::Select;

                        match &mapping.mesh.mesh_type {
                            mapmap_core::mesh::MeshType::Quad => {
                                if mapping.mesh.vertices.len() >= 4 {
                                    mesh_editor.set_from_quad(
                                        egui::Pos2::new(
                                            mapping.mesh.vertices[0].position.x * scale,
                                            mapping.mesh.vertices[0].position.y * scale,
                                        ),
                                        egui::Pos2::new(
                                            mapping.mesh.vertices[1].position.x * scale,
                                            mapping.mesh.vertices[1].position.y * scale,
                                        ),
                                        egui::Pos2::new(
                                            mapping.mesh.vertices[2].position.x * scale,
                                            mapping.mesh.vertices[2].position.y * scale,
                                        ),
                                        egui::Pos2::new(
                                            mapping.mesh.vertices[3].position.x * scale,
                                            mapping.mesh.vertices[3].position.y * scale,
                                        ),
                                    );
                                }
                            }
                            _ => {
                                mesh_editor.create_quad(egui::Pos2::new(100.0, 100.0), 200.0);
                            }
                        }
                    }

                    // Render UI
                    if let Some(_edit_action) = mesh_editor.ui(ui) {
                        // Sync back to a clone of the mesh
                        let mut new_mesh = mapping.mesh.clone();
                        if new_mesh.mesh_type == mapmap_core::mesh::MeshType::Quad {
                            if let Some((p_tl, p_tr, p_br, p_bl)) = mesh_editor.get_quad_corners() {
                                if new_mesh.vertices.len() >= 4 {
                                    new_mesh.vertices[0].position =
                                        glam::Vec2::new(p_tl.x / scale, p_tl.y / scale);
                                    new_mesh.vertices[1].position =
                                        glam::Vec2::new(p_tr.x / scale, p_tr.y / scale);
                                    new_mesh.vertices[2].position =
                                        glam::Vec2::new(p_br.x / scale, p_br.y / scale);
                                    new_mesh.vertices[3].position =
                                        glam::Vec2::new(p_bl.x / scale, p_bl.y / scale);
                                    new_mesh.revision += 1;
                                    action = Some(InspectorAction::UpdateMappingMesh(
                                        mapping.id, new_mesh,
                                    ));
                                }
                            }
                        }
                    }

                    if ui.button("Reset Mesh").clicked() {
                        let mut reset_mesh = mapmap_core::Mesh::quad();
                        reset_mesh.revision += 1;
                        action = Some(InspectorAction::UpdateMappingMesh(mapping.id, reset_mesh));
                        *last_mesh_edit_id = None; // Trigger resync
                    }
                } else {
                    ui.label(
                        egui::RichText::new("No mapping available to edit mesh.")
                            .weak()
                            .italics(),
                    );
                }
            });
    });

    action
}
