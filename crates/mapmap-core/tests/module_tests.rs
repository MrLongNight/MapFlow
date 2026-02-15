use mapmap_core::module::{
    HueMappingMode, MapFlowModule, ModuleManager, ModulePartType, ModulePlaybackMode,
    ModuleSocketType, OutputType, PartType, SourceType, TriggerType,
};
use std::collections::HashMap;

#[test]
fn test_create_module() {
    let mut manager = ModuleManager::new();
    let id = manager.create_module("Test Module".to_string());
    assert_eq!(id, 1);
    let modules = manager.list_modules();
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "Test Module");
}

#[test]
fn test_delete_module() {
    let mut manager = ModuleManager::new();
    let id = manager.create_module("Test Module".to_string());
    manager.delete_module(id);
    assert!(manager.list_modules().is_empty());
}

#[test]
fn test_set_module_color() {
    let mut manager = ModuleManager::new();
    let id = manager.create_module("Test Module".to_string());
    let new_color = [0.1, 0.2, 0.3, 1.0];
    manager.set_module_color(id, new_color);
    let modules = manager.list_modules();
    assert_eq!(modules[0].color, new_color);
}

#[test]
fn test_module_color_rotation() {
    let mut manager = ModuleManager::new();
    let id1 = manager.create_module("Module 1".to_string());
    let id2 = manager.create_module("Module 2".to_string());
    let modules1 = manager
        .list_modules()
        .iter()
        .find(|m| m.id == id1)
        .unwrap()
        .color;
    let modules2 = manager
        .list_modules()
        .iter()
        .find(|m| m.id == id2)
        .unwrap()
        .color;
    assert_ne!(modules1, modules2);
}

#[test]
fn test_socket_generation_coverage() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // 1. Trigger (Beat)
    let pid_trigger = module.add_part(PartType::Trigger, (0.0, 0.0));
    let part_trigger = module.parts.iter().find(|p| p.id == pid_trigger).unwrap();
    // Should have 1 Trigger Out
    assert!(part_trigger
        .outputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Trigger));
    assert!(part_trigger.inputs.is_empty());

    // 2. Source (Media File)
    let pid_source = module.add_part(PartType::Source, (100.0, 0.0));
    let part_source = module.parts.iter().find(|p| p.id == pid_source).unwrap();
    // 1 Trigger In, 1 Media Out
    assert!(part_source
        .inputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Trigger));
    assert!(part_source
        .outputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Media));

    // 3. Layer
    let pid_layer = module.add_part(PartType::Layer, (200.0, 0.0));
    let part_layer = module.parts.iter().find(|p| p.id == pid_layer).unwrap();
    // Input: Media In, Trigger In. Output: Layer Out
    assert!(part_layer
        .inputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Media));
    assert!(part_layer
        .inputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Trigger));
    assert!(part_layer
        .outputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Layer));

    // 4. Output (Projector)
    let pid_output = module.add_part(PartType::Output, (300.0, 0.0));
    let part_output = module.parts.iter().find(|p| p.id == pid_output).unwrap();
    // Input: Layer In. No Output.
    assert!(part_output
        .inputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Layer));
    assert!(part_output.outputs.is_empty());

    // 5. Hue Output
    let hue_output = ModulePartType::Output(OutputType::Hue {
        bridge_ip: "127.0.0.1".into(),
        username: "test".into(),
        client_key: "key".into(),
        entertainment_area: "area".into(),
        lamp_positions: HashMap::new(),
        mapping_mode: HueMappingMode::Ambient,
    });
    let pid_hue = module.add_part_with_type(hue_output, (400.0, 0.0));
    let part_hue = module.parts.iter().find(|p| p.id == pid_hue).unwrap();
    // Input: Layer In, Trigger In. Output: None
    assert_eq!(part_hue.inputs.len(), 2);
    assert!(part_hue.outputs.is_empty());

    // 6. Mask
    let pid_mask = module.add_part(PartType::Mask, (500.0, 0.0));
    let part_mask = module.parts.iter().find(|p| p.id == pid_mask).unwrap();
    // Inputs: Media In, Mask In. Output: Media Out
    assert_eq!(part_mask.inputs.len(), 2);
    assert_eq!(part_mask.outputs.len(), 1);

    // 7. Modulizer
    let pid_mod = module.add_part(PartType::Modulator, (600.0, 0.0));
    let part_mod = module.parts.iter().find(|p| p.id == pid_mod).unwrap();
    // Inputs: Media In, Trigger In. Output: Media Out
    assert_eq!(part_mod.inputs.len(), 2);
    assert_eq!(part_mod.outputs.len(), 1);

    // 8. Mesh
    let pid_mesh = module.add_part(PartType::Mesh, (700.0, 0.0));
    let part_mesh = module.parts.iter().find(|p| p.id == pid_mesh).unwrap();
    // Inputs: Vertex In, Control In. Output: Geometry Out
    assert_eq!(part_mesh.inputs.len(), 2);
    assert_eq!(part_mesh.outputs.len(), 1);
}

#[test]
fn test_comprehensive_source_sockets() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // Helper to add part and check socket counts
    let check_source = |module: &mut MapFlowModule, source_type: SourceType, expected_ins: usize, expected_outs: usize| {
        let part_type = ModulePartType::Source(source_type);
        let pid = module.add_part_with_type(part_type, (0.0, 0.0));
        let part = module.parts.iter().find(|p| p.id == pid).unwrap();
        assert_eq!(part.inputs.len(), expected_ins, "Inputs mismatch");
        assert_eq!(part.outputs.len(), expected_outs, "Outputs mismatch");
    };

    // 1. MediaFile
    check_source(&mut module, SourceType::new_media_file("test.mp4".into()), 1, 1);

    // 2. Shader
    check_source(
        &mut module,
        SourceType::Shader {
            name: "Test".into(),
            params: vec![],
        },
        1,
        1,
    );

    // 3. LiveInput
    check_source(&mut module, SourceType::LiveInput { device_id: 0 }, 1, 1);

    // 4. NdiInput
    check_source(&mut module, SourceType::NdiInput { source_name: None }, 1, 1);

    // 5. BevyParticles (Has Spawn Trigger + Media Out = 1 In, 1 Out)
    check_source(
        &mut module,
        SourceType::BevyParticles {
            rate: 10.0,
            lifetime: 1.0,
            speed: 1.0,
            color_start: [1.0; 4],
            color_end: [1.0; 4],
            position: [0.0; 3],
            rotation: [0.0; 3],
        },
        1,
        1,
    );

    // Check specific name for BevyParticles input
    let last_part = module.parts.last().unwrap();
    assert_eq!(last_part.inputs[0].name, "Spawn Trigger");

    // 6. BevyAtmosphere
    check_source(
        &mut module,
        SourceType::BevyAtmosphere {
            turbidity: 1.0,
            rayleigh: 1.0,
            mie_coeff: 1.0,
            mie_directional_g: 1.0,
            sun_position: (0.0, 0.0),
            exposure: 1.0,
        },
        1,
        1,
    );

    // 7. BevyHexGrid
    check_source(
        &mut module,
        SourceType::BevyHexGrid {
            radius: 1.0,
            rings: 1,
            pointy_top: true,
            spacing: 1.0,
            position: [0.0; 3],
            rotation: [0.0; 3],
            scale: 1.0,
        },
        1,
        1,
    );

    // 8. Bevy3DText
    check_source(
        &mut module,
        SourceType::Bevy3DText {
            text: "Hello".into(),
            font_size: 10.0,
            color: [1.0; 4],
            position: [0.0; 3],
            rotation: [0.0; 3],
            alignment: "Center".into(),
        },
        1,
        1,
    );
}

#[test]
fn test_comprehensive_trigger_sockets() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // 1. AudioFFT (tested extensively in trigger_system_tests, but check basic)
    let fft = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: mapmap_core::module::AudioBand::Bass,
        threshold: 0.5,
        output_config: Default::default(), // Defaults to Beat Out
    });
    let pid = module.add_part_with_type(fft, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == pid).unwrap();
    assert_eq!(part.outputs.len(), 1);
    assert_eq!(part.outputs[0].name, "Beat Out");

    // 2. Random
    let rnd = ModulePartType::Trigger(TriggerType::Random {
        min_interval_ms: 100,
        max_interval_ms: 1000,
        probability: 0.5,
    });
    let pid = module.add_part_with_type(rnd, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == pid).unwrap();
    assert_eq!(part.outputs.len(), 1);
    assert_eq!(part.outputs[0].name, "Trigger Out");

    // 3. Midi
    let midi = ModulePartType::Trigger(TriggerType::Midi {
        device: "Nano".into(),
        channel: 1,
        note: 60,
    });
    let pid = module.add_part_with_type(midi, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == pid).unwrap();
    assert_eq!(part.outputs.len(), 1);
}
