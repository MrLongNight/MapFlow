#[cfg(test)]
mod tests {
    use crate::module::ModuleManager;

    #[test]
    fn test_manager_new_initial_state() {
        let manager = ModuleManager::new();
        assert!(manager.modules.is_empty());
        assert_eq!(manager.next_module_id, 1);
        assert_eq!(manager.next_part_id, 1);
        assert_eq!(manager.next_color_index, 0);
        assert_eq!(manager.graph_revision, 1);
    }

    #[test]
    fn test_manager_create_module_success() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("Test Module".to_string());

        assert_eq!(id, 1);
        assert_eq!(manager.modules.len(), 1);
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

        assert!(manager.modules.is_empty());
        assert!(manager.get_module(id).is_none());
        assert_eq!(manager.graph_revision, 3);
    }

    #[test]
    fn test_manager_duplicate_module_creates_copy() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("Original".to_string());

        let dup_id = manager.duplicate_module(id).unwrap();

        assert_eq!(manager.modules.len(), 2);
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
}
#[cfg(test)]
mod test_manager_repair {
    use crate::module::{ModuleManager, types::PartType};

    #[test]
    fn test_manager_repair_module_repairs_graph_and_marks_dirty() {
        let mut manager = ModuleManager::new();
        let module_id = manager.create_module("Test".to_string());

        // Initial graph_revision after create_module
        let initial_revision = manager.graph_revision;

        // Add two parts and a connection between them
        let part1 = manager.add_part_to_module(module_id, PartType::Trigger, (0.0, 0.0)).unwrap();
        let part2 = manager.add_part_to_module(module_id, PartType::Trigger, (0.0, 0.0)).unwrap();

        let module = manager.get_module_mut(module_id).unwrap();
        module.add_connection(part1, 0, part2, 0); // Invalid sockets conceptually, but structural

        // Simulate removing part2 without removing the connection, leaving a dangling connection
        module.parts.retain(|p| p.id != part2);

        // At this point, the module has a dangling connection.
        // repair_module should clean this up and mark the manager dirty.
        let report = manager.repair_module(module_id).unwrap();

        assert_eq!(report.removed_connections, 1);
        assert!(report.changed());

        // Check if graph_revision increased
        assert!(manager.graph_revision > initial_revision);

        let repaired_module = manager.get_module(module_id).unwrap();
        assert!(repaired_module.connections.is_empty());
    }

    #[test]
    fn test_manager_repair_modules_repairs_multiple() {
        let mut manager = ModuleManager::new();
        let module_id1 = manager.create_module("Test1".to_string());
        let module_id2 = manager.create_module("Test2".to_string());

        // Setup module 1 with a dangling connection
        let part1 = manager.add_part_to_module(module_id1, PartType::Trigger, (0.0, 0.0)).unwrap();
        let part2 = manager.add_part_to_module(module_id1, PartType::Trigger, (0.0, 0.0)).unwrap();
        let module1 = manager.get_module_mut(module_id1).unwrap();
        module1.add_connection(part1, 0, part2, 0);
        module1.parts.retain(|p| p.id != part2); // Delete part2

        // Setup module 2 with a dangling connection
        let part3 = manager.add_part_to_module(module_id2, PartType::Trigger, (0.0, 0.0)).unwrap();
        let part4 = manager.add_part_to_module(module_id2, PartType::Trigger, (0.0, 0.0)).unwrap();
        let module2 = manager.get_module_mut(module_id2).unwrap();
        module2.add_connection(part3, 0, part4, 0);
        module2.parts.retain(|p| p.id != part4); // Delete part4

        let reports = manager.repair_modules(vec![module_id1, module_id2]);

        assert_eq!(reports.len(), 2);

        // Both modules should have been repaired
        let report1 = reports.iter().find(|r| r.0 == module_id1).unwrap();
        assert_eq!(report1.1.removed_connections, 1);

        let report2 = reports.iter().find(|r| r.0 == module_id2).unwrap();
        assert_eq!(report2.1.removed_connections, 1);
    }
}
