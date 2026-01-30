//! Effect Chain data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Effect types available in the chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    /// Color adjustments (brightness, contrast, saturation)
    ColorAdjust,
    /// Gaussian blur effect
    Blur,
    /// Chromatic aberration (RGB split)
    ChromaticAberration,
    /// Edge detection (Sobel filter)
    EdgeDetect,
    /// Glow/bloom effect
    Glow,
    /// Kaleidoscope mirror effect
    Kaleidoscope,
    /// Invert colors
    Invert,
    /// Pixelation effect
    Pixelate,
    /// Vignette darkening at edges
    Vignette,
    /// Film grain noise
    FilmGrain,
    /// Wave distortion effect
    Wave,
    /// Digital glitch effect
    Glitch,
    /// RGB channel split
    RgbSplit,
    /// Mirror reflection effect
    Mirror,
    /// Hue shift effect
    HueShift,
    /// Custom shader from shader graph
    Custom,
    /// Custom Node-based Shader Graph
    ShaderGraph(crate::shader_graph::GraphId),
}

impl EffectType {
    /// Get the display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            EffectType::ColorAdjust => "Color Adjust",
            EffectType::Blur => "Blur",
            EffectType::ChromaticAberration => "Chromatic Aberration",
            EffectType::EdgeDetect => "Edge Detect",
            EffectType::Glow => "Glow",
            EffectType::Kaleidoscope => "Kaleidoscope",
            EffectType::Invert => "Invert",
            EffectType::Pixelate => "Pixelate",
            EffectType::Vignette => "Vignette",
            EffectType::FilmGrain => "Film Grain",
            EffectType::Wave => "Wave",
            EffectType::Glitch => "Glitch",
            EffectType::RgbSplit => "RGB Split",
            EffectType::Mirror => "Mirror",
            EffectType::HueShift => "Hue Shift",
            EffectType::Custom => "Custom Shader",
            EffectType::ShaderGraph(_) => "Shader Graph",
        }
    }
}

/// An effect instance in the chain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effect {
    /// Unique ID for this effect instance
    pub id: u64,
    /// Type of effect
    pub effect_type: EffectType,
    /// Whether this effect is enabled
    pub enabled: bool,
    /// Effect intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Effect-specific parameters
    pub parameters: HashMap<String, f32>,
    /// Custom shader source (for Custom effect type)
    #[serde(skip)]
    pub custom_shader: Option<String>,
}

impl Effect {
    /// Create a new effect instance
    pub fn new(id: u64, effect_type: EffectType) -> Self {
        let mut parameters = HashMap::new();

        // Set default parameters based on effect type
        match effect_type {
            EffectType::ColorAdjust => {
                parameters.insert("brightness".to_string(), 0.0);
                parameters.insert("contrast".to_string(), 1.0);
                parameters.insert("saturation".to_string(), 1.0);
            }
            EffectType::Blur => {
                parameters.insert("radius".to_string(), 5.0);
                parameters.insert("samples".to_string(), 9.0);
            }
            EffectType::ChromaticAberration => {
                parameters.insert("amount".to_string(), 0.01);
            }
            EffectType::Glow => {
                parameters.insert("threshold".to_string(), 0.5);
                parameters.insert("radius".to_string(), 10.0);
            }
            EffectType::Kaleidoscope => {
                parameters.insert("segments".to_string(), 6.0);
                parameters.insert("rotation".to_string(), 0.0);
            }
            EffectType::Pixelate => {
                parameters.insert("pixel_size".to_string(), 8.0);
            }
            EffectType::Vignette => {
                parameters.insert("radius".to_string(), 0.5);
                parameters.insert("softness".to_string(), 0.5);
            }
            EffectType::FilmGrain => {
                parameters.insert("amount".to_string(), 0.1);
                parameters.insert("speed".to_string(), 1.0);
            }
            _ => {}
        }

        Self {
            id,
            effect_type,
            enabled: true,
            intensity: 1.0,
            parameters,
            custom_shader: None,
        }
    }

    /// Get a parameter value with default fallback
    pub fn get_param(&self, name: &str, default: f32) -> f32 {
        *self.parameters.get(name).unwrap_or(&default)
    }

    /// Set a parameter value
    pub fn set_param(&mut self, name: &str, value: f32) {
        self.parameters.insert(name.to_string(), value);
    }
}

/// Effect chain containing multiple effects
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EffectChain {
    /// Effects in order of application
    pub effects: Vec<Effect>,
    /// Next effect ID to assign
    next_id: u64,
}

impl EffectChain {
    /// Create a new empty effect chain
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            next_id: 1,
        }
    }

    /// Add an effect to the chain
    pub fn add_effect(&mut self, effect_type: EffectType) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.effects.push(Effect::new(id, effect_type));
        id
    }

    /// Remove an effect by ID
    pub fn remove_effect(&mut self, id: u64) -> Option<Effect> {
        if let Some(pos) = self.effects.iter().position(|e| e.id == id) {
            Some(self.effects.remove(pos))
        } else {
            None
        }
    }

    /// Move an effect up in the chain (earlier processing)
    pub fn move_up(&mut self, id: u64) {
        if let Some(pos) = self.effects.iter().position(|e| e.id == id) {
            if pos > 0 {
                self.effects.swap(pos, pos - 1);
            }
        }
    }

    /// Move an effect down in the chain (later processing)
    pub fn move_down(&mut self, id: u64) {
        if let Some(pos) = self.effects.iter().position(|e| e.id == id) {
            if pos < self.effects.len() - 1 {
                self.effects.swap(pos, pos + 1);
            }
        }
    }

    /// Get enabled effects only
    pub fn enabled_effects(&self) -> impl Iterator<Item = &Effect> {
        self.effects.iter().filter(|e| e.enabled)
    }

    /// Get mutable reference to an effect by ID
    pub fn get_effect_mut(&mut self, id: u64) -> Option<&mut Effect> {
        self.effects.iter_mut().find(|e| e.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_chain_creation() {
        let mut chain = EffectChain::new();

        let id1 = chain.add_effect(EffectType::Blur);
        let id2 = chain.add_effect(EffectType::ColorAdjust);

        assert_eq!(chain.effects.len(), 2);
        assert_eq!(chain.effects[0].id, id1);
        assert_eq!(chain.effects[1].id, id2);
    }

    #[test]
    fn test_effect_chain_reorder() {
        let mut chain = EffectChain::new();

        let id1 = chain.add_effect(EffectType::Blur);
        let id2 = chain.add_effect(EffectType::ColorAdjust);

        chain.move_up(id2);

        assert_eq!(chain.effects[0].id, id2);
        assert_eq!(chain.effects[1].id, id1);
    }

    #[test]
    fn test_effect_params() {
        let mut effect = Effect::new(1, EffectType::Blur);

        assert_eq!(effect.get_param("radius", 0.0), 5.0);

        effect.set_param("radius", 10.0);
        assert_eq!(effect.get_param("radius", 0.0), 10.0);
    }

    #[test]
    fn test_effect_chain_enable_disable() {
        let mut chain = EffectChain::new();

        chain.add_effect(EffectType::Blur);
        let id2 = chain.add_effect(EffectType::ColorAdjust);

        // Disable second effect
        if let Some(effect) = chain.get_effect_mut(id2) {
            effect.enabled = false;
        }

        let enabled: Vec<_> = chain.enabled_effects().collect();
        assert_eq!(enabled.len(), 1);
    }
}
