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
                    output_config: _,
                }) = &part.part_type
                {
                    // Check each of the 9 frequency bands
                    for i in 0..9 {
                        if audio_data.band_energies[i] > *threshold {
                            self.active_triggers.insert((part.id, i));
                        }
                    }
                    // Check RMS, Peak, Beat, BPM
                    if audio_data.rms_volume > *threshold {
                        self.active_triggers.insert((part.id, 9));
                    }
                    if audio_data.peak_volume > *threshold {
                        self.active_triggers.insert((part.id, 10));
                    }
                    if audio_data.beat_detected {
                        self.active_triggers.insert((part.id, 11));
                    }
                    // For BPM, the trigger is usually just the beat itself.
                    // A continuous BPM value doesn't make sense as a trigger here.
                    // The "Beat Out" socket (index 11) handles the primary beat trigger.
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
