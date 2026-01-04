use mapmap_core::module::{
    AudioTriggerOutputConfig, MapFlowModule, ModulePartType, ModulePlaybackMode, PartType,
    TriggerType,
};

#[test]
fn test_module_part_addition() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test Module".to_string(),
        color: [1.0, 1.0, 1.0, 1.0],
        parts: Vec::new(),
        connections: Vec::new(),
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    let part_id_1 = module.add_part(PartType::Trigger, (100.0, 100.0));
    let part_id_2 = module.add_part(PartType::Source, (300.0, 100.0));

    assert_eq!(module.parts.len(), 2);
    assert_ne!(part_id_1, part_id_2);

    let part1 = module.parts.iter().find(|p| p.id == part_id_1).unwrap();
    assert!(matches!(part1.part_type, ModulePartType::Trigger(_)));
    assert_eq!(part1.position, (100.0, 100.0));
}

#[test]
fn test_socket_generation() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test Module".to_string(),
        color: [1.0, 1.0, 1.0, 1.0],
        parts: Vec::new(),
        connections: Vec::new(),
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    // Add a Trigger (Source)
    let trigger_id = module.add_part(PartType::Trigger, (0.0, 0.0));
    let trigger_part = module.parts.iter().find(|p| p.id == trigger_id).unwrap();

    // Triggers should have 0 inputs and at least 1 output
    assert_eq!(trigger_part.inputs.len(), 0);
    assert!(trigger_part.outputs.len() >= 1);

    // Add a Layer (Sink/Pass-through)
    let layer_id = module.add_part(PartType::Layer, (0.0, 0.0));
    let layer_part = module.parts.iter().find(|p| p.id == layer_id).unwrap();

    // Layers typically have Input and Output
    assert!(layer_part.inputs.len() >= 1);
    assert!(layer_part.outputs.len() >= 1);
}

#[test]
fn test_connection_management() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test Module".to_string(),
        color: [1.0, 1.0, 1.0, 1.0],
        parts: Vec::new(),
        connections: Vec::new(),
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    let p1 = module.add_part(PartType::Trigger, (0.0, 0.0));
    let p2 = module.add_part(PartType::Layer, (100.0, 0.0));

    module.add_connection(p1, 0, p2, 0);
    assert_eq!(module.connections.len(), 1);

    let conn = &module.connections[0];
    assert_eq!(conn.from_part, p1);
    assert_eq!(conn.to_part, p2);

    module.remove_connection(p1, 0, p2, 0);
    assert_eq!(module.connections.len(), 0);
}

#[test]
fn test_audio_trigger_socket_update() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test Module".to_string(),
        color: [1.0, 1.0, 1.0, 1.0],
        parts: Vec::new(),
        connections: Vec::new(),
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    let p_id = module.add_part(PartType::Trigger, (0.0, 0.0));

    // Default should have some outputs
    {
        let part = module.parts.iter().find(|p| p.id == p_id).unwrap();
        // By default, only Beat Out is enabled for TriggerType::Beat (which add_part creates)
        // Check what kind of trigger it is first
        if let ModulePartType::Trigger(TriggerType::Beat) = part.part_type {
            assert!(part.outputs.iter().any(|s| s.name == "Trigger Out"));
        }
    }

    // Update to AudioFFT type to test dynamic sockets
    if let Some(part) = module.parts.iter_mut().find(|p| p.id == p_id) {
        part.part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
            band: mapmap_core::module::AudioBand::Bass,
            threshold: 0.5,
            output_config: AudioTriggerOutputConfig {
                frequency_bands: true,
                ..Default::default()
            }
        });
    }

    // Trigger update
    module.update_part_sockets(p_id);

    // Check results
    let part = module.parts.iter().find(|p| p.id == p_id).unwrap();
    // 9 frequency bands + default enabled outputs (Beat Out is default true in OutputConfig)
    assert!(part.outputs.len() >= 9);
    assert!(part.outputs.iter().any(|s| s.name == "SubBass Out"));
}
