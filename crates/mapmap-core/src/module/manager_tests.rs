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
