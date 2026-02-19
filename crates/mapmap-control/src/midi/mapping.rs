//! MIDI message to control target mapping
//!
//! Provides HashMap-based mapping for MIDI messages.

use super::MidiMessage;
use crate::error::Result;
use crate::target::{ControlTarget, ControlValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Key for MIDI mapping (ignores value fields)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MidiMappingKey {
    Note(u8, u8),      // channel, note
    Control(u8, u8),   // channel, controller
    PitchBend(u8),     // channel
    ProgramChange(u8), // channel
}

impl From<&MidiMessage> for Option<MidiMappingKey> {
    fn from(msg: &MidiMessage) -> Self {
        match msg {
            MidiMessage::NoteOn { channel, note, .. } => {
                Some(MidiMappingKey::Note(*channel, *note))
            }
            MidiMessage::NoteOff { channel, note } => Some(MidiMappingKey::Note(*channel, *note)),
            MidiMessage::ControlChange {
                channel,
                controller,
                ..
            } => Some(MidiMappingKey::Control(*channel, *controller)),
            MidiMessage::PitchBend { channel, .. } => Some(MidiMappingKey::PitchBend(*channel)),
            MidiMessage::ProgramChange { channel, .. } => {
                Some(MidiMappingKey::ProgramChange(*channel))
            }
            _ => None,
        }
    }
}

/// Maps MIDI messages to control targets
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MidiMapping {
    /// Mapping storage
    pub map: HashMap<MidiMappingKey, MidiControlMapping>,
}

/// A single MIDI to control mapping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MidiControlMapping {
    pub target: ControlTarget,
    pub min_value: f32,
    pub max_value: f32,
    pub curve: MappingCurve,
}

/// Value mapping curve
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MappingCurve {
    Linear,
    Exponential,
    Logarithmic,
    SCurve,
}

impl MidiMapping {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mapping
    pub fn add_mapping(
        &mut self,
        key: MidiMappingKey,
        target: ControlTarget,
        min_value: f32,
        max_value: f32,
        curve: MappingCurve,
    ) {
        self.map.insert(
            key,
            MidiControlMapping {
                target,
                min_value,
                max_value,
                curve,
            },
        );
    }

    /// Remove a mapping
    pub fn remove_mapping(&mut self, key: &MidiMappingKey) {
        self.map.remove(key);
    }

    /// Get the control value for a MIDI message
    pub fn get_control_value(
        &self,
        message: &MidiMessage,
    ) -> Option<(ControlTarget, ControlValue)> {
        let key: Option<MidiMappingKey> = message.into();
        let key = key?;

        let mapping = self.map.get(&key)?;

        // Get the normalized value (0.0-1.0) from the MIDI message
        let normalized = match message {
            MidiMessage::ControlChange { value, .. } => *value as f32 / 127.0,
            MidiMessage::NoteOn { velocity, .. } => *velocity as f32 / 127.0,
            MidiMessage::PitchBend { value, .. } => *value as f32 / 16383.0,
            MidiMessage::NoteOff { .. } => 0.0,
            MidiMessage::ProgramChange { program, .. } => *program as f32 / 127.0,
            _ => return None,
        };

        // Apply curve
        let curved = mapping.curve.apply(normalized);

        // Map to target range
        let value = mapping.min_value + curved * (mapping.max_value - mapping.min_value);

        Some((mapping.target.clone(), ControlValue::Float(value)))
    }

    /// Load from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Save to JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

impl MappingCurve {
    /// Apply the curve to a normalized value (0.0-1.0)
    pub fn apply(&self, value: f32) -> f32 {
        let value = value.clamp(0.0, 1.0);
        match self {
            MappingCurve::Linear => value,
            MappingCurve::Exponential => value * value,
            MappingCurve::Logarithmic => value.sqrt(),
            MappingCurve::SCurve => {
                // Smoothstep function
                value * value * (3.0 - 2.0 * value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_mapping_hashmap() {
        let mut mapping = MidiMapping::new();

        let key = MidiMappingKey::Control(0, 7); // CC 7 on Ch 0

        mapping.add_mapping(
            key,
            ControlTarget::LayerOpacity(0),
            0.0,
            1.0,
            MappingCurve::Linear,
        );

        let msg = MidiMessage::ControlChange {
            channel: 0,
            controller: 7,
            value: 64,
        };

        let (target, value) = mapping.get_control_value(&msg).unwrap();
        assert_eq!(target, ControlTarget::LayerOpacity(0));

        if let ControlValue::Float(v) = value {
            assert!((v - 0.503).abs() < 0.01);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_mapping_curve_linear() {
        let curve = MappingCurve::Linear;
        assert_eq!(curve.apply(0.0), 0.0);
        assert_eq!(curve.apply(0.5), 0.5);
        assert_eq!(curve.apply(1.0), 1.0);
    }

    #[test]
    fn test_mapping_curve_exponential() {
        let curve = MappingCurve::Exponential;
        assert_eq!(curve.apply(0.0), 0.0);
        assert_eq!(curve.apply(0.5), 0.25); // 0.5 * 0.5
        assert_eq!(curve.apply(1.0), 1.0);
    }

    #[test]
    fn test_mapping_curve_logarithmic() {
        let curve = MappingCurve::Logarithmic;
        assert_eq!(curve.apply(0.0), 0.0);
        assert!((curve.apply(0.25) - 0.5).abs() < 1e-6); // sqrt(0.25) = 0.5
        assert_eq!(curve.apply(1.0), 1.0);
    }

    #[test]
    fn test_mapping_curve_scurve() {
        let curve = MappingCurve::SCurve;
        assert_eq!(curve.apply(0.0), 0.0);
        assert_eq!(curve.apply(0.5), 0.5);
        assert_eq!(curve.apply(1.0), 1.0);

        // Check smoothstep property: slower at edges
        // x=0.1 -> 3(0.01) - 2(0.001) = 0.03 - 0.002 = 0.028 < 0.1
        let val_0_1 = curve.apply(0.1);
        assert!(val_0_1 < 0.1);

        // x=0.9 -> 3(0.81) - 2(0.729) = 2.43 - 1.458 = 0.972 > 0.9
        let val_0_9 = curve.apply(0.9);
        assert!(val_0_9 > 0.9);
    }

    #[test]
    fn test_mapping_curve_clamping() {
        let curve = MappingCurve::Linear;
        assert_eq!(curve.apply(-0.5), 0.0);
        assert_eq!(curve.apply(1.5), 1.0);
    }
}
