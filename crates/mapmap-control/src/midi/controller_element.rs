//! Controller Element definitions for visual overlay
//!
//! Defines the structure of physical controller elements (knobs, faders, buttons)
//! for rendering in the UI overlay.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Type of controller element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    /// Rotary knob (sends CC 0-127)
    Knob,
    /// Linear fader (sends CC 0-127)
    Fader,
    /// Momentary push button (sends Note On/Off)
    Button,
    /// Toggle switch with on/off state
    Toggle,
    /// Endless rotary encoder (sends relative CC)
    Encoder,
    /// Crossfader (horizontal fader)
    Crossfader,
}

/// Visual position and size of an element in the overlay
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ElementPosition {
    /// X position (0.0-1.0 normalized, relative to overlay width)
    pub x: f32,
    /// Y position (0.0-1.0 normalized, relative to overlay height)
    pub y: f32,
    /// Width (0.0-1.0 normalized)
    pub width: f32,
    /// Height (0.0-1.0 normalized)
    pub height: f32,
}

impl Default for ElementPosition {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.05,
            height: 0.05,
        }
    }
}

/// MIDI message configuration for an element
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MidiConfig {
    /// Control Change message
    Cc {
        /// MIDI Channel
        channel: u8,
        /// Controller Number
        controller: u8
    },
    /// Note On/Off message
    Note {
        /// MIDI Channel
        channel: u8,
        /// Note Number
        note: u8
    },
    /// Relative CC (for encoders)
    CcRelative {
        /// MIDI Channel
        channel: u8,
        /// Controller Number
        controller: u8
    },
}

/// A single controller element definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerElement {
    /// Unique identifier (e.g., "ch2_gain", "encoder_1_a1")
    pub id: String,
    /// Element type
    pub element_type: ElementType,
    /// Section/group this element belongs to
    pub section: String,
    /// Human-readable label
    pub label: String,
    /// Visual position in overlay
    pub position: ElementPosition,
    /// MIDI configuration
    pub midi: Option<MidiConfig>,
    /// Whether this element is affected by LAYOUT switch
    #[serde(default)]
    pub layout_aware: bool,
    /// Whether this element is affected by A/B switch
    #[serde(default)]
    pub ab_aware: bool,
    /// Asset/image file for this element type
    #[serde(default)]
    pub asset: Option<String>,
    /// Animation range [top, bottom] normalized (0.0-1.0) relative to element height
    #[serde(default)]
    pub animation_range: Option<[f32; 2]>,
}

/// Runtime state of an element
#[derive(Debug, Clone)]
pub struct ElementState {
    /// Raw MIDI value (0-127)
    pub value: u8,
    /// Normalized value (0.0-1.0)
    pub normalized: f32,
    /// Active state for buttons/toggles
    pub active: bool,
    /// Last update timestamp
    pub last_update: Instant,
}

impl Default for ElementState {
    fn default() -> Self {
        Self {
            value: 0,
            normalized: 0.0,
            active: false,
            last_update: Instant::now(),
        }
    }
}

impl ElementState {
    /// Update from a CC value (0-127)
    pub fn update_cc(&mut self, value: u8) {
        self.value = value;
        self.normalized = value as f32 / 127.0;
        self.last_update = Instant::now();
    }

    /// Update from a Note On event
    pub fn update_note_on(&mut self, velocity: u8) {
        self.value = velocity;
        self.normalized = velocity as f32 / 127.0;
        self.active = true;
        self.last_update = Instant::now();
    }

    /// Update from a Note Off event
    pub fn update_note_off(&mut self) {
        self.active = false;
        self.last_update = Instant::now();
    }

    /// Toggle the active state (for toggle switches)
    pub fn toggle(&mut self) {
        self.active = !self.active;
        self.normalized = if self.active { 1.0 } else { 0.0 };
        self.last_update = Instant::now();
    }
}

/// Controller element registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerElements {
    /// Controller name
    pub controller: String,
    /// All elements
    pub elements: Vec<ControllerElement>,
}

impl ControllerElements {
    /// Load from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Save to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Find element by ID
    pub fn find(&self, id: &str) -> Option<&ControllerElement> {
        self.elements.iter().find(|e| e.id == id)
    }

    /// Get elements by section
    pub fn by_section(&self, section: &str) -> Vec<&ControllerElement> {
        self.elements
            .iter()
            .filter(|e| e.section == section)
            .collect()
    }

    /// Get all unique sections
    pub fn sections(&self) -> Vec<&str> {
        let mut sections: Vec<&str> = self.elements.iter().map(|e| e.section.as_str()).collect();
        sections.sort();
        sections.dedup();
        sections
    }
}

/// Runtime element state manager
#[derive(Debug, Default)]
pub struct ElementStateManager {
    states: HashMap<String, ElementState>,
}

impl ElementStateManager {
    /// Create a new element state manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Get state for an element
    pub fn get(&self, id: &str) -> Option<&ElementState> {
        self.states.get(id)
    }

    /// Get mutable state for an element, creating if needed
    pub fn get_or_create(&mut self, id: &str) -> &mut ElementState {
        self.states.entry(id.to_string()).or_default()
    }

    /// Update element from CC message
    pub fn update_cc(&mut self, id: &str, value: u8) {
        self.get_or_create(id).update_cc(value);
    }

    /// Update element from Note On message
    pub fn update_note_on(&mut self, id: &str, velocity: u8) {
        self.get_or_create(id).update_note_on(velocity);
    }

    /// Update element from Note Off message
    pub fn update_note_off(&mut self, id: &str) {
        self.get_or_create(id).update_note_off();
    }

    /// Get all states
    pub fn all_states(&self) -> &HashMap<String, ElementState> {
        &self.states
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_state_cc_update() {
        let mut state = ElementState::default();
        state.update_cc(64);

        assert_eq!(state.value, 64);
        assert!((state.normalized - 0.504).abs() < 0.01);
    }

    #[test]
    fn test_element_state_note_on_off() {
        let mut state = ElementState::default();

        state.update_note_on(127);
        assert!(state.active);
        assert_eq!(state.normalized, 1.0);

        state.update_note_off();
        assert!(!state.active);
    }

    #[test]
    fn test_element_state_toggle() {
        let mut state = ElementState::default();

        assert!(!state.active);
        state.toggle();
        assert!(state.active);
        state.toggle();
        assert!(!state.active);
    }

    #[test]
    fn test_controller_elements_json() {
        let elements = ControllerElements {
            controller: "Test Controller".to_string(),
            elements: vec![ControllerElement {
                id: "test_knob".to_string(),
                element_type: ElementType::Knob,
                section: "test".to_string(),
                label: "Test Knob".to_string(),
                position: ElementPosition::default(),
                midi: Some(MidiConfig::Cc {
                    channel: 0,
                    controller: 16,
                }),
                layout_aware: false,
                ab_aware: false,
                asset: None,
                animation_range: None,
            }],
        };

        let json = elements.to_json().unwrap();
        let loaded = ControllerElements::from_json(&json).unwrap();

        assert_eq!(loaded.controller, "Test Controller");
        assert_eq!(loaded.elements.len(), 1);
        assert_eq!(loaded.elements[0].id, "test_knob");
    }

    #[test]
    fn test_element_state_manager() {
        let mut manager = ElementStateManager::new();

        manager.update_cc("knob_1", 100);
        manager.update_note_on("button_1", 127);

        assert_eq!(manager.get("knob_1").unwrap().value, 100);
        assert!(manager.get("button_1").unwrap().active);
        assert!(manager.get("nonexistent").is_none());
    }
}
