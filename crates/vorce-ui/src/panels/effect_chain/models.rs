use super::types::EffectType;
use serde::{Deserialize, Serialize};

/// Effect instance for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIEffect {
    /// Unique identifier for this entity.
    pub id: u64,
    pub effect_type: EffectType,
    pub enabled: bool,
    pub intensity: f32,
    pub expanded: bool,
    pub lut_path: String,
    pub error: Option<String>,
    pub parameters: std::collections::HashMap<String, f32>,
}

impl UIEffect {
    /// Unique identifier for this entity.
    pub fn new(id: u64, effect_type: EffectType) -> Self {
        Self {
            id,
            effect_type,
            enabled: true,
            intensity: 1.0,
            expanded: true,
            lut_path: String::new(),
            error: None,
            parameters: effect_type.default_params(),
        }
    }

    /// Human-readable display name.
    pub fn get_param(&self, name: &str, default: f32) -> f32 {
        *self.parameters.get(name).unwrap_or(&default)
    }

    /// Human-readable display name.
    pub fn set_param(&mut self, name: &str, value: f32) {
        self.parameters.insert(name.to_string(), value);
    }
}

/// Effect chain for UI
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UIEffectChain {
    pub effects: Vec<UIEffect>,
    next_id: u64,
}

impl UIEffectChain {
    /// Creates a new, uninitialized instance with default settings.
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_effect(&mut self, effect_type: EffectType) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.effects.push(UIEffect::new(id, effect_type));
        id
    }

    /// Unique identifier for this entity.
    pub fn remove_effect(&mut self, id: u64) {
        self.effects.retain(|e| e.id != id);
    }

    /// Unique identifier for this entity.
    pub fn move_up(&mut self, id: u64) {
        if let Some(pos) = self.effects.iter().position(|e| e.id == id) {
            if pos > 0 {
                self.effects.swap(pos, pos - 1);
            }
        }
    }

    /// Unique identifier for this entity.
    pub fn move_down(&mut self, id: u64) {
        if let Some(pos) = self.effects.iter().position(|e| e.id == id) {
            if pos < self.effects.len() - 1 {
                self.effects.swap(pos, pos + 1);
            }
        }
    }

    /// Unique identifier for this entity.
    pub fn move_effect(&mut self, id: u64, to_idx: usize) {
        if let Some(from_idx) = self.effects.iter().position(|e| e.id == id) {
            if from_idx == to_idx {
                return;
            }
            if to_idx >= self.effects.len() {
                return;
            }
            let effect = self.effects.remove(from_idx);
            self.effects.insert(to_idx, effect);
        }
    }

    /// Unique identifier for this entity.
    pub fn get_effect_mut(&mut self, id: u64) -> Option<&mut UIEffect> {
        self.effects.iter_mut().find(|e| e.id == id)
    }
}

/// Actions from the effect chain panel
#[derive(Debug, Clone)]
pub enum EffectChainAction {
    /// Add a new effect of the given type
    AddEffect(EffectType),
    /// Add a new effect with specific parameters
    AddEffectWithParams(EffectType, std::collections::HashMap<String, f32>),
    /// Remove an effect by ID
    RemoveEffect(u64),
    /// Move effect up in chain
    MoveUp(u64),
    /// Move effect down in chain
    MoveDown(u64),
    /// Move effect to specific index
    MoveEffect(u64, usize),
    /// Toggle effect enabled state
    ToggleEnabled(u64),
    /// Set effect intensity
    SetIntensity(u64, f32),
    /// Set effect parameter
    SetParameter(u64, String, f32),
    /// Set LUT path
    SetLUTPath(u64, String),
    /// Reset effect parameters to default
    ResetEffect(u64),
    /// Load a preset by name
    LoadPreset(String),
    /// Save current chain as preset
    SavePreset(String),
    /// Clear all effects
    ClearAll,
}

/// Preset entry for the browser
#[derive(Debug, Clone)]
pub struct PresetEntry {
    /// Human-readable display name.
    pub name: String,
    pub name_lower: String,
    pub category: String,
    /// File system path to the asset or resource.
    pub path: String,
    pub is_favorite: bool,
}
