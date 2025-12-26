//! MIDI Learn Mode
//!
//! Allows users to map MIDI controls by pressing buttons/turning knobs
//! and automatically detecting the MIDI message.

use super::{MidiMappingKey, MidiMessage};
use std::time::{Duration, Instant};

/// State of the MIDI learn mode
#[derive(Debug, Clone)]
pub enum MidiLearnState {
    /// Not in learn mode
    Inactive,
    /// Waiting for user to trigger a MIDI control
    WaitingForInput {
        /// Target element ID to map
        target_element: String,
        /// When learn mode started
        started_at: Instant,
        /// Timeout duration
        timeout: Duration,
    },
    /// A MIDI message was detected
    Detected {
        /// The detected MIDI message
        message: MidiMessage,
        /// Target element ID
        target_element: String,
    },
    /// Learn mode timed out
    TimedOut {
        /// Target element ID
        target_element: String,
    },
    /// Learn mode was cancelled
    Cancelled,
}

impl Default for MidiLearnState {
    fn default() -> Self {
        Self::Inactive
    }
}

impl MidiLearnState {
    /// Start learn mode for a specific element
    pub fn start(target_element: String, timeout_secs: u64) -> Self {
        Self::WaitingForInput {
            target_element,
            started_at: Instant::now(),
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Check if learn mode is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::WaitingForInput { .. })
    }

    /// Check if we have a detected message
    pub fn has_detection(&self) -> bool {
        matches!(self, Self::Detected { .. })
    }

    /// Get the detected mapping key if available
    pub fn get_detected_key(&self) -> Option<MidiMappingKey> {
        match self {
            Self::Detected { message, .. } => midi_message_to_key(message),
            _ => None,
        }
    }

    /// Get the target element ID if in active state
    pub fn target_element(&self) -> Option<&str> {
        match self {
            Self::WaitingForInput { target_element, .. } => Some(target_element),
            Self::Detected { target_element, .. } => Some(target_element),
            Self::TimedOut { target_element } => Some(target_element),
            _ => None,
        }
    }

    /// Process an incoming MIDI message during learn mode
    pub fn process_message(&mut self, message: MidiMessage) -> bool {
        if let Self::WaitingForInput { target_element, .. } = self {
            // Ignore clock and transport messages
            match message {
                MidiMessage::Clock
                | MidiMessage::Start
                | MidiMessage::Stop
                | MidiMessage::Continue => {
                    return false;
                }
                _ => {}
            }

            *self = Self::Detected {
                message,
                target_element: target_element.clone(),
            };
            true
        } else {
            false
        }
    }

    /// Check for timeout and update state
    pub fn check_timeout(&mut self) -> bool {
        if let Self::WaitingForInput {
            target_element,
            started_at,
            timeout,
        } = self
        {
            if started_at.elapsed() > *timeout {
                *self = Self::TimedOut {
                    target_element: target_element.clone(),
                };
                return true;
            }
        }
        false
    }

    /// Cancel learn mode
    pub fn cancel(&mut self) {
        *self = Self::Cancelled;
    }

    /// Reset to inactive
    pub fn reset(&mut self) {
        *self = Self::Inactive;
    }

    /// Get remaining time if waiting
    pub fn remaining_time(&self) -> Option<Duration> {
        if let Self::WaitingForInput {
            started_at,
            timeout,
            ..
        } = self
        {
            let elapsed = started_at.elapsed();
            if elapsed < *timeout {
                Some(*timeout - elapsed)
            } else {
                Some(Duration::ZERO)
            }
        } else {
            None
        }
    }
}

/// Convert a MidiMessage to a MidiMappingKey (ignoring value)
fn midi_message_to_key(message: &MidiMessage) -> Option<MidiMappingKey> {
    match message {
        MidiMessage::NoteOn { channel, note, .. } => Some(MidiMappingKey::Note(*channel, *note)),
        MidiMessage::NoteOff { channel, note } => Some(MidiMappingKey::Note(*channel, *note)),
        MidiMessage::ControlChange {
            channel,
            controller,
            ..
        } => Some(MidiMappingKey::Control(*channel, *controller)),
        MidiMessage::PitchBend { channel, .. } => Some(MidiMappingKey::PitchBend(*channel)),
        MidiMessage::ProgramChange { channel, .. } => Some(MidiMappingKey::ProgramChange(*channel)),
        _ => None,
    }
}

/// MIDI Learn Manager
#[derive(Debug, Default)]
pub struct MidiLearnManager {
    state: MidiLearnState,
    /// Default timeout in seconds
    default_timeout: u64,
}

impl MidiLearnManager {
    pub fn new() -> Self {
        Self {
            state: MidiLearnState::Inactive,
            default_timeout: 10,
        }
    }

    /// Set default timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.default_timeout = seconds;
        self
    }

    /// Start learning for an element
    pub fn start_learning(&mut self, element_id: &str) {
        self.state = MidiLearnState::start(element_id.to_string(), self.default_timeout);
    }

    /// Get current state
    pub fn state(&self) -> &MidiLearnState {
        &self.state
    }

    /// Process incoming MIDI message
    pub fn process(&mut self, message: MidiMessage) -> bool {
        self.state.process_message(message)
    }

    /// Update state (check timeout)
    pub fn update(&mut self) -> bool {
        self.state.check_timeout()
    }

    /// Cancel learning
    pub fn cancel(&mut self) {
        self.state.cancel();
    }

    /// Accept detected mapping and reset
    pub fn accept(&mut self) -> Option<(String, MidiMappingKey)> {
        if let MidiLearnState::Detected {
            target_element,
            message,
        } = &self.state
        {
            let result = midi_message_to_key(message).map(|key| (target_element.clone(), key));
            self.state.reset();
            result
        } else {
            None
        }
    }

    /// Reset to inactive
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Check if currently learning
    pub fn is_learning(&self) -> bool {
        self.state.is_active()
    }

    /// Check if a message was detected
    pub fn has_detection(&self) -> bool {
        self.state.has_detection()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_learn_flow() {
        let mut manager = MidiLearnManager::new().with_timeout(5);

        // Start learning
        manager.start_learning("test_knob");
        assert!(manager.is_learning());

        // Simulate MIDI CC message
        let cc = MidiMessage::ControlChange {
            channel: 0,
            controller: 16,
            value: 64,
        };

        let detected = manager.process(cc);
        assert!(detected);
        assert!(manager.has_detection());

        // Accept the mapping
        let result = manager.accept();
        assert!(result.is_some());

        let (element_id, key) = result.unwrap();
        assert_eq!(element_id, "test_knob");
        assert_eq!(key, MidiMappingKey::Control(0, 16));
    }

    #[test]
    fn test_midi_learn_ignores_clock() {
        let mut manager = MidiLearnManager::new();
        manager.start_learning("test_button");

        // Clock messages should be ignored
        let detected = manager.process(MidiMessage::Clock);
        assert!(!detected);
        assert!(manager.is_learning()); // Still waiting

        // But note messages should work
        let note = MidiMessage::NoteOn {
            channel: 15,
            note: 36,
            velocity: 127,
        };
        let detected = manager.process(note);
        assert!(detected);
    }

    #[test]
    fn test_midi_learn_cancel() {
        let mut manager = MidiLearnManager::new();
        manager.start_learning("test");

        manager.cancel();
        assert!(!manager.is_learning());
        assert!(matches!(manager.state(), MidiLearnState::Cancelled));
    }

    #[test]
    fn test_midi_learn_state_remaining_time() {
        let state = MidiLearnState::start("test".to_string(), 5);

        if let Some(remaining) = state.remaining_time() {
            assert!(remaining.as_secs() <= 5);
        } else {
            panic!("Should have remaining time");
        }
    }
}
