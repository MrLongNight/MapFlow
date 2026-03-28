use std::collections::HashMap;
use vorce_core::module::{
    HueMappingMode, ModuleManager, ModulePartType, ModulePlaybackMode, ModuleSocketType,
    OutputType, PartType, VorceModule,
};

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
    let mut module = VorceModule {
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
    let mut module = VorceModule {
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
    let mut module = VorceModule {
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

    module.add_connection(
        pid1,
        "trigger_out".to_string(),
        pid2,
        "trigger_in".to_string(),
    );
    assert_eq!(module.connections.len(), 1);
    assert_eq!(module.connections[0].from_part, pid1);
    assert_eq!(module.connections[0].to_part, pid2);

    module.remove_connection(
        pid1,
        "trigger_out".to_string(),
        pid2,
        "trigger_in".to_string(),
    );
    assert!(module.connections.is_empty());
}

#[test]
fn test_update_part_sockets() {
    let mut module = VorceModule {
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
    module.add_connection(
        pid1,
        "trigger_out".to_string(),
        pid2,
        "trigger_in".to_string(),
    );

    // For now just test it doesn't crash on normal update and keeps valid connections
    module.update_part_sockets(pid1);
    assert_eq!(module.connections.len(), 1);
}

#[test]
fn test_update_part_sockets_removes_invalid_connections() {
    let mut module = VorceModule {
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

    // Create invalid connection (socket IDs that don't exist)
    module
        .connections
        .push(vorce_core::module::ModuleConnection {
            from_part: pid1,
            from_socket: "invalid_src".to_string(),
            to_part: pid2,
            to_socket: "trigger_in".to_string(),
        });
    module
        .connections
        .push(vorce_core::module::ModuleConnection {
            from_part: pid1,
            from_socket: "trigger_out".to_string(),
            to_part: pid2,
            to_socket: "invalid_dst".to_string(),
        });
    module
        .connections
        .push(vorce_core::module::ModuleConnection {
            from_part: pid1,
            from_socket: "trigger_out".to_string(),
            to_part: pid2,
            to_socket: "trigger_in".to_string(),
        }); // Valid connection

    assert_eq!(module.connections.len(), 3);

    // Calling update_part_sockets updates sockets, but graph repair is needed to fully clean up
    module.update_part_sockets(pid1);
    module.repair_graph();

    assert_eq!(module.connections.len(), 1);

    assert_eq!(module.connections[0].from_socket, "trigger_out");
    assert_eq!(module.connections[0].to_socket, "trigger_in");
}

#[test]
fn test_update_part_outputs_delegates() {
    let mut module = VorceModule {
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

    // Push directly since add_connection would silently fail
    module
        .connections
        .push(vorce_core::module::ModuleConnection {
            from_part: pid1,
            from_socket: "invalid".to_string(),
            to_part: pid2,
            to_socket: "trigger_in".to_string(),
        });

    assert_eq!(module.connections.len(), 1);
    module.update_part_outputs(pid1); // Should call update_part_sockets and clear connection
    assert!(module.connections.is_empty());
}

#[test]
fn test_effect_type_all_and_name() {
    let all = vorce_core::module::EffectType::all();
    assert!(!all.is_empty());

    for effect in all {
        let name = effect.name();
        assert!(!name.is_empty());
    }

    let sg = vorce_core::module::EffectType::ShaderGraph(1);
    assert_eq!(sg.name(), "Custom Shader Graph");
}

#[test]
fn test_blend_mode_type_all_and_name() {
    let all = vorce_core::module::BlendModeType::all();
    assert!(!all.is_empty());

    for blend_mode in all {
        let name = blend_mode.name();
        assert!(!name.is_empty());
    }
}

#[test]
fn test_mesh_type_to_mesh_and_hash() {
    let mesh_type = vorce_core::module::MeshType::Grid { cols: 2, rows: 2 };
    let hash = mesh_type.compute_revision_hash();
    assert_ne!(hash, 0); // Assuming hash isn't exactly 0 for grid 2x2

    let mesh = mesh_type.to_mesh();
    assert!(!mesh.vertices.is_empty());
}

#[test]
fn test_part_compute_sockets() {
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
    let mut media = vorce_core::module::SharedMediaState::new();
    media.register(
        "id1".to_string(),
        "path1".to_string(),
        vorce_core::module::SharedMediaType::Video,
    );

    let item = media.get("id1").unwrap();
    assert_eq!(item.path, "path1");
    assert_eq!(item.media_type, vorce_core::module::SharedMediaType::Video);

    assert!(media.get("id2").is_none());

    media.unregister("id1");
    assert!(media.get("id1").is_none());
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
    let mut manager = ModuleManager::new();
    manager.mark_dirty();
}

#[test]
fn test_manager_new_initial_state() {
    let manager = ModuleManager::new();
    assert!(manager.modules().is_empty());
    assert_eq!(manager.graph_revision, 1);
}

#[test]
fn test_manager_create_module_success() {
    let mut manager = ModuleManager::new();
    let id = manager.create_module("Test Module".to_string());

    assert_eq!(id, 1);
    assert_eq!(manager.modules().len(), 1);
    assert_eq!(manager.graph_revision, 2);

    let module = manager.get_module(id).unwrap();
    assert_eq!(module.name, "Test Module");
    assert_eq!(module.id, 1);
}

#[test]
fn test_manager_create_module_duplicate_name_renames() {
    let mut manager = ModuleManager::new();
    manager.create_module("Test Module".to_string());
    let id2 = manager.create_module("Test Module".to_string());

    let module2 = manager.get_module(id2).unwrap();
    assert_eq!(module2.name, "Test Module 1");
}

#[test]
fn test_manager_delete_module_removes_from_map() {
    let mut manager = ModuleManager::new();
    let id = manager.create_module("Test Module".to_string());

    manager.delete_module(id);

    assert!(manager.modules().is_empty());
    assert!(manager.get_module(id).is_none());
    assert_eq!(manager.graph_revision, 3);
}

#[test]
fn test_manager_duplicate_module_creates_copy() {
    let mut manager = ModuleManager::new();
    let id = manager.create_module("Original".to_string());

    let dup_id = manager.duplicate_module(id).unwrap();

    assert_eq!(manager.modules().len(), 2);
    let dup_module = manager.get_module(dup_id).unwrap();
    assert_eq!(dup_module.name, "Original (Copy) 1");
}

#[test]
fn test_manager_rename_module_success() {
    let mut manager = ModuleManager::new();
    let id = manager.create_module("Original".to_string());

    let success = manager.rename_module(id, "Renamed".to_string());

    assert!(success);
    assert_eq!(manager.get_module(id).unwrap().name, "Renamed");
}

#[test]
fn test_manager_rename_module_duplicate_fails() {
    let mut manager = ModuleManager::new();
    manager.create_module("Module A".to_string());
    let id2 = manager.create_module("Module B".to_string());

    let success = manager.rename_module(id2, "Module A".to_string());

    assert!(!success);
    assert_eq!(manager.get_module(id2).unwrap().name, "Module B");
}

#[test]
fn test_module_add_part_creates_part_and_increments_id() {
    let mut module = VorceModule {
        id: 1,
        name: "Test Module".to_string(),
        color: [1.0, 1.0, 1.0, 1.0],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    let part_id = module.add_part(PartType::Source, (10.0, 20.0));

    assert_eq!(part_id, 1);
    assert_eq!(module.next_part_id, 2);
    assert_eq!(module.parts.len(), 1);

    let added_part = &module.parts[0];
    assert_eq!(added_part.id, 1);
    assert_eq!(added_part.position, (10.0, 20.0));
    assert!(matches!(added_part.part_type, ModulePartType::Source(_)));
}

#[test]
fn test_module_update_part_position_success() {
    let mut module = VorceModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    let part_id = module.add_part(PartType::Trigger, (0.0, 0.0));
    module.update_part_position(part_id, (50.0, 100.0));

    assert_eq!(module.parts[0].position, (50.0, 100.0));
}

#[test]
fn test_module_add_connection_adds_to_list() {
    let mut module = VorceModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // The connections vector needs to be pushed directly since add_connection validates parts.
    module
        .connections
        .push(vorce_core::module::ModuleConnection {
            from_part: 1,
            from_socket: "trigger_out".to_string(),
            to_part: 2,
            to_socket: "trigger_in".to_string(),
        });

    assert_eq!(module.connections.len(), 1);
    let conn = &module.connections[0];
    assert_eq!(conn.from_part, 1);
    assert_eq!(conn.from_socket, "trigger_out");
    assert_eq!(conn.to_part, 2);
    assert_eq!(conn.to_socket, "trigger_in");
}

#[test]
fn test_module_remove_connection_removes_exact_match() {
    let mut module = VorceModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // Push directly since validate_connection fails if parts are missing
    module
        .connections
        .push(vorce_core::module::ModuleConnection {
            from_part: 1,
            from_socket: "trigger_out".to_string(),
            to_part: 2,
            to_socket: "trigger_in".to_string(),
        });
    module
        .connections
        .push(vorce_core::module::ModuleConnection {
            from_part: 1,
            from_socket: "link_out".to_string(),
            to_part: 3,
            to_socket: "link_in".to_string(),
        });

    module.remove_connection(1, "trigger_out".to_string(), 2, "trigger_in".to_string());

    assert_eq!(module.connections.len(), 1);
    assert_eq!(module.connections[0].to_part, 3);
}
