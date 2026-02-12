//! System for processing module triggers (e.g., AudioFFT)

use crate::audio_reactive::AudioTriggerData;
use crate::module::{ModuleManager, ModulePartType, TriggerType};
use std::collections::HashSet;

/// A set of active trigger outputs. Each entry is (part_id, socket_idx).
pub type ActiveTriggers = HashSet<(u64, usize)>;

/// System for processing and tracking active trigger states
#[derive(Default)]
pub struct TriggerSystem {
    active_triggers: ActiveTriggers,
}

impl TriggerSystem {
    /// Create a new trigger system
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the trigger states based on the current audio data and module configuration.
    pub fn update(&mut self, module_manager: &ModuleManager, audio_data: &AudioTriggerData) {
        self.active_triggers.clear();

        for module in module_manager.modules() {
            for part in &module.parts {
                match &part.part_type {
                    ModulePartType::Trigger(TriggerType::AudioFFT {
                        band: _,
                        threshold,
                        output_config,
                    }) => {
                        let mut current_socket_idx = 0;
                        let mut sockets_generated = false;

                        // Check Frequency Bands (9 bands)
                        if output_config.frequency_bands {
                            sockets_generated = true;
                            for i in 0..9 {
                                // Check if band i is active
                                if audio_data.band_energies[i] > *threshold {
                                    self.active_triggers.insert((part.id, current_socket_idx));
                                }
                                current_socket_idx += 1;
                            }
                        }

                        // Check Volume Outputs (RMS, Peak)
                        if output_config.volume_outputs {
                            sockets_generated = true;
                            // RMS
                            if audio_data.rms_volume > *threshold {
                                self.active_triggers.insert((part.id, current_socket_idx));
                            }
                            current_socket_idx += 1;

                            // Peak
                            if audio_data.peak_volume > *threshold {
                                self.active_triggers.insert((part.id, current_socket_idx));
                            }
                            current_socket_idx += 1;
                        }

                        // Check Beat Output
                        if output_config.beat_output {
                            sockets_generated = true;
                            if audio_data.beat_detected {
                                self.active_triggers.insert((part.id, current_socket_idx));
                            }
                            current_socket_idx += 1;
                        }

                        // Check BPM Output
                        if output_config.bpm_output {
                            sockets_generated = true;
                            // BPM is a value, not a momentary trigger.
                            // For boolean active state (e.g. UI light), we might just consider it always "active"
                            // if BPM is detected, or pulsing on beat.
                            // For now, let's say it's active if BPM is present.
                            if audio_data.bpm.is_some() {
                                self.active_triggers.insert((part.id, current_socket_idx));
                            }
                            current_socket_idx += 1;
                        }

                        // Fallback: If no outputs generated, a default "Beat Out" is added
                        if !sockets_generated {
                            if audio_data.beat_detected {
                                self.active_triggers.insert((part.id, 0));
                            }
                        }
                    }
                    ModulePartType::Trigger(TriggerType::Beat) => {
                        // Legacy Beat trigger has 1 output: "Trigger Out"
                        if audio_data.beat_detected {
                            self.active_triggers.insert((part.id, 0));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Check if a specific trigger output is currently active.
    pub fn is_active(&self, part_id: u64, socket_idx: usize) -> bool {
        self.active_triggers.contains(&(part_id, socket_idx))
    }

    /// Get all active triggers.
    pub fn get_active_triggers(&self) -> &ActiveTriggers {
        &self.active_triggers
    }
}
