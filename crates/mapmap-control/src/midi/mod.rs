//! MIDI input/output system

#[cfg(feature = "midi")]
mod clock;
#[cfg(feature = "midi")]
mod controller_element;
#[cfg(feature = "midi")]
mod ecler_nuo4;
#[cfg(feature = "midi")]
mod input;
#[cfg(feature = "midi")]
mod mapping;
#[cfg(feature = "midi")]
mod midi_learn;
#[cfg(feature = "midi")]
mod output;
#[cfg(feature = "midi")]
mod profiles;

#[cfg(feature = "midi")]
pub use clock::*;
#[cfg(feature = "midi")]
pub use controller_element::*;
#[cfg(feature = "midi")]
pub use ecler_nuo4::*;
#[cfg(feature = "midi")]
pub use input::*;
#[cfg(feature = "midi")]
pub use mapping::*;
#[cfg(feature = "midi")]
pub use midi_learn::*;
#[cfg(feature = "midi")]
pub use output::*;
#[cfg(feature = "midi")]
pub use profiles::*;

use serde::{Deserialize, Serialize};

/// MIDI message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MidiMessage {
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    PitchBend {
        channel: u8,
        value: u16,
    },
    Clock,
    Start,
    Stop,
    Continue,
}

impl MidiMessage {
    /// Parse a MIDI message from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        let status = bytes[0];

        // Real-time messages (single byte)
        match status {
            0xF8 => return Some(MidiMessage::Clock),
            0xFA => return Some(MidiMessage::Start),
            0xFC => return Some(MidiMessage::Stop),
            0xFB => return Some(MidiMessage::Continue),
            _ => {}
        }

        // Channel messages (need at least 2 bytes)
        if bytes.len() < 2 {
            return None;
        }

        let message_type = status & 0xF0;
        let channel = status & 0x0F;

        match message_type {
            0x90 => {
                // Note On
                let velocity = bytes[2];
                if velocity == 0 {
                    // Note On with velocity 0 is treated as Note Off
                    Some(MidiMessage::NoteOff {
                        channel,
                        note: bytes[1],
                    })
                } else {
                    Some(MidiMessage::NoteOn {
                        channel,
                        note: bytes[1],
                        velocity,
                    })
                }
            }
            0x80 => {
                // Note Off
                Some(MidiMessage::NoteOff {
                    channel,
                    note: bytes[1],
                })
            }
            0xB0 => {
                // Control Change
                Some(MidiMessage::ControlChange {
                    channel,
                    controller: bytes[1],
                    value: bytes[2],
                })
            }
            0xC0 => {
                // Program Change
                Some(MidiMessage::ProgramChange {
                    channel,
                    program: bytes[1],
                })
            }
            0xE0 => {
                // Pitch Bend
                let value = ((bytes[2] as u16) << 7) | (bytes[1] as u16);
                Some(MidiMessage::PitchBend { channel, value })
            }
            _ => None,
        }
    }

    /// Checks if this message matches another, ignoring value fields for mapping
    pub fn matches(&self, other: &MidiMessage) -> bool {
        match (self, other) {
            (
                MidiMessage::NoteOn {
                    channel: ch1,
                    note: n1,
                    ..
                },
                MidiMessage::NoteOn {
                    channel: ch2,
                    note: n2,
                    ..
                },
            ) => ch1 == ch2 && n1 == n2,
            (
                MidiMessage::ControlChange {
                    channel: ch1,
                    controller: c1,
                    ..
                },
                MidiMessage::ControlChange {
                    channel: ch2,
                    controller: c2,
                    ..
                },
            ) => ch1 == ch2 && c1 == c2,
            (
                MidiMessage::PitchBend { channel: ch1, .. },
                MidiMessage::PitchBend { channel: ch2, .. },
            ) => ch1 == ch2,
            _ => self == other,
        }
    }

    /// Convert to raw MIDI bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            MidiMessage::NoteOn {
                channel,
                note,
                velocity,
            } => vec![0x90 | channel, *note, *velocity],
            MidiMessage::NoteOff { channel, note } => vec![0x80 | channel, *note, 0],
            MidiMessage::ControlChange {
                channel,
                controller,
                value,
            } => vec![0xB0 | channel, *controller, *value],
            MidiMessage::ProgramChange { channel, program } => vec![0xC0 | channel, *program],
            MidiMessage::PitchBend { channel, value } => {
                vec![0xE0 | channel, (*value & 0x7F) as u8, (*value >> 7) as u8]
            }
            MidiMessage::Clock => vec![0xF8],
            MidiMessage::Start => vec![0xFA],
            MidiMessage::Stop => vec![0xFC],
            MidiMessage::Continue => vec![0xFB],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_message_parsing() {
        // Note On
        let msg = MidiMessage::from_bytes(&[0x90, 60, 100]);
        assert_eq!(
            msg,
            Some(MidiMessage::NoteOn {
                channel: 0,
                note: 60,
                velocity: 100
            })
        );

        // Note Off (via Note On with velocity 0)
        let msg = MidiMessage::from_bytes(&[0x90, 60, 0]);
        assert_eq!(
            msg,
            Some(MidiMessage::NoteOff {
                channel: 0,
                note: 60
            })
        );

        // Control Change
        let msg = MidiMessage::from_bytes(&[0xB0, 7, 64]);
        assert_eq!(
            msg,
            Some(MidiMessage::ControlChange {
                channel: 0,
                controller: 7,
                value: 64
            })
        );

        // Clock
        let msg = MidiMessage::from_bytes(&[0xF8]);
        assert_eq!(msg, Some(MidiMessage::Clock));
    }

    #[test]
    fn test_midi_message_to_bytes() {
        let msg = MidiMessage::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100,
        };
        assert_eq!(msg.to_bytes(), vec![0x90, 60, 100]);

        let msg = MidiMessage::ControlChange {
            channel: 0,
            controller: 7,
            value: 64,
        };
        assert_eq!(msg.to_bytes(), vec![0xB0, 7, 64]);

        let msg = MidiMessage::Clock;
        assert_eq!(msg.to_bytes(), vec![0xF8]);
    }

    #[test]
    fn test_midi_message_parsing_extended() {
        // Note Off Explicit (0x80)
        let msg = MidiMessage::from_bytes(&[0x80, 60, 0]);
        assert_eq!(
            msg,
            Some(MidiMessage::NoteOff {
                channel: 0,
                note: 60
            })
        );

        // Program Change (0xC0)
        let msg = MidiMessage::from_bytes(&[0xC0, 10]);
        assert_eq!(
            msg,
            Some(MidiMessage::ProgramChange {
                channel: 0,
                program: 10
            })
        );

        // Pitch Bend (0xE0)
        // LSB = 0x00, MSB = 0x40 (center = 8192)
        // 0x40 << 7 = 0x2000 = 8192
        let msg = MidiMessage::from_bytes(&[0xE0, 0x00, 0x40]);
        assert_eq!(
            msg,
            Some(MidiMessage::PitchBend {
                channel: 0,
                value: 8192
            })
        );

        // Pitch Bend Max
        // LSB = 0x7F, MSB = 0x7F
        // 0x7F | (0x7F << 7) = 127 | 16256 = 16383
        let msg = MidiMessage::from_bytes(&[0xE0, 0x7F, 0x7F]);
        assert_eq!(
            msg,
            Some(MidiMessage::PitchBend {
                channel: 0,
                value: 16383
            })
        );

        // Start
        let msg = MidiMessage::from_bytes(&[0xFA]);
        assert_eq!(msg, Some(MidiMessage::Start));

        // Stop
        let msg = MidiMessage::from_bytes(&[0xFC]);
        assert_eq!(msg, Some(MidiMessage::Stop));

        // Continue
        let msg = MidiMessage::from_bytes(&[0xFB]);
        assert_eq!(msg, Some(MidiMessage::Continue));
    }

    #[test]
    fn test_midi_matches() {
        // Note On Matches
        let n1 = MidiMessage::NoteOn {
            channel: 1,
            note: 60,
            velocity: 100,
        };
        let n2 = MidiMessage::NoteOn {
            channel: 1,
            note: 60,
            velocity: 50,
        }; // Different velocity
        let n3 = MidiMessage::NoteOn {
            channel: 1,
            note: 61,
            velocity: 100,
        }; // Different note

        assert!(n1.matches(&n2));
        assert!(!n1.matches(&n3));

        // CC Matches
        let c1 = MidiMessage::ControlChange {
            channel: 2,
            controller: 10,
            value: 0,
        };
        let c2 = MidiMessage::ControlChange {
            channel: 2,
            controller: 10,
            value: 127,
        };
        let c3 = MidiMessage::ControlChange {
            channel: 2,
            controller: 11,
            value: 0,
        };

        assert!(c1.matches(&c2));
        assert!(!c1.matches(&c3));

        // Pitch Bend Matches (channel only)
        let p1 = MidiMessage::PitchBend {
            channel: 3,
            value: 0,
        };
        let p2 = MidiMessage::PitchBend {
            channel: 3,
            value: 16383,
        };
        let p3 = MidiMessage::PitchBend {
            channel: 4,
            value: 0,
        };

        assert!(p1.matches(&p2));
        assert!(!p1.matches(&p3));
    }
}




