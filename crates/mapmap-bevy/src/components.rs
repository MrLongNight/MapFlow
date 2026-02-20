use bevy::prelude::*;

/// Component to make an entity react to audio input.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AudioReactive {
    /// What property to effect
    pub target: AudioReactiveTarget,
    /// Which audio data source to use
    pub source: AudioReactiveSource,
    /// Multiplier for the audio value.
    pub intensity: f32,
    /// Base value when audio is 0
    pub base: f32,
}

#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioReactiveTarget {
    #[default]
    Scale,
    ScaleX,
    ScaleY,
    ScaleZ,
    RotateX,
    RotateY,
    RotateZ,
    PositionY,
    /// Emissive color intensity (requires StandardMaterial)
    EmissiveIntensity,
}

#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioReactiveSource {
    #[default]
    Bass, // Band 0-1
    LowMid,  // Band 2-3
    Mid,     // Band 4-5
    HighMid, // Band 6-7
    High,    // Band 8
    Rms,     // Overall volume
    Peak,    // Peak volume
}
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BevyAtmosphere {
    pub turbidity: f32,
    pub rayleigh: f32,
    pub mie_coeff: f32,
    pub mie_directional_g: f32,
    pub sun_position: (f32, f32),
    pub exposure: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BevyHexGrid {
    pub radius: f32,
    pub rings: u32,
    pub pointy_top: bool,
    pub spacing: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BevyParticles {
    pub rate: f32,
    pub lifetime: f32,
    pub speed: f32,
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
}

/// Internal state for particle emitter
#[derive(Component, Default)]
pub struct ParticleEmitter {
    pub particles: Vec<Particle>,
    pub spawn_accumulator: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Bevy3DShape {
    #[reflect(ignore)]
    pub shape_type: mapmap_core::module::BevyShapeType,
    pub color: [f32; 4],
    pub unlit: bool,
    pub outline_width: f32,
    pub outline_color: [f32; 4],
}

impl Default for Bevy3DShape {
    fn default() -> Self {
        Self {
            shape_type: mapmap_core::module::BevyShapeType::Cube,
            color: [1.0, 1.0, 1.0, 1.0],
            unlit: false,
            outline_width: 0.0,
            outline_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Component for 3D Model loading and transform control
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Bevy3DModel {
    pub path: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    pub outline_width: f32,
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

/// Individual particle data
#[derive(Debug, Clone, Copy, Default)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub age: f32,
    pub color_start: LinearRgba,
    pub color_end: LinearRgba,
}

/// Tag component for the Shared Engine instance
#[derive(Component)]
pub struct SharedEngineCamera;

#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default)]
pub enum BevyTextAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Bevy3DText {
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub alignment: BevyTextAlignment,
}

#[derive(Reflect, Clone, PartialEq)]
pub enum BevyCameraMode {
    Orbit {
        radius: f32,
        speed: f32,
        target: Vec3,
        height: f32,
    },
    Fly {
        speed: f32,
        sensitivity: f32,
    },
    Static {
        position: Vec3,
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

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BevyCamera {
    pub mode: BevyCameraMode,
    pub fov: f32,
    pub active: bool,
}
