//! Module Configuration Defaults

use crate::module::types::{MeshType, ModulePartId};

/// Returns the default playback speed for media.
pub fn default_speed() -> f32 {
    1.0
}
/// Returns the default opacity level.
pub fn default_opacity() -> f32 {
    1.0
}
/// Returns the default white color in RGBA format.
pub fn default_white_rgba() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}
/// Returns the default contrast adjustment value.
pub fn default_contrast() -> f32 {
    1.0
}
/// Returns the default saturation adjustment value.
pub fn default_saturation() -> f32 {
    1.0
}
/// Returns the default uniform scale factor.
pub fn default_scale() -> f32 {
    1.0
}
/// Returns the default starting ID for module parts.
pub fn default_next_part_id() -> ModulePartId {
    1
}
/// Returns a default quad mesh spanning from (0,0) to (1,1).
pub fn default_mesh_quad() -> MeshType {
    MeshType::Quad {
        tl: (0.0, 0.0),
        tr: (1.0, 0.0),
        br: (1.0, 1.0),
        bl: (0.0, 1.0),
    }
}
/// Returns a default boolean true value for deserialization.
pub fn default_true() -> bool {
    true
}
/// Returns the default output framerate in FPS.
pub fn default_output_fps() -> f32 {
    60.0
}
/// Returns the default hue color as an RGB array.
pub fn default_hue_color() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}
/// Returns the default color palette with a variety of preset colors.
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
