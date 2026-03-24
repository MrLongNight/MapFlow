use super::types::*;
use egui::{Pos2, Vec2};

/// Advanced mesh editor
pub struct MeshEditor {
    /// Mesh vertices
    pub vertices: Vec<Vertex>,
    /// Mesh faces (triangles)
    pub(crate) faces: Vec<Face>,
    /// Editor mode
    pub mode: EditMode,
    /// Symmetry settings
    pub(crate) symmetry: SymmetryMode,
    /// Snap settings
    pub snap_to_grid: bool,
    pub(crate) grid_size: f32,
    /// Element currently being dragged
    pub(crate) dragging_element: Option<DragElement>,
    /// Cached center for symmetry operations
    pub(crate) cached_center: Option<Pos2>,
}

impl Default for MeshEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshEditor {
    /// Creates a new, uninitialized instance with default settings.
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
    pub(crate) fn snap_to_grid_pos(&self, pos: Pos2) -> Pos2 {
        if self.snap_to_grid {
            Pos2::new(
                (pos.x / self.grid_size).round() * self.grid_size,
                (pos.y / self.grid_size).round() * self.grid_size,
            )
        } else {
            pos
        }
    }
}
