use super::state::ModuleCanvas;
use egui::{Pos2, Ui};
use mapmap_core::module::{LayerType, MeshType, ModulePart, ModulePartId, ModulePartType};

impl ModuleCanvas {
    /// Sync the mesh editor with the current selection's mesh
    pub fn sync_mesh_editor_to_current_selection(&mut self, part: &ModulePart) {
        // Extract MeshType from part
        let mesh = match &part.part_type {
            ModulePartType::Layer(LayerType::Single { mesh, .. }) => mesh,
            ModulePartType::Layer(LayerType::Group { mesh, .. }) => mesh,
            ModulePartType::Mesh(mesh) => mesh,
            _ => return, // Not a mesh-capable part
        };

        // Only reset if it's a different part
        if self.last_mesh_edit_id == Some(part.id) {
            return;
        }

        self.last_mesh_edit_id = Some(part.id);
        self.mesh_editor.mode = crate::editors::mesh_editor::EditMode::Select;

        // Visual scale for editor (0-1 -> 0-200)
        let scale = 200.0;

        match mesh {
            MeshType::Quad { tl, tr, br, bl } => {
                self.mesh_editor.set_from_quad(
                    Pos2::new(tl.0 * scale, tl.1 * scale),
                    Pos2::new(tr.0 * scale, tr.1 * scale),
                    Pos2::new(br.0 * scale, br.1 * scale),
                    Pos2::new(bl.0 * scale, bl.1 * scale),
                );
            }
            MeshType::BezierSurface { control_points } => {
                // Deserialize scaled points
                let points: Vec<(f32, f32)> = control_points
                    .iter()
                    .map(|(x, y)| (x * scale, y * scale))
                    .collect();
                self.mesh_editor.set_from_bezier_points(&points);
            }
            // Fallback for unsupported types - reset to default quad for now
            _ => {
                self.mesh_editor.create_quad(Pos2::new(100.0, 100.0), 200.0);
            }
        }
    }

    /// Apply mesh editor changes back to the selection
    pub fn apply_mesh_editor_to_selection(&mut self, part: &mut ModulePart) {
        // Get mutable reference to mesh
        let mesh = match &mut part.part_type {
            ModulePartType::Layer(LayerType::Single { mesh, .. }) => mesh,
            ModulePartType::Layer(LayerType::Group { mesh, .. }) => mesh,
            ModulePartType::Mesh(mesh) => mesh,
            _ => return,
        };

        let scale = 200.0;

        // Try to update current mesh type
        match mesh {
            MeshType::Quad { tl, tr, br, bl } => {
                if let Some((p_tl, p_tr, p_br, p_bl)) = self.mesh_editor.get_quad_corners() {
                    *tl = (p_tl.x / scale, p_tl.y / scale);
                    *tr = (p_tr.x / scale, p_tr.y / scale);
                    *br = (p_br.x / scale, p_br.y / scale);
                    *bl = (p_bl.x / scale, p_bl.y / scale);
                }
            }
            MeshType::BezierSurface { control_points } => {
                let points = self.mesh_editor.get_bezier_points();
                *control_points = points.iter().map(|(x, y)| (x / scale, y / scale)).collect();
            }
            _ => {
                // Other types not yet supported for write-back
            }
        }
    }

    /// Render the unified mesh editor UI for a given mesh
    pub fn render_mesh_editor_ui(
        &mut self,
        ui: &mut Ui,
        mesh: &mut MeshType,
        part_id: ModulePartId,
        id_salt: u64,
    ) {
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("🕸️ï¸  Mesh/Geometry").strong());
            ui.separator();

            egui::ComboBox::from_id_salt(format!("mesh_type_{}", id_salt))
                .selected_text(match mesh {
                    MeshType::Quad { .. } => "Quad",
                    MeshType::Grid { .. } => "Grid",
                    MeshType::BezierSurface { .. } => "Bezier",
                    MeshType::Polygon { .. } => "Polygon",
                    MeshType::TriMesh => "Triangle",
                    MeshType::Circle { .. } => "Circle",
                    MeshType::Cylinder { .. } => "Cylinder",
                    MeshType::Sphere { .. } => "Sphere",
                    MeshType::Custom { .. } => "Custom",
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(matches!(mesh, MeshType::Quad { .. }), "Quad")
                        .clicked()
                    {
                        *mesh = MeshType::Quad {
                            tl: (0.0, 0.0),
                            tr: (1.0, 0.0),
                            br: (1.0, 1.0),
                            bl: (0.0, 1.0),
                        };
                        self.last_mesh_edit_id = None; // Trigger resync
                    }
                    if ui
                        .selectable_label(matches!(mesh, MeshType::Grid { .. }), "Grid")
                        .clicked()
                    {
                        *mesh = MeshType::Grid { rows: 4, cols: 4 };
                        self.last_mesh_edit_id = None; // Trigger resync
                    }
                    if ui
                        .selectable_label(matches!(mesh, MeshType::BezierSurface { .. }), "Bezier")
                        .clicked()
                    {
                        // Default bezier
                        *mesh = MeshType::BezierSurface {
                            control_points: vec![],
                        };
                        self.last_mesh_edit_id = None;
                    }
                });

            // Resync logic if type changed (handled by caller passing part, but here we just have mesh)
            if self.last_mesh_edit_id.is_none() {
                let scale = 200.0;
                match mesh {
                    MeshType::Quad { tl, tr, br, bl } => {
                        self.mesh_editor.set_from_quad(
                            Pos2::new(tl.0 * scale, tl.1 * scale),
                            Pos2::new(tr.0 * scale, tr.1 * scale),
                            Pos2::new(br.0 * scale, br.1 * scale),
                            Pos2::new(bl.0 * scale, bl.1 * scale),
                        );
                        self.last_mesh_edit_id = Some(part_id);
                    }
                    MeshType::BezierSurface { control_points } => {
                        // Deserialize scaled points
                        let points: Vec<(f32, f32)> = control_points
                            .iter()
                            .map(|(x, y)| (x * scale, y * scale))
                            .collect();
                        self.mesh_editor.set_from_bezier_points(&points);
                        self.last_mesh_edit_id = Some(part_id);
                    }
                    _ => {
                        // Fallback
                        self.mesh_editor.create_quad(Pos2::new(100.0, 100.0), 200.0);
                        self.last_mesh_edit_id = Some(part_id);
                    }
                }
            }

            ui.separator();
            ui.label("Visual Editor:");

            if let Some(_action) = self.mesh_editor.ui(ui) {
                // Sync back
                let scale = 200.0;
                match mesh {
                    MeshType::Quad { tl, tr, br, bl } => {
                        if let Some((p_tl, p_tr, p_br, p_bl)) = self.mesh_editor.get_quad_corners()
                        {
                            *tl = (p_tl.x / scale, p_tl.y / scale);
                            *tr = (p_tr.x / scale, p_tr.y / scale);
                            *br = (p_br.x / scale, p_br.y / scale);
                            *bl = (p_bl.x / scale, p_bl.y / scale);
                        }
                    }
                    MeshType::BezierSurface { control_points } => {
                        let points = self.mesh_editor.get_bezier_points();
                        *control_points =
                            points.iter().map(|(x, y)| (x / scale, y / scale)).collect();
                    }
                    _ => {}
                }
            }
        });
    }
}
