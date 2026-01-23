use mapmap_core::module::{TriggerConfig, TriggerMappingMode, TriggerTarget};

#[test]
fn test_trigger_config_defaults() {
    let config = TriggerConfig::default();
    assert_eq!(config.target, TriggerTarget::None);
    assert_eq!(config.mode, TriggerMappingMode::Direct);
    assert_eq!(config.min_value, 0.0);
    assert_eq!(config.max_value, 1.0);
    assert!(!config.invert);
    assert_eq!(config.threshold, 0.5);
}

#[test]
fn test_trigger_config_apply_direct() {
    let mut config = TriggerConfig::default();
    config.min_value = 0.0;
    config.max_value = 100.0;

    // Direct mapping
    assert_eq!(config.apply(0.0), 0.0);
    assert_eq!(config.apply(0.5), 50.0);
    assert_eq!(config.apply(1.0), 100.0);

    // With inversion
    config.invert = true;
    assert_eq!(config.apply(0.0), 100.0);
    assert_eq!(config.apply(0.5), 50.0);
    assert_eq!(config.apply(1.0), 0.0);
}

#[test]
fn test_trigger_config_apply_fixed() {
    let mut config = TriggerConfig::default();
    config.mode = TriggerMappingMode::Fixed;
    config.min_value = 10.0;
    config.max_value = 20.0;
    config.threshold = 0.5;

    // Below threshold
    assert_eq!(config.apply(0.4), 10.0);
    assert_eq!(config.apply(0.5), 10.0); // Exact threshold -> min value

    // Above threshold
    assert_eq!(config.apply(0.6), 20.0);

    // With inversion: input becomes (1.0 - input) before threshold check
    config.invert = true;
    // Input 0.4 -> 0.6 (> 0.5) -> max value
    assert_eq!(config.apply(0.4), 20.0);
    // Input 0.6 -> 0.4 (<= 0.5) -> min value
    assert_eq!(config.apply(0.6), 10.0);
}

#[test]
fn test_trigger_config_apply_random() {
    let mut config = TriggerConfig::default();
    config.mode = TriggerMappingMode::RandomInRange;
    config.min_value = 10.0;
    config.max_value = 20.0;

    // When trigger is inactive (0.0), returns min value
    assert_eq!(config.apply(0.0), 10.0);

    // When trigger is active (> 0.0), returns value in range
    for _ in 0..100 {
        let val = config.apply(1.0);
        assert!(val >= 10.0 && val <= 20.0);
    }
}

#[test]
fn test_trigger_config_for_target() {
    let config = TriggerConfig::for_target(TriggerTarget::Opacity);
    assert_eq!(config.target, TriggerTarget::Opacity);
    assert_eq!(config.mode, TriggerMappingMode::Direct);
}
