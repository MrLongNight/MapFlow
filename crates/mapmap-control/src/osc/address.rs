//! OSC address space parser
//!
//! Parses OSC addresses like `/mapmap/layer/0/opacity` to control targets

use crate::{error::ControlError, ControlTarget, Result};

/// Maximum length of an OSC address string
const MAX_OSC_ADDRESS_LENGTH: usize = 1024;
/// Maximum length of a parameter name (e.g. paint/effect parameter)
const MAX_NAME_LENGTH: usize = 256;

/// Parse an OSC address to a control target
///
/// Supported address patterns:
/// - `/mapmap/layer/{id}/opacity` - Layer opacity (0.0-1.0)
/// - `/mapmap/layer/{id}/position` - Layer position (x, y)
/// - `/mapmap/layer/{id}/rotation` - Layer rotation (degrees)
/// - `/mapmap/layer/{id}/scale` - Layer scale
/// - `/mapmap/layer/{id}/visibility` - Layer visibility (bool)
/// - `/mapmap/paint/{id}/parameter/{name}` - Paint parameter
/// - `/mapmap/effect/{id}/parameter/{name}` - Effect parameter
/// - `/mapmap/playback/speed` - Playback speed
/// - `/mapmap/playback/position` - Playback position
/// - `/mapmap/output/{id}/brightness` - Output brightness
pub fn parse_osc_address(address: &str) -> Result<ControlTarget> {
    if address.len() > MAX_OSC_ADDRESS_LENGTH {
        return Err(ControlError::InvalidMessage(format!(
            "OSC address too long (max {} chars)",
            MAX_OSC_ADDRESS_LENGTH
        )));
    }

    let parts: Vec<&str> = address.trim_start_matches('/').split('/').collect();

    if parts.is_empty() || parts[0] != "mapmap" {
        return Err(ControlError::InvalidMessage(format!(
            "OSC address must start with /mapmap: {}",
            address
        )));
    }

    if parts.len() < 2 {
        return Err(ControlError::InvalidMessage(format!(
            "Invalid OSC address: {}",
            address
        )));
    }

    match parts[1] {
        "master" => parse_master_address(&parts[2..]),
        "layer" => parse_layer_address(&parts[2..]),
        "paint" => parse_paint_address(&parts[2..]),
        "effect" => parse_effect_address(&parts[2..]),
        "playback" => parse_playback_address(&parts[2..]),
        "output" => parse_output_address(&parts[2..]),
        _ => Err(ControlError::InvalidMessage(format!(
            "Unknown OSC category: {}",
            parts[1]
        ))),
    }
}

fn parse_master_address(parts: &[&str]) -> Result<ControlTarget> {
    if parts.is_empty() {
        return Err(ControlError::InvalidMessage(
            "Missing master parameter".to_string(),
        ));
    }

    match parts[0] {
        "opacity" => Ok(ControlTarget::MasterOpacity),
        "blackout" => Ok(ControlTarget::MasterBlackout),
        _ => Err(ControlError::InvalidMessage(format!(
            "Unknown master parameter: {}",
            parts[0]
        ))),
    }
}

fn parse_layer_address(parts: &[&str]) -> Result<ControlTarget> {
    if parts.is_empty() {
        return Err(ControlError::InvalidMessage("Missing layer ID".to_string()));
    }

    let layer_id: u32 = parts[0]
        .parse()
        .map_err(|_| ControlError::InvalidMessage(format!("Invalid layer ID: {}", parts[0])))?;

    if parts.len() < 2 {
        return Err(ControlError::InvalidMessage(
            "Missing layer parameter".to_string(),
        ));
    }

    match parts[1] {
        "opacity" => Ok(ControlTarget::LayerOpacity(layer_id)),
        "position" => Ok(ControlTarget::LayerPosition(layer_id)),
        "rotation" => Ok(ControlTarget::LayerRotation(layer_id)),
        "scale" => Ok(ControlTarget::LayerScale(layer_id)),
        "visibility" => Ok(ControlTarget::LayerVisibility(layer_id)),
        _ => Err(ControlError::InvalidMessage(format!(
            "Unknown layer parameter: {}",
            parts[1]
        ))),
    }
}

fn parse_paint_address(parts: &[&str]) -> Result<ControlTarget> {
    if parts.is_empty() {
        return Err(ControlError::InvalidMessage("Missing paint ID".to_string()));
    }

    let paint_id: u32 = parts[0]
        .parse()
        .map_err(|_| ControlError::InvalidMessage(format!("Invalid paint ID: {}", parts[0])))?;

    if parts.len() < 3 || parts[1] != "parameter" {
        return Err(ControlError::InvalidMessage(
            "Paint address must be /paint/{id}/parameter/{name}".to_string(),
        ));
    }

    let name = parts[2];
    if name.len() > MAX_NAME_LENGTH {
        return Err(ControlError::InvalidMessage(format!(
            "Parameter name too long (max {} chars)",
            MAX_NAME_LENGTH
        )));
    }

    Ok(ControlTarget::PaintParameter(paint_id, name.to_string()))
}

fn parse_effect_address(parts: &[&str]) -> Result<ControlTarget> {
    if parts.is_empty() {
        return Err(ControlError::InvalidMessage(
            "Missing effect ID".to_string(),
        ));
    }

    let effect_id: u32 = parts[0]
        .parse()
        .map_err(|_| ControlError::InvalidMessage(format!("Invalid effect ID: {}", parts[0])))?;

    if parts.len() < 3 || parts[1] != "parameter" {
        return Err(ControlError::InvalidMessage(
            "Effect address must be /effect/{id}/parameter/{name}".to_string(),
        ));
    }

    let name = parts[2];
    if name.len() > MAX_NAME_LENGTH {
        return Err(ControlError::InvalidMessage(format!(
            "Parameter name too long (max {} chars)",
            MAX_NAME_LENGTH
        )));
    }

    Ok(ControlTarget::EffectParameter(effect_id, name.to_string()))
}

fn parse_playback_address(parts: &[&str]) -> Result<ControlTarget> {
    if parts.is_empty() {
        return Err(ControlError::InvalidMessage(
            "Missing playback parameter".to_string(),
        ));
    }

    match parts[0] {
        "speed" => Ok(ControlTarget::PlaybackSpeed(None)),
        "position" => Ok(ControlTarget::PlaybackPosition),
        _ => Err(ControlError::InvalidMessage(format!(
            "Unknown playback parameter: {}",
            parts[0]
        ))),
    }
}

fn parse_output_address(parts: &[&str]) -> Result<ControlTarget> {
    if parts.is_empty() {
        return Err(ControlError::InvalidMessage(
            "Missing output ID".to_string(),
        ));
    }

    let output_id: u32 = parts[0]
        .parse()
        .map_err(|_| ControlError::InvalidMessage(format!("Invalid output ID: {}", parts[0])))?;

    if parts.len() < 2 {
        return Err(ControlError::InvalidMessage(
            "Missing output parameter".to_string(),
        ));
    }

    match parts[1] {
        "brightness" => Ok(ControlTarget::OutputBrightness(output_id)),
        _ => Err(ControlError::InvalidMessage(format!(
            "Unknown output parameter: {}",
            parts[1]
        ))),
    }
}

/// Generate OSC address from control target
pub fn control_target_to_address(target: &ControlTarget) -> String {
    match target {
        ControlTarget::LayerOpacity(id) => format!("/mapmap/layer/{}/opacity", id),
        ControlTarget::LayerPosition(id) => format!("/mapmap/layer/{}/position", id),
        ControlTarget::LayerScale(id) => format!("/mapmap/layer/{}/scale", id),
        ControlTarget::LayerRotation(id) => format!("/mapmap/layer/{}/rotation", id),
        ControlTarget::LayerVisibility(id) => format!("/mapmap/layer/{}/visibility", id),
        ControlTarget::PaintParameter(id, name) => {
            format!("/mapmap/paint/{}/parameter/{}", id, name)
        }
        ControlTarget::EffectParameter(id, name) => {
            format!("/mapmap/effect/{}/parameter/{}", id, name)
        }
        ControlTarget::PlaybackSpeed(_) => "/mapmap/playback/speed".to_string(),
        ControlTarget::PlaybackPosition => "/mapmap/playback/position".to_string(),
        ControlTarget::OutputBrightness(id) => format!("/mapmap/output/{}/brightness", id),
        ControlTarget::OutputEdgeBlend(id, edge) => {
            format!("/mapmap/output/{}/edge_blend/{:?}", id, edge)
        }
        ControlTarget::MasterOpacity => "/mapmap/master/opacity".to_string(),
        ControlTarget::MasterBlackout => "/mapmap/master/blackout".to_string(),
        ControlTarget::Custom(name) => format!("/mapmap/custom/{}", name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_layer_opacity() {
        let target = parse_osc_address("/mapmap/layer/0/opacity").unwrap();
        assert_eq!(target, ControlTarget::LayerOpacity(0));
    }

    #[test]
    fn test_parse_layer_position() {
        let target = parse_osc_address("/mapmap/layer/5/position").unwrap();
        assert_eq!(target, ControlTarget::LayerPosition(5));
    }

    #[test]
    fn test_parse_paint_parameter() {
        let target = parse_osc_address("/mapmap/paint/3/parameter/speed").unwrap();
        assert_eq!(
            target,
            ControlTarget::PaintParameter(3, "speed".to_string())
        );
    }

    #[test]
    fn test_parse_effect_parameter() {
        let target = parse_osc_address("/mapmap/effect/1/parameter/intensity").unwrap();
        assert_eq!(
            target,
            ControlTarget::EffectParameter(1, "intensity".to_string())
        );
    }

    #[test]
    fn test_parse_playback_speed() {
        let target = parse_osc_address("/mapmap/playback/speed").unwrap();
        assert_eq!(target, ControlTarget::PlaybackSpeed(None));
    }

    #[test]
    fn test_invalid_address() {
        assert!(parse_osc_address("/invalid/address").is_err());
        assert!(parse_osc_address("/mapmap").is_err());
        assert!(parse_osc_address("/mapmap/layer").is_err());
        assert!(parse_osc_address("/mapmap/layer/notanumber/opacity").is_err());
    }

    #[test]
    fn test_control_target_to_address() {
        let target = ControlTarget::LayerOpacity(0);
        assert_eq!(
            control_target_to_address(&target),
            "/mapmap/layer/0/opacity"
        );

        let target = ControlTarget::PaintParameter(3, "speed".to_string());
        assert_eq!(
            control_target_to_address(&target),
            "/mapmap/paint/3/parameter/speed"
        );
    }

    #[test]
    fn test_parse_layer_rotation() {
        let target = parse_osc_address("/mapmap/layer/2/rotation").unwrap();
        assert_eq!(target, ControlTarget::LayerRotation(2));
    }

    #[test]
    fn test_parse_layer_scale() {
        let target = parse_osc_address("/mapmap/layer/7/scale").unwrap();
        assert_eq!(target, ControlTarget::LayerScale(7));
    }

    #[test]
    fn test_parse_layer_visibility() {
        let target = parse_osc_address("/mapmap/layer/10/visibility").unwrap();
        assert_eq!(target, ControlTarget::LayerVisibility(10));
    }

    #[test]
    fn test_parse_playback_position() {
        let target = parse_osc_address("/mapmap/playback/position").unwrap();
        assert_eq!(target, ControlTarget::PlaybackPosition);
    }

    #[test]
    fn test_parse_output_brightness() {
        let target = parse_osc_address("/mapmap/output/0/brightness").unwrap();
        assert_eq!(target, ControlTarget::OutputBrightness(0));
    }

    #[test]
    fn test_parse_master_opacity() {
        let target = parse_osc_address("/mapmap/master/opacity").unwrap();
        assert_eq!(target, ControlTarget::MasterOpacity);
    }

    #[test]
    fn test_parse_master_blackout() {
        let target = parse_osc_address("/mapmap/master/blackout").unwrap();
        assert_eq!(target, ControlTarget::MasterBlackout);
    }

    #[test]
    fn test_round_trip_layer_targets() {
        // Test that parsing the address generated from a target gives back the same target
        let targets = vec![
            ControlTarget::LayerOpacity(5),
            ControlTarget::LayerPosition(3),
            ControlTarget::LayerScale(1),
            ControlTarget::LayerRotation(8),
            ControlTarget::LayerVisibility(0),
        ];

        for target in targets {
            let address = control_target_to_address(&target);
            let parsed = parse_osc_address(&address).unwrap();
            assert_eq!(parsed, target);
        }
    }

    #[test]
    fn test_round_trip_master_targets() {
        let targets = vec![ControlTarget::MasterOpacity, ControlTarget::MasterBlackout];

        for target in targets {
            let address = control_target_to_address(&target);
            let parsed = parse_osc_address(&address).unwrap();
            assert_eq!(parsed, target);
        }
    }

    #[test]
    fn test_round_trip_playback_targets() {
        let targets = vec![
            ControlTarget::PlaybackSpeed(None),
            ControlTarget::PlaybackPosition,
        ];

        for target in targets {
            let address = control_target_to_address(&target);
            let parsed = parse_osc_address(&address).unwrap();
            assert_eq!(parsed, target);
        }
    }

    #[test]
    fn test_invalid_category() {
        assert!(parse_osc_address("/mapmap/unknown/test").is_err());
    }

    #[test]
    fn test_invalid_output_address() {
        assert!(parse_osc_address("/mapmap/output").is_err());
        assert!(parse_osc_address("/mapmap/output/abc").is_err());
        assert!(parse_osc_address("/mapmap/output/0").is_err());
        assert!(parse_osc_address("/mapmap/output/0/unknown").is_err());
    }

    #[test]
    fn test_invalid_master_address() {
        assert!(parse_osc_address("/mapmap/master").is_err());
        assert!(parse_osc_address("/mapmap/master/unknown").is_err());
    }

    #[test]
    fn test_parse_huge_address() {
        // Construct a valid address with a very long parameter name
        let huge_name = "a".repeat(10000);
        let address = format!("/mapmap/paint/0/parameter/{}", huge_name);

        // This should now fail due to length limits
        let result = parse_osc_address(&address);
        assert!(result.is_err());

        // Also verify the total address limit
        let huge_address = format!("/mapmap/{}", "a".repeat(2000));
        let result_total = parse_osc_address(&huge_address);
        assert!(result_total.is_err());
    }
}
