use mapmap_core::state::{AppState, AppSettings};

#[test]
fn test_app_state_default() {
    let state = AppState::default();
    assert_eq!(state.name, "Untitled Project");
    assert_eq!(state.version, "0.1.0");
    assert!(!state.dirty);
}

#[test]
fn test_app_state_new() {
    let state = AppState::new("My Show");
    assert_eq!(state.name, "My Show");
}

#[test]
fn test_app_state_serialization() {
    let mut state = AppState::default();
    state.name = "Serialization Test".to_string();
    state.settings.master_volume = 0.5;

    let json = serde_json::to_string(&state).expect("Failed to serialize AppState");
    let deserialized: AppState = serde_json::from_str(&json).expect("Failed to deserialize AppState");

    assert_eq!(state.name, deserialized.name);
    assert_eq!(state.settings.master_volume, deserialized.settings.master_volume);
    assert_eq!(state, deserialized);
}

#[test]
fn test_app_settings_default() {
    let settings = AppSettings::default();
    assert_eq!(settings.master_volume, 1.0);
    assert!(settings.dark_mode);
    assert_eq!(settings.output_count, 1);
}
