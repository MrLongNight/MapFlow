use mapmap_core::module::ModulePlaybackMode;

#[test]
fn test_module_playback_mode_variants() {
    let loop_mode = ModulePlaybackMode::LoopUntilManualSwitch;
    let timeline_mode = ModulePlaybackMode::TimelineDuration { duration_ms: 5000 };

    assert_ne!(loop_mode, timeline_mode);
}

#[test]
fn test_module_playback_mode_serialization() {
    // Loop mode
    let loop_mode = ModulePlaybackMode::LoopUntilManualSwitch;
    let serialized_loop = serde_json::to_string(&loop_mode).unwrap();
    let deserialized_loop: ModulePlaybackMode = serde_json::from_str(&serialized_loop).unwrap();
    assert_eq!(loop_mode, deserialized_loop);

    // Timeline mode
    let timeline_mode = ModulePlaybackMode::TimelineDuration { duration_ms: 1234 };
    let serialized_timeline = serde_json::to_string(&timeline_mode).unwrap();
    let deserialized_timeline: ModulePlaybackMode =
        serde_json::from_str(&serialized_timeline).unwrap();
    assert_eq!(timeline_mode, deserialized_timeline);

    if let ModulePlaybackMode::TimelineDuration { duration_ms } = deserialized_timeline {
        assert_eq!(duration_ms, 1234);
    } else {
        panic!("Wrong variant deserialized");
    }
}
