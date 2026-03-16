use crate::i18n::LocaleManager;
use crate::icons::AppIcon;
use serde::{Deserialize, Serialize};

/// Available effect types (mirror of mapmap-render::EffectType)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    LoadLUT,
    ColorAdjust,
    Blur,
    ChromaticAberration,
    EdgeDetect,
    Glow,
    Kaleidoscope,
    Invert,
    Pixelate,
    Vignette,
    FilmGrain,
    Wave,
    Glitch,
    RgbSplit,
    Mirror,
    HueShift,
    Voronoi,
    Tunnel,
    Galaxy,
    Custom,
}

impl EffectType {
    pub fn display_name(&self, locale: &LocaleManager) -> String {
        match self {
            EffectType::LoadLUT => locale.t("effect-name-load-lut"),
            EffectType::ColorAdjust => locale.t("effect-name-color-adjust"),
            EffectType::Blur => locale.t("effect-name-blur"),
            EffectType::ChromaticAberration => locale.t("effect-name-chromatic-aberration"),
            EffectType::EdgeDetect => locale.t("effect-name-edge-detect"),
            EffectType::Glow => locale.t("effect-name-glow"),
            EffectType::Kaleidoscope => locale.t("effect-name-kaleidoscope"),
            EffectType::Invert => locale.t("effect-name-invert"),
            EffectType::Pixelate => locale.t("effect-name-pixelate"),
            EffectType::Vignette => locale.t("effect-name-vignette"),
            EffectType::FilmGrain => locale.t("effect-name-film-grain"),
            EffectType::Wave => locale.t("effect-name-wave"),
            EffectType::Glitch => locale.t("effect-name-glitch"),
            EffectType::RgbSplit => locale.t("effect-name-rgb-split"),
            EffectType::Mirror => locale.t("effect-name-mirror"),
            EffectType::HueShift => locale.t("effect-name-hue-shift"),
            EffectType::Voronoi => locale.t("effect-name-voronoi"),
            EffectType::Tunnel => locale.t("effect-name-tunnel"),
            EffectType::Galaxy => locale.t("effect-name-galaxy"),
            EffectType::Custom => locale.t("effect-name-custom"),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            EffectType::LoadLUT => "🧊",
            EffectType::ColorAdjust => "🎨",
            EffectType::Blur => "🌫️",
            EffectType::ChromaticAberration => "🌈",
            EffectType::EdgeDetect => "📐",
            EffectType::Glow => "✨",
            EffectType::Kaleidoscope => "🔮",
            EffectType::Invert => "🔄",
            EffectType::Pixelate => "🟩",
            EffectType::Vignette => "🌑",
            EffectType::FilmGrain => "🎞️",
            EffectType::Wave => "🌊",
            EffectType::Glitch => "👾",
            EffectType::RgbSplit => "🌈",
            EffectType::Mirror => "🪞",
            EffectType::HueShift => "🎨",
            EffectType::Voronoi => "💠",
            EffectType::Tunnel => "🌀",
            EffectType::Galaxy => "🌌",
            EffectType::Custom => "⚙️",
        }
    }

    pub fn app_icon(&self) -> AppIcon {
        match self {
            EffectType::LoadLUT => AppIcon::ImageFile,
            EffectType::ColorAdjust => AppIcon::MagicWand,
            EffectType::Blur => AppIcon::MagicWand,
            EffectType::ChromaticAberration => AppIcon::MagicWand,
            EffectType::EdgeDetect => AppIcon::Pencil,
            EffectType::Glow => AppIcon::MagicWand,
            EffectType::Kaleidoscope => AppIcon::MagicWand,
            EffectType::Invert => AppIcon::Repeat,
            EffectType::Pixelate => AppIcon::Screen,
            EffectType::Vignette => AppIcon::AppWindow,
            EffectType::FilmGrain => AppIcon::VideoFile,
            EffectType::Wave => AppIcon::MagicWand,
            EffectType::Glitch => AppIcon::Screen,
            EffectType::RgbSplit => AppIcon::MagicWand,
            EffectType::Mirror => AppIcon::Repeat,
            EffectType::HueShift => AppIcon::PaintBucket,
            EffectType::Voronoi => AppIcon::MagicWand,
            EffectType::Tunnel => AppIcon::MagicWand,
            EffectType::Galaxy => AppIcon::MagicWand,
            EffectType::Custom => AppIcon::Cog,
        }
    }

    pub fn all() -> &'static [EffectType] {
        &[
            EffectType::LoadLUT,
            EffectType::ColorAdjust,
            EffectType::Blur,
            EffectType::ChromaticAberration,
            EffectType::EdgeDetect,
            EffectType::Glow,
            EffectType::Kaleidoscope,
            EffectType::Invert,
            EffectType::Pixelate,
            EffectType::Vignette,
            EffectType::FilmGrain,
            EffectType::Wave,
            EffectType::Glitch,
            EffectType::RgbSplit,
            EffectType::Mirror,
            EffectType::HueShift,
            EffectType::Voronoi,
            EffectType::Tunnel,
            EffectType::Galaxy,
            EffectType::Custom,
        ]
    }

    pub fn default_params(&self) -> std::collections::HashMap<String, f32> {
        let mut parameters = std::collections::HashMap::new();

        match self {
            EffectType::LoadLUT => {
                parameters.insert("intensity".to_string(), 1.0);
            }
            EffectType::ColorAdjust => {
                parameters.insert("brightness".to_string(), 0.0);
                parameters.insert("contrast".to_string(), 1.0);
                parameters.insert("saturation".to_string(), 1.0);
            }
            EffectType::Blur => {
                parameters.insert("radius".to_string(), 5.0);
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
            EffectType::Voronoi => {
                parameters.insert("scale".to_string(), 10.0);
                parameters.insert("offset".to_string(), 1.0);
                parameters.insert("cell_size".to_string(), 1.0);
                parameters.insert("distortion".to_string(), 0.5);
            }
            EffectType::Tunnel => {
                parameters.insert("speed".to_string(), 0.5);
                parameters.insert("rotation".to_string(), 0.5);
                parameters.insert("scale".to_string(), 0.5);
                parameters.insert("distortion".to_string(), 0.5);
            }
            EffectType::Galaxy => {
                parameters.insert("zoom".to_string(), 0.5);
                parameters.insert("speed".to_string(), 0.2);
                parameters.insert("radius".to_string(), 1.0);
                parameters.insert("brightness".to_string(), 1.0);
            }
            _ => {}
        }
        parameters
    }
}

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
    pub category: String,
    /// File system path to the asset or resource.
    pub path: String,
    pub is_favorite: bool,
}
