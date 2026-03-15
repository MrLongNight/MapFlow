//! Universal Trigger Router
//!
//! Maps external inputs (MIDI, OSC, GPIO, etc.) to timeline and show-control actions,
//! specifically designed for the Trackline Mode.

use serde::{Deserialize, Serialize};

#[cfg(feature = "midi")]
use crate::midi::MidiMessage;

/// Sources of external triggers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExternalTriggerSource {
    #[cfg(feature = "midi")]
    /// MIDI message pattern
    Midi {
        channel: u8,
        /// Match type: NoteOn, CC, PC
        message_type: MidiMatchType,
        /// Match value 1: Note number, CC number, PC number
        index: u8,
        /// Optional Match value 2: Velocity, CC value
        value: Option<u8>,
    },
    #[cfg(feature = "osc")]
    /// OSC address pattern
    Osc {
        address: String,
        /// Optional argument value to match
        value_pattern: Option<String>,
    },
    /// GPIO pin state change
    Gpio { pin: u8, state: bool },
    /// Keyboard Shortcut or other custom string-based source
    Custom(String),
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiMatchType {
    NoteOn,
    ControlChange,
    ProgramChange,
}

/// Actions resulting from a trigger, mostly timeline/trackline oriented
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerAction {
    /// Jump to a specific timeline marker by name
    TimelineGotoMarker(String),
    /// Jump to a specific timeline marker by index
    TimelineGotoIndex(usize),
    /// Advance to the next marker
    TimelineNextMarker,
    /// Return to the previous marker
    TimelinePrevMarker,
    /// Play/Pause toggle
    TimelinePlayPause,
    /// Override a specific control target value
    ParameterOverride {
        target: crate::ControlTarget,
        value: crate::ControlValue,
    },
    /// Custom string action for app specific behavior
    Custom(String),
}

/// A mapping from a source to an action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerMapping {
    pub id: String,
    pub source: ExternalTriggerSource,
    pub action: TriggerAction,
    pub enabled: bool,
}

/// The Universal Trigger Router
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TriggerRouter {
    pub mappings: Vec<TriggerMapping>,
}

impl TriggerRouter {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    pub fn add_mapping(&mut self, mapping: TriggerMapping) {
        self.mappings.push(mapping);
    }

    pub fn remove_mapping(&mut self, id: &str) {
        self.mappings.retain(|m| m.id != id);
    }

    pub fn get_mappings(&self) -> &[TriggerMapping] {
        &self.mappings
    }

    pub fn get_mappings_mut(&mut self) -> &mut Vec<TriggerMapping> {
        &mut self.mappings
    }

    /// Process a MIDI message and return triggered actions
    #[cfg(feature = "midi")]
    pub fn process_midi(&self, message: &MidiMessage) -> Vec<TriggerAction> {
        let mut triggered = Vec::new();

        for mapping in self.mappings.iter().filter(|m| m.enabled) {
            if let ExternalTriggerSource::Midi {
                channel,
                message_type,
                index,
                value,
            } = &mapping.source
            {
                let matches = match (message_type, message) {
                    (
                        MidiMatchType::NoteOn,
                        MidiMessage::NoteOn {
                            channel: msg_ch,
                            note: msg_note,
                            velocity: msg_vel,
                        },
                    ) => {
                        *msg_ch == *channel
                            && *msg_note == *index
                            && value.map_or(true, |v| v == *msg_vel)
                    }
                    (
                        MidiMatchType::ControlChange,
                        MidiMessage::ControlChange {
                            channel: msg_ch,
                            controller: msg_cc,
                            value: msg_val,
                        },
                    ) => {
                        *msg_ch == *channel
                            && *msg_cc == *index
                            && value.map_or(true, |v| v == *msg_val)
                    }
                    (
                        MidiMatchType::ProgramChange,
                        MidiMessage::ProgramChange {
                            channel: msg_ch,
                            program: msg_prog,
                        },
                    ) => *msg_ch == *channel && *msg_prog == *index,
                    _ => false,
                };

                if matches {
                    triggered.push(mapping.action.clone());
                }
            }
        }

        triggered
    }

    /// Process an OSC message and return triggered actions
    #[cfg(feature = "osc")]
    pub fn process_osc(&self, address: &str, args: &[rosc::OscType]) -> Vec<TriggerAction> {
        let mut triggered = Vec::new();

        for mapping in self.mappings.iter().filter(|m| m.enabled) {
            if let ExternalTriggerSource::Osc {
                address: map_addr,
                value_pattern,
            } = &mapping.source
            {
                if map_addr == address {
                    let matches_value = match value_pattern {
                        Some(pat) => {
                            if let Some(first_arg) = args.first() {
                                // Simple string representation match for now
                                match first_arg {
                                    rosc::OscType::Float(f) => f.to_string() == *pat,
                                    rosc::OscType::Int(i) => i.to_string() == *pat,
                                    rosc::OscType::String(s) => s == pat,
                                    rosc::OscType::Bool(b) => b.to_string() == *pat,
                                    _ => false,
                                }
                            } else {
                                false
                            }
                        }
                        None => true, // Ignore value if pattern is None
                    };

                    if matches_value {
                        triggered.push(mapping.action.clone());
                    }
                }
            }
        }

        triggered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "midi")]
    #[test]
    fn test_midi_routing() {
        let mut router = TriggerRouter::new();

        router.add_mapping(TriggerMapping {
            id: "goto_start".to_string(),
            source: ExternalTriggerSource::Midi {
                channel: 0,
                message_type: MidiMatchType::NoteOn,
                index: 60,
                value: None, // Any velocity
            },
            action: TriggerAction::TimelineGotoMarker("Start".to_string()),
            enabled: true,
        });

        router.add_mapping(TriggerMapping {
            id: "next_marker".to_string(),
            source: ExternalTriggerSource::Midi {
                channel: 1,
                message_type: MidiMatchType::ControlChange,
                index: 7,
                value: Some(127), // Specific value
            },
            action: TriggerAction::TimelineNextMarker,
            enabled: true,
        });

        // Test NoteOn (should match)
        let actions = router.process_midi(&MidiMessage::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100,
        });
        assert_eq!(actions.len(), 1);
        assert_eq!(
            actions[0],
            TriggerAction::TimelineGotoMarker("Start".to_string())
        );

        // Test NoteOn wrong note (should not match)
        let actions = router.process_midi(&MidiMessage::NoteOn {
            channel: 0,
            note: 61,
            velocity: 100,
        });
        assert_eq!(actions.len(), 0);

        // Test CC wrong value (should not match)
        let actions = router.process_midi(&MidiMessage::ControlChange {
            channel: 1,
            controller: 7,
            value: 64,
        });
        assert_eq!(actions.len(), 0);

        // Test CC correct value (should match)
        let actions = router.process_midi(&MidiMessage::ControlChange {
            channel: 1,
            controller: 7,
            value: 127,
        });
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], TriggerAction::TimelineNextMarker);
    }

    #[cfg(feature = "osc")]
    #[test]
    fn test_osc_routing() {
        let mut router = TriggerRouter::new();

        router.add_mapping(TriggerMapping {
            id: "goto_chorus".to_string(),
            source: ExternalTriggerSource::Osc {
                address: "/trackline/goto".to_string(),
                value_pattern: Some("Chorus".to_string()),
            },
            action: TriggerAction::TimelineGotoMarker("Chorus".to_string()),
            enabled: true,
        });

        router.add_mapping(TriggerMapping {
            id: "play".to_string(),
            source: ExternalTriggerSource::Osc {
                address: "/trackline/play".to_string(),
                value_pattern: None,
            },
            action: TriggerAction::TimelinePlayPause,
            enabled: true,
        });

        // Match address with string value
        let actions = router.process_osc(
            "/trackline/goto",
            &[rosc::OscType::String("Chorus".to_string())],
        );
        assert_eq!(actions.len(), 1);
        assert_eq!(
            actions[0],
            TriggerAction::TimelineGotoMarker("Chorus".to_string())
        );

        // Mismatch string value
        let actions = router.process_osc(
            "/trackline/goto",
            &[rosc::OscType::String("Verse".to_string())],
        );
        assert_eq!(actions.len(), 0);

        // Match address without caring about value
        let actions = router.process_osc("/trackline/play", &[rosc::OscType::Float(1.0)]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], TriggerAction::TimelinePlayPause);
    }
}
