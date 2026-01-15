//! Control target abstraction
//!
//! This module provides a unified abstraction for all controllable parameters in MapFlow.

use serde::{Deserialize, Serialize};

/// Maximum length for names (targets, paints, effects)
const MAX_NAME_LENGTH: usize = 256;

/// Maximum length for string values
const MAX_STRING_VALUE_LENGTH: usize = 4096;

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
    /// Validate the target
    pub fn validate(&self) -> Result<(), String> {
        match self {
            ControlTarget::PaintParameter(_, name) => {
                if name.len() > MAX_NAME_LENGTH {
                    return Err(format!("Paint parameter name exceeds {} characters", MAX_NAME_LENGTH));
                }
            }
            ControlTarget::EffectParameter(_, name) => {
                if name.len() > MAX_NAME_LENGTH {
                    return Err(format!("Effect parameter name exceeds {} characters", MAX_NAME_LENGTH));
                }
            }
            ControlTarget::Custom(name) => {
                if name.len() > MAX_NAME_LENGTH {
                    return Err(format!("Custom target name exceeds {} characters", MAX_NAME_LENGTH));
                }
            }
            _ => {}
        }
        Ok(())
    }

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
    /// Validate the value
    pub fn validate(&self) -> Result<(), String> {
        match self {
            ControlValue::Float(v) => {
                if !v.is_finite() {
                    return Err("Float value must be finite".to_string());
                }
            }
            ControlValue::Vec2(x, y) => {
                if !x.is_finite() || !y.is_finite() {
                    return Err("Vec2 values must be finite".to_string());
                }
            }
            ControlValue::Vec3(x, y, z) => {
                if !x.is_finite() || !y.is_finite() || !z.is_finite() {
                    return Err("Vec3 values must be finite".to_string());
                }
            }
            ControlValue::String(s) => {
                if s.len() > MAX_STRING_VALUE_LENGTH {
                    return Err(format!("String value exceeds {} characters", MAX_STRING_VALUE_LENGTH));
                }
            }
            _ => {}
        }
        Ok(())
    }

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
        let valid = ControlTarget::Custom("valid".to_string());
        assert!(valid.validate().is_ok());

        let invalid = ControlTarget::Custom("a".repeat(MAX_NAME_LENGTH + 1));
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_control_value_validation() {
        let valid_float = ControlValue::Float(1.0);
        assert!(valid_float.validate().is_ok());

        let invalid_float = ControlValue::Float(f32::NAN);
        assert!(invalid_float.validate().is_err());

        let invalid_inf = ControlValue::Float(f32::INFINITY);
        assert!(invalid_inf.validate().is_err());

        let valid_string = ControlValue::String("short".to_string());
        assert!(valid_string.validate().is_ok());

        let invalid_string = ControlValue::String("a".repeat(MAX_STRING_VALUE_LENGTH + 1));
        assert!(invalid_string.validate().is_err());
    }
}
