use mapmap_core::audio_reactive::AudioTriggerData;
use mapmap_core::module::{
    AudioBand, AudioTriggerOutputConfig, ModuleManager, ModulePartType, TriggerType,
};
use mapmap_core::trigger_system::TriggerSystem;

#[test]
fn test_trigger_system_initialization() {
    let system = TriggerSystem::new();
    assert!(system.get_active_triggers().is_empty());
}

#[test]
fn test_trigger_system_update_basic() {
    // 1. Setup
    let mut system = TriggerSystem::new();
    let mut module_manager = ModuleManager::new();
    let module_id = module_manager.create_module("Test Module".to_string());
    let module = module_manager.get_module_mut(module_id).unwrap();

    // Add AudioFFT Trigger (Bass Band, Threshold 0.5)
    let config = AudioTriggerOutputConfig {
        frequency_bands: true,
        ..Default::default()
    };
    let part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });
    let part_id = module.add_part_with_type(part_type, (0.0, 0.0));

    // 2. Stimulate with High Bass Energy
    let mut audio_data = AudioTriggerData::default();
    audio_data.band_energies[1] = 0.8; // Bass is index 1

    // 3. Update
    system.update(&module_manager, &audio_data);

    // 4. Assert
    // Bass output is socket index 1 in AudioTriggerOutputConfig::generate_outputs
    assert!(
        system.is_active(part_id, 1),
        "Bass trigger (socket 1) should be active"
    );
    assert!(
        !system.is_active(part_id, 0),
        "SubBass trigger (socket 0) should NOT be active"
    );
}

#[test]
fn test_trigger_system_update_volume_beats() {
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
    let mut audio_data = AudioTriggerData::default();
    audio_data.rms_volume = 0.6; // > 0.5
    audio_data.peak_volume = 0.4; // < 0.5
    audio_data.beat_detected = true;

    // 3. Update
    system.update(&module_manager, &audio_data);

    // 4. Assert
    // Socket indices hardcoded in TriggerSystem::update:
    // 9: RMS, 10: Peak, 11: Beat
    assert!(
        system.is_active(part_id, 9),
        "RMS trigger (socket 9) should be active"
    );
    assert!(
        !system.is_active(part_id, 10),
        "Peak trigger (socket 10) should NOT be active"
    );
    assert!(
        system.is_active(part_id, 11),
        "Beat trigger (socket 11) should be active"
    );
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
    let mut audio_data = AudioTriggerData::default();
    audio_data.band_energies[1] = 0.79;
    system.update(&module_manager, &audio_data);
    assert!(!system.is_active(part_id, 1));

    // 3. Test Above Threshold
    audio_data.band_energies[1] = 0.81;
    system.update(&module_manager, &audio_data);
    assert!(system.is_active(part_id, 1));
}

#[test]
fn test_trigger_system_clearing() {
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
    let mut audio_data = AudioTriggerData::default();
    audio_data.beat_detected = true;
    system.update(&module_manager, &audio_data);
    assert!(system.is_active(part_id, 11)); // Beat is socket 11

    // 3. Deactivate (next frame)
    audio_data.beat_detected = false;
    system.update(&module_manager, &audio_data);
    assert!(!system.is_active(part_id, 11));
}
