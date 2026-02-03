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
