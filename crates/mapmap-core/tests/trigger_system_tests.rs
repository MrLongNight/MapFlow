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

    system.update(&module_manager, &audio_data, 0.016);
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

        system.update(&module_manager, &audio_data, 0.016);

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
    system.update(&module_manager, &audio_data, 0.016);

    // 4. Assert
    // Dynamic Socket indices:
    // Bands: Disabled (0)
    // Volume: Enabled -> RMS at 0, Peak at 1
    // Beat: Enabled -> Beat at 2
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

    // Config with only beat output enabled explicitly
    // Default has beat_output = true, but let's be explicit
    let config = AudioTriggerOutputConfig {
        beat_output: true,
        frequency_bands: false,
        volume_outputs: false,
        bpm_output: false,
        inverted_outputs: Default::default(),
    };
    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // 2. Activate
    // Bands(0) + Vol(0) + Beat(1) -> Beat is at index 0
    let mut audio_data = AudioTriggerData {
        beat_detected: true,
        ..AudioTriggerData::default()
    };
    system.update(&module_manager, &audio_data, 0.016);
    assert!(system.is_active(part_id, 0));

    // 3. Deactivate (next frame)
    audio_data.beat_detected = false;
    system.update(&module_manager, &audio_data, 0.016);
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
    system.update(&module_manager, &audio_data, 0.016);
    assert!(!system.is_active(part_id, 1));

    // 3. Test Above Threshold
    audio_data.band_energies[1] = 0.81;
    system.update(&module_manager, &audio_data, 0.016);
    assert!(system.is_active(part_id, 1));
}

#[test]
fn test_dynamic_socket_indexing() {
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    // Config: Volume (2) + Beat (1) + BPM (1, reserved)
    // Bands: Disabled
    // Indices:
    // RMS: 0
    // Peak: 1
    // Beat: 2
    // BPM: 3 (reserved, never active)
    let config = AudioTriggerOutputConfig {
        frequency_bands: false,
        volume_outputs: true,
        beat_output: true,
        bpm_output: true,
        inverted_outputs: Default::default(),
    };
    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    let audio_data = AudioTriggerData {
        rms_volume: 0.9,
        peak_volume: 0.9,
        beat_detected: true,
        bpm: Some(120.0),
        ..AudioTriggerData::default()
    };

    system.update(&module_manager, &audio_data, 0.016);

    assert!(system.is_active(part_id, 0), "RMS should be active");
    assert!(system.is_active(part_id, 1), "Peak should be active");
    assert!(system.is_active(part_id, 2), "Beat should be active");
    assert!(
        !system.is_active(part_id, 3),
        "BPM should NOT be active (reserved)"
    );
}

#[test]
fn test_fallback_behavior() {
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    // Config: EVERYTHING DISABLED
    // Fallback logic should enable Beat output at index 0
    let config = AudioTriggerOutputConfig {
        frequency_bands: false,
        volume_outputs: false,
        beat_output: false,
        bpm_output: false,
        inverted_outputs: Default::default(),
    };
    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    let audio_data = AudioTriggerData {
        beat_detected: true,
        ..AudioTriggerData::default()
    };

    system.update(&module_manager, &audio_data, 0.016);

    assert!(
        system.is_active(part_id, 0),
        "Fallback Beat Output should be active"
    );
}

#[test]
fn test_update_robustness_nan_inf() {
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    let config = AudioTriggerOutputConfig {
        frequency_bands: true,
        volume_outputs: true,
        beat_output: true,
        bpm_output: true,
        inverted_outputs: Default::default(),
    };
    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });
    // This part should have sockets for all outputs
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // Create bad audio data
    let mut audio_data = default_audio_data();
    audio_data.band_energies[0] = f32::NAN;
    audio_data.band_energies[1] = f32::INFINITY;
    audio_data.band_energies[2] = f32::NEG_INFINITY;
    audio_data.rms_volume = f32::NAN;
    audio_data.peak_volume = f32::INFINITY;
    audio_data.beat_strength = f32::NAN;

    // Update should not panic
    system.update(&module_manager, &audio_data, 0.016);

    // Verify behavior:
    // NaN > 0.5 is false. So socket 0 (SubBass) should be inactive.
    assert!(!system.is_active(part_id, 0), "NaN input should not trigger");

    // Inf > 0.5 is true. So socket 1 (Bass) should be active.
    assert!(system.is_active(part_id, 1), "Infinity input should trigger");

    // -Inf > 0.5 is false. So socket 2 (LowMid) should be inactive.
    assert!(!system.is_active(part_id, 2), "Negative Infinity input should not trigger");

    // RMS NaN -> Inactive
    // Socket index: 9 bands (0-8) -> RMS is 9
    assert!(!system.is_active(part_id, 9), "NaN RMS should not trigger");

    // Peak Inf -> Active
    // Socket index: 9 bands -> RMS(9) -> Peak(10)
    assert!(system.is_active(part_id, 10), "Inf Peak should trigger");
}

#[test]
fn test_audio_fft_inverted_output() {
    // 1. Setup
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    let mut inverted_outputs = std::collections::HashSet::new();
    inverted_outputs.insert("Bass Out".to_string());

    let config = AudioTriggerOutputConfig {
        frequency_bands: true,
        inverted_outputs,
        ..Default::default()
    };

    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // 2. Test Below Threshold
    // Should be ACTIVE because it's inverted!
    let mut audio_data = default_audio_data();
    audio_data.band_energies[1] = 0.4; // 0.4 < 0.5
    system.update(&module_manager, &audio_data, 0.016);
    assert!(
        system.is_active(part_id, 1),
        "Inverted Bass Out (socket 1) should be ACTIVE when below threshold"
    );

    // 3. Test Above Threshold
    // Should be INACTIVE because it's inverted!
    audio_data.band_energies[1] = 0.6; // 0.6 > 0.5
    system.update(&module_manager, &audio_data, 0.016);
    assert!(
        !system.is_active(part_id, 1),
        "Inverted Bass Out (socket 1) should be INACTIVE when above threshold"
    );

    // 4. Test Another Band (Non-Inverted)
    // "SubBass Out" (index 0) is not inverted
    audio_data.band_energies[0] = 0.6; // 0.6 > 0.5
    system.update(&module_manager, &audio_data, 0.016);
    assert!(
        system.is_active(part_id, 0),
        "SubBass Out (socket 0) should be ACTIVE when above threshold (normal)"
    );
    audio_data.band_energies[0] = 0.4; // 0.4 < 0.5
    system.update(&module_manager, &audio_data, 0.016);
    assert!(
        !system.is_active(part_id, 0),
        "SubBass Out (socket 0) should be INACTIVE when below threshold (normal)"
    );
}

#[test]
fn test_audio_fft_volume_inverted_output() {
    // 1. Setup
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    let mut inverted_outputs = std::collections::HashSet::new();
    inverted_outputs.insert("RMS Volume".to_string());

    let config = AudioTriggerOutputConfig {
        frequency_bands: false,
        volume_outputs: true,
        beat_output: false,
        bpm_output: false,
        inverted_outputs,
    };

    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // 2. Test Below Threshold
    // RMS (index 0) is inverted -> Should be ACTIVE
    let mut audio_data = default_audio_data();
    audio_data.rms_volume = 0.4;
    system.update(&module_manager, &audio_data, 0.016);
    assert!(
        system.is_active(part_id, 0),
        "Inverted RMS Volume should be ACTIVE when below threshold"
    );

    // 3. Test Above Threshold
    // RMS (index 0) is inverted -> Should be INACTIVE
    audio_data.rms_volume = 0.6;
    system.update(&module_manager, &audio_data, 0.016);
    assert!(
        !system.is_active(part_id, 0),
        "Inverted RMS Volume should be INACTIVE when above threshold"
    );
}
