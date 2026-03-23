#[cfg(test)]
mod tests {
    use crate::module::types::{ModulePartType, PartType};
    use crate::module::{MapFlowModule, ModulePlaybackMode};

    #[test]
    fn test_module_add_part_creates_part_and_increments_id() {
        let mut module = MapFlowModule {
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
        let mut module = MapFlowModule {
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
        let mut module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        module.add_connection(1, 0, 2, 0);

        assert_eq!(module.connections.len(), 1);
        let conn = &module.connections[0];
        assert_eq!(conn.from_part, 1);
        assert_eq!(conn.from_socket, 0);
        assert_eq!(conn.to_part, 2);
        assert_eq!(conn.to_socket, 0);
    }

    #[test]
    fn test_module_remove_connection_removes_exact_match() {
        let mut module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        module.add_connection(1, 0, 2, 0);
        module.add_connection(1, 1, 3, 0);

        module.remove_connection(1, 0, 2, 0);

        assert_eq!(module.connections.len(), 1);
        assert_eq!(module.connections[0].to_part, 3);
    }

    #[test]
    fn test_normalize_inserted_part_type_resolves_output_id_conflict() {
        use crate::module::types::output::OutputType;
        let mut module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        // Add a part with id 1
        module.add_part_with_type(
            ModulePartType::Output(OutputType::Projector {
                id: 1,
                name: "Existing".to_string(),
                hide_cursor: false,
                target_screen: None,
                show_in_preview_panel: true,
                extra_preview_window: false,
                output_width: 1920,
                output_height: 1080,
                output_fps: 60,
                ndi_enabled: false,
                ndi_stream_name: "NDI".to_string(),
            }),
            (0.0, 0.0),
        );

        // Add another part trying to use id 1, and no specific name
        let part_id2 = module.add_part_with_type(
            ModulePartType::Output(OutputType::Projector {
                id: 1,
                name: " ".to_string(),
                hide_cursor: false,
                target_screen: None,
                show_in_preview_panel: true,
                extra_preview_window: false,
                output_width: 1920,
                output_height: 1080,
                output_fps: 60,
                ndi_enabled: false,
                ndi_stream_name: "NDI".to_string(),
            }),
            (10.0, 0.0),
        );

        let added_part2 = module.get_part(part_id2).unwrap();
        if let ModulePartType::Output(OutputType::Projector { id, name, .. }) = &added_part2.part_type {
            assert_eq!(*id, 2); // ID incremented to 2
            assert_eq!(name, "Output 2"); // Auto-generated name used 2
        } else {
            panic!("Expected Output Projector part type");
        }
    }

    #[test]
    fn test_normalize_inserted_part_type_resolves_layer_id_conflict() {
        use crate::module::types::layer::LayerType;
        use crate::layer::types::{BlendMode, MappingMode};

        let mut module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        // Add a part with id 1
        module.add_part_with_type(
            ModulePartType::Layer(LayerType::Single {
                id: 1,
                name: "CustomName".to_string(),
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                mesh: None,
                mapping_mode: MappingMode::Texture,
            }),
            (0.0, 0.0),
        );

        // Add another part trying to use id 1, and empty name
        let part_id2 = module.add_part_with_type(
            ModulePartType::Layer(LayerType::Single {
                id: 1,
                name: "".to_string(),
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                mesh: None,
                mapping_mode: MappingMode::Texture,
            }),
            (10.0, 0.0),
        );

        let added_part2 = module.get_part(part_id2).unwrap();
        if let ModulePartType::Layer(LayerType::Single { id, name, .. }) = &added_part2.part_type {
            assert_eq!(*id, 2); // ID incremented to 2
            assert_eq!(name, "Layer 2"); // Auto-generated name used 2
        } else {
            panic!("Expected Layer Single part type");
        }
    }

    #[test]
    fn test_normalize_inserted_part_type_preserves_other_types() {
        use crate::module::types::trigger::TriggerType;

        let mut module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        let part_id = module.add_part_with_type(
            ModulePartType::Trigger(TriggerType::Beat),
            (0.0, 0.0),
        );

        let added_part = module.get_part(part_id).unwrap();
        assert!(matches!(added_part.part_type, ModulePartType::Trigger(TriggerType::Beat)));
    }
}
