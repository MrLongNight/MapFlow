//! System for processing module triggers (e.g., AudioFFT)

use crate::audio_reactive::AudioTriggerData;
use crate::module::{ModuleManager, ModulePartType, TriggerType};
use std::collections::{HashMap, HashSet};

/// A set of active trigger outputs. Each entry is (part_id, socket_idx).
pub type ActiveTriggers = HashSet<(u64, usize)>;

/// System for processing and tracking active trigger states
#[derive(Default)]
pub struct TriggerSystem {
    active_triggers: ActiveTriggers,
    /// Timer states for time-based triggers (Part ID -> Elapsed Time)
    timer_states: HashMap<u64, f32>,
    /// Timer targets for random triggers (Part ID -> Target Interval)
    random_targets: HashMap<u64, f32>,
}

impl TriggerSystem {
    /// Create a new trigger system
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the trigger states based on the current audio data and module configuration.
    pub fn update(&mut self, module_manager: &ModuleManager, audio_data: &AudioTriggerData, dt: f32) {
        self.active_triggers.clear();

        for module in module_manager.modules() {
            for part in &module.parts {
                if let ModulePartType::Trigger(trigger) = &part.part_type {
                    match trigger {
                        TriggerType::AudioFFT {
                            band: _,
                            threshold,
                            output_config,
                        } => {
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
                                socket_index += 1;
                            }

                            // Fallback: If no outputs are enabled, we default to a single Beat output (index 0)
                            if !any_output_enabled && audio_data.beat_detected {
                                self.active_triggers.insert((part.id, 0));
                            }

                            // Silence unused assignment warning for the last increment
                            let _ = socket_index;
                        }
                        TriggerType::Beat => {
                            if audio_data.beat_detected {
                                self.active_triggers.insert((part.id, 0));
                            }
                        }
                        TriggerType::Fixed { interval_ms, .. } => {
                            let interval = *interval_ms as f32 / 1000.0;
                            let timer = self.timer_states.entry(part.id).or_insert(0.0);
                            *timer += dt;
                            if *timer >= interval {
                                *timer -= interval;
                                self.active_triggers.insert((part.id, 0));
                            }
                        }
                        TriggerType::Random {
                            min_interval_ms,
                            max_interval_ms,
                            ..
                        } => {
                            let timer = self.timer_states.entry(part.id).or_insert(0.0);
                            *timer += dt;

                            let target = *self.random_targets.entry(part.id).or_insert_with(|| {
                                use rand::Rng;
                                let mut rng = rand::rng();
                                rng.random_range(*min_interval_ms..=*max_interval_ms) as f32
                                    / 1000.0
                            });

                            if *timer >= target {
                                *timer = 0.0;
                                self.active_triggers.insert((part.id, 0));
                                // Pick new target
                                use rand::Rng;
                                let mut rng = rand::rng();
                                let next_target = rng.random_range(*min_interval_ms..=*max_interval_ms)
                                    as f32
                                    / 1000.0;
                                self.random_targets.insert(part.id, next_target);
                            }
                        }
                        // Other triggers (Midi, Osc, Shortcut) handled by event system or direct inputs
                        _ => {}
                    }
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
