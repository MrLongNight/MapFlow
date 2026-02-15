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
                if let ModulePartType::Trigger(TriggerType::AudioFFT {
                    band: _,
                    threshold,
                    output_config,
                }) = &part.part_type
                {
                    let mut socket_index = 0;
                    let mut any_output_enabled = false;

                    // 1. Frequency Bands (9 outputs)
                    if output_config.frequency_bands {
                        any_output_enabled = true;
                        for i in 0..9 {
                            if audio_data.band_energies[i] > *threshold {
                                self.active_triggers.insert((part.id, socket_index));
                            }
                            socket_index += 1;
                        }
                    }

                    // 2. Volume Outputs (RMS, Peak)
                    if output_config.volume_outputs {
                        any_output_enabled = true;
                        // RMS
                        if audio_data.rms_volume > *threshold {
                            self.active_triggers.insert((part.id, socket_index));
                        }
                        socket_index += 1;

                        // Peak
                        if audio_data.peak_volume > *threshold {
                            self.active_triggers.insert((part.id, socket_index));
                        }
                        socket_index += 1;
                    }

                    // 3. Beat Output
                    if output_config.beat_output {
                        any_output_enabled = true;
                        if audio_data.beat_detected {
                            self.active_triggers.insert((part.id, socket_index));
                        }
                        socket_index += 1;
                    }

                    // 4. BPM Output (Reserved Index)
                    if output_config.bpm_output {
                        any_output_enabled = true;
                        // BPM is a continuous value, not a trigger event.
                        // However, we must reserve the socket index to maintain alignment
                        // with the module graph (which generates a "BPM Out" socket).
                        // No trigger is inserted here.
                        // socket_index += 1; // Unused increment (BPM is last)
                    }

                    // Fallback: If no outputs are enabled, we default to a single Beat output (index 0)
                    if !any_output_enabled && audio_data.beat_detected {
                        self.active_triggers.insert((part.id, 0));
                    }

                    // Silence unused assignment warning for the last increment
                    let _ = socket_index;
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
