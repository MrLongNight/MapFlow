use super::state::*;
use super::types::*;
use egui::{Pos2, Vec2};

pub trait MeshEditorInteraction {
    fn handle_interaction(&mut self, input: InteractionInput) -> Option<MeshEditorAction>;
}

impl MeshEditorInteraction for MeshEditor {
    /// Process interaction event
    fn handle_interaction(&mut self, input: InteractionInput) -> Option<MeshEditorAction> {
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
}
