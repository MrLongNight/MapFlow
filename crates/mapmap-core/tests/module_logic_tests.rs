use mapmap_core::module::{
    AudioTriggerOutputConfig, LinkMode, MapFlowModule, ModulePartType, ModulePlaybackMode,
    ModuleSocketType, PartType,
};

#[test]
fn test_module_creation_defaults() {
    let module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    assert_eq!(module.parts.len(), 0);
    assert_eq!(module.connections.len(), 0);
}

#[test]
fn test_add_part_creates_sockets() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    // Add a Source part
    let source_id = module.add_part(PartType::Source, (0.0, 0.0));
    let source_part = module.parts.iter().find(|p| p.id == source_id).unwrap();

    // Check Source sockets: Trigger In, Media Out
    assert_eq!(source_part.inputs.len(), 1);
    assert_eq!(source_part.inputs[0].socket_type, ModuleSocketType::Trigger);
    assert_eq!(source_part.outputs.len(), 1);
    assert_eq!(source_part.outputs[0].socket_type, ModuleSocketType::Media);

    // Add a Layer part
    let layer_id = module.add_part(PartType::Layer, (100.0, 0.0));
    let layer_part = module.parts.iter().find(|p| p.id == layer_id).unwrap();

    // Check Layer sockets: Input (Media), Output (Layer)
    assert_eq!(layer_part.inputs.len(), 1);
    assert_eq!(layer_part.inputs[0].socket_type, ModuleSocketType::Media);
    assert_eq!(layer_part.outputs.len(), 1);
    assert_eq!(layer_part.outputs[0].socket_type, ModuleSocketType::Layer);
}

#[test]
fn test_connection_management() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    let p1 = module.add_part(PartType::Source, (0.0, 0.0));
    let p2 = module.add_part(PartType::Layer, (100.0, 0.0));

    // Add connection
    module.add_connection(p1, 0, p2, 0);
    assert_eq!(module.connections.len(), 1);

    let conn = &module.connections[0];
    assert_eq!(conn.from_part, p1);
    assert_eq!(conn.to_part, p2);

    // Remove connection
    module.remove_connection(p1, 0, p2, 0);
    assert_eq!(module.connections.len(), 0);
}

#[test]
fn test_link_mode_sockets() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    let pid = module.add_part(PartType::Source, (0.0, 0.0));

    // Default: Off
    {
        let part = module.parts.iter().find(|p| p.id == pid).unwrap();
        assert_eq!(part.link_data.mode, LinkMode::Off);
        // Source has 1 output by default
        assert!(!part
            .outputs
            .iter()
            .any(|s| s.socket_type == ModuleSocketType::Link));
    }

    // Set to Master
    {
        let part = module.parts.iter_mut().find(|p| p.id == pid).unwrap();
        part.link_data.mode = LinkMode::Master;
        module.update_part_sockets(pid);

        let part = module.parts.iter().find(|p| p.id == pid).unwrap();
        // Should now have a Link Out socket
        assert!(part
            .outputs
            .iter()
            .any(|s| s.name == "Link Out" && s.socket_type == ModuleSocketType::Link));
    }

    // Set to Slave
    {
        let part = module.parts.iter_mut().find(|p| p.id == pid).unwrap();
        part.link_data.mode = LinkMode::Slave;
        module.update_part_sockets(pid);

        let part = module.parts.iter().find(|p| p.id == pid).unwrap();
        // Should now have a Link In socket
        assert!(part
            .inputs
            .iter()
            .any(|s| s.name == "Link In" && s.socket_type == ModuleSocketType::Link));
    }
}

#[test]
fn test_audio_trigger_outputs() {
    let config = AudioTriggerOutputConfig {
        frequency_bands: true,
        volume_outputs: true,
        beat_output: true,
        bpm_output: true,
        inverted_outputs: Default::default(),
    };

    let outputs = config.generate_outputs();

    // 9 bands + 2 volumes + 1 beat + 1 bpm = 13 outputs
    assert_eq!(outputs.len(), 13);

    let types: Vec<_> = outputs.iter().map(|s| s.socket_type).collect();
    assert!(types.iter().all(|&t| t == ModuleSocketType::Trigger));
}

#[test]
fn test_update_part_sockets_cleans_connections() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
    };

    let p1 = module.add_part(PartType::Source, (0.0, 0.0));
    let p2 = module.add_part(PartType::Layer, (100.0, 0.0));

    // Connect Source Output 0 -> Layer Input 0
    module.add_connection(p1, 0, p2, 0);
    assert_eq!(module.connections.len(), 1);

    // Verify socket count before change (Source has 1 output)
    {
        let part = module.parts.iter().find(|p| p.id == p1).unwrap();
        assert_eq!(part.outputs.len(), 1);
    }

    // Change p1 to a type that has NO outputs (e.g. Output type acts as sink, though PartType::Output has 0 outputs)
    // Actually, let's use a trick: Manually change the PartType to something with 0 outputs
    // But we can only use `add_part_with_type` or modify the internal field.
    // Let's modify the field directly since we have access in the test.

    // Let's assume we change p1 to an Output type which has 0 outputs.
    {
        let part = module.parts.iter_mut().find(|p| p.id == p1).unwrap();
        // Output type has 0 outputs
        part.part_type = ModulePartType::Output(mapmap_core::module::OutputType::Projector {
            id: 0,
            name: "Out".into(),
            fullscreen: false,
            hide_cursor: true,
            target_screen: 0,
            show_in_preview_panel: true,
            extra_preview_window: false,
        });
        // We must call update_part_sockets to refresh inputs/outputs
    }

    module.update_part_sockets(p1);

    // Now p1 should have 0 outputs.
    // The connection from p1 output 0 should be removed.
    assert_eq!(module.connections.len(), 0);
}
