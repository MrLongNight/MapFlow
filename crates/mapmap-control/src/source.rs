//! Control Source Definitions
//!
//! This module defines the data structures for identifying control sources
//! in a stable, serializable way.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Represents a unique source of control values.
///
/// This enum is designed to be extensible to include other sources like LFOs,
/// audio analysis, etc. in the future.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlSource {
    /// A MIDI message from a specific device and channel.
    Midi(MidiSource),
    /// An OSC message from a specific address.
    Osc(OscSource),
}

impl ControlSource {
    /// Generates a stable, unique u64 ID for the control source.
    ///
    /// This is achieved by hashing the enum's contents.
    pub fn to_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// Identifies a specific MIDI source.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MidiSource {
    /// A unique identifier for the MIDI device (e.g., its name or a generated ID).
    pub device_id: String,
    /// MIDI channel (0-15).
    pub channel: u8,
    /// The type of MIDI message.
    pub message_type: MidiMessageType,
}

/// The type of MIDI message to listen for.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MidiMessageType {
    /// Control Change message.
    ControlChange { control: u8 },
    /// Note On message.
    NoteOn { note: u8 },
    /// Note Off message.
    NoteOff { note: u8 },
}

/// Identifies a specific OSC source.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OscSource {
    /// The OSC address path (e.g., "/layer/1/opacity").
    pub address: String,
}
