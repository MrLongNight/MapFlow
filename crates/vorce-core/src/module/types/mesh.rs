//!
//! Mesh data and definitions.
//!

use serde::{Deserialize, Serialize};

/// Mesh geometry definitions for projection mapping surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MeshType {
    /// A simple four-cornered quadrilateral.
    Quad {
        /// Top-left corner coordinates.
        tl: (f32, f32),
        /// Top-right corner coordinates.
        tr: (f32, f32),
        /// Bottom-right corner coordinates.
        br: (f32, f32),
        /// Bottom-left corner coordinates.
        bl: (f32, f32),
    },
    /// A rectangular grid of vertices, useful for complex warping.
    Grid {
        /// Number of horizontal divisions.
        rows: u32,
        /// Number of vertical divisions.
        cols: u32,
    },
    /// A smooth surface defined by Bezier control points.
    BezierSurface {
        /// List of 2D control points influencing the surface curvature.
        control_points: Vec<(f32, f32)>,
    },
    /// An arbitrary flat shape defined by an ordered list of vertices.
    Polygon {
        /// List of corner points forming the polygon boundary.
        vertices: Vec<(f32, f32)>,
    },
    /// A generic triangle-based mesh.
    TriMesh,
    /// A procedural circular or arc-shaped mesh.
    Circle {
        /// Number of radial divisions (smoothness).
        segments: u32,
        /// Total angle covered by the arc (360.0 for a full circle).
        arc_angle: f32,
    },
    /// A procedural cylindrical surface.
    Cylinder {
        /// Number of vertical divisions.
        segments: u32,
        /// Total height of the cylinder.
        height: f32,
    },
    /// A procedural spherical surface.
    Sphere {
        /// Number of latitude (vertical) divisions.
        lat_segments: u32,
        /// Number of longitude (horizontal) divisions.
        lon_segments: u32,
    },
    /// A mesh loaded from an external 3D file format.
    Custom {
        /// Path to the mesh file.
        path: String,
    },
}

impl Default for MeshType {
    fn default() -> Self {
        Self::Quad {
            tl: (0.0, 0.0),
            tr: (1.0, 0.0),
            br: (1.0, 1.0),
            bl: (0.0, 1.0),
        }
    }
}

impl MeshType {
    /// Generates a hash representing the current geometric configuration.
    /// Used to detect when GPU buffers need to be updated.
    pub fn compute_revision_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        match self {
            MeshType::Quad { tl, tr, br, bl } => {
                0u8.hash(&mut hasher);
                tl.0.to_bits().hash(&mut hasher);
                tl.1.to_bits().hash(&mut hasher);
                tr.0.to_bits().hash(&mut hasher);
                tr.1.to_bits().hash(&mut hasher);
                br.0.to_bits().hash(&mut hasher);
                br.1.to_bits().hash(&mut hasher);
                bl.0.to_bits().hash(&mut hasher);
                bl.1.to_bits().hash(&mut hasher);
            }
            MeshType::Grid { rows, cols } => {
                1u8.hash(&mut hasher);
                rows.hash(&mut hasher);
                cols.hash(&mut hasher);
            }
            MeshType::TriMesh => {
                2u8.hash(&mut hasher);
            }
            MeshType::Circle {
                segments,
                arc_angle,
            } => {
                3u8.hash(&mut hasher);
                segments.hash(&mut hasher);
                arc_angle.to_bits().hash(&mut hasher);
            }
            MeshType::BezierSurface { control_points } => {
                4u8.hash(&mut hasher);
                control_points.len().hash(&mut hasher);
                for (x, y) in control_points {
                    x.to_bits().hash(&mut hasher);
                    y.to_bits().hash(&mut hasher);
                }
            }
            MeshType::Polygon { vertices } => {
                5u8.hash(&mut hasher);
                vertices.len().hash(&mut hasher);
                for (x, y) in vertices {
                    x.to_bits().hash(&mut hasher);
                    y.to_bits().hash(&mut hasher);
                }
            }
            MeshType::Cylinder { segments, height } => {
                6u8.hash(&mut hasher);
                segments.hash(&mut hasher);
                height.to_bits().hash(&mut hasher);
            }
            MeshType::Sphere {
                lat_segments,
                lon_segments,
            } => {
                7u8.hash(&mut hasher);
                lat_segments.hash(&mut hasher);
                lon_segments.hash(&mut hasher);
            }
            MeshType::Custom { path } => {
                8u8.hash(&mut hasher);
                path.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Converts the procedural definition into a concrete Mesh object for rendering.
    pub fn to_mesh(&self) -> crate::mesh::Mesh {
        use crate::mesh::Mesh;
        use glam::Vec2;

        let mut mesh = match self {
            MeshType::Quad { tl, tr, br, bl } => {
                let mut mesh = Mesh::quad();
                let corners = [
                    Vec2::new(tl.0, tl.1),
                    Vec2::new(tr.0, tr.1),
                    Vec2::new(br.0, br.1),
                    Vec2::new(bl.0, bl.1),
                ];
                mesh.apply_keystone(corners);
                mesh
            }
            MeshType::Grid { rows, cols } => Mesh::create_grid(*rows, *cols),
            MeshType::TriMesh => Mesh::triangle(),
            MeshType::Circle { segments, .. } => {
                Mesh::ellipse(Vec2::new(0.5, 0.5), 0.5, 0.5, *segments)
            }
            MeshType::BezierSurface { control_points } => {
                if control_points.len() == 16 {
                    let mut patch = crate::mesh::BezierPatch::new();
                    for (i, p) in control_points.iter().take(16).enumerate() {
                        let row = i / 4;
                        let col = i % 4;
                        patch.control_points[row][col] = Vec2::new(p.0, p.1);
                    }

                    let mut mesh = Mesh::create_grid(8, 8);
                    patch.apply_to_mesh(&mut mesh);
                    mesh
                } else {
                    Mesh::quad()
                }
            }
            MeshType::Polygon { vertices } => {
                if vertices.len() < 3 {
                    Mesh::quad()
                } else {
                    use crate::mesh::{MeshType as CoreMeshType, MeshVertex};

                    let center = vertices
                        .iter()
                        .fold((0.0, 0.0), |acc, v| (acc.0 + v.0, acc.1 + v.1));
                    let center = (
                        center.0 / vertices.len() as f32,
                        center.1 / vertices.len() as f32,
                    );

                    let mut mesh_vertices = Vec::with_capacity(vertices.len() + 1);
                    mesh_vertices.push(MeshVertex::new(
                        Vec2::new(center.0, center.1),
                        Vec2::new(0.5, 0.5),
                    ));

                    for v in vertices {
                        mesh_vertices
                            .push(MeshVertex::new(Vec2::new(v.0, v.1), Vec2::new(v.0, v.1)));
                    }

                    let mut indices = Vec::with_capacity(vertices.len() * 3);
                    for i in 0..vertices.len() {
                        indices.push(0);
                        indices.push((i + 1) as u16);
                        indices.push(((i + 1) % vertices.len() + 1) as u16);
                    }

                    Mesh {
                        mesh_type: CoreMeshType::Custom,
                        vertices: mesh_vertices,
                        indices,
                        revision: 0,
                    }
                }
            }
            MeshType::Cylinder { segments, height } => {
                let rows = (height * 10.0).max(2.0) as u32;
                let cols = (*segments).max(3);
                Mesh::create_grid(rows, cols)
            }
            MeshType::Sphere {
                lat_segments,
                lon_segments,
            } => {
                use crate::mesh::{MeshType as CoreMeshType, MeshVertex};

                let lat_segs = (*lat_segments).max(3);
                let lon_segs = (*lon_segments).max(3);

                let mut mesh_vertices = Vec::new();
                let mut indices = Vec::new();

                for lat in 0..=lat_segs {
                    let theta = (lat as f32 / lat_segs as f32) * std::f32::consts::PI;
                    let sin_theta = theta.sin();
                    let cos_theta = theta.cos();

                    for lon in 0..=lon_segs {
                        let phi = (lon as f32 / lon_segs as f32) * std::f32::consts::TAU;
                        let cos_phi = phi.cos();

                        let x = 0.5 + 0.5 * sin_theta * cos_phi;
                        let y = 0.5 + 0.5 * cos_theta;
                        let u = lon as f32 / lon_segs as f32;
                        let v = lat as f32 / lat_segs as f32;

                        mesh_vertices.push(MeshVertex::new(Vec2::new(x, y), Vec2::new(u, v)));
                    }
                }

                for lat in 0..lat_segs {
                    for lon in 0..lon_segs {
                        let first = (lat * (lon_segs + 1) + lon) as u16;
                        let second = first + lon_segs as u16 + 1;

                        indices.push(first);
                        indices.push(second);
                        indices.push(first + 1);

                        indices.push(second);
                        indices.push(second + 1);
                        indices.push(first + 1);
                    }
                }

                Mesh {
                    mesh_type: CoreMeshType::Custom,
                    vertices: mesh_vertices,
                    indices,
                    revision: 0,
                }
            }
            MeshType::Custom { path } => {
                if path.ends_with(".obj") {
                    if let Ok((models, _)) = tobj::load_obj(
                        path,
                        &tobj::LoadOptions {
                            single_index: true,
                            triangulate: true,
                            ignore_points: true,
                            ignore_lines: true,
                        },
                    ) {
                        if let Some(model) = models.first() {
                            use crate::mesh::{MeshType as CoreMeshType, MeshVertex};
                            let mesh = &model.mesh;
                            let mut mesh_vertices = Vec::with_capacity(mesh.positions.len() / 3);

                            // Normalize coordinates to 0..1 bounding box
                            let mut min_x = f32::MAX;
                            let mut min_y = f32::MAX;
                            let mut max_x = f32::MIN;
                            let mut max_y = f32::MIN;

                            for i in (0..mesh.positions.len()).step_by(3) {
                                let x = mesh.positions[i];
                                let y = mesh.positions[i + 1];
                                min_x = min_x.min(x);
                                min_y = min_y.min(y);
                                max_x = max_x.max(x);
                                max_y = max_y.max(y);
                            }

                            let width = (max_x - min_x).max(0.0001);
                            let height = (max_y - min_y).max(0.0001);

                            for i in (0..mesh.positions.len()).step_by(3) {
                                let x = mesh.positions[i];
                                let y = mesh.positions[i + 1];
                                let nx = (x - min_x) / width;
                                let ny = (y - min_y) / height;

                                let (u, v) = if !mesh.texcoords.is_empty() {
                                    let uv_idx = (i / 3) * 2;
                                    if uv_idx + 1 < mesh.texcoords.len() {
                                        (mesh.texcoords[uv_idx], mesh.texcoords[uv_idx + 1])
                                    } else {
                                        (nx, ny)
                                    }
                                } else {
                                    (nx, ny)
                                };

                                mesh_vertices
                                    .push(MeshVertex::new(Vec2::new(nx, ny), Vec2::new(u, v)));
                            }

                            let mut indices = Vec::with_capacity(mesh.indices.len());
                            for idx in &mesh.indices {
                                indices.push(*idx as u16);
                            }

                            Mesh {
                                mesh_type: CoreMeshType::Custom,
                                vertices: mesh_vertices,
                                indices,
                                revision: 0,
                            }
                        } else {
                            Mesh::quad()
                        }
                    } else {
                        Mesh::quad()
                    }
                } else if path.ends_with(".svg") {
                    if let Ok(svg_data) = std::fs::read(path) {
                        let opt = usvg::Options::default();
                        if let Ok(tree) = usvg::Tree::from_data(&svg_data, &opt) {
                            use crate::mesh::{MeshType as CoreMeshType, MeshVertex};
                            let mut vertices = Vec::new();

                            // Extract path points
                            fn extract_paths(group: &usvg::Group, vertices: &mut Vec<(f32, f32)>) {
                                for node in group.children() {
                                    match node {
                                        usvg::Node::Group(g) => extract_paths(g, vertices),
                                        usvg::Node::Path(path) => {
                                            let path_data = path.data();
                                            for seg in path_data.segments() {
                                                match seg {
                                                    usvg::tiny_skia_path::PathSegment::MoveTo(
                                                        p,
                                                    ) => {
                                                        vertices.push((p.x, p.y));
                                                    }
                                                    usvg::tiny_skia_path::PathSegment::LineTo(
                                                        p,
                                                    ) => {
                                                        vertices.push((p.x, p.y));
                                                    }
                                                    usvg::tiny_skia_path::PathSegment::QuadTo(
                                                        _,
                                                        p,
                                                    ) => {
                                                        vertices.push((p.x, p.y));
                                                    }
                                                    usvg::tiny_skia_path::PathSegment::CubicTo(
                                                        _,
                                                        _,
                                                        p,
                                                    ) => {
                                                        vertices.push((p.x, p.y));
                                                    }
                                                    usvg::tiny_skia_path::PathSegment::Close => {}
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            extract_paths(tree.root(), &mut vertices);

                            if vertices.len() >= 3 {
                                // Normalize
                                let mut min_x = f32::MAX;
                                let mut min_y = f32::MAX;
                                let mut max_x = f32::MIN;
                                let mut max_y = f32::MIN;

                                for (x, y) in &vertices {
                                    min_x = min_x.min(*x);
                                    min_y = min_y.min(*y);
                                    max_x = max_x.max(*x);
                                    max_y = max_y.max(*y);
                                }

                                let width = (max_x - min_x).max(0.0001);
                                let height = (max_y - min_y).max(0.0001);

                                let mut norm_vertices = Vec::new();
                                for (x, y) in vertices {
                                    let nx = (x - min_x) / width;
                                    let ny = (y - min_y) / height;
                                    norm_vertices.push((nx, ny));
                                }

                                // Triangle fan construction (simple polygon fallback)
                                let center = norm_vertices
                                    .iter()
                                    .fold((0.0, 0.0), |acc, v| (acc.0 + v.0, acc.1 + v.1));
                                let center = (
                                    center.0 / norm_vertices.len() as f32,
                                    center.1 / norm_vertices.len() as f32,
                                );

                                let mut mesh_vertices = Vec::with_capacity(norm_vertices.len() + 1);
                                mesh_vertices.push(MeshVertex::new(
                                    Vec2::new(center.0, center.1),
                                    Vec2::new(0.5, 0.5),
                                ));

                                for v in &norm_vertices {
                                    mesh_vertices.push(MeshVertex::new(
                                        Vec2::new(v.0, v.1),
                                        Vec2::new(v.0, v.1),
                                    ));
                                }

                                let mut indices = Vec::with_capacity(norm_vertices.len() * 3);
                                for i in 0..norm_vertices.len() {
                                    indices.push(0);
                                    indices.push((i + 1) as u16);
                                    indices.push(((i + 1) % norm_vertices.len() + 1) as u16);
                                }

                                Mesh {
                                    mesh_type: CoreMeshType::Custom,
                                    vertices: mesh_vertices,
                                    indices,
                                    revision: 0,
                                }
                            } else {
                                Mesh::quad()
                            }
                        } else {
                            Mesh::quad()
                        }
                    } else {
                        Mesh::quad()
                    }
                } else {
                    Mesh::quad()
                }
            }
        };

        mesh.revision = self.compute_revision_hash();
        mesh
    }
}
