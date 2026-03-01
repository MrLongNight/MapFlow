//! Control target abstraction
//!
//! This module provides a unified abstraction for all controllable parameters in MapFlow.

use serde::{Deserialize, Serialize};
use std::path::{Component, Path};

/// A controllable parameter in the application
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlTarget {
    /// Layer opacity (layer_id, opacity: 0.0-1.0)
    LayerOpacity(u32),
    /// Layer position (layer_id)
    LayerPosition(u32),
    /// Layer scale (layer_id)
    LayerScale(u32),
    /// Layer rotation (layer_id, degrees)
    LayerRotation(u32),
    /// Layer visibility (layer_id)
    LayerVisibility(u32),
    /// Paint parameter (paint_id, param_name)
    PaintParameter(u32, String),
    /// Effect parameter (effect_id, param_name)
    EffectParameter(u32, String),
    /// Playback speed (global or per-layer)
    PlaybackSpeed(Option<u32>),
    /// Playback position (0.0-1.0)
    PlaybackPosition,
    /// Output brightness (output_id, brightness: 0.0-1.0)
    OutputBrightness(u32),
    /// Output edge blend (output_id, edge, width: 0.0-1.0)
    OutputEdgeBlend(u32, EdgeSide),
    /// Master opacity
    MasterOpacity,
    /// Master blackout
    MasterBlackout,
    /// Custom parameter (name)
    Custom(String),
}

impl ControlTarget {
    /// Returns a human-readable name for the target
    pub fn name(&self) -> String {
        match self {
            ControlTarget::LayerOpacity(id) => format!("Layer {} Opacity", id),
            ControlTarget::LayerPosition(id) => format!("Layer {} Position", id),
            ControlTarget::LayerScale(id) => format!("Layer {} Scale", id),
            ControlTarget::LayerRotation(id) => format!("Layer {} Rotation", id),
            ControlTarget::LayerVisibility(id) => format!("Layer {} Visibility", id),
            ControlTarget::PaintParameter(id, name) => format!("Paint {} {}", id, name),
            ControlTarget::EffectParameter(id, name) => format!("Effect {} {}", id, name),
            ControlTarget::PlaybackSpeed(Some(id)) => format!("Layer {} Speed", id),
            ControlTarget::PlaybackSpeed(None) => "Global Speed".to_string(),
            ControlTarget::PlaybackPosition => "Global Position".to_string(),
            ControlTarget::OutputBrightness(id) => format!("Output {} Brightness", id),
            ControlTarget::OutputEdgeBlend(id, _) => format!("Output {} Edge Blend", id),
            ControlTarget::MasterOpacity => "Master Opacity".to_string(),
            ControlTarget::MasterBlackout => "Master Blackout".to_string(),
            ControlTarget::Custom(name) => name.clone(),
        }
    }

    /// Returns a unique string identifier for the target (e.g., for serialization/maps)
    pub fn to_id_string(&self) -> String {
        // We can reuse the JSON serialization or a custom format
        // For simplicity and stability, we use a custom format here
        // that matches what might be used in mapping files or OSC addresses
        match self {
            ControlTarget::LayerOpacity(id) => format!("layer/{}/opacity", id),
            ControlTarget::LayerPosition(id) => format!("layer/{}/position", id),
            ControlTarget::LayerScale(id) => format!("layer/{}/scale", id),
            ControlTarget::LayerRotation(id) => format!("layer/{}/rotation", id),
            ControlTarget::LayerVisibility(id) => format!("layer/{}/visibility", id),
            ControlTarget::PaintParameter(id, name) => format!("paint/{}/{}", id, name),
            ControlTarget::EffectParameter(id, name) => format!("effect/{}/{}", id, name),
            ControlTarget::PlaybackSpeed(Some(id)) => format!("layer/{}/speed", id),
            ControlTarget::PlaybackSpeed(None) => "playback/speed".to_string(),
            ControlTarget::PlaybackPosition => "playback/position".to_string(),
            ControlTarget::OutputBrightness(id) => format!("output/{}/brightness", id),
            ControlTarget::OutputEdgeBlend(id, edge) => format!("output/{}/blend/{:?}", id, edge),
            ControlTarget::MasterOpacity => "master/opacity".to_string(),
            ControlTarget::MasterBlackout => "master/blackout".to_string(),
            ControlTarget::Custom(name) => format!("custom/{}", name),
        }
    }

    /// Validate the target (e.g. check string lengths)
    pub fn validate(&self) -> Result<(), String> {
        const MAX_NAME_LEN: usize = 256;
        match self {
            ControlTarget::PaintParameter(_, name)
            | ControlTarget::EffectParameter(_, name)
            | ControlTarget::Custom(name) => {
                if name.len() > MAX_NAME_LEN {
                    return Err(format!("Name exceeds maximum length of {}", MAX_NAME_LEN));
                }
                if name.chars().any(|c| c.is_control()) {
                    return Err("Name contains control characters".to_string());
                }
                // Prevent path traversal and potential injection
                if name.contains('/') || name.contains('\\') || name.contains("..") {
                    return Err("Name contains invalid characters (/, \\, ..)".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Edge sides for edge blending
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeSide {
    /// Left edge
    Left,
    /// Right edge
    Right,
    /// Top edge
    Top,
    /// Bottom edge
    Bottom,
}

/// Control value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlValue {
    /// Float value (e.g. 0.0 - 1.0)
    Float(f32),
    /// Integer value
    Int(i32),
    /// Boolean value
    Bool(bool),
    /// String value
    String(String),
    /// Color value (RGBA u32)
    Color(u32), // RGBA
    /// 2D Vector (x, y)
    Vec2(f32, f32),
    /// 3D Vector (x, y, z)
    Vec3(f32, f32, f32),
}

impl ControlValue {
    /// Get as float, converting if necessary
    pub fn as_float(&self) -> Option<f32> {
        match self {
            ControlValue::Float(v) => Some(*v),
            ControlValue::Int(v) => Some(*v as f32),
            ControlValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Get as int, converting if necessary
    pub fn as_int(&self) -> Option<i32> {
        match self {
            ControlValue::Int(v) => Some(*v),
            ControlValue::Float(v) => Some(*v as i32),
            ControlValue::Bool(v) => Some(if *v { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Get as bool, converting if necessary
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ControlValue::Bool(v) => Some(*v),
            ControlValue::Int(v) => Some(*v != 0),
            ControlValue::Float(v) => Some(*v != 0.0),
            _ => None,
        }
    }

    /// Get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ControlValue::String(v) => Some(v),
            _ => None,
        }
    }

    /// Validate the value (check string length, finite floats)
    pub fn validate(&self) -> Result<(), String> {
        const MAX_STRING_LEN: usize = 4096;
        match self {
            ControlValue::String(s) => {
                if s.len() > MAX_STRING_LEN {
                    return Err(format!(
                        "String value exceeds maximum length of {}",
                        MAX_STRING_LEN
                    ));
                }
                // Path traversal check
                if Path::new(s)
                    .components()
                    .any(|c| matches!(c, Component::ParentDir))
                {
                    return Err("String value contains path traversal attempt (..)".to_string());
                }
            }
            ControlValue::Float(f) => {
                if !f.is_finite() {
                    return Err("Float value must be finite".to_string());
                }
            }
            ControlValue::Vec2(x, y) => {
                if !x.is_finite() || !y.is_finite() {
                    return Err("Vec2 components must be finite".to_string());
                }
            }
            ControlValue::Vec3(x, y, z) => {
                if !x.is_finite() || !y.is_finite() || !z.is_finite() {
                    return Err("Vec3 components must be finite".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl From<f32> for ControlValue {
    fn from(v: f32) -> Self {
        ControlValue::Float(v)
    }
}

impl From<i32> for ControlValue {
    fn from(v: i32) -> Self {
        ControlValue::Int(v)
    }
}

impl From<bool> for ControlValue {
    fn from(v: bool) -> Self {
        ControlValue::Bool(v)
    }
}

impl From<String> for ControlValue {
    fn from(v: String) -> Self {
        ControlValue::String(v)
    }
}

impl From<(f32, f32)> for ControlValue {
    fn from((x, y): (f32, f32)) -> Self {
        ControlValue::Vec2(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_value_conversions() {
        let float_val = ControlValue::Float(0.75);
        assert_eq!(float_val.as_float(), Some(0.75));
        assert_eq!(float_val.as_int(), Some(0));

        let int_val = ControlValue::Int(42);
        assert_eq!(int_val.as_int(), Some(42));
        assert_eq!(int_val.as_float(), Some(42.0));

        let bool_val = ControlValue::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.as_float(), Some(1.0));
        assert_eq!(bool_val.as_int(), Some(1));
    }

    #[test]
    fn test_control_target_serialization() {
        let target = ControlTarget::LayerOpacity(5);
        let json = serde_json::to_string(&target).unwrap();
        let deserialized: ControlTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(target, deserialized);
    }

    #[test]
    fn test_control_target_validation() {
        let valid = ControlTarget::Custom("Valid Name".to_string());
        assert!(valid.validate().is_ok());

        let long_name = "a".repeat(300);
        let invalid = ControlTarget::Custom(long_name);
        assert!(invalid.validate().is_err());

        let control_char = ControlTarget::Custom("Name\nWith\tControl".to_string());
        assert!(control_char.validate().is_err());

        // Test path traversal protection
        let path_trav = ControlTarget::Custom("../secret".to_string());
        assert!(path_trav.validate().is_err());

        let slash = ControlTarget::Custom("foo/bar".to_string());
        assert!(slash.validate().is_err());

        let backslash = ControlTarget::Custom("foo\\bar".to_string());
        assert!(backslash.validate().is_err());
    }

    #[test]
    fn test_control_value_validation() {
        let valid = ControlValue::String("Valid".to_string());
        assert!(valid.validate().is_ok());

        let long_string = "a".repeat(5000);
        let invalid = ControlValue::String(long_string);
        assert!(invalid.validate().is_err());

        let inf_float = ControlValue::Float(f32::INFINITY);
        assert!(inf_float.validate().is_err());

        let nan_vec = ControlValue::Vec2(0.0, f32::NAN);
        assert!(nan_vec.validate().is_err());

        // Path traversal
        let traversal = ControlValue::String("../secret".to_string());
        assert!(traversal.validate().is_err());

        let traversal2 = ControlValue::String("foo/../bar".to_string());
        assert!(traversal2.validate().is_err());

        let valid_dots = ControlValue::String("Loading...".to_string());
        assert!(valid_dots.validate().is_ok());

        let dots_in_name = ControlValue::String("my..file".to_string());
        assert!(dots_in_name.validate().is_ok());
    }

    #[test]
    fn test_control_target_to_id_string() {
        assert_eq!(
            ControlTarget::LayerOpacity(5).to_id_string(),
            "layer/5/opacity"
        );
        assert_eq!(
            ControlTarget::PaintParameter(2, "brightness".into()).to_id_string(),
            "paint/2/brightness"
        );
        assert_eq!(
            ControlTarget::MasterOpacity.to_id_string(),
            "master/opacity"
        );
        assert_eq!(
            ControlTarget::Custom("my_param".into()).to_id_string(),
            "custom/my_param"
        );
    }

    #[test]
    fn test_control_value_as_string() {
        let s = ControlValue::String("hello".to_string());
        assert_eq!(s.as_string(), Some("hello"));

        let f = ControlValue::Float(1.0);
        assert_eq!(f.as_string(), None);
    }
}
