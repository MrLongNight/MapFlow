//! HTTP request handlers

use serde::{Deserialize, Serialize};

use crate::{ControlTarget, ControlValue};

/// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// System status response
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub version: String,
    pub uptime_seconds: u64,
    pub active_layers: usize,
    pub fps: f32,
}

/// Layer info response
#[derive(Debug, Serialize, Deserialize)]
pub struct LayerInfo {
    pub id: u32,
    pub name: String,
    pub opacity: f32,
    pub visible: bool,
}

/// Parameter update request
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateParameterRequest {
    pub target: ControlTarget,
    pub value: ControlValue,
}

/// Layer update request
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLayerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<(f32, f32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
}

impl UpdateLayerRequest {
    /// Check if the request is empty
    pub fn is_empty(&self) -> bool {
        self.opacity.is_none()
            && self.visible.is_none()
            && self.position.is_none()
            && self.rotation.is_none()
            && self.scale.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success(42);
        assert!(response.success);
        assert_eq!(response.data, Some(42));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<()> = ApiResponse::error("Test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_update_layer_request_empty() {
        let request = UpdateLayerRequest {
            opacity: None,
            visible: None,
            position: None,
            rotation: None,
            scale: None,
        };
        assert!(request.is_empty());

        let request = UpdateLayerRequest {
            opacity: Some(0.5),
            visible: None,
            position: None,
            rotation: None,
            scale: None,
        };
        assert!(!request.is_empty());
    }

    #[test]
    fn test_serialization() {
        let response = ApiResponse::success(StatusResponse {
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            active_layers: 5,
            fps: 60.0,
        });

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("version"));
    }

    #[test]
    fn test_layer_info_serialization() {
        let info = LayerInfo {
            id: 1,
            name: "Layer 1".to_string(),
            opacity: 1.0,
            visible: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("Layer 1"));
        assert!(json.contains("opacity"));

        let deserialized: LayerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.name, "Layer 1");
    }

    #[test]
    fn test_update_layer_request_serialization() {
        let request = UpdateLayerRequest {
            opacity: Some(0.5),
            visible: Some(true),
            position: Some((100.0, 200.0)),
            rotation: None,
            scale: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("opacity"));
        assert!(json.contains("visible"));
        assert!(!json.contains("rotation"));

        let deserialized: UpdateLayerRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.opacity, Some(0.5));
        assert_eq!(deserialized.visible, Some(true));
        assert_eq!(deserialized.position, Some((100.0, 200.0)));
        assert_eq!(deserialized.rotation, None);
    }

    #[test]
    fn test_update_parameter_request_serialization() {
        let request = UpdateParameterRequest {
            target: ControlTarget::LayerOpacity(0),
            value: ControlValue::Float(0.75),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("target"));
        assert!(json.contains("value"));

        let deserialized: UpdateParameterRequest = serde_json::from_str(&json).unwrap();
        if let ControlTarget::LayerOpacity(id) = deserialized.target {
            assert_eq!(id, 0);
        } else {
            panic!("Wrong target type");
        }

        if let ControlValue::Float(val) = deserialized.value {
            assert_eq!(val, 0.75);
        } else {
            panic!("Wrong value type");
        }
    }
}
