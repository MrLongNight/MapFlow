use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct HueConfig {
    pub bridge_ip: String,
    pub username: String,       // Used as "hue-application-key" in REST headers
    pub client_key: String,     // Used as PSK for DTLS encryption
    pub application_id: String, // Used as PSK Identity for DTLS (from /auth/v1)
    pub entertainment_group_id: String,
}

impl std::fmt::Debug for HueConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HueConfig")
            .field("bridge_ip", &self.bridge_ip)
            .field("username", &"***REDACTED***")
            .field("client_key", &"***REDACTED***")
            .field("application_id", &self.application_id)
            .field("entertainment_group_id", &self.entertainment_group_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hue_config_debug_redaction() {
        let config = HueConfig {
            bridge_ip: "192.168.1.5".to_string(),
            username: "secret_user_123".to_string(),
            client_key: "secret_key_456".to_string(),
            application_id: "app_789".to_string(),
            entertainment_group_id: "group_001".to_string(),
        };
        let debug_str = format!("{:?}", config);

        // Assert sensitive fields are redacted
        assert!(debug_str.contains("***REDACTED***"));
        assert!(!debug_str.contains("secret_user_123"));
        assert!(!debug_str.contains("secret_key_456"));

        // Assert non-sensitive fields are present
        assert!(debug_str.contains("192.168.1.5"));
        assert!(debug_str.contains("app_789"));
        assert!(debug_str.contains("group_001"));
    }
}

/// Represents a light channel in an entertainment configuration.
/// Note: `channel_id` is the streaming ID (0, 1, 2...), NOT the light's REST API ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightNode {
    pub id: String,     // REST API light ID (for reference)
    pub channel_id: u8, // Streaming channel ID (0-based index for DTLS messages)
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
