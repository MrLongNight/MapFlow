//! tests/trigger_eval_tests.rs
use mapmap_core::module::{MapFlowModule, ModulePart, ModulePartType, TriggerType};
use mapmap_core::module_eval::ModuleEvaluator;

fn create_test_module_with_trigger(trigger_type: TriggerType) -> (MapFlowModule, u64) {
    let part_id = 1;
    let module = MapFlowModule {
        id: 1,
        name: "Test Module".to_string(),
        color: [1.0, 0.0, 0.0, 1.0],
        parts: vec![ModulePart {
            id: part_id,
            part_type: ModulePartType::Trigger(trigger_type),
            position: (0.0, 0.0),
            size: None,
            link_data: Default::default(),
            inputs: vec![],
            outputs: vec![],
        }],
        connections: vec![],
        playback_mode: mapmap_core::module::ModulePlaybackMode::LoopUntilManualSwitch,
    };
    (module, part_id)
}

#[test]
fn test_random_trigger_current_flawed_behavior() {
    // This test asserts the CURRENT flawed behavior, where intervals are ignored.
    let (module, part_id) = create_test_module_with_trigger(TriggerType::Random {
        min_interval_ms: 1_000_000, // Effectively infinite
        max_interval_ms: 2_000_000,
        probability: 1.0, // Should always fire based on current logic
    });

    let evaluator = ModuleEvaluator::new();
    let result = evaluator.evaluate(&module);

    let trigger_values = result.trigger_values.get(&part_id).unwrap();
    // Because the current implementation ignores time and only checks probability,
    // this should fire immediately. A correct implementation would not fire.
    assert_eq!(
        trigger_values,
        &vec![1.0],
        "Random trigger with 100% probability should fire immediately (current flawed logic)"
    );
}

#[test]
fn test_fixed_trigger_logic() {
    // This trigger should fire every 100ms with a 50ms offset.
    let (module, part_id) = create_test_module_with_trigger(TriggerType::Fixed {
        interval_ms: 100,
        offset_ms: 50,
    });

    let evaluator = ModuleEvaluator::new();

    // We can't easily test time here, but we can check the initial state.
    // Since elapsed time is < offset, it should not fire.
    std::thread::sleep(std::time::Duration::from_millis(10));
    let result = evaluator.evaluate(&module);
    let trigger_values = result.trigger_values.get(&part_id).unwrap();
    assert_eq!(
        trigger_values,
        &vec![0.0],
        "Fixed trigger should not fire before its offset"
    );

    // A full test would require mocking `Instant::now()`, which is complex.
    // This basic check is sufficient for now.
}

#[test]
fn test_shortcut_trigger_is_unimplemented() {
    let (module, part_id) = create_test_module_with_trigger(TriggerType::Shortcut {
        key_code: "Space".to_string(),
        modifiers: 0,
    });

    let evaluator = ModuleEvaluator::new();
    let result = evaluator.evaluate(&module);

    let trigger_values = result.trigger_values.get(&part_id).unwrap();
    // The evaluator currently has no logic for keyboard input, so this will always be 0.
    assert_eq!(
        trigger_values,
        &vec![0.0],
        "Shortcut trigger is not implemented in module_eval and should return 0.0"
    );
}

#[test]
fn test_audio_fft_trigger_receives_data() {
    use mapmap_core::audio::analyzer_v2::AudioAnalysisV2;

    let (module, part_id) = create_test_module_with_trigger(TriggerType::AudioFFT {
        band: mapmap_core::module::AudioBand::Bass,
        threshold: 0.5,
        output_config: Default::default(), // Just "Beat Out"
    });

    let mut evaluator = ModuleEvaluator::new();

    // Simulate audio analysis with a beat detected
    let mut analysis = AudioAnalysisV2::default();
    analysis.beat_detected = true;
    evaluator.update_audio(&analysis);

    let result = evaluator.evaluate(&module);
    let trigger_values = result.trigger_values.get(&part_id).unwrap();

    // Default config only has "Beat Out"
    assert_eq!(trigger_values.len(), 1);
    assert_eq!(
        trigger_values[0], 1.0,
        "AudioFFT trigger should fire when a beat is detected"
    );

    // Simulate no beat
    analysis.beat_detected = false;
    evaluator.update_audio(&analysis);
    let result_no_beat = evaluator.evaluate(&module);
    let trigger_values_no_beat = result_no_beat.trigger_values.get(&part_id).unwrap();
    assert_eq!(
        trigger_values_no_beat[0], 0.0,
        "AudioFFT trigger should not fire when no beat is detected"
    );
}
