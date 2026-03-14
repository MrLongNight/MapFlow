#[cfg(test)]
mod mapping_tests {
    use crate::osc::address::{control_target_to_address, parse_osc_address};
    use crate::target::ControlTarget;

    #[test]
    fn test_mcp_parity_timeline() {
        assert_eq!(
            parse_osc_address("/mapmap/timeline/play").unwrap(),
            ControlTarget::TimelinePlay
        );
        assert_eq!(
            parse_osc_address("/mapmap/timeline/stop").unwrap(),
            ControlTarget::TimelineStop
        );
        assert_eq!(
            parse_osc_address("/mapmap/timeline/speed").unwrap(),
            ControlTarget::TimelineSpeed
        );
        assert_eq!(
            parse_osc_address("/mapmap/timeline/loop").unwrap(),
            ControlTarget::TimelineLoop
        );

        assert_eq!(
            control_target_to_address(&ControlTarget::TimelinePlay),
            "/mapmap/timeline/play"
        );
    }

    #[test]
    fn test_mcp_parity_effect() {
        assert_eq!(
            parse_osc_address("/mapmap/layer/1/effect/add").unwrap(),
            ControlTarget::EffectAdd(1)
        );
        assert_eq!(
            parse_osc_address("/mapmap/layer/1/effect/2/remove").unwrap(),
            ControlTarget::EffectRemove(1, 2)
        );
        assert_eq!(
            parse_osc_address("/mapmap/layer/1/effect/2/parameter/intensity").unwrap(),
            ControlTarget::LayerEffectParameter(1, 2, "intensity".to_string())
        );
    }

    #[test]
    fn test_mcp_parity_surface() {
        assert_eq!(
            parse_osc_address("/mapmap/surface/3/corner/2/position").unwrap(),
            ControlTarget::SurfaceCornerPosition(3, 2)
        );
    }

    #[test]
    fn test_mcp_parity_scene() {
        assert_eq!(
            parse_osc_address("/mapmap/scene/switch/4").unwrap(),
            ControlTarget::SceneSwitch(4)
        );
    }

    #[test]
    fn test_mcp_parity_cue() {
        assert_eq!(
            parse_osc_address("/mapmap/cue/trigger/5").unwrap(),
            ControlTarget::CueTrigger(5)
        );
    }
}
