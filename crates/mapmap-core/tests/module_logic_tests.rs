use mapmap_core::module::{LinkMode, MapFlowModule, ModulePartType, ModulePlaybackMode, PartType};

#[test]
fn test_add_part_creates_correct_sockets() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test Module".to_string(),
        color: [1.0; 4],
        parts: Vec::new(),
        connections: Vec::new(),
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    // Test Source Part (Trigger In, Media Out)
    let source_id = module.add_part(PartType::Source, (0.0, 0.0));
    let source = module.parts.iter().find(|p| p.id == source_id).unwrap();
    assert_eq!(source.inputs.len(), 1);
    assert_eq!(source.inputs[0].name, "Trigger In");
    assert_eq!(source.outputs.len(), 1);
    assert_eq!(source.outputs[0].name, "Media Out");

    // Test Output Part (Layer In, No Outputs)
    let output_id = module.add_part(PartType::Output, (100.0, 0.0));
    let output = module.parts.iter().find(|p| p.id == output_id).unwrap();
    assert_eq!(output.inputs.len(), 1);
    assert_eq!(output.inputs[0].name, "Layer In");
    assert!(output.outputs.is_empty());
}

#[test]
fn test_connection_management() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test Module".to_string(),
        color: [1.0; 4],
        parts: Vec::new(),
        connections: Vec::new(),
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    let p1 = module.add_part(PartType::Source, (0.0, 0.0));
    let p2 = module.add_part(PartType::Output, (100.0, 0.0));

    // Add connection
    module.add_connection(p1, 0, p2, 0);
    assert_eq!(module.connections.len(), 1);
    assert_eq!(module.connections[0].from_part, p1);
    assert_eq!(module.connections[0].to_part, p2);

    // Remove connection
    module.remove_connection(p1, 0, p2, 0);
    assert!(module.connections.is_empty());
}

#[test]
fn test_socket_update_removes_invalid_connections() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test Module".to_string(),
        color: [1.0; 4],
        parts: Vec::new(),
        connections: Vec::new(),
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    let p1 = module.add_part(PartType::Source, (0.0, 0.0)); // 1 output
    let p2 = module.add_part(PartType::Output, (100.0, 0.0)); // 1 input

    module.add_connection(p1, 0, p2, 0);
    assert_eq!(module.connections.len(), 1);

    // Manually modify p1 type to Trigger (0 inputs, X outputs) via a simulated change
    // Since add_part returns a new ID, we have to cheat a bit to simulate an "edit"
    // or we can just modify the part in place if we can access it mutably.
    // However, update_part_sockets uses the *current* configuration.
    // Let's modify the part's link_data to add a socket, connect it, then remove it.

    if let Some(part) = module.parts.iter_mut().find(|p| p.id == p1) {
        part.link_data.mode = LinkMode::Master; // Should add "Link Out" at index 1 (since Source has 1 output already at 0)
    }
    module.update_part_sockets(p1);

    // Verify Link Out socket appeared
    let part = module.parts.iter().find(|p| p.id == p1).unwrap();
    assert_eq!(part.outputs.len(), 2);
    assert_eq!(part.outputs[1].name, "Link Out");

    // Connect to this new socket
    module.add_connection(p1, 1, p2, 0);
    assert_eq!(module.connections.len(), 2);

    // Now turn off Master mode
    if let Some(part) = module.parts.iter_mut().find(|p| p.id == p1) {
        part.link_data.mode = LinkMode::Off;
    }
    module.update_part_sockets(p1);

    // Verify socket is gone and connection is removed
    let part = module.parts.iter().find(|p| p.id == p1).unwrap();
    assert_eq!(part.outputs.len(), 1);
    assert_eq!(module.connections.len(), 1); // Only original connection remains
    assert_eq!(module.connections[0].from_socket, 0);
}

#[test]
fn test_link_system_sockets() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Link Test".to_string(),
        color: [1.0; 4],
        parts: Vec::new(),
        connections: Vec::new(),
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    let p1 = module.add_part(PartType::Source, (0.0, 0.0));

    // Enable Master Mode
    if let Some(part) = module.parts.iter_mut().find(|p| p.id == p1) {
        part.link_data.mode = LinkMode::Master;
        part.link_data.trigger_input_enabled = true;
    }
    module.update_part_sockets(p1);

    let part = module.parts.iter().find(|p| p.id == p1).unwrap();
    // Source default inputs: 1 (Trigger In)
    // Master mode + trigger enabled: +1 Trigger In (Vis)
    // Source default outputs: 1 (Media Out)
    // Master mode: +1 Link Out

    // Actually, Source default inputs is 1.
    // Link In (Slave) adds one.
    // Trigger In (Vis) adds one.
    // So inputs should be 2.

    assert!(part.inputs.iter().any(|s| s.name == "Trigger In (Vis)"));
    assert!(part.outputs.iter().any(|s| s.name == "Link Out"));
}

#[test]
fn test_audio_trigger_sockets() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Audio Test".to_string(),
        color: [1.0; 4],
        parts: Vec::new(),
        connections: Vec::new(),
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    // Add Trigger Part (default is Beat)
    let p1 = module.add_part(PartType::Trigger, (0.0, 0.0));

    // Default Beat trigger has 1 output
    let part = module.parts.iter().find(|p| p.id == p1).unwrap();
    assert_eq!(part.outputs.len(), 1);
    assert_eq!(part.outputs[0].name, "Trigger Out");

    // Change to AudioFFT with frequency bands
    if let Some(part) = module.parts.iter_mut().find(|p| p.id == p1) {
        if let ModulePartType::Trigger(ref mut t_type) = part.part_type {
            *t_type = mapmap_core::module::TriggerType::AudioFFT {
                band: mapmap_core::module::AudioBand::Bass,
                threshold: 0.5,
                output_config: mapmap_core::module::AudioTriggerOutputConfig {
                    frequency_bands: true,
                    ..Default::default()
                },
            };
        }
    }
    module.update_part_sockets(p1);

    let part = module.parts.iter().find(|p| p.id == p1).unwrap();
    // 9 frequency bands + 1 beat (default fallback if others off, but we set freq bands true)
    // Wait, let's check generate_outputs logic:
    // if frequency_bands: push 9
    // if outputs is empty: push beat
    // Here outputs won't be empty.
    // But default beat_output is true in Default impl!
    // So it should be 9 + 1 = 10 outputs.

    assert_eq!(part.outputs.len(), 10);
    assert_eq!(part.outputs[0].name, "SubBass Out");
}
