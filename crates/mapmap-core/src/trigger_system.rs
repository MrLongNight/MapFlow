//! System for processing module triggers (e.g., AudioFFT)

use crate::audio_reactive::AudioTriggerData;
use crate::module::{ModuleManager, ModulePartType, TriggerType};
use rand::Rng;
use std::collections::{HashMap, HashSet};

/// A set of active trigger outputs. Each entry is (part_id, socket_idx).
pub type ActiveTriggers = HashSet<(u64, usize)>;

/// State for a trigger (timer, target interval, etc.)
#[derive(Debug, Clone, Copy)]
pub struct TriggerState {
    /// Accumulated time since last trigger
    pub timer: f32,
    /// Target interval for the next trigger (used for Random triggers)
    ///
    /// A value < 0.0 indicates that the target has not been initialized.
    pub target: f32,
}

impl Default for TriggerState {
    fn default() -> Self {
        Self {
            timer: 0.0,
            target: -1.0,
        }
    }
}

/// System for processing and tracking active trigger states
#[derive(Default)]
pub struct TriggerSystem {
    active_triggers: ActiveTriggers,
    /// Unified states for triggers (Part ID -> State)
    ///
    /// Optimized to reduce hash lookups by storing timer and target together.
    states: HashMap<u64, TriggerState>,
}

impl TriggerSystem {
    /// Create a new trigger system
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the trigger states based on the current audio data and module configuration.
    pub fn update(
        &mut self,
        module_manager: &ModuleManager,
        audio_data: &AudioTriggerData,
        dt: f32,
    ) {
        self.active_triggers.clear();

<<<<<<< HEAD
        // Hoist RNG initialization to avoid repeated thread-local access in the loop
=======
        // Hoist RNG to avoid repeated initialization inside the loop
        use rand::Rng;
>>>>>>> 23d054a3 (Guardian: Fix TriggerSystem memory leak and add consistency tests)
        let mut rng = rand::rng();

        // Track parts that actively use state to perform Garbage Collection
        let mut active_state_users = HashSet::new();

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
                            active_state_users.insert(part.id); // Mark as using state

                            let interval = *interval_ms as f32 / 1000.0;
                            // Unified state lookup (O(1))
                            let state = self.states.entry(part.id).or_default();
                            state.timer += dt;
                            if state.timer >= interval {
                                state.timer -= interval;
                                self.active_triggers.insert((part.id, 0));
                            }
                        }
                        TriggerType::Random {
                            min_interval_ms,
                            max_interval_ms,
                            ..
                        } => {
                            active_state_users.insert(part.id); // Mark as using state

                            // Unified state lookup (O(1)) - Handles both timer and target
                            let state = self.states.entry(part.id).or_default();

                            // Initialize target if needed (first run or after type switch)
                            if state.target < 0.0 {
                                state.target = rng.random_range(*min_interval_ms..=*max_interval_ms)
                                    as f32
                                    / 1000.0;
                            }

                            state.timer += dt;

                            if state.timer >= state.target {
                                state.timer = 0.0;
                                self.active_triggers.insert((part.id, 0));

                                // Pick new target using hoisted RNG
                                state.target = rng.random_range(*min_interval_ms..=*max_interval_ms)
                                    as f32
                                    / 1000.0;
                            }
                        }
                        // Other triggers (Midi, Osc, Shortcut) handled by event system or direct inputs
                        _ => {}
                    }
                }
            }
        }

        // Garbage Collection: Remove states for parts that no longer exist or don't use state
        self.states.retain(|id, _| active_state_users.contains(id));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_reactive::AudioTriggerData;
    use crate::module::{ModuleManager, ModulePartType, PartType, TriggerType};

    #[test]
    fn test_fixed_trigger() {
        let mut manager = ModuleManager::new();
        let module_id = manager.create_module("Test".to_string());

        let trigger_type = ModulePartType::Trigger(TriggerType::Fixed {
            interval_ms: 100, // 0.1s
            offset_ms: 0,
        });

        // add_part creates a default trigger (Beat), we replace it
        let part_id = manager
            .add_part_to_module(module_id, PartType::Trigger, (0.0, 0.0))
            .unwrap();

        if let Some(module) = manager.get_module_mut(module_id) {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                part.part_type = trigger_type;
            }
        }

        let mut system = TriggerSystem::new();
        let audio = AudioTriggerData::default();

        // 0.0s -> 0.05s (No trigger)
        system.update(&manager, &audio, 0.05);
        assert!(!system.is_active(part_id, 0));

        // 0.05s -> 0.10s (Trigger!)
        system.update(&manager, &audio, 0.05);
        assert!(system.is_active(part_id, 0));

        // 0.10s -> 0.15s (No trigger, timer reset to 0.0)
        system.update(&manager, &audio, 0.05);
        assert!(!system.is_active(part_id, 0));

        // 0.15s -> 0.20s (Trigger again!)
        system.update(&manager, &audio, 0.05);
        assert!(system.is_active(part_id, 0));
    }

    #[test]
    fn test_random_trigger_initialization_and_firing() {
        let mut manager = ModuleManager::new();
        let module_id = manager.create_module("Test Random".to_string());

        // Random interval between 100ms and 200ms
        let trigger_type = ModulePartType::Trigger(TriggerType::Random {
            min_interval_ms: 100,
            max_interval_ms: 200,
            probability: 1.0,
        });

        let part_id = manager
            .add_part_to_module(module_id, PartType::Trigger, (0.0, 0.0))
            .unwrap();

        if let Some(module) = manager.get_module_mut(module_id) {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                part.part_type = trigger_type;
            }
        }

        let mut system = TriggerSystem::new();
        let audio = AudioTriggerData::default();

        // First update: should initialize target
        system.update(&manager, &audio, 0.01);

        // Verify state exists and has valid target
        let state = system
            .states
            .get(&part_id)
            .expect("State should be initialized");
        assert!(state.target >= 0.1 && state.target <= 0.2);
        assert!(state.timer > 0.0);

        // Advance time until it definitely fires (max 0.2s)
        // We already did 0.01s. Add 0.3s.
        system.update(&manager, &audio, 0.3);

        // Should have fired
        assert!(system.is_active(part_id, 0));

        // Timer should be reset (low value) and target should be new
        let new_state = system.states.get(&part_id).unwrap();
        assert!(new_state.timer < 0.1);
        // Note: timer is reset to 0.0 upon firing, but  adds  *before* check?
        // Wait, looking at code:
        // state.timer += dt;
        // if state.timer >= state.target { state.timer = 0.0; ... }
        // So timer is 0.0 at the end of the frame it fired.
        assert_eq!(new_state.timer, 0.0);
        assert!(new_state.target >= 0.1 && new_state.target <= 0.2);
    }

    #[test]
    fn test_trigger_system_garbage_collection() {
        let mut manager = ModuleManager::new();
        let module_id = manager.create_module("Test GC".to_string());

        // Create a Random trigger (which uses state)
        let trigger_type = ModulePartType::Trigger(TriggerType::Random {
            min_interval_ms: 100,
            max_interval_ms: 200,
            probability: 1.0,
        });

        let part_id = manager
            .add_part_to_module(module_id, PartType::Trigger, (0.0, 0.0))
            .unwrap();

        if let Some(module) = manager.get_module_mut(module_id) {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                part.part_type = trigger_type;
            }
        }

        let mut system = TriggerSystem::new();
        let audio = AudioTriggerData::default();

        // 1. Update to initialize state
        system.update(&manager, &audio, 0.01);
<<<<<<< HEAD
        assert!(
            system.states.contains_key(&part_id),
            "State should be created"
        );

        // 2. Remove the part
        if let Some(module) = manager.get_module_mut(module_id) {
            module.parts.retain(|p| p.id != part_id);
=======
        assert!(system.states.contains_key(&part_id), "State should be created");

        // 2. Remove the part
        if let Some(module) = manager.get_module_mut(module_id) {
             module.parts.retain(|p| p.id != part_id);
>>>>>>> 23d054a3 (Guardian: Fix TriggerSystem memory leak and add consistency tests)
        }

        // 3. Update again
        system.update(&manager, &audio, 0.01);

        // 4. Assert state is gone
<<<<<<< HEAD
        assert!(
            !system.states.contains_key(&part_id),
            "State should be garbage collected"
        );
=======
        assert!(!system.states.contains_key(&part_id), "State should be garbage collected");
>>>>>>> 23d054a3 (Guardian: Fix TriggerSystem memory leak and add consistency tests)
    }

    #[test]
    fn test_audio_fft_socket_consistency() {
        // This test ensures TriggerSystem logic matches AudioTriggerOutputConfig logic
        let mut manager = ModuleManager::new();
        let module_id = manager.create_module("Test FFT".to_string());

        let config = crate::module::AudioTriggerOutputConfig {
            frequency_bands: true,
            volume_outputs: true,
            beat_output: true,
            bpm_output: true,
            inverted_outputs: Default::default(),
        };

        // Get expected sockets
        let expected_sockets = config.generate_outputs();

        // Setup part
        let trigger_type = ModulePartType::Trigger(TriggerType::AudioFFT {
            band: crate::module::AudioBand::Bass, // Irrelevant for this test
<<<<<<< HEAD
            threshold: 0.0,                       // Low threshold to trigger everything
=======
            threshold: 0.0, // Low threshold to trigger everything
>>>>>>> 23d054a3 (Guardian: Fix TriggerSystem memory leak and add consistency tests)
            output_config: config,
        });

        let part_id = manager
            .add_part_to_module(module_id, PartType::Trigger, (0.0, 0.0))
            .unwrap();

        if let Some(module) = manager.get_module_mut(module_id) {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                part.part_type = trigger_type;
            }
        }

        let mut system = TriggerSystem::new();
        // Mock audio data that triggers EVERYTHING
<<<<<<< HEAD
        let mut audio = AudioTriggerData {
            beat_detected: true,
            peak_volume: 1.0,
            rms_volume: 1.0,
            ..Default::default()
        };
=======
        let mut audio = AudioTriggerData::default();
        audio.beat_detected = true;
        audio.peak_volume = 1.0;
        audio.rms_volume = 1.0;
>>>>>>> 23d054a3 (Guardian: Fix TriggerSystem memory leak and add consistency tests)
        for i in 0..9 {
            audio.band_energies[i] = 1.0;
        }

        system.update(&manager, &audio, 0.01);

        // Check each expected socket index is active
        for (i, socket) in expected_sockets.iter().enumerate() {
            if socket.name == "BPM Out" {
                assert!(
                    !system.is_active(part_id, i),
                    "BPM Out should NOT be active in TriggerSystem (handled separately)"
                );
                continue;
            }

            assert!(
                system.is_active(part_id, i),
                "Socket '{}' at index {} should be active",
                socket.name,
                i
            );
        }

        // Check no extra sockets active (e.g. index 100)
<<<<<<< HEAD
        assert!(
            !system.is_active(part_id, expected_sockets.len()),
            "Index out of bounds should not be active"
        );
=======
        assert!(!system.is_active(part_id, expected_sockets.len()), "Index out of bounds should not be active");
>>>>>>> 23d054a3 (Guardian: Fix TriggerSystem memory leak and add consistency tests)
    }
}
