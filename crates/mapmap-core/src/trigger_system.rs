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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::{AudioBand, AudioTriggerOutputConfig};

    fn create_test_module_manager() -> ModuleManager {
        let mut manager = ModuleManager::new();
        manager.create_module("Test Module".to_string());
        manager
    }

    #[test]
    fn test_trigger_system_initialization() {
        let system = TriggerSystem::new();
        assert!(system.active_triggers.is_empty());
    }

    #[test]
    fn test_trigger_system_band_trigger() {
        let mut system = TriggerSystem::new();
        let mut manager = create_test_module_manager();
        let module_id = 1; // First module ID is 1

        // Add AudioFFT trigger part
        let config = AudioTriggerOutputConfig {
            frequency_bands: true,
            ..Default::default()
        };
        let fft_type = ModulePartType::Trigger(TriggerType::AudioFFT {
            band: AudioBand::Bass, // Primary band setting doesn't affect all-band output check logic
            threshold: 0.5,
            output_config: config,
        });

        // We use add_part_with_type to inject our specific config
        let part_id = manager
            .get_module_mut(module_id)
            .unwrap()
            .add_part_with_type(fft_type, (0.0, 0.0));

        // Create Audio Data with high energy in Bass (index 1)
        let mut audio_data = AudioTriggerData::default();
        audio_data.band_energies[1] = 0.8; // > 0.5 threshold

        // Act
        system.update(&manager, &audio_data);

        // Assert
        assert!(
            system.is_active(part_id, 1),
            "Bass band (index 1) should be active"
        );
        assert!(
            !system.is_active(part_id, 0),
            "SubBass band (index 0) should NOT be active"
        );
    }

    #[test]
    fn test_trigger_system_threshold_logic() {
        let mut system = TriggerSystem::new();
        let mut manager = create_test_module_manager();
        let module_id = 1;

        let config = AudioTriggerOutputConfig {
            frequency_bands: true,
            ..Default::default()
        };
        let fft_type = ModulePartType::Trigger(TriggerType::AudioFFT {
            band: AudioBand::Bass,
            threshold: 0.5,
            output_config: config,
        });
        let part_id = manager
            .get_module_mut(module_id)
            .unwrap()
            .add_part_with_type(fft_type, (0.0, 0.0));

        let mut audio_data = AudioTriggerData::default();

        // 1. Below Threshold
        audio_data.band_energies[1] = 0.4;
        system.update(&manager, &audio_data);
        assert!(
            !system.is_active(part_id, 1),
            "Should not trigger below threshold"
        );

        // 2. Above Threshold
        audio_data.band_energies[1] = 0.51;
        system.update(&manager, &audio_data);
        assert!(
            system.is_active(part_id, 1),
            "Should trigger above threshold"
        );
    }

    #[test]
    fn test_trigger_system_volume_triggers() {
        let mut system = TriggerSystem::new();
        let mut manager = create_test_module_manager();
        let module_id = 1;

        let config = AudioTriggerOutputConfig {
            volume_outputs: true,
            ..Default::default()
        };
        let fft_type = ModulePartType::Trigger(TriggerType::AudioFFT {
            band: AudioBand::Bass,
            threshold: 0.5,
            output_config: config,
        });
        let part_id = manager
            .get_module_mut(module_id)
            .unwrap()
            .add_part_with_type(fft_type, (0.0, 0.0));

        let mut audio_data = AudioTriggerData::default();

        // Test RMS (Index 9)
        audio_data.rms_volume = 0.8;
        system.update(&manager, &audio_data);
        assert!(system.is_active(part_id, 9), "RMS trigger should be active");

        // Test Peak (Index 10)
        audio_data.peak_volume = 0.8;
        system.update(&manager, &audio_data);
        assert!(
            system.is_active(part_id, 10),
            "Peak trigger should be active"
        );
    }

    #[test]
    fn test_trigger_system_beat_trigger() {
        let mut system = TriggerSystem::new();
        let mut manager = create_test_module_manager();
        let module_id = 1;

        let config = AudioTriggerOutputConfig {
            beat_output: true,
            ..Default::default()
        };
        // Note: threshold doesn't affect boolean 'beat_detected' in current implementation,
        // but let's set it anyway.
        let fft_type = ModulePartType::Trigger(TriggerType::AudioFFT {
            band: AudioBand::Bass,
            threshold: 0.5,
            output_config: config,
        });
        let part_id = manager
            .get_module_mut(module_id)
            .unwrap()
            .add_part_with_type(fft_type, (0.0, 0.0));

        let mut audio_data = AudioTriggerData::default();
        audio_data.beat_detected = true;

        system.update(&manager, &audio_data);
        assert!(
            system.is_active(part_id, 11),
            "Beat trigger should be active"
        );
    }
}
