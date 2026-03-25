use crate::i18n::LocaleManager;
use crate::icons::AppIcon;
use serde::{Deserialize, Serialize};

/// Available effect types (mirror of vorce-render::EffectType)
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
