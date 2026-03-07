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
}
