use std::collections::HashSet;
use vorce_core::module::{MapFlowModule, ModulePartType, ModulePlaybackMode, TriggerType};
use vorce_core::module_eval::ModuleEvaluator;

#[test]
fn test_manual_trigger() {
    let mut evaluator = ModuleEvaluator::new();
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // Add a trigger node
    let t_id = module.add_part_with_type(ModulePartType::Trigger(TriggerType::Beat), (0.0, 0.0));

    // Evaluate without manual trigger
    let shared = vorce_core::module::SharedMediaState::default();
    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 0.0);

    // Fire manual trigger
    evaluator.trigger_node(t_id);
    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 1.0);

    // Verify it's cleared next frame
    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 0.0);
}

#[test]
fn test_shortcut_trigger() {
    let mut evaluator = ModuleEvaluator::new();
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // Add a shortcut trigger
    let t_id = module.add_part_with_type(
        ModulePartType::Trigger(TriggerType::Shortcut {
            key_code: "Space".to_string(),
            modifiers: 0,
        }),
        (0.0, 0.0),
    );

    let shared = vorce_core::module::SharedMediaState::default();

    // No key pressed
    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 0.0);

    // Press Space
    let mut keys = HashSet::new();
    keys.insert("Space".to_string());
    evaluator.update_keys(&keys);

    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 1.0);

    // Release Space
    keys.clear();
    evaluator.update_keys(&keys);
    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 0.0);
}

#[test]
fn test_midi_trigger() {
    let mut evaluator = ModuleEvaluator::new();
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // Add a MIDI trigger (Channel 1, Note 60)
    let t_id = module.add_part_with_type(
        ModulePartType::Trigger(TriggerType::Midi {
            device: "Any".to_string(),
            channel: 1,
            note: 60,
        }),
        (0.0, 0.0),
    );

    let shared = vorce_core::module::SharedMediaState::default();

    // No MIDI
    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 0.0);

    // Send MIDI
    let mut shared_midi = shared.clone();
    shared_midi.active_midi_events.push((1, 60, 127));
    let res = evaluator.evaluate(&module, &shared_midi, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 1.0);

    // Verify cleared
    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 0.0);
}

#[test]
fn test_osc_trigger() {
    let mut evaluator = ModuleEvaluator::new();
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // Add an OSC trigger
    let t_id = module.add_part_with_type(
        ModulePartType::Trigger(TriggerType::Osc {
            address: "/trigger/1".to_string(),
        }),
        (0.0, 0.0),
    );

    let mut shared = vorce_core::module::SharedMediaState::default();

    // No OSC
    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 0.0);

    // Send OSC
    shared
        .active_osc_messages
        .insert("/trigger/1".to_string(), vec![1.0]);
    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 1.0);

    // Verify cleared
    shared.active_osc_messages.clear();
    let res = evaluator.evaluate(&module, &shared, 0);
    let val = res
        .trigger_values
        .get(&t_id)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(0.0);
    assert_eq!(val, 0.0);
}
