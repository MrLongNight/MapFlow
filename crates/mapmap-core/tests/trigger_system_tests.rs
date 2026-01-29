use mapmap_core::audio_reactive::AudioTriggerData;
use mapmap_core::module::{
    AudioBand, AudioTriggerOutputConfig, ModuleManager, ModulePartType, TriggerType,
};
use mapmap_core::trigger_system::TriggerSystem;

#[test]
fn test_initialization() {
    let system = TriggerSystem::new();
    assert!(system.get_active_triggers().is_empty());
}

#[test]
fn test_update_empty_manager() {
    let mut system = TriggerSystem::new();
    let manager = ModuleManager::new();
    let audio_data = AudioTriggerData::default();

    system.update(&manager, &audio_data);
    assert!(system.get_active_triggers().is_empty());
}

#[test]
fn test_update_audio_fft_bands() {
    let mut system = TriggerSystem::new();
    let mut manager = ModuleManager::new();
    let module_id = manager.create_module("Test Module".to_string());

    // Configure output to enable frequency bands
    let config = AudioTriggerOutputConfig {
        frequency_bands: true,
        ..Default::default()
    };

    let trigger_type = TriggerType::AudioFFT {
        band: AudioBand::Bass, // This is the "primary" band, but the system checks all 9 if enabled
        threshold: 0.5,
        output_config: config,
    };

    let part_type = ModulePartType::Trigger(trigger_type);

    // Add part to module
    let module = manager.get_module_mut(module_id).unwrap();
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // Prepare audio data
    let mut audio_data = AudioTriggerData::default();
    // Set Band 0 (SubBass) and Band 8 (Air) above threshold (0.5)
    audio_data.band_energies[0] = 0.8;
    audio_data.band_energies[1] = 0.2; // Below
    audio_data.band_energies[8] = 0.6;

    // Run update
    system.update(&manager, &audio_data);

    // Verify
    assert!(system.is_active(part_id, 0)); // Band 0 active
    assert!(!system.is_active(part_id, 1)); // Band 1 inactive
    assert!(system.is_active(part_id, 8)); // Band 8 active
}

#[test]
fn test_update_audio_volume_beat() {
    let mut system = TriggerSystem::new();
    let mut manager = ModuleManager::new();
    let module_id = manager.create_module("Test Module".to_string());

    // Configure output to enable volume and beat
    let config = AudioTriggerOutputConfig {
        volume_outputs: true,
        beat_output: true,
        ..Default::default()
    };

    let trigger_type = TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    };

    let part_type = ModulePartType::Trigger(trigger_type);

    let module = manager.get_module_mut(module_id).unwrap();
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // Case 1: RMS above threshold
    let audio_data = AudioTriggerData {
        rms_volume: 0.6,
        peak_volume: 0.1,
        beat_detected: false,
        ..Default::default()
    };

    system.update(&manager, &audio_data);
    assert!(system.is_active(part_id, 9)); // RMS (index 9)
    assert!(!system.is_active(part_id, 10)); // Peak (index 10)
    assert!(!system.is_active(part_id, 11)); // Beat (index 11)

    // Case 2: Peak above threshold and Beat detected
    // We create a new struct to avoid mutation if possible, or just reuse
    let audio_data = AudioTriggerData {
        rms_volume: 0.1,
        peak_volume: 0.9,
        beat_detected: true,
        ..Default::default()
    };

    system.update(&manager, &audio_data);
    assert!(!system.is_active(part_id, 9));
    assert!(system.is_active(part_id, 10)); // Peak
    assert!(system.is_active(part_id, 11)); // Beat
}

#[test]
fn test_update_clears_previous_state() {
    let mut system = TriggerSystem::new();
    let mut manager = ModuleManager::new();
    let module_id = manager.create_module("Test".to_string());

    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: AudioTriggerOutputConfig { beat_output: true, ..Default::default() },
    });

    let module = manager.get_module_mut(module_id).unwrap();
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // Frame 1: Beat active
    let mut audio_data = AudioTriggerData {
        beat_detected: true,
        ..Default::default()
    };
    system.update(&manager, &audio_data);
    assert!(system.is_active(part_id, 11));

    // Frame 2: Beat inactive
    audio_data.beat_detected = false;
    system.update(&manager, &audio_data);
    assert!(!system.is_active(part_id, 11));
}
