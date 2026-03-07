use egui::{Color32, Pos2, Sense, Stroke, Ui, Vec2};
use mapmap_core::module::{LayerType, MeshType, ModulePart, ModulePartId, ModulePartType};

pub fn sync_mesh_editor_to_current_selection(
    mesh_editor: &mut crate::editors::mesh_editor::MeshEditor,
    last_mesh_edit_id: &mut Option<u64>,
    part: &ModulePart,
) {
    // Extract MeshType from part
    let mesh = match &part.part_type {
        ModulePartType::Layer(LayerType::Single { mesh, .. }) => mesh,
        ModulePartType::Layer(LayerType::Group { mesh, .. }) => mesh,
        ModulePartType::Mesh(mesh) => mesh,
        _ => return, // Not a mesh-capable part
    };

    // Only reset if it's a different part
    if *last_mesh_edit_id == Some(part.id) {
        return;
    }

    *last_mesh_edit_id = Some(part.id);
    mesh_editor.mode = crate::editors::mesh_editor::EditMode::Select;

    // Visual scale for editor (0-1 -> 0-200)
    let scale = 200.0;

    match mesh {
        MeshType::Quad { tl, tr, br, bl } => {
            mesh_editor.set_from_quad(
                egui::Pos2::new(tl.0 * scale, tl.1 * scale),
                egui::Pos2::new(tr.0 * scale, tr.1 * scale),
                egui::Pos2::new(br.0 * scale, br.1 * scale),
                egui::Pos2::new(bl.0 * scale, bl.1 * scale),
            );
        }
        MeshType::BezierSurface { control_points } => {
            // Deserialize scaled points
            let points: Vec<(f32, f32)> = control_points
                .iter()
                .map(|(x, y)| (x * scale, y * scale))
                .collect();
            mesh_editor.set_from_bezier_points(&points);
        }
        // Fallback for unsupported types - reset to default quad for now
        _ => {
            mesh_editor.create_quad(egui::Pos2::new(100.0, 100.0), 200.0);
        }
    }
}

pub fn apply_mesh_editor_to_selection(
    mesh_editor: &crate::editors::mesh_editor::MeshEditor,
    part: &mut ModulePart,
) {
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
            if let Some((p_tl, p_tr, p_br, p_bl)) = mesh_editor.get_quad_corners() {
                *tl = (p_tl.x / scale, p_tl.y / scale);
                *tr = (p_tr.x / scale, p_tr.y / scale);
                *br = (p_br.x / scale, p_br.y / scale);
                *bl = (p_bl.x / scale, p_bl.y / scale);
            }
        }
        MeshType::BezierSurface { control_points } => {
            let points = mesh_editor.get_bezier_points();
            *control_points = points.iter().map(|(x, y)| (x / scale, y / scale)).collect();
        }
        _ => {
            // Other types not yet supported for write-back
        }
    }
}

pub fn render_mesh_editor_ui(
    mesh_editor: &mut crate::editors::mesh_editor::MeshEditor,
    last_mesh_edit_id: &mut Option<u64>,
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
                    *last_mesh_edit_id = None; // Trigger resync
                }
                if ui
                    .selectable_label(matches!(mesh, MeshType::Grid { .. }), "Grid")
                    .clicked()
                {
                    *mesh = MeshType::Grid { rows: 4, cols: 4 };
                    *last_mesh_edit_id = None; // Trigger resync
                }
                if ui
                    .selectable_label(matches!(mesh, MeshType::BezierSurface { .. }), "Bezier")
                    .clicked()
                {
                    // Default bezier
                    *mesh = MeshType::BezierSurface {
                        control_points: vec![],
                    };
                    *last_mesh_edit_id = None;
                }
            });

        // Resync logic if type changed (handled by caller passing part, but here we just have mesh)
        if last_mesh_edit_id.is_none() {
            let scale = 200.0;
            match mesh {
                MeshType::Quad { tl, tr, br, bl } => {
                    mesh_editor.set_from_quad(
                        egui::Pos2::new(tl.0 * scale, tl.1 * scale),
                        egui::Pos2::new(tr.0 * scale, tr.1 * scale),
                        egui::Pos2::new(br.0 * scale, br.1 * scale),
                        egui::Pos2::new(bl.0 * scale, bl.1 * scale),
                    );
                    *last_mesh_edit_id = Some(part_id);
                }
                MeshType::BezierSurface { control_points } => {
                    // Deserialize scaled points
                    let points: Vec<(f32, f32)> = control_points
                        .iter()
                        .map(|(x, y)| (x * scale, y * scale))
                        .collect();
                    mesh_editor.set_from_bezier_points(&points);
                    *last_mesh_edit_id = Some(part_id);
                }
                _ => {
                    // Fallback
                    mesh_editor.create_quad(egui::Pos2::new(100.0, 100.0), 200.0);
                    *last_mesh_edit_id = Some(part_id);
                }
            }
        }

        ui.separator();
        ui.label("Visual Editor:");

        if let Some(_action) = mesh_editor.ui(ui) {
            // Sync back
            let scale = 200.0;
            match mesh {
                MeshType::Quad { tl, tr, br, bl } => {
                    if let Some((p_tl, p_tr, p_br, p_bl)) = mesh_editor.get_quad_corners() {
                        *tl = (p_tl.x / scale, p_tl.y / scale);
                        *tr = (p_tr.x / scale, p_tr.y / scale);
                        *br = (p_br.x / scale, p_br.y / scale);
                        *bl = (p_bl.x / scale, p_bl.y / scale);
                    }
                }
                MeshType::BezierSurface { control_points } => {
                    let points = mesh_editor.get_bezier_points();
                    *control_points = points.iter().map(|(x, y)| (x / scale, y / scale)).collect();
                }
                _ => {}
            }
        }
    });
}

pub fn render_hue_spatial_editor(
    ui: &mut Ui,
    lamp_positions: &mut std::collections::HashMap<String, (f32, f32)>,
) {
    let editor_size = Vec2::new(300.0, 300.0);
    let (response, painter) = ui.allocate_painter(editor_size, Sense::click_and_drag());
    let rect = response.rect;

    // Draw background (Room representation)
    painter.rect_filled(rect, 4.0, Color32::from_gray(30));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, Color32::GRAY),
        egui::StrokeKind::Middle,
    );

    // Draw grid
    let grid_steps = 5;
    for i in 1..grid_steps {
        let t = i as f32 / grid_steps as f32;
        let x = rect.min.x + t * rect.width();
        let y = rect.min.y + t * rect.height();

        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(1.0, Color32::from_white_alpha(20)),
        );
        painter.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            Stroke::new(1.0, Color32::from_white_alpha(20)),
        );
    }

    // Labels
    painter.text(
        rect.center_top() + Vec2::new(0.0, 10.0),
        egui::Align2::CENTER_TOP,
        "Front (TV/Screen)",
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );

    // If empty, add dummy lamps for visualization/testing
    if lamp_positions.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No Lamps Mapped",
            egui::FontId::proportional(14.0),
            Color32::GRAY,
        );
        // Typically we would populate this from the Entertainment Area config
        if ui.button("Add Test Lamps").clicked() {
            lamp_positions.insert("1".to_string(), (0.2, 0.2)); // Front Left
            lamp_positions.insert("2".to_string(), (0.8, 0.2)); // Front Right
            lamp_positions.insert("3".to_string(), (0.2, 0.8)); // Rear Left
            lamp_positions.insert("4".to_string(), (0.8, 0.8)); // Rear Right
        }
        return;
    }

    let to_screen = |x: f32, y: f32| -> Pos2 {
        Pos2::new(
            rect.min.x + x.clamp(0.0, 1.0) * rect.width(),
            rect.min.y + y.clamp(0.0, 1.0) * rect.height(),
        )
    };

    // Handle lamp dragging
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let _is_dragging = ui.input(|i| i.pointer.primary_down());

    let mut dragged_lamp = None;

    // If dragging, find closest lamp
    if response.dragged() {
        if let Some(pos) = pointer_pos {
            // Find closest lamp within radius
            let mut min_dist = f32::MAX;
            let mut closest_id = None;

            for (id, (lx, ly)) in lamp_positions.iter() {
                let lamp_pos = to_screen(*lx, *ly);
                let dist = lamp_pos.distance(pos);
                if dist < 20.0 && dist < min_dist {
                    min_dist = dist;
                    closest_id = Some(id.clone());
                }
            }

            if let Some(id) = closest_id {
                dragged_lamp = Some(id);
            }
        }
    }

    if let Some(id) = dragged_lamp {
        if let Some(pos) = pointer_pos {
            // Update position
            let nx = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            let ny = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
            lamp_positions.insert(id, (nx, ny));
        }
    }

    // Draw Lamps
    for (id, (lx, ly)) in lamp_positions.iter() {
        let pos = to_screen(*lx, *ly);

        // Draw lamp body
        painter.circle_filled(pos, 8.0, Color32::from_rgb(255, 200, 100));
        painter.circle_stroke(pos, 8.0, Stroke::new(2.0, Color32::WHITE));

        // Draw Label
        painter.text(
            pos + Vec2::new(0.0, 12.0),
            egui::Align2::CENTER_TOP,
            id,
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );
    }
}
