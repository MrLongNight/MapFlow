use vorce_core::audio::analyzer_v2::AudioAnalysisV2;
use vorce_core::module::{
    AudioBand, AudioTriggerOutputConfig, MapFlowModule, ModulePartType, ModulePlaybackMode,
    PartType, TriggerConfig, TriggerMappingMode, TriggerTarget, TriggerType,
};
use vorce_core::module_eval::ModuleEvaluator;

#[test]
fn test_trigger_inversion_logic() {
    let mut evaluator = ModuleEvaluator::new();

    let analysis = AudioAnalysisV2 {
        beat_detected: true,
        rms_volume: 0.8,
        ..Default::default()
    };
    evaluator.update_audio(&analysis);

    let mut module = MapFlowModule {
        id: 1,
        name: "Inversion Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    let mut config = AudioTriggerOutputConfig {
        beat_output: true,
        volume_outputs: false,
        ..Default::default()
    };
    config.inverted_outputs.insert("Beat Out".to_string());

    let trigger_type = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: config,
    });

    let t_id = module.add_part_with_type(trigger_type, (0.0, 0.0));

    let shared = vorce_core::module::SharedMediaState::default();
    let result = evaluator.evaluate(&module, &shared, 0);

    let values = &result.trigger_values[&t_id];
    assert_eq!(values[0], 0.0, "Beat output should be inverted to 0.0");

    let analysis_no_beat = AudioAnalysisV2 {
        beat_detected: false,
        ..Default::default()
    };
    evaluator.update_audio(&analysis_no_beat);
    let result_no_beat = evaluator.evaluate(&module, &shared, 0);
    let values_no_beat = &result_no_beat.trigger_values[&t_id];
    assert_eq!(values_no_beat[0], 1.0, "No beat should be inverted to 1.0");
}

#[test]
fn test_trigger_target_range_mapping() {
    let mut evaluator = ModuleEvaluator::new();
    let mut module = MapFlowModule {
        id: 2,
        name: "Range Mapping Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // 1. Constant Trigger (1.0)
    let t_id = module.add_part_with_type(
        ModulePartType::Trigger(TriggerType::Fixed {
            interval_ms: 0,
            offset_ms: 0,
        }),
        (0.0, 0.0),
    );

    // 2. Source Node
    let s_id = module.add_part(PartType::Source, (200.0, 0.0));

    // 3. Layer Node
    let l_id = module.add_part(PartType::Layer, (400.0, 0.0));

    // 4. Output Node
    let o_id = module.add_part(PartType::Output, (600.0, 0.0));

    // CONNECTIONS:
    // Trigger -> Source Trigger In (Socket 0)
    module.add_connection(
        t_id,
        "trigger_out".to_string(),
        s_id,
        "trigger_in".to_string(),
    );
    // Source -> Layer Input (Socket 0)
    module.add_connection(s_id, "media_out".to_string(), l_id, "media_in".to_string());
    // Layer -> Output Input (Socket 0)
    module.add_connection(l_id, "layer_out".to_string(), o_id, "layer_in".to_string());

    // Configure Trigger Mapping on Source: Trigger In -> Brightness (Range 0.5 to 0.8)
    if let Some(part) = module.parts.iter_mut().find(|p| p.id == s_id) {
        part.trigger_targets.insert(
            0,
            TriggerConfig {
                target: TriggerTarget::Brightness,
                mode: TriggerMappingMode::Direct,
                min_value: 0.5,
                max_value: 0.8,
                ..Default::default()
            },
        );
    }

    let shared = vorce_core::module::SharedMediaState::default();
    let result = evaluator.evaluate(&module, &shared, 0);

    // Verify render op
    let mut found = false;
    for op in &result.render_ops {
        if op.source_part_id == Some(s_id) {
            assert!(
                (op.source_props.brightness - 0.8).abs() < 0.001,
                "Brightness {} should be 0.8",
                op.source_props.brightness
            );
            found = true;
        }
    }
    assert!(found, "Should have found a render op for the source");
}
