use mapmap_core::module::{
    HueMappingMode, MapFlowModule, ModuleManager, ModulePartType, ModulePlaybackMode,
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

