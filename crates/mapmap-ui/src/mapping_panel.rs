//! Egui-based Mapping Manager Panel
use crate::i18n::LocaleManager;
use egui::*;
use mapmap_core::{Mapping, MappingId, MappingManager, MeshType};

#[derive(Debug, Clone)]
pub enum MappingAction {
    SelectMapping(MappingId),
    AddMapping(MeshType),
    RemoveMapping(MappingId),
    ToggleVisibility(MappingId, bool),
    UpdateMapping(MappingId, MappingUpdate),
    MoveMappingUp(MappingId),
    MoveMappingDown(MappingId),
}

#[derive(Debug, Clone, Default)]
pub struct MappingUpdate {
    pub name: Option<String>,
    pub opacity: Option<f32>,
    pub depth: Option<f32>,
    pub solo: Option<bool>,
    pub locked: Option<bool>,
    // TODO: Add mesh transformation updates if needed
}

#[derive(Debug, Default)]
pub struct MappingPanel {
    pub visible: bool,
    pub selected_mapping_id: Option<MappingId>,
    last_action: Option<MappingAction>,
}

impl MappingPanel {
    /// Take the last action performed in the panel.
    pub fn take_action(&mut self) -> Option<MappingAction> {
        self.last_action.take()
    }

    /// Render the mapping panel.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        i18n: &LocaleManager,
        mapping_manager: &MappingManager,
    ) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-mappings"))
            .open(&mut open)
            .default_size([380.0, 500.0])
            .show(ctx, |ui| {
                ui.heading(i18n.t_args(
                    "label-total-mappings",
                    &[("count", &mapping_manager.mappings().len().to_string())],
                ));
                ui.separator();

                // 1. Mapping List
                self.render_mapping_list(ui, i18n, mapping_manager);

                ui.separator();

                // 2. Selected Mapping Details
                if let Some(selected_id) = self.selected_mapping_id {
                    if let Some(mapping) = mapping_manager.get_mapping(selected_id) {
                        self.render_mapping_details(ui, i18n, mapping);
                    } else {
                        // Selected mapping no longer exists
                        self.selected_mapping_id = None;
                    }
                } else {
                    ui.label(i18n.t("label-no-mapping-selected"));
                }

                ui.separator();

                // 3. Add New Mapping Controls
                ui.horizontal(|ui| {
                    if ui.button(i18n.t("btn-add-quad")).clicked() {
                        self.last_action = Some(MappingAction::AddMapping(MeshType::Quad));
                    }
                    if ui.button(i18n.t("btn-add-triangle")).clicked() {
                        self.last_action = Some(MappingAction::AddMapping(MeshType::Triangle));
                    }
                });
            });
        self.visible = open;
    }

    fn render_mapping_list(
        &mut self,
        ui: &mut egui::Ui,
        _i18n: &LocaleManager,
        mapping_manager: &MappingManager,
    ) {
        // Use a scroll area for the list
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for mapping in mapping_manager.mappings() {
                    ui.push_id(mapping.id, |ui| {
                        ui.horizontal(|ui| {
                            // Visibility toggle
                            let mut visible = mapping.visible;
                            if ui.checkbox(&mut visible, "").changed() {
                                self.last_action =
                                    Some(MappingAction::ToggleVisibility(mapping.id, visible));
                            }

                            // Selection (clickable label)
                            let is_selected = self.selected_mapping_id == Some(mapping.id);
                            if ui.selectable_label(is_selected, &mapping.name).clicked() {
                                self.selected_mapping_id = Some(mapping.id);
                                self.last_action = Some(MappingAction::SelectMapping(mapping.id));
                            }

                            // Mesh Type Icon/Label
                            ui.label(match mapping.mesh.mesh_type {
                                MeshType::Quad => "[Quad]",
                                MeshType::Triangle => "[Tri]",
                                MeshType::Ellipse => "[Circle]",
                                MeshType::Custom => "[Custom]",
                            });
                        });
                    });
                }
            });
    }

    fn render_mapping_details(
        &mut self,
        ui: &mut egui::Ui,
        i18n: &LocaleManager,
        mapping: &Mapping,
    ) {
        ui.heading(format!("{}: {}", i18n.t("label-editing"), mapping.name));
        ui.separator();

        let mut update = MappingUpdate::default();
        let mut changed = false;

        // Name
        let mut name = mapping.name.clone();
        ui.horizontal(|ui| {
            ui.label(i18n.t("label-name"));
            if ui.text_edit_singleline(&mut name).lost_focus() && name != mapping.name {
                update.name = Some(name);
                changed = true;
            }
        });

        // Opacity
        let mut opacity = mapping.opacity;
        if ui
            .add(Slider::new(&mut opacity, 0.0..=1.0).text(i18n.t("label-opacity")))
            .changed()
        {
            update.opacity = Some(opacity);
            changed = true;
        }

        // Depth (Z-Order)
        let mut depth = mapping.depth;
        if ui
            .add(Slider::new(&mut depth, -10.0..=10.0).text(i18n.t("label-depth")))
            .changed()
        {
            update.depth = Some(depth);
            changed = true;
        }

        ui.horizontal(|ui| {
            // Solo
            let mut solo = mapping.solo;
            if ui.checkbox(&mut solo, i18n.t("check-solo")).changed() {
                update.solo = Some(solo);
                changed = true;
            }

            // Locked
            let mut locked = mapping.locked;
            if ui.checkbox(&mut locked, i18n.t("check-lock")).changed() {
                update.locked = Some(locked);
                changed = true;
            }
        });

        ui.separator();

        // Mesh Stats (Read-only for now)
        ui.label(i18n.t("header-mesh-info"));
        ui.label(format!("Type: {:?}", mapping.mesh.mesh_type));
        ui.label(format!("Vertices: {}", mapping.mesh.vertex_count()));
        ui.label(format!("Triangles: {}", mapping.mesh.triangle_count()));

        ui.separator();

        // Z-Order Controls
        ui.horizontal(|ui| {
            if ui.button("Move Up").clicked() {
                self.last_action = Some(MappingAction::MoveMappingUp(mapping.id));
            }
            if ui.button("Move Down").clicked() {
                self.last_action = Some(MappingAction::MoveMappingDown(mapping.id));
            }
        });

        ui.separator();

        // Actions
        if ui.button(i18n.t("btn-remove-this")).clicked() {
            self.last_action = Some(MappingAction::RemoveMapping(mapping.id));
            self.selected_mapping_id = None;
        }

        if changed {
            self.last_action = Some(MappingAction::UpdateMapping(mapping.id, update));
        }
    }
}
