use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::module::types::socket::BlendModeType;

/// Types of modulizers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulizerType {
    /// Enumeration variant.
    Effect {
        /// Component property or field.
        effect_type: EffectType,
        #[serde(default)]
        /// Dynamic parameters for the component, usually as (Name, Value) pairs.
        params: HashMap<String, f32>,
    },
    /// Enumeration variant.
    BlendMode(BlendModeType),
    /// Enumeration variant.
    AudioReactive {
        /// Component property or field.
        source: String,
    },
}

/// Available visual effects
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectType {
    /// Enumeration variant.
    ShaderGraph(crate::shader_graph::GraphId),
    /// Enumeration variant.
    Blur,
    /// Enumeration variant.
    Sharpen,
    /// Enumeration variant.
    Invert,
    /// Enumeration variant.
    Threshold,
    /// Brightness factor.
    Brightness,
    /// Contrast factor.
    Contrast,
    /// Saturation adjustment.
    Saturation,
    /// Hue shift in degrees.
    HueShift,
    /// Enumeration variant.
    Colorize,
    /// Enumeration variant.
    Wave,
    /// Enumeration variant.
    Spiral,
    /// Enumeration variant.
    Pinch,
    /// Enumeration variant.
    Mirror,
    /// Enumeration variant.
    Kaleidoscope,
    /// Enumeration variant.
    Pixelate,
    /// Enumeration variant.
    Halftone,
    /// Enumeration variant.
    EdgeDetect,
    /// Enumeration variant.
    Posterize,
    /// Enumeration variant.
    Glitch,
    /// Enumeration variant.
    RgbSplit,
    /// Enumeration variant.
    ChromaticAberration,
    /// Enumeration variant.
    VHS,
    /// Enumeration variant.
    FilmGrain,
    /// Enumeration variant.
    Vignette,
}

impl EffectType {
    /// Associated function.
    pub fn all() -> &'static [EffectType] {
        &[
            EffectType::Blur,
            EffectType::Sharpen,
            EffectType::Invert,
            EffectType::Threshold,
            EffectType::Brightness,
            EffectType::Contrast,
            EffectType::Saturation,
            EffectType::HueShift,
            EffectType::Colorize,
            EffectType::Wave,
            EffectType::Spiral,
            EffectType::Pinch,
            EffectType::Mirror,
            EffectType::Kaleidoscope,
            EffectType::Pixelate,
            EffectType::Halftone,
            EffectType::EdgeDetect,
            EffectType::Posterize,
            EffectType::Glitch,
            EffectType::RgbSplit,
            EffectType::ChromaticAberration,
            EffectType::VHS,
            EffectType::FilmGrain,
            EffectType::Vignette,
        ]
    }

    /// Human-readable display name.
    pub fn name(&self) -> &'static str {
        match self {
            EffectType::Blur => "Blur",
            EffectType::Sharpen => "Sharpen",
            EffectType::Invert => "Invert",
            EffectType::Threshold => "Threshold",
            EffectType::Brightness => "Brightness",
            EffectType::Contrast => "Contrast",
            EffectType::Saturation => "Saturation",
            EffectType::HueShift => "Hue Shift",
            EffectType::Colorize => "Colorize",
            EffectType::Wave => "Wave",
            EffectType::Spiral => "Spiral",
            EffectType::Pinch => "Pinch",
            EffectType::Mirror => "Mirror",
            EffectType::Kaleidoscope => "Kaleidoscope",
            EffectType::Pixelate => "Pixelate",
            EffectType::Halftone => "Halftone",
            EffectType::EdgeDetect => "Edge Detect",
            EffectType::Posterize => "Posterize",
            EffectType::Glitch => "Glitch",
            EffectType::RgbSplit => "RGB Split",
            EffectType::ChromaticAberration => "Chromatic Aberration",
            EffectType::VHS => "VHS",
            EffectType::FilmGrain => "Film Grain",
            EffectType::Vignette => "Vignette",
            EffectType::ShaderGraph(_) => "Custom Shader Graph",
        }
    }
}
