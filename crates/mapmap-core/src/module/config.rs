//! Module Configuration Defaults

use crate::module::types::{MeshType, ModulePartId};

pub fn default_speed() -> f32 {
    1.0
}
pub fn default_opacity() -> f32 {
    1.0
}
pub fn default_white_rgba() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}
pub fn default_contrast() -> f32 {
    1.0
}
pub fn default_saturation() -> f32 {
    1.0
}
pub fn default_scale() -> f32 {
    1.0
}
pub fn default_next_part_id() -> ModulePartId {
    1
}
pub fn default_mesh_quad() -> MeshType {
    MeshType::Quad {
        tl: (0.0, 0.0),
        tr: (1.0, 0.0),
        br: (1.0, 1.0),
        bl: (0.0, 1.0),
    }
}
pub fn default_true() -> bool {
    true
}
pub fn default_output_fps() -> f32 {
    60.0
}
pub fn default_hue_color() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}
pub fn default_color_palette() -> Vec<[f32; 4]> {
    vec![
        [1.0, 0.2, 0.2, 1.0],
        [1.0, 0.5, 0.2, 1.0],
        [1.0, 1.0, 0.2, 1.0],
        [0.5, 1.0, 0.2, 1.0],
        [0.2, 1.0, 0.2, 1.0],
        [0.2, 1.0, 0.5, 1.0],
        [0.2, 1.0, 1.0, 1.0],
        [0.2, 0.5, 1.0, 1.0],
        [0.2, 0.2, 1.0, 1.0],
        [0.5, 0.2, 1.0, 1.0],
        [1.0, 0.2, 1.0, 1.0],
        [1.0, 0.2, 0.5, 1.0],
        [0.5, 0.5, 0.5, 1.0],
        [1.0, 0.5, 0.8, 1.0],
        [0.5, 1.0, 0.8, 1.0],
        [0.8, 0.5, 1.0, 1.0],
    ]
}
