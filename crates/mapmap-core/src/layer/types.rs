use serde::{Deserialize, Serialize};
use glam::Vec2;

/// Blend mode for compositing layers
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BlendMode {
    /// Normal alpha blending (default)
    #[default]
    Normal,
    /// Add colors (lighten)
    Add,
    /// Subtract colors (darken)
    Subtract,
    /// Multiply colors (darken)
    Multiply,
    /// Screen colors (lighten)
    Screen,
    /// Overlay (combination of multiply and screen)
    Overlay,
    /// Soft light
    SoftLight,
    /// Hard light
    HardLight,
    /// Lighten only (max)
    Lighten,
    /// Darken only (min)
    Darken,
    /// Color dodge
    ColorDodge,
    /// Color burn
    ColorBurn,
    /// Difference
    Difference,
    /// Exclusion
    Exclusion,
}

impl BlendMode {
    /// Get shader function name for this blend mode
    pub fn shader_function(&self) -> &'static str {
        match self {
            BlendMode::Normal => "blend_normal",
            BlendMode::Add => "blend_add",
            BlendMode::Subtract => "blend_subtract",
            BlendMode::Multiply => "blend_multiply",
            BlendMode::Screen => "blend_screen",
            BlendMode::Overlay => "blend_overlay",
            BlendMode::SoftLight => "blend_soft_light",
            BlendMode::HardLight => "blend_hard_light",
            BlendMode::Lighten => "blend_lighten",
            BlendMode::Darken => "blend_darken",
            BlendMode::ColorDodge => "blend_color_dodge",
            BlendMode::ColorBurn => "blend_color_burn",
            BlendMode::Difference => "blend_difference",
            BlendMode::Exclusion => "blend_exclusion",
        }
    }

    /// List all available blend modes
    pub fn all() -> &'static [BlendMode] {
        &[
            BlendMode::Normal,
            BlendMode::Add,
            BlendMode::Subtract,
            BlendMode::Multiply,
            BlendMode::Screen,
            BlendMode::Overlay,
            BlendMode::SoftLight,
            BlendMode::HardLight,
            BlendMode::Lighten,
            BlendMode::Darken,
            BlendMode::ColorDodge,
            BlendMode::ColorBurn,
            BlendMode::Difference,
            BlendMode::Exclusion,
        ]
    }
}

/// Resize mode for automatic content fitting (Phase 1, Month 6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResizeMode {
    /// Fill - Scale to cover entire composition, crop excess
    Fill,
    /// Fit - Scale to fit within composition, letterbox/pillarbox
    #[default]
    Fit,
    /// Stretch - Non-uniform scale to fill composition exactly
    Stretch,
    /// Original - 1:1 pixel mapping, no scaling
    Original,
}

impl ResizeMode {
    /// Calculate transform matrix for this resize mode
    /// Returns scale and translation to apply
    pub fn calculate_transform(&self, source_size: Vec2, target_size: Vec2) -> (Vec2, Vec2) {
        // Prevent division by zero if source is empty
        if source_size.x.abs() < f32::EPSILON || source_size.y.abs() < f32::EPSILON {
            return (Vec2::ZERO, Vec2::ZERO);
        }

        // Prevent weird behavior if target is empty
        if target_size.x.abs() < f32::EPSILON || target_size.y.abs() < f32::EPSILON {
            return (Vec2::ZERO, Vec2::ZERO);
        }

        match self {
            ResizeMode::Fill => {
                // Scale to cover (largest dimension fills, crop other)
                let scale_x = target_size.x / source_size.x;
                let scale_y = target_size.y / source_size.y;
                let scale = scale_x.max(scale_y);
                (Vec2::splat(scale), Vec2::ZERO)
            }
            ResizeMode::Fit => {
                // Scale to fit (smallest dimension fills, letterbox other)
                let scale_x = target_size.x / source_size.x;
                let scale_y = target_size.y / source_size.y;
                let scale = scale_x.min(scale_y);
                (Vec2::splat(scale), Vec2::ZERO)
            }
            ResizeMode::Stretch => {
                // Non-uniform scale to fill exactly
                let scale = target_size / source_size;
                (scale, Vec2::ZERO)
            }
            ResizeMode::Original => {
                // No scaling, 1:1 pixel mapping
                (Vec2::ONE, Vec2::ZERO)
            }
        }
    }
}
