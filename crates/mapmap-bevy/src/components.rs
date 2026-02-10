use bevy::prelude::*;

/// Component to make an entity react to audio input.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AudioReactive {
    /// What property to effect
    pub target: AudioReactiveTarget,
    /// Which audio data source to use
    pub source: AudioReactiveSource,
    /// Multiplier for the audio value
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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Bevy3DShape {
    #[reflect(ignore)]
    pub shape_type: mapmap_core::module::BevyShapeType,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub color: LinearRgba,
    pub unlit: bool,
}

impl Default for Bevy3DShape {
    fn default() -> Self {
        Self {
            shape_type: mapmap_core::module::BevyShapeType::Cube,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: LinearRgba::WHITE,
            unlit: false,
        }
    }
}

#[derive(Component)]
pub struct BevyShapeCache {
    pub shape_type: mapmap_core::module::BevyShapeType,
    pub color: LinearRgba,
    pub unlit: bool,
}
