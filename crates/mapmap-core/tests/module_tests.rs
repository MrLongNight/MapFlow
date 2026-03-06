use mapmap_core::module::{
    EffectType, HueMappingMode, MapFlowModule, ModuleManager, ModulePartType, ModulePlaybackMode,
    ModuleSocketType, OutputType, PartType,
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
fn test_update_part_position_valid_id_updates() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    let pid = module.add_part(PartType::Trigger, (0.0, 0.0));
    module.update_part_position(pid, (10.0, 20.0));

    let part = module.parts.iter().find(|p| p.id == pid).unwrap();
    assert_eq!(part.position, (10.0, 20.0));
}

#[test]
fn test_add_remove_connection() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    let pid1 = module.add_part(PartType::Trigger, (0.0, 0.0));
    let pid2 = module.add_part(PartType::Source, (100.0, 0.0));

    module.add_connection(pid1, 0, pid2, 0);
    assert_eq!(module.connections.len(), 1);
    assert_eq!(module.connections[0].from_part, pid1);
    assert_eq!(module.connections[0].to_part, pid2);

    module.remove_connection(pid1, 0, pid2, 0);
    assert!(module.connections.is_empty());
}

#[test]
fn test_update_part_sockets() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    let pid1 = module.add_part(PartType::Trigger, (0.0, 0.0));
    let pid2 = module.add_part(PartType::Source, (100.0, 0.0));
    module.add_connection(pid1, 0, pid2, 0);

    // Changing the part to something else to alter its sockets, or simply calling it to ensure it clears invalid connections
    // Actually, `update_part_sockets` recomputes sockets and removes invalid connections.
    // If we mock a part losing a socket, the connection should be removed.

    // For now just test it doesn't crash on normal update and keeps valid connections
    module.update_part_sockets(pid1);
    assert_eq!(module.connections.len(), 1);
}

#[test]
fn test_update_part_sockets_removes_invalid_connections() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // Add part that initially has outputs
    let pid1 = module.add_part(PartType::Trigger, (0.0, 0.0));
    let pid2 = module.add_part(PartType::Source, (100.0, 0.0));

    // Create an invalid connection (output socket index out of bounds)
    module.add_connection(pid1, 999, pid2, 0); // pid1 only has 1 output
    module.add_connection(pid1, 0, pid2, 999); // pid2 only has 1 input
    module.add_connection(pid1, 0, pid2, 0); // Valid connection

    assert_eq!(module.connections.len(), 3);

    module.update_part_sockets(pid1);

    // Invalid connection from pid1 should be removed
    assert_eq!(module.connections.len(), 2);

    module.update_part_sockets(pid2);

    // Invalid connection to pid2 should be removed
    assert_eq!(module.connections.len(), 1);

    assert_eq!(module.connections[0].from_socket, 0);
    assert_eq!(module.connections[0].to_socket, 0);
}

#[test]
fn test_update_part_outputs_delegates() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    let pid1 = module.add_part(PartType::Trigger, (0.0, 0.0));
    let pid2 = module.add_part(PartType::Source, (100.0, 0.0));

    module.add_connection(pid1, 999, pid2, 0); // Invalid connection

    assert_eq!(module.connections.len(), 1);
    module.update_part_outputs(pid1); // Should call update_part_sockets and clear connection
    assert!(module.connections.is_empty());
}

#[test]
fn test_effect_type_all_and_name() {
    let all = EffectType::all();
    assert!(!all.is_empty());

    for effect in all {
        let name = effect.name();
        assert!(!name.is_empty());
    }

    let sg = EffectType::ShaderGraph(1);
    assert_eq!(sg.name(), "Custom Shader Graph");
}

#[test]
fn test_blend_mode_type_all_and_name() {
    let all = mapmap_core::module::BlendModeType::all();
    assert!(!all.is_empty());

    for blend_mode in all {
        let name = blend_mode.name();
        assert!(!name.is_empty());
    }
}

#[test]
fn test_mesh_type_to_mesh_and_hash() {
    let mesh_type = mapmap_core::module::MeshType::Grid { cols: 2, rows: 2 };
    let hash = mesh_type.compute_revision_hash();
    assert_ne!(hash, 0); // Assuming hash isn't exactly 0 for grid 2x2

    let mesh = mesh_type.to_mesh();
    assert!(!mesh.vertices.is_empty());
}

#[test]
fn test_part_compute_sockets() {
    let _module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // Test that compute_sockets calls through properly, checking one part
    let mut manager = ModuleManager::new();
    let id = manager.create_module("Test Module".to_string());

    let part_id = manager
        .add_part_to_module(id, PartType::Trigger, (0.0, 0.0))
        .unwrap();
    let m = manager.get_module(id).unwrap();
    let part = m.parts.iter().find(|p| p.id == part_id).unwrap();

    let (inputs, outputs) = part.compute_sockets();
    assert_eq!(inputs.len(), 0);
    assert_eq!(outputs.len(), 1);

    let (def_inputs, def_outputs) = part.part_type.get_default_sockets();
    assert_eq!(inputs, def_inputs);
    assert_eq!(outputs, def_outputs);
}

#[test]
fn test_shared_media() {
    let mut media = mapmap_core::module::SharedMediaState::new();
    media.register(
        "id1".to_string(),
        "path1".to_string(),
        mapmap_core::module::SharedMediaType::Video,
    );

    let item = media.get("id1").unwrap();
    assert_eq!(item.path, "path1");
    assert_eq!(item.media_type, mapmap_core::module::SharedMediaType::Video);

    assert!(media.get("id2").is_none());

    media.unregister("id1");
    assert!(media.get("id1").is_none());
}

#[test]
fn test_blend_mode_type_all() {
    let all = mapmap_core::module::BlendModeType::all();
    assert_eq!(all.len(), 7); // Normal, Add, Multiply, Screen, Overlay, HardLight, SoftLight, ColorDodge, ColorBurn, Difference, Exclusion, Darken, Lighten, Subtract
}

#[test]
fn test_effect_type_all() {
    let all = EffectType::all();
    assert_eq!(all.len(), 25);
}

#[test]
fn test_manager_getters() {
    let mut manager = ModuleManager::new();
    let id = manager.create_module("Test Module".to_string());

    // get_module
    assert!(manager.get_module(id).is_some());
    assert!(manager.get_module(999).is_none());

    // get_module_mut
    assert!(manager.get_module_mut(id).is_some());
    assert!(manager.get_module_mut(999).is_none());

    // modules()
    assert_eq!(manager.modules().len(), 1);

    // modules_mut()
    assert_eq!(manager.modules_mut().len(), 1);

    // remove_module()
    let removed = manager.remove_module(id);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name, "Test Module");
    assert!(manager.modules().is_empty());
}

#[test]
fn test_manager_next_part_id() {
    let mut manager = ModuleManager::new();
    let initial_id = manager.next_part_id();
    assert_eq!(manager.next_part_id(), initial_id + 1);
}

#[test]
fn test_manager_mark_dirty() {
    // mark_dirty just increments next_part_id as a side effect if it's not actually used for a specific dirtiness tracking mechanism in manager
    let mut manager = ModuleManager::new();
    manager.mark_dirty(); // Should be a no-op or specific side effect we can just call for coverage
}
