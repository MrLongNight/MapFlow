//!
//! Source related data.
//!

use crate::module::types::socket::BlendModeType;
use serde::{Deserialize, Serialize};

/// Types of media sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    /// Enumeration variant.
    MediaFile {
        /// File path to asset.
        path: String,
        #[serde(default = "crate::module::config::default_speed")]
        /// Playback speed multiplier.
        speed: f32,
        #[serde(default)]
        /// Whether the media should automatically restart after reaching the end.
        loop_enabled: bool,
        #[serde(default)]
        /// Timestamp (in seconds) where playback should begin within the media file.
        start_time: f32,
        #[serde(default)]
        /// Timestamp (in seconds) where playback should stop within the media file. 0.0 means end of file.
        end_time: f32,
        #[serde(default = "crate::module::config::default_opacity")]
        /// Opacity value (0.0 to 1.0).
        opacity: f32,
        #[serde(default)]
        /// Blending mode used for rendering.
        blend_mode: Option<BlendModeType>,
        #[serde(default)]
        /// Brightness factor.
        brightness: f32,
        #[serde(default = "crate::module::config::default_contrast")]
        /// Contrast factor.
        contrast: f32,
        #[serde(default = "crate::module::config::default_saturation")]
        /// Saturation adjustment.
        saturation: f32,
        #[serde(default)]
        /// Hue shift in degrees.
        hue_shift: f32,
        #[serde(default = "crate::module::config::default_scale")]
        /// Scale factor along the horizontal axis.
        scale_x: f32,
        #[serde(default = "crate::module::config::default_scale")]
        /// Scaling factor applied to the Y axis of the source content.
        scale_y: f32,
        #[serde(default)]
        /// Rotation in degrees around the center point of the source.
        rotation: f32,
        #[serde(default)]
        /// Horizontal translation offset in normalized coordinates.
        offset_x: f32,
        #[serde(default)]
        /// Vertical translation offset in normalized coordinates.
        offset_y: f32,
        #[serde(default)]
        /// Optional width constraint for the media player's internal buffer.
        target_width: Option<u32>,
        #[serde(default)]
        /// Optional height constraint for the media player's internal buffer.
        target_height: Option<u32>,
        #[serde(default)]
        /// Desired playback frame rate. If None, the source's native FPS is used.
        target_fps: Option<f32>,
        #[serde(default)]
        /// Flips the texture horizontally. Essential for mirror-based projections or back-projection screens.
        flip_horizontal: bool,
        #[serde(default)]
        /// Flips the texture vertically. Useful for correcting inverted camera feeds or projector mounts.
        flip_vertical: bool,
        #[serde(default)]
        /// When enabled, the media file is played from end to beginning.
        reverse_playback: bool,
    },
    /// A source that renders content using a custom WGSL shader.
    Shader {
        /// Human-readable label for identifying this shader instance in the UI.
        name: String,
        /// Dynamic parameters exposed by the shader, represented as (Name, Value) pairs.
        params: Vec<(String, f32)>,
    },
    /// Raw video feed from a locally connected capture device (e.g., Webcam, Capture Card).
    LiveInput {
        /// System-specific index of the capture device.
        device_id: u32,
    },
    /// Network video stream received via the NewTek NDI protocol.
    NdiInput {
        /// The name of the NDI source as broadcast on the network (e.g., "STUDIO-PC (Output)").
        source_name: Option<String>,
    },
    /// Enumeration variant.
    Bevy,
    /// Simulates realistic sky and atmospheric scattering.
    BevyAtmosphere {
        /// Atmospheric haze intensity. Higher values make the sky look more "dusty" or polluted.
        turbidity: f32,
        /// Determines the strength of Rayleigh scattering, which gives the sky its blue color and red sunsets.
        rayleigh: f32,
        /// Determines the strength of Mie scattering, affecting the glow around the sun.
        mie_coeff: f32,
        /// Controls the directionality of Mie scattering (how much light is scattered forward).
        mie_directional_g: f32,
        /// Position of the sun in azimuth/elevation coordinates.
        sun_position: (f32, f32),
        /// Overall brightness/exposure of the atmospheric effect.
        exposure: f32,
    },
    /// Renders a grid of hexagons in 3D space.
    BevyHexGrid {
        /// Radius from the center to a corner of a single hexagon.
        radius: f32,
        /// Number of concentric rings of hexagons around the center.
        rings: u32,
        /// If true, hexagons are oriented with a point at the top. If false, they have a flat top.
        pointy_top: bool,
        /// Distance between the centers of adjacent hexagons.
        spacing: f32,
        /// World-space position [x, y, z] of the grid origin.
        position: [f32; 3],
        /// Rotation of the entire grid in degrees for each axis.
        rotation: [f32; 3],
        /// Uniform scale factor for the entire grid.
        scale: f32,
    },
    /// GPU-accelerated 3D particle system.
    BevyParticles {
        /// Number of particles spawned per second.
        rate: f32,
        /// Maximum duration in seconds a particle exists before disappearing.
        lifetime: f32,
        /// Speed at which particles move away from the emitter.
        speed: f32,
        /// Color of particles at the moment they are spawned.
        color_start: [f32; 4],
        /// Color of particles just before they disappear.
        color_end: [f32; 4],
        /// Origin point [x, y, z] of the particle emitter.
        position: [f32; 3],
        /// Directional orientation of the emitter.
        rotation: [f32; 3],
    },
    /// Standard 3D geometric primitive (Cube, Sphere, etc.).
    Bevy3DShape {
        /// The geometric form of the 3D object (e.g., Cube, Sphere, Plane).
        shape_type: BevyShapeType,
        /// 3D position coordinates [x, y, z].
        position: [f32; 3],
        /// Rotation angles in degrees.
        rotation: [f32; 3],
        /// Scale factors for the object's dimensions.
        scale: [f32; 3],
        /// RGBA color value.
        color: [f32; 4],
        /// Disables lighting calculations, making the object appear with its full emission/color regardless of light sources.
        unlit: bool,
        #[serde(default)]
        /// Thickness of the selection or decorative outline.
        outline_width: f32,
        #[serde(default = "crate::module::config::default_white_rgba")]
        /// The color used for the object's outline.
        outline_color: [f32; 4],
    },
    /// External 3D asset loaded from a file (GLTF, OBJ).
    Bevy3DModel {
        /// File path to asset.
        path: String,
        /// 3D position coordinates [x, y, z].
        position: [f32; 3],
        /// Rotation angles in degrees.
        rotation: [f32; 3],
        /// Scale factors for the object's dimensions.
        scale: [f32; 3],
        /// RGBA color value.
        color: [f32; 4],
        /// Disables lighting calculations, making the object appear with its full emission/color regardless of light sources.
        unlit: bool,
        #[serde(default)]
        /// Thickness of the selection or decorative outline.
        outline_width: f32,
        #[serde(default = "crate::module::config::default_white_rgba")]
        /// The color used for the object's outline.
        outline_color: [f32; 4],
    },
    /// Three-dimensional text rendered in the scene.
    Bevy3DText {
        /// The literal string content to be rendered.
        text: String,
        /// The size of the characters in pixels or points.
        font_size: f32,
        /// RGBA color value.
        color: [f32; 4],
        /// 3D position coordinates [x, y, z].
        position: [f32; 3],
        /// Rotation angles in degrees.
        rotation: [f32; 3],
        /// Alignment of the content (e.g., 'Center', 'Left', 'Right').
        alignment: String,
    },
    /// Virtual camera defining the viewpoint of the Bevy engine.
    BevyCamera {
        /// Projection mode: Perspective (3D) or Orthographic (2D/Flat).
        mode: BevyCameraMode,
        /// Field of view in degrees. Higher values show more of the scene but cause distortion at the edges.
        fov: f32,
        /// Toggle to enable or disable this camera view within the scene.
        active: bool,
    },
    #[cfg(target_os = "windows")]
    /// Inter-process video sharing via Spout (Windows).
    SpoutInput {
        /// The name of the Spout sender to connect to.
        sender_name: String,
    },
    /// Single-instance video source with full transform controls.
    VideoUni {
        /// File path to the video or image asset.
        path: String,
        #[serde(default = "crate::module::config::default_speed")]
        /// Playback speed multiplier (1.0 is normal speed).
        speed: f32,
        #[serde(default)]
        /// Automatically restart playback when the end of the file is reached.
        loop_enabled: bool,
        #[serde(default)]
        /// Start position in seconds within the media file.
        start_time: f32,
        #[serde(default)]
        /// End position in seconds within the media file (0.0 plays to the end).
        end_time: f32,
        #[serde(default = "crate::module::config::default_opacity")]
        /// Transparency level of the source (0.0 to 1.0).
        opacity: f32,
        #[serde(default)]
        /// The algorithm used to blend this source with lower layers (e.g., Add, Multiply).
        blend_mode: Option<BlendModeType>,
        #[serde(default)]
        /// Increases or decreases the overall light level of the video.
        brightness: f32,
        #[serde(default = "crate::module::config::default_contrast")]
        /// Adjusts the range between light and dark areas.
        contrast: f32,
        #[serde(default = "crate::module::config::default_saturation")]
        /// Intensifies or dulls the colors (0.0 is black and white).
        saturation: f32,
        #[serde(default)]
        /// Shifts all colors around the color wheel in degrees.
        hue_shift: f32,
        #[serde(default = "crate::module::config::default_scale")]
        /// Horizontal scaling factor.
        scale_x: f32,
        #[serde(default = "crate::module::config::default_scale")]
        /// Vertical scaling factor.
        scale_y: f32,
        #[serde(default)]
        /// Clockwise rotation in degrees.
        rotation: f32,
        #[serde(default)]
        /// Horizontal shift from the center point.
        offset_x: f32,
        #[serde(default)]
        /// Vertical shift from the center point.
        offset_y: f32,
        #[serde(default)]
        /// Force the internal player to use this specific width.
        target_width: Option<u32>,
        #[serde(default)]
        /// Force the internal player to use this specific height.
        target_height: Option<u32>,
        #[serde(default)]
        /// Target frame rate for the media decoder.
        target_fps: Option<f32>,
        #[serde(default)]
        /// Mirror the image horizontally.
        flip_horizontal: bool,
        #[serde(default)]
        /// Mirror the image vertically.
        flip_vertical: bool,
        #[serde(default)]
        /// Reverse the playback direction.
        reverse_playback: bool,
    },
    /// Media source sharing its state across multiple instances.
    VideoMulti {
        /// Unique key for accessing a shared resource or media pool.
        shared_id: String,
        #[serde(default = "crate::module::config::default_opacity")]
        /// Opacity value (0.0 to 1.0).
        opacity: f32,
        #[serde(default)]
        /// Blending mode used for rendering.
        blend_mode: Option<BlendModeType>,
        #[serde(default)]
        /// Brightness factor.
        brightness: f32,
        #[serde(default = "crate::module::config::default_contrast")]
        /// Contrast factor.
        contrast: f32,
        #[serde(default = "crate::module::config::default_saturation")]
        /// Saturation adjustment.
        saturation: f32,
        #[serde(default)]
        /// Hue shift in degrees.
        hue_shift: f32,
        #[serde(default = "crate::module::config::default_scale")]
        /// Scale factor along the horizontal axis.
        scale_x: f32,
        #[serde(default = "crate::module::config::default_scale")]
        /// Scale factor along the vertical axis.
        scale_y: f32,
        #[serde(default)]
        /// Rotation angles in degrees.
        rotation: f32,
        #[serde(default)]
        /// Horizontal translation offset.
        offset_x: f32,
        #[serde(default)]
        /// Vertical translation offset.
        offset_y: f32,
        #[serde(default)]
        /// Horizontal flip flag.
        flip_horizontal: bool,
        #[serde(default)]
        /// Vertical flip flag.
        flip_vertical: bool,
    },
    /// Single-instance static image source.
    ImageUni {
        /// File path to the image file (PNG, JPG, etc.).
        path: String,
        #[serde(default = "crate::module::config::default_opacity")]
        /// Transparency level (0.0 to 1.0).
        opacity: f32,
        #[serde(default)]
        /// Blending algorithm for compositing.
        blend_mode: Option<BlendModeType>,
        #[serde(default)]
        /// Brightness adjustment.
        brightness: f32,
        #[serde(default = "crate::module::config::default_contrast")]
        /// Contrast adjustment.
        contrast: f32,
        #[serde(default = "crate::module::config::default_saturation")]
        /// Saturation adjustment.
        saturation: f32,
        #[serde(default)]
        /// Hue shift in degrees.
        hue_shift: f32,
        #[serde(default = "crate::module::config::default_scale")]
        /// Horizontal scale.
        scale_x: f32,
        #[serde(default = "crate::module::config::default_scale")]
        /// Vertical scale.
        scale_y: f32,
        #[serde(default)]
        /// Rotation in degrees.
        rotation: f32,
        #[serde(default)]
        /// X-axis offset.
        offset_x: f32,
        #[serde(default)]
        /// Y-axis offset.
        offset_y: f32,
        #[serde(default)]
        /// Force a specific internal buffer width.
        target_width: Option<u32>,
        #[serde(default)]
        /// Force a specific internal buffer height.
        target_height: Option<u32>,
        #[serde(default)]
        /// Mirror horizontally.
        flip_horizontal: bool,
        #[serde(default)]
        /// Mirror vertically.
        flip_vertical: bool,
    },
    /// Static image source that shares its resource ID with other instances.
    ImageMulti {
        /// Identifier for the shared image resource.
        shared_id: String,
        #[serde(default = "crate::module::config::default_opacity")]
        /// Transparency level (0.0 to 1.0).
        opacity: f32,
        #[serde(default)]
        /// Blending algorithm for compositing.
        blend_mode: Option<BlendModeType>,
        #[serde(default)]
        /// Brightness adjustment.
        brightness: f32,
        #[serde(default = "crate::module::config::default_contrast")]
        /// Contrast adjustment.
        contrast: f32,
        #[serde(default = "crate::module::config::default_saturation")]
        /// Saturation adjustment.
        saturation: f32,
        #[serde(default)]
        /// Hue shift in degrees.
        hue_shift: f32,
        #[serde(default = "crate::module::config::default_scale")]
        /// Horizontal scale.
        scale_x: f32,
        #[serde(default = "crate::module::config::default_scale")]
        /// Vertical scale.
        scale_y: f32,
        #[serde(default)]
        /// Rotation in degrees.
        rotation: f32,
        #[serde(default)]
        /// X-axis offset.
        offset_x: f32,
        #[serde(default)]
        /// Y-axis offset.
        offset_y: f32,
        #[serde(default)]
        /// Mirror horizontally.
        flip_horizontal: bool,
        #[serde(default)]
        /// Mirror vertically.
        flip_vertical: bool,
    },
}

impl SourceType {
    /// Creates a new source configuration from a media file path.
    pub fn new_media_file(path: String) -> Self {
        SourceType::MediaFile {
            path,
            speed: 1.0,
            loop_enabled: true,
            start_time: 0.0,
            end_time: 0.0,
            opacity: 1.0,
            blend_mode: None,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            hue_shift: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            target_width: None,
            target_height: None,
            target_fps: None,
            flip_horizontal: false,
            flip_vertical: false,
            reverse_playback: false,
        }
    }
}

/// Types of 3D shapes available in Bevy nodes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum BevyShapeType {
    #[default]
    /// Enumeration variant.
    Cube,
    /// Enumeration variant.
    Sphere,
    /// Enumeration variant.
    Capsule,
    /// Enumeration variant.
    Torus,
    /// Enumeration variant.
    Cylinder,
    /// Enumeration variant.
    Plane,
}

/// Modes for Bevy Camera
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BevyCameraMode {
    /// Enumeration variant.
    Orbit {
        /// Radius of the hexagonal grid or shape.
        radius: f32,
        /// Playback speed multiplier.
        speed: f32,
        /// Component property or field.
        target: [f32; 3],
        /// Component property or field.
        height: f32,
    },
    /// Enumeration variant.
    Fly {
        /// Playback speed multiplier.
        speed: f32,
        /// Component property or field.
        sensitivity: f32,
    },
    /// Enumeration variant.
    Static {
        /// 3D position coordinates [x, y, z].
        position: [f32; 3],
        /// Component property or field.
        look_at: [f32; 3],
    },
}

impl Default for BevyCameraMode {
    fn default() -> Self {
        BevyCameraMode::Orbit {
            radius: 10.0,
            speed: 20.0,
            target: [0.0, 0.0, 0.0],
            height: 2.0,
        }
    }
}
