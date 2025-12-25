use mapmap_core::module::ModuleManager;

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
