use bevy::prelude::*;

/// Component to make an entity react to audio input.
///
/// When attached to an entity, this component modulates the specified `target` property
/// based on the intensity of the audio signal from the selected `source`.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AudioReactive {
    /// The property of the entity to modulate (e.g., Scale, Rotation, Position).
    pub target: AudioReactiveTarget,
    /// The source of the audio data to use (e.g., Bass, Mid, RMS).
    pub source: AudioReactiveSource,
    /// Multiplier for the audio value. Higher values result in stronger reactions.
    pub intensity: f32,
    /// The base value of the property when the audio signal is zero.
    pub base: f32,
}

/// Specifies which property of the entity should be affected by audio.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioReactiveTarget {
    /// Uniformly scale the entity on all axes.
    #[default]
    Scale,
    /// Scale only along the X axis.
    ScaleX,
    /// Scale only along the Y axis.
    ScaleY,
    /// Scale only along the Z axis.
    ScaleZ,
    /// Rotate around the X axis.
    RotateX,
    /// Rotate around the Y axis.
    RotateY,
    /// Rotate around the Z axis.
    RotateZ,
    /// Modify the Y position (height).
    PositionY,
    /// Modulate the emissive color intensity (requires a `StandardMaterial`).
    EmissiveIntensity,
}

/// Specifies the audio frequency band or volume metric to use as input.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioReactiveSource {
    /// Low frequencies (Band 0-1).
    #[default]
    Bass,
    /// Low-mid frequencies (Band 2-3).
    LowMid,
    /// Mid frequencies (Band 4-5).
    Mid,
    /// High-mid frequencies (Band 6-7).
    HighMid,
    /// High frequencies (Band 8).
    High,
    /// Root Mean Square (RMS) volume (overall loudness).
    Rms,
    /// Peak volume.
    Peak,
}

/// Component for rendering a realistic atmosphere and sky.
///
/// This component controls the parameters for atmospheric scattering, allowing for
/// dynamic time-of-day and weather effects.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BevyAtmosphere {
    /// Turbidity of the atmosphere (haziness). Higher values make the sky look dustier.
    pub turbidity: f32,
    /// Rayleigh scattering coefficient. Affects the color of the sky (blue/red).
    pub rayleigh: f32,
    /// Mie scattering coefficient. Affects the brightness of the sun halo.
    pub mie_coeff: f32,
    /// Mie scattering directionality (g). Controls how focused the sun halo is.
    pub mie_directional_g: f32,
    /// Position of the sun in the sky (azimuth, inclination).
    pub sun_position: (f32, f32),
    /// Exposure level for the sky rendering.
    pub exposure: f32,
}

/// Component for generating a procedural 3D hexagonal grid.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BevyHexGrid {
    /// The radius of individual hexagonal cells.
    pub radius: f32,
    /// The number of rings in the grid (controls grid size).
    pub rings: u32,
    /// Whether the hexagons are oriented with a pointy top (vs. flat top).
    pub pointy_top: bool,
    /// Spacing between hexagonal cells.
    pub spacing: f32,
}

/// Component for a GPU-accelerated particle emitter.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BevyParticles {
    /// The rate at which particles are spawned (particles per second).
    pub rate: f32,
    /// The lifetime of each particle in seconds.
    pub lifetime: f32,
    /// The initial speed of spawned particles.
    pub speed: f32,
    /// The starting color of particles (RGBA).
    pub color_start: [f32; 4],
    /// The ending color of particles (RGBA), interpolated over their lifetime.
    pub color_end: [f32; 4],
}

/// Internal state for a particle emitter system.
///
/// This component stores the active particles and the accumulator for spawning logic.
#[derive(Component, Default)]
pub struct ParticleEmitter {
    /// List of currently active particles.
    pub particles: Vec<Particle>,
    /// Accumulator for fractional spawn counts.
    pub spawn_accumulator: f32,
}

/// Component representing a basic 3D geometric shape.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Bevy3DShape {
    /// The type of geometric shape (Cube, Sphere, etc.).
    #[reflect(ignore)]
    pub shape_type: vorce_core::module::BevyShapeType,
    /// The base color of the shape (RGBA).
    pub color: [f32; 4],
    /// Whether the material should be unlit (ignore lighting) or PBR.
    pub unlit: bool,
    /// Width of the outline effect (0.0 to disable).
    pub outline_width: f32,
    /// Color of the outline effect (RGBA).
    pub outline_color: [f32; 4],
}

impl Default for Bevy3DShape {
    fn default() -> Self {
        Self {
            shape_type: vorce_core::module::BevyShapeType::Cube,
            color: [1.0, 1.0, 1.0, 1.0],
            unlit: false,
            outline_width: 0.0,
            outline_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Component for loading and displaying a 3D model (glTF).
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Bevy3DModel {
    /// Path to the 3D model file.
    pub path: String,
    /// Position offset (x, y, z).
    pub position: [f32; 3],
    /// Rotation (euler angles in degrees: x, y, z).
    pub rotation: [f32; 3],
    /// Scale factor (x, y, z).
    pub scale: [f32; 3],
    /// Width of the outline effect (0.0 to disable).
    pub outline_width: f32,
    /// Color of the outline effect (RGBA).
    pub outline_color: [f32; 4],
}

impl Default for Bevy3DModel {
    fn default() -> Self {
        Self {
            path: String::new(),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            outline_width: 0.0,
            outline_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Data for a single particle instance.
#[derive(Debug, Clone, Copy, Default)]
pub struct Particle {
    /// Current position of the particle.
    pub position: Vec3,
    /// Velocity vector of the particle.
    pub velocity: Vec3,
    /// Total lifetime of the particle in seconds.
    pub lifetime: f32,
    /// Current age of the particle in seconds.
    pub age: f32,
    /// Color at the start of the particle's life.
    pub color_start: LinearRgba,
    /// Color at the end of the particle's life.
    pub color_end: LinearRgba,
}

/// Tag component for the Shared Engine Camera.
///
/// This camera renders the scene to a texture that is shared with MapFlow.
#[derive(Component)]
pub struct SharedEngineCamera;

/// Text alignment options for 3D text.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default)]
pub enum BevyTextAlignment {
    /// Align text to the left.
    #[default]
    Left,
    /// Center text horizontally.
    Center,
    /// Align text to the right.
    Right,
    /// Justify text (spread to fill width).
    Justify,
}

/// Component for rendering 3D text in the scene.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Bevy3DText {
    /// The text content string.
    pub text: String,
    /// Font size in pixels (world units).
    pub font_size: f32,
    /// Text color (RGBA).
    pub color: [f32; 4],
    /// Horizontal alignment of the text.
    pub alignment: BevyTextAlignment,
}

/// Camera control modes for the Bevy scene.
#[derive(Reflect, Clone, PartialEq)]
pub enum BevyCameraMode {
    /// Camera orbits around a target point.
    Orbit {
        /// Distance from the target.
        radius: f32,
        /// Orbit speed.
        speed: f32,
        /// The point to look at.
        target: Vec3,
        /// Height offset relative to the target.
        height: f32,
    },
    /// First-person "fly" camera mode.
    Fly {
        /// Movement speed.
        speed: f32,
        /// Mouse look sensitivity.
        sensitivity: f32,
    },
    /// Static camera at a fixed position looking at a point.
    Static {
        /// Camera position.
        position: Vec3,
        /// Point to look at.
        look_at: Vec3,
    },
}

impl Default for BevyCameraMode {
    fn default() -> Self {
        Self::Orbit {
            radius: 10.0,
            speed: 20.0,
            target: Vec3::ZERO,
            height: 2.0,
        }
    }
}

/// Component for controlling the main camera in the Bevy scene.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BevyCamera {
    /// The active camera control mode.
    pub mode: BevyCameraMode,
    /// Field of View (FOV) in degrees.
    pub fov: f32,
    /// Whether this camera is currently active.
    pub active: bool,
}
