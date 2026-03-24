use egui::{Pos2, Vec2};
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

/// Mesh vertex with Bezier control points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    /// 3D position coordinates [x, y, z].
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

/// Actions that can be triggered by the mesh editor
#[derive(Debug, Clone)]
pub enum MeshEditorAction {
    VertexAdded,
    VertexRemoved,
    MeshSubdivided,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DragElement {
    Vertex(usize, Vec2),
    ControlIn(usize, Vec2),
    ControlOut(usize, Vec2),
}
