//! Project serialization and deserialization tests

use mapmap_core::{AppSettings, AppState};
use mapmap_io::error::IoError;
use mapmap_io::project::{load_project, save_project};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

/// Creates a sample AppState for testing.
fn create_sample_app_state() -> AppState {
    let mut app_state = AppState::new("Test Project");
    app_state.version = "1.2.3".to_string();
    app_state.settings = std::sync::Arc::new(AppSettings {
        master_volume: 0.8,
        dark_mode: false,
        ui_scale: 1.2,
        language: "de".to_string(),
        log_config: Default::default(),
        output_count: 1,
    });
    
    // Create and fill managers before they are wrapped in Arc (if AppState allows)
    // Or use make_mut if they are already in Arc
    {
        let layer_manager = std::sync::Arc::make_mut(&mut app_state.layer_manager);
        let layer = mapmap_core::Layer::new(1, "Test Layer");
        layer_manager.add_layer(layer);
    }
    
    {
        let mapping_manager = std::sync::Arc::make_mut(&mut app_state.mapping_manager);
        let mapping = mapmap_core::Mapping::new(
            1, 
            "Test Mapping", 
            1, 
            mapmap_core::Mesh::quad()
        );
        mapping_manager.add_mapping(mapping);
    }
    
    app_state
}

#[test]
fn test_project_ron_roundtrip() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_project.mflow");

    let original_state = create_sample_app_state();
    save_project(&original_state, &file_path).unwrap();

    let loaded_state = load_project(&file_path).unwrap();

    assert_eq!(original_state, loaded_state);
}

#[test]
fn test_project_json_roundtrip() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_project.json");

    let original_state = create_sample_app_state();
    save_project(&original_state, &file_path).unwrap();

    let loaded_state = load_project(&file_path).unwrap();

    assert_eq!(original_state, loaded_state);
}

#[test]
fn test_load_missing_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("non_existent_project.mflow");

    let result = load_project(&file_path);
    assert!(matches!(result, Err(IoError::Io(_))));
}

#[test]
fn test_load_invalid_ron() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("invalid.mflow");

    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "this is not valid ron").unwrap();

    let result = load_project(&file_path);
    assert!(matches!(result, Err(IoError::RonDeserialization(_))));
}
