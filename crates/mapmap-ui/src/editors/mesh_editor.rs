//! Phase 6: Advanced Warp Mesh Editor
//!
//! Advanced mesh editing with Bezier control points, subdivision surfaces,
//! symmetry mode, snap to grid/guides, and copy/paste functionality.

use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Input state for mesh editor interactions
#[derive(Debug, Clone, Copy)]
pub struct InteractionInput {
    pub pointer_pos: Pos2,
    pub clicked: bool,
    pub dragged: bool,
    pub drag_delta: Vec2,
    pub drag_started: bool,
    pub drag_stopped: bool,
}

/// Advanced mesh editor
pub struct MeshEditor {
    /// Mesh vertices
    pub vertices: Vec<Vertex>,
    /// Mesh faces (triangles)
    faces: Vec<Face>,
    /// Editor mode
    pub mode: EditMode,
    /// Symmetry settings
    symmetry: SymmetryMode,
    /// Snap settings
    pub snap_to_grid: bool,
    grid_size: f32,
    /// Element currently being dragged
    dragging_element: Option<DragElement>,
    /// Cached center for symmetry operations
    cached_center: Option<Pos2>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DragElement {
    Vertex(usize, Vec2),
    ControlIn(usize, Vec2),
    ControlOut(usize, Vec2),
}

/// Mesh vertex with Bezier control points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Pos2,
    pub control_in: Option<Vec2>,
    pub control_out: Option<Vec2>,
    pub selected: bool,
}

/// Mesh face (triangle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face {
    pub vertices: [usize; 3],
}

/// Edit mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    /// Select and move vertices
    Select,
    /// Add new vertices
    Add,
    /// Remove vertices
    Remove,
    /// Edit Bezier control points
    Bezier,
}

/// Symmetry mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetryMode {
    None,
    Horizontal,
    Vertical,
    Both,
}

impl Default for MeshEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshEditor {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            faces: Vec::new(),
            mode: EditMode::Select,
            symmetry: SymmetryMode::None,
            snap_to_grid: false,
            grid_size: 20.0,
            dragging_element: None,
            cached_center: None,
        }
    }

    /// Create a default quad mesh
    pub fn create_quad(&mut self, center: Pos2, size: f32) {
        let half = size / 2.0;
        self.set_from_quad(
            Pos2::new(center.x - half, center.y - half),
            Pos2::new(center.x + half, center.y - half),
            Pos2::new(center.x + half, center.y + half),
            Pos2::new(center.x - half, center.y + half),
        );
    }

    /// Set mesh from 4 quad corners (TL, TR, BR, BL)
    pub fn set_from_quad(&mut self, tl: Pos2, tr: Pos2, br: Pos2, bl: Pos2) {
        self.vertices.clear();
        self.faces.clear();

        self.vertices.push(Vertex {
            position: tl,
            control_in: None,
            control_out: None,
            selected: false,
        });
        self.vertices.push(Vertex {
            position: tr,
            control_in: None,
            control_out: None,
            selected: false,
        });
        self.vertices.push(Vertex {
            position: br,
            control_in: None,
            control_out: None,
            selected: false,
        });
        self.vertices.push(Vertex {
            position: bl,
            control_in: None,
            control_out: None,
            selected: false,
        });

        self.faces.push(Face {
            vertices: [0, 1, 2],
        });
        self.faces.push(Face {
            vertices: [0, 2, 3],
        });
    }

    /// Get quad corners if the mesh is a simple quad
    pub fn get_quad_corners(&self) -> Option<(Pos2, Pos2, Pos2, Pos2)> {
        if self.vertices.len() == 4 {
            Some((
                self.vertices[0].position,
                self.vertices[1].position,
                self.vertices[2].position,
                self.vertices[3].position,
            ))
        } else {
            None
        }
    }

    /// Populate mesh from a flattened list of bezier points (Pos, In, Out)
    pub fn set_from_bezier_points(&mut self, points: &[(f32, f32)]) {
        self.vertices.clear();
        self.faces.clear();

        if points.is_empty() {
            // Default if empty
            return;
        }

        let num_verts = points.len() / 3;
        for i in 0..num_verts {
            let p_pos = points[i * 3];
            let p_in = points[i * 3 + 1];
            let p_out = points[i * 3 + 2];

            let pos = Pos2::new(p_pos.0, p_pos.1);
            // If control point == position, treat as None
            let ctrl_in = if (p_in.0 - p_pos.0).abs() < 0.001 && (p_in.1 - p_pos.1).abs() < 0.001 {
                None
            } else {
                Some(Vec2::new(p_in.0 - p_pos.0, p_in.1 - p_pos.1))
            };
            let ctrl_out = if (p_out.0 - p_pos.0).abs() < 0.001 && (p_out.1 - p_pos.1).abs() < 0.001
            {
                None
            } else {
                Some(Vec2::new(p_out.0 - p_pos.0, p_out.1 - p_pos.1))
            };

            self.vertices.push(Vertex {
                position: pos,
                control_in: ctrl_in,
                control_out: ctrl_out,
                selected: false,
            });
        }

        // Rebuild simple quad faces if 4 vertices (Standard Quad topology)
        if self.vertices.len() == 4 {
            self.faces.push(Face {
                vertices: [0, 1, 2],
            });
            self.faces.push(Face {
                vertices: [0, 2, 3],
            });
        }
    }

    /// Serialize mesh to flattened list of bezier points (Pos, In, Out)
    pub fn get_bezier_points(&self) -> Vec<(f32, f32)> {
        let mut points = Vec::new();
        for v in &self.vertices {
            let pos = v.position;
            let p_in = v.control_in.map(|o| pos + o).unwrap_or(pos);
            let p_out = v.control_out.map(|o| pos + o).unwrap_or(pos);

            points.push((pos.x, pos.y));
            points.push((p_in.x, p_in.y));
            points.push((p_out.x, p_out.y));
        }
        points
    }

    /// Subdivide the mesh
    pub fn subdivide(&mut self) {
        // Catmull-Clark subdivision (simplified)
        let mut new_vertices = self.vertices.clone();
        let mut new_faces = Vec::new();

        for face in &self.faces {
            // Calculate face center
            let face_center = Pos2::new(
                (self.vertices[face.vertices[0]].position.x
                    + self.vertices[face.vertices[1]].position.x
                    + self.vertices[face.vertices[2]].position.x)
                    / 3.0,
                (self.vertices[face.vertices[0]].position.y
                    + self.vertices[face.vertices[1]].position.y
                    + self.vertices[face.vertices[2]].position.y)
                    / 3.0,
            );

            let center_idx = new_vertices.len();
            new_vertices.push(Vertex {
                position: face_center,
                control_in: None,
                control_out: None,
                selected: false,
            });

            // Create new faces
            for i in 0..3 {
                let v0 = face.vertices[i];
                let v1 = face.vertices[(i + 1) % 3];

                // Edge midpoint
                let edge_mid = Pos2::new(
                    (self.vertices[v0].position.x + self.vertices[v1].position.x) / 2.0,
                    (self.vertices[v0].position.y + self.vertices[v1].position.y) / 2.0,
                );

                let mid_idx = new_vertices.len();
                new_vertices.push(Vertex {
                    position: edge_mid,
                    control_in: None,
                    control_out: None,
                    selected: false,
                });

                new_faces.push(Face {
                    vertices: [v0, mid_idx, center_idx],
                });
            }
        }

        self.vertices = new_vertices;
        self.faces = new_faces;
    }

    /// Snap position to grid
    fn snap_to_grid_pos(&self, pos: Pos2) -> Pos2 {
        if self.snap_to_grid {
            Pos2::new(
                (pos.x / self.grid_size).round() * self.grid_size,
                (pos.y / self.grid_size).round() * self.grid_size,
            )
        } else {
            pos
        }
    }

    /// Process interaction event
    pub fn handle_interaction(&mut self, input: InteractionInput) -> Option<MeshEditorAction> {
        let mut action = None;
        let pointer_pos = input.pointer_pos;

        match self.mode {
            EditMode::Select => {
                if input.clicked {
                    // Select vertex under pointer
                    let mut found = false;
                    for vertex in self.vertices.iter_mut() {
                        if vertex.position.distance(pointer_pos) < 10.0 {
                            vertex.selected = !vertex.selected;
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        // Deselect all
                        for vertex in &mut self.vertices {
                            vertex.selected = false;
                        }
                    }
                }

                if input.dragged {
                    // Drag selected vertices
                    let delta = input.drag_delta;
                    let snap_to_grid = self.snap_to_grid;
                    let grid_size = self.grid_size;
                    for vertex in &mut self.vertices {
                        if vertex.selected {
                            let new_pos = vertex.position + delta;
                            // Inline snap_to_grid_pos logic to avoid borrow conflict
                            vertex.position = if snap_to_grid {
                                Pos2::new(
                                    (new_pos.x / grid_size).round() * grid_size,
                                    (new_pos.y / grid_size).round() * grid_size,
                                )
                            } else {
                                new_pos
                            };
                        }
                    }
                }
            }
            EditMode::Add => {
                if input.clicked {
                    let pos = self.snap_to_grid_pos(pointer_pos);
                    self.vertices.push(Vertex {
                        position: pos,
                        control_in: None,
                        control_out: None,
                        selected: false,
                    });
                    action = Some(MeshEditorAction::VertexAdded);
                }
            }
            EditMode::Remove => {
                if input.clicked {
                    // Remove vertex under pointer
                    if let Some(idx) = self
                        .vertices
                        .iter()
                        .position(|v| v.position.distance(pointer_pos) < 10.0)
                    {
                        self.vertices.remove(idx);
                        // Remove faces referencing this vertex
                        self.faces.retain(|f| !f.vertices.contains(&idx));
                        action = Some(MeshEditorAction::VertexRemoved);
                    }
                }
            }
            EditMode::Bezier => {
                // Handle drag start
                if input.drag_started {
                    // Calculate bounds center for symmetry
                    if !self.vertices.is_empty() {
                        let mut min = self.vertices[0].position;
                        let mut max = self.vertices[0].position;
                        for v in &self.vertices {
                            min = min.min(v.position);
                            max = max.max(v.position);
                        }
                        self.cached_center = Some(min + (max - min) / 2.0);
                    }

                    // Check control points first
                    let mut found = false;
                    for (idx, vertex) in self.vertices.iter().enumerate() {
                        if let Some(ctrl_in) = vertex.control_in {
                            let pos = vertex.position + ctrl_in;
                            if pos.distance(pointer_pos) < 6.0 {
                                let offset = pos - pointer_pos;
                                self.dragging_element = Some(DragElement::ControlIn(idx, offset));
                                found = true;
                                break;
                            }
                        }
                        if let Some(ctrl_out) = vertex.control_out {
                            let pos = vertex.position + ctrl_out;
                            if pos.distance(pointer_pos) < 6.0 {
                                let offset = pos - pointer_pos;
                                self.dragging_element = Some(DragElement::ControlOut(idx, offset));
                                found = true;
                                break;
                            }
                        }
                    }

                    if !found {
                        // Check vertices
                        for (idx, vertex) in self.vertices.iter().enumerate() {
                            if vertex.position.distance(pointer_pos) < 10.0 {
                                let offset = vertex.position - pointer_pos;
                                self.dragging_element = Some(DragElement::Vertex(idx, offset));
                                break;
                            }
                        }
                    }
                }

                // Handle dragging
                if input.dragged {
                    if let Some(element) = self.dragging_element {
                        let snap_to_grid = self.snap_to_grid;
                        let grid_size = self.grid_size;
                        let symmetry = self.symmetry;
                        let center = self.cached_center.unwrap_or(Pos2::ZERO);

                        let snap_pos = |pos: Pos2| -> Pos2 {
                            if snap_to_grid {
                                Pos2::new(
                                    (pos.x / grid_size).round() * grid_size,
                                    (pos.y / grid_size).round() * grid_size,
                                )
                            } else {
                                pos
                            }
                        };

                        // Helper to find symmetric vertex index
                        let find_symmetric =
                            |target_idx: usize, mode: SymmetryMode| -> Option<usize> {
                                if mode == SymmetryMode::None {
                                    return None;
                                }
                                let target_pos = self.vertices[target_idx].position;

                                // Find counterpart
                                for (i, v) in self.vertices.iter().enumerate() {
                                    if i == target_idx {
                                        continue;
                                    }

                                    let mut is_sym = false;
                                    match mode {
                                        SymmetryMode::Horizontal => {
                                            // Mirror X around center.x
                                            let sym_x = 2.0 * center.x - target_pos.x;
                                            if (v.position.x - sym_x).abs() < 1.0
                                                && (v.position.y - target_pos.y).abs() < 1.0
                                            {
                                                is_sym = true;
                                            }
                                        }
                                        SymmetryMode::Vertical => {
                                            // Mirror Y around center.y
                                            let sym_y = 2.0 * center.y - target_pos.y;
                                            if (v.position.x - target_pos.x).abs() < 1.0
                                                && (v.position.y - sym_y).abs() < 1.0
                                            {
                                                is_sym = true;
                                            }
                                        }
                                        SymmetryMode::Both => {
                                            let sym_x = 2.0 * center.x - target_pos.x;
                                            let sym_y = 2.0 * center.y - target_pos.y;
                                            if (v.position.x - sym_x).abs() < 1.0
                                                && (v.position.y - sym_y).abs() < 1.0
                                            {
                                                is_sym = true;
                                            }
                                        }
                                        _ => {}
                                    }
                                    if is_sym {
                                        return Some(i);
                                    }
                                }
                                None
                            };

                        match element {
                            DragElement::Vertex(idx, offset) => {
                                // Find symmetric partner BEFORE moving
                                let sym_idx = find_symmetric(idx, symmetry);

                                if let Some(vertex) = self.vertices.get_mut(idx) {
                                    let target_pos = pointer_pos + offset;
                                    vertex.position = snap_pos(target_pos);
                                }

                                // Update symmetric partner
                                if let Some(s_idx) = sym_idx {
                                    // Recalculate target for symmetric partner
                                    let prime_pos = self.vertices[idx].position; // New pos of dragged
                                    let mut sym_pos = prime_pos;

                                    if symmetry == SymmetryMode::Horizontal
                                        || symmetry == SymmetryMode::Both
                                    {
                                        sym_pos.x = 2.0 * center.x - prime_pos.x;
                                    }
                                    if symmetry == SymmetryMode::Vertical
                                        || symmetry == SymmetryMode::Both
                                    {
                                        sym_pos.y = 2.0 * center.y - prime_pos.y;
                                    }

                                    if let Some(s_vert) = self.vertices.get_mut(s_idx) {
                                        s_vert.position = sym_pos; // Already snapped indirectly
                                    }
                                }
                            }
                            DragElement::ControlIn(idx, offset) => {
                                // Find symmetric partner BEFORE moving
                                let sym_idx = find_symmetric(idx, symmetry);

                                if let Some(vertex) = self.vertices.get_mut(idx) {
                                    if let Some(ctrl) = &mut vertex.control_in {
                                        let target_abs_pos = pointer_pos + offset;
                                        let snapped_abs = snap_pos(target_abs_pos);
                                        *ctrl = snapped_abs - vertex.position;
                                    }
                                }

                                // Update symmetric partner
                                if let Some(s_idx) = sym_idx {
                                    // Extract control value first to avoid overlapping borrows
                                    let mut sym_ctrl = None;
                                    if let Some(vertex) = self.vertices.get(idx) {
                                        if let Some(ctrl) = vertex.control_in {
                                            let mut c = ctrl;
                                            if symmetry == SymmetryMode::Horizontal
                                                || symmetry == SymmetryMode::Both
                                            {
                                                c.x = -c.x;
                                            }
                                            if symmetry == SymmetryMode::Vertical
                                                || symmetry == SymmetryMode::Both
                                            {
                                                c.y = -c.y;
                                            }
                                            sym_ctrl = Some(c);
                                        }
                                    }

                                    // Apply mutation
                                    if let Some(val) = sym_ctrl {
                                        if let Some(s_vert) = self.vertices.get_mut(s_idx) {
                                            if let Some(s_ctrl) = &mut s_vert.control_in {
                                                *s_ctrl = val;
                                            }
                                        }
                                    }
                                }
                            }
                            DragElement::ControlOut(idx, offset) => {
                                // Find symmetric partner BEFORE moving
                                let sym_idx = find_symmetric(idx, symmetry);

                                if let Some(vertex) = self.vertices.get_mut(idx) {
                                    if let Some(ctrl) = &mut vertex.control_out {
                                        let target_abs_pos = pointer_pos + offset;
                                        let snapped_abs = snap_pos(target_abs_pos);
                                        *ctrl = snapped_abs - vertex.position;
                                    }
                                }

                                // Update symmetric partner
                                if let Some(s_idx) = sym_idx {
                                    // Extract control value first to avoid overlapping borrows
                                    let mut sym_ctrl = None;
                                    if let Some(vertex) = self.vertices.get(idx) {
                                        if let Some(ctrl) = vertex.control_out {
                                            let mut c = ctrl;
                                            if symmetry == SymmetryMode::Horizontal
                                                || symmetry == SymmetryMode::Both
                                            {
                                                c.x = -c.x;
                                            }
                                            if symmetry == SymmetryMode::Vertical
                                                || symmetry == SymmetryMode::Both
                                            {
                                                c.y = -c.y;
                                            }
                                            sym_ctrl = Some(c);
                                        }
                                    }

                                    // Apply mutation
                                    if let Some(val) = sym_ctrl {
                                        if let Some(s_vert) = self.vertices.get_mut(s_idx) {
                                            if let Some(s_ctrl) = &mut s_vert.control_out {
                                                *s_ctrl = val;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Handle drag stop
                if input.drag_stopped {
                    self.dragging_element = None;
                    self.cached_center = None;
                }

                // Handle click (to add controls)
                if input.clicked {
                    for vertex in self.vertices.iter_mut() {
                        if vertex.position.distance(pointer_pos) < 10.0 {
                            // Add default controls if none exist
                            if vertex.control_in.is_none() && vertex.control_out.is_none() {
                                vertex.control_in = Some(Vec2::new(-30.0, 0.0));
                                vertex.control_out = Some(Vec2::new(30.0, 0.0));
                            }
                            break;
                        }
                    }
                }
            }
        }

        action
    }

    /// Render the mesh editor UI
    pub fn ui(&mut self, ui: &mut Ui) -> Option<MeshEditorAction> {
        let mut action = None;

        // Toolbar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.mode, EditMode::Select, "Select");
            ui.selectable_value(&mut self.mode, EditMode::Add, "Add");
            ui.selectable_value(&mut self.mode, EditMode::Remove, "Remove");
            ui.selectable_value(&mut self.mode, EditMode::Bezier, "Bezier");

            ui.separator();

            ui.checkbox(&mut self.snap_to_grid, "Snap to Grid");
            if self.snap_to_grid {
                ui.add(egui::DragValue::new(&mut self.grid_size).prefix("Grid: "));
            }

            ui.separator();

            ui.label("Symmetry:");
            ui.selectable_value(&mut self.symmetry, SymmetryMode::None, "None");
            ui.selectable_value(&mut self.symmetry, SymmetryMode::Horizontal, "H");
            ui.selectable_value(&mut self.symmetry, SymmetryMode::Vertical, "V");
            ui.selectable_value(&mut self.symmetry, SymmetryMode::Both, "Both");

            ui.separator();

            if ui.button("Subdivide").clicked() {
                self.subdivide();
            }

            if ui.button("Create Quad").clicked() {
                self.create_quad(Pos2::new(400.0, 300.0), 200.0);
            }
        });

        ui.separator();

        // Canvas
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        // Draw grid if enabled
        if self.snap_to_grid {
            self.draw_grid(&painter, response.rect);
        }

        // Draw mesh faces
        for face in &self.faces {
            let points = [
                self.vertices[face.vertices[0]].position,
                self.vertices[face.vertices[1]].position,
                self.vertices[face.vertices[2]].position,
            ];

            painter.add(egui::Shape::convex_polygon(
                points.to_vec(),
                Color32::from_rgba_premultiplied(100, 100, 150, 50),
                Stroke::new(1.0, Color32::from_rgb(150, 150, 200)),
            ));
        }

        // Draw vertices
        for vertex in self.vertices.iter() {
            let color = if vertex.selected {
                Color32::from_rgb(255, 200, 100)
            } else {
                Color32::from_rgb(200, 200, 200)
            };

            painter.circle_filled(vertex.position, 6.0, color);
            painter.circle_stroke(vertex.position, 6.0, Stroke::new(2.0, Color32::WHITE));

            // Draw Bezier control points if in Bezier mode
            if self.mode == EditMode::Bezier {
                if let Some(ctrl_in) = vertex.control_in {
                    let ctrl_pos = vertex.position + ctrl_in;
                    painter.line_segment(
                        [vertex.position, ctrl_pos],
                        Stroke::new(1.0, Color32::from_rgb(100, 200, 255)),
                    );
                    painter.circle_filled(ctrl_pos, 4.0, Color32::from_rgb(100, 200, 255));
                }

                if let Some(ctrl_out) = vertex.control_out {
                    let ctrl_pos = vertex.position + ctrl_out;
                    painter.line_segment(
                        [vertex.position, ctrl_pos],
                        Stroke::new(1.0, Color32::from_rgb(255, 200, 100)),
                    );
                    painter.circle_filled(ctrl_pos, 4.0, Color32::from_rgb(255, 200, 100));
                }
            }
        }

        // Handle interactions
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let input = InteractionInput {
                pointer_pos,
                clicked: response.clicked(),
                dragged: response.dragged(),
                drag_delta: response.drag_delta(),
                drag_started: response.drag_started(),
                drag_stopped: response.drag_stopped(),
            };

            if let Some(act) = self.handle_interaction(input) {
                action = Some(act);
            }
        }

        action
    }

    /// Draw grid background
    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let color = Color32::from_rgb(50, 50, 50);

        let mut x = 0.0;
        while x < rect.width() {
            let pos_x = rect.min.x + x;
            painter.line_segment(
                [Pos2::new(pos_x, rect.min.y), Pos2::new(pos_x, rect.max.y)],
                Stroke::new(1.0, color),
            );
            x += self.grid_size;
        }

        let mut y = 0.0;
        while y < rect.height() {
            let pos_y = rect.min.y + y;
            painter.line_segment(
                [Pos2::new(rect.min.x, pos_y), Pos2::new(rect.max.x, pos_y)],
                Stroke::new(1.0, color),
            );
            y += self.grid_size;
        }
    }
}

/// Actions that can be triggered by the mesh editor
#[derive(Debug, Clone)]
pub enum MeshEditorAction {
    VertexAdded,
    VertexRemoved,
    MeshSubdivided,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_detection_and_dragging() {
        let mut editor = MeshEditor::new();
        // Create a single vertex at (100, 100)
        editor.vertices.push(Vertex {
            position: Pos2::new(100.0, 100.0),
            control_in: Some(Vec2::new(-20.0, 0.0)),
            control_out: Some(Vec2::new(20.0, 0.0)),
            selected: false,
        });
        editor.mode = EditMode::Bezier;

        // 1. Test Hit Detection (Control Point)
        // Click near control_out (100 + 20 = 120, 100)
        let input_click = InteractionInput {
            pointer_pos: Pos2::new(122.0, 100.0), // Within 6.0 radius
            clicked: false,
            dragged: false,
            drag_delta: Vec2::ZERO,
            drag_started: true,
            drag_stopped: false,
        };
        editor.handle_interaction(input_click);

        // Should be dragging ControlOut of vertex 0
        match editor.dragging_element {
            Some(DragElement::ControlOut(0, _)) => {}
            _ => panic!(
                "Should be dragging ControlOut, got {:?}",
                editor.dragging_element
            ),
        }

        // 2. Test Dragging
        // Drag to (150, 100).
        // Original offset was (120, 100) - (122, 100) = (-2, 0).
        // Target abs pos = (150, 100) + (-2, 0) = (148, 100).
        // Snap OFF.
        // New ctrl = (148, 100) - (100, 100) = (48, 0).
        let input_drag = InteractionInput {
            pointer_pos: Pos2::new(150.0, 100.0),
            clicked: false,
            dragged: true,
            drag_delta: Vec2::new(28.0, 0.0),
            drag_started: false,
            drag_stopped: false,
        };
        editor.handle_interaction(input_drag);

        let v = &editor.vertices[0];
        let ctrl_out = v.control_out.unwrap();
        assert!(
            (ctrl_out.x - 48.0).abs() < 0.001,
            "Expected 48.0, got {}",
            ctrl_out.x
        );

        // 3. Test Drag Stop
        let input_stop = InteractionInput {
            pointer_pos: Pos2::new(150.0, 100.0),
            clicked: false,
            dragged: false,
            drag_delta: Vec2::ZERO,
            drag_started: false,
            drag_stopped: true,
        };
        editor.handle_interaction(input_stop);
        assert!(editor.dragging_element.is_none());
    }

    #[test]
    fn test_grid_snapping() {
        let mut editor = MeshEditor::new();
        editor.vertices.push(Vertex {
            position: Pos2::new(100.0, 100.0),
            control_in: None,
            control_out: None,
            selected: false,
        });
        editor.mode = EditMode::Bezier;
        editor.snap_to_grid = true;
        editor.grid_size = 20.0;

        // Start drag on vertex (100, 100)
        let input_start = InteractionInput {
            pointer_pos: Pos2::new(101.0, 101.0), // Slight offset
            clicked: false,
            dragged: false,
            drag_delta: Vec2::ZERO,
            drag_started: true,
            drag_stopped: false,
        };
        editor.handle_interaction(input_start);

        // Drag to (115, 115)
        // Offset = (100, 100) - (101, 101) = (-1, -1)
        // Target = (115, 115) + (-1, -1) = (114, 114)
        // Snap (114, 114) to grid 20 -> (120, 120) or (100, 120)?
        // 114/20 = 5.7 -> 6 * 20 = 120.
        let input_drag = InteractionInput {
            pointer_pos: Pos2::new(115.0, 115.0),
            clicked: false,
            dragged: true,
            drag_delta: Vec2::ZERO,
            drag_started: false,
            drag_stopped: false,
        };
        editor.handle_interaction(input_drag);

        let v = &editor.vertices[0];
        assert_eq!(v.position, Pos2::new(120.0, 120.0));
    }

    #[test]
    fn test_control_point_creation() {
        let mut editor = MeshEditor::new();
        editor.vertices.push(Vertex {
            position: Pos2::new(100.0, 100.0),
            control_in: None,
            control_out: None,
            selected: false,
        });
        editor.mode = EditMode::Bezier;

        // Click on vertex
        let input_click = InteractionInput {
            pointer_pos: Pos2::new(102.0, 102.0),
            clicked: true,
            dragged: false,
            drag_delta: Vec2::ZERO,
            drag_started: false,
            drag_stopped: false,
        };
        editor.handle_interaction(input_click);

        let v = &editor.vertices[0];
        assert!(v.control_in.is_some());
        assert!(v.control_out.is_some());
        assert_eq!(v.control_in.unwrap(), Vec2::new(-30.0, 0.0));
    }
}




