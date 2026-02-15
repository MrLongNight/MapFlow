use mapmap_core::audio_reactive::AudioTriggerData;
use mapmap_core::module::{
    AudioBand, AudioTriggerOutputConfig, ModuleManager, ModulePartType, TriggerType,
};
use mapmap_core::trigger_system::TriggerSystem;

fn default_audio_data() -> AudioTriggerData {
    AudioTriggerData {
        band_energies: [0.0; 9],
        rms_volume: 0.0,
        peak_volume: 0.0,
        beat_detected: false,
        beat_strength: 0.0,
        bpm: None,
    }
}

#[test]
fn test_initialization() {
    let system = TriggerSystem::new();
    assert!(system.get_active_triggers().is_empty());
}

#[test]
fn test_update_empty_manager() {
    let mut system = TriggerSystem::new();
    let module_manager = ModuleManager::new();
    let audio_data = default_audio_data();

    system.update(&module_manager, &audio_data);
    assert!(system.get_active_triggers().is_empty());
}

#[test]
fn test_update_audio_fft_bands() {
    // 1. Setup
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    // Add AudioFFT Trigger with all frequency bands enabled
    let config = AudioTriggerOutputConfig {
        frequency_bands: true,
        ..Default::default()
    };
    // The band parameter here is technically the "primary" band,
    // but the output_config enables all individual band outputs.
    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // 2. Test each band individually
    for i in 0..9 {
        let mut audio_data = default_audio_data();
        audio_data.band_energies[i] = 0.8; // Trigger threshold is 0.5

        system.update(&module_manager, &audio_data);

        assert!(
            system.is_active(part_id, i),
            "Band index {} should be active",
            i
        );

        // Ensure others are not active (basic check)
        let active_count = system.get_active_triggers().len();
        assert_eq!(active_count, 1, "Only one band should be active");
    }
}

#[test]
fn test_update_audio_volume_beat() {
    // 1. Setup
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    // Add AudioFFT Trigger checking Volume and Beats
    let config = AudioTriggerOutputConfig {
        frequency_bands: false,
        volume_outputs: true,
        beat_output: true,
        bpm_output: false,
        inverted_outputs: Default::default(),
    };
    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Peak, // Doesn't matter for specific outputs
        threshold: 0.5,
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // 2. Stimulate
    let audio_data = AudioTriggerData {
        rms_volume: 0.6,  // > 0.5
        peak_volume: 0.4, // < 0.5
        beat_detected: true,
        ..AudioTriggerData::default()
    };

    // 3. Update
    system.update(&module_manager, &audio_data);

    // 4. Assert
    // Dynamic Socket Indices:
    // Frequency Bands (0) -> Volume RMS (0) -> Volume Peak (1) -> Beat (2) -> BPM (3)
    // Since frequency_bands=false, start with Volume.
    assert!(
        system.is_active(part_id, 0),
        "RMS trigger (socket 0) should be active"
    );
    assert!(
        !system.is_active(part_id, 1),
        "Peak trigger (socket 1) should NOT be active"
    );
    assert!(
        system.is_active(part_id, 2),
        "Beat trigger (socket 2) should be active"
    );
}

#[test]
fn test_update_clears_previous_state() {
    // 1. Setup
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    let config = AudioTriggerOutputConfig {
        beat_output: true,
        ..Default::default()
    };
    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // 2. Activate
    let mut audio_data = AudioTriggerData {
        beat_detected: true,
        ..AudioTriggerData::default()
    };
    system.update(&module_manager, &audio_data);
    assert!(system.is_active(part_id, 0)); // Beat is socket 0 (no bands, no volume)

    // 3. Deactivate (next frame)
    audio_data.beat_detected = false;
    system.update(&module_manager, &audio_data);
    assert!(!system.is_active(part_id, 0));
}

#[test]
fn test_trigger_system_update_thresholds() {
    // 1. Setup
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    let config = AudioTriggerOutputConfig {
        frequency_bands: true,
        ..Default::default()
    };
    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.8, // High threshold
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // 2. Test Below Threshold
    let mut audio_data = default_audio_data();
    audio_data.band_energies[1] = 0.79;
    system.update(&module_manager, &audio_data);
    assert!(!system.is_active(part_id, 1));

    // 3. Test Above Threshold
    audio_data.band_energies[1] = 0.81;
    system.update(&module_manager, &audio_data);
    assert!(system.is_active(part_id, 1));
}

#[test]
fn test_trigger_system_dynamic_indices() {
    // 1. Setup
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    // Configure AudioFFT Trigger with:
    // frequency_bands: FALSE (0-8 skipped)
    // volume_outputs: TRUE (RMS=0, Peak=1)
    // beat_output: TRUE (Beat=2)
    // bpm_output: FALSE
    let config = AudioTriggerOutputConfig {
        frequency_bands: false,
        volume_outputs: true,
        beat_output: true,
        bpm_output: false,
        inverted_outputs: Default::default(),
    };
    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // 2. Stimulate RMS, Peak and Beat
    let audio_data = AudioTriggerData {
        rms_volume: 0.8,
        peak_volume: 0.8,
        beat_detected: true,
        ..default_audio_data()
    };

    // 3. Update
    system.update(&module_manager, &audio_data);

    // 4. Assert Correct Indices (Dynamic)
    // RMS should be at index 0 (because bands are skipped)
    assert!(system.is_active(part_id, 0), "RMS should be at index 0 when bands are disabled");
    // Peak should be at index 1
    assert!(system.is_active(part_id, 1), "Peak should be at index 1");
    // Beat should be at index 2
    assert!(system.is_active(part_id, 2), "Beat should be at index 2");
}
