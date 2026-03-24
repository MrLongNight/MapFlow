#[allow(unused_imports)]
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, TextureHandle, Ui, Vec2};

#[allow(unused_imports)]
use crate::config::{MidiAssignment, MidiAssignmentTarget, UserConfig};

#[cfg(feature = "midi")]
use vorce_control::midi::{ControllerElements, MidiMessage};
use vorce_core::runtime_paths;
#[allow(unused_imports)]
use std::collections::{HashMap, HashSet};

use super::panel::{ControllerOverlayPanel, MidiLearnTarget};

impl ControllerOverlayPanel {
    #[cfg(feature = "midi")]
    pub fn load_elements(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let elements = ControllerElements::from_json(json)?;
        // Dynamic expansion removed - now using static elements from JSON for better control
        self.elements = Some(elements);
        Ok(())
    }
    #[cfg(feature = "midi")]
    pub fn process_midi(&mut self, message: MidiMessage) {
        // Check if in learn mode
        if self.learn_manager.process(message) {
            return; // Message was consumed by learn mode
        }

        // Update element states based on message
        if let Some(elements) = &self.elements {
            for element in &elements.elements {
                if let Some(midi_config) = &element.midi {
                    if Self::message_matches_config(&message, midi_config) {
                        // Track activity for global learn
                        self.last_active_element = Some(element.id.clone());
                        self.last_active_time = Some(std::time::Instant::now());

                        match message {
                            MidiMessage::ControlChange { value, .. } => {
                                self.state_manager.update_cc(&element.id, value);
                            }
                            MidiMessage::NoteOn { velocity, .. } => {
                                self.state_manager.update_note_on(&element.id, velocity);
                            }
                            MidiMessage::NoteOff { .. } => {
                                self.state_manager.update_note_off(&element.id);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    #[cfg(feature = "midi")]
    fn message_matches_config(
        message: &MidiMessage,
        config: &vorce_control::midi::MidiConfig,
    ) -> bool {
        // Use full path to avoid import conflict or unused import warnings
        // if we import MidiConfig at module level
        match (message, config) {
            (
                MidiMessage::ControlChange {
                    channel,
                    controller,
                    ..
                },
                vorce_control::midi::MidiConfig::Cc {
                    channel: cfg_ch,
                    controller: cfg_cc,
                },
            ) => *channel == *cfg_ch && *controller == *cfg_cc,
            (
                MidiMessage::ControlChange {
                    channel,
                    controller,
                    ..
                },
                vorce_control::midi::MidiConfig::CcRelative {
                    channel: cfg_ch,
                    controller: cfg_cc,
                },
            ) => *channel == *cfg_ch && *controller == *cfg_cc,
            (
                MidiMessage::NoteOn { channel, note, .. },
                vorce_control::midi::MidiConfig::Note {
                    channel: cfg_ch,
                    note: cfg_note,
                },
            ) => *channel == *cfg_ch && *note == *cfg_note,
            (
                MidiMessage::NoteOff { channel, note },
                vorce_control::midi::MidiConfig::Note {
                    channel: cfg_ch,
                    note: cfg_note,
                },
            ) => *channel == *cfg_ch && *note == *cfg_note,
            _ => false,
        }
    }
    #[cfg(feature = "midi")]
    pub fn start_learn(&mut self, element_id: &str, target: MidiLearnTarget) {
        self.learn_target = Some(target);
        self.learn_manager.start_learning(element_id);
    }
    #[cfg(feature = "midi")]
    pub fn cancel_learn(&mut self) {
        self.learn_target = None;
        self.learn_manager.cancel();
    }
    #[cfg(feature = "midi")]
    pub fn is_learning(&self) -> bool {
        self.learn_manager.is_learning()
    }

    pub(crate) fn save_elements(&self) {
        #[cfg(feature = "midi")]
        if let Some(elements) = &self.elements {
            if let Some(path) =
                runtime_paths::existing_resource_path("controllers/ecler_nuo4/elements.json")
            {
                match serde_json::to_string_pretty(elements) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(&path, json) {
                            tracing::error!("Failed to save elements to {:?}: {}", path, e);
                        } else {
                            tracing::info!("Saved elements to {:?}", path);
                        }
                    }
                    Err(e) => tracing::error!("Failed to serialize elements: {}", e),
                }
                return;
            }
            tracing::error!("Could not find elements.json to save to.");
        }
    }
}
