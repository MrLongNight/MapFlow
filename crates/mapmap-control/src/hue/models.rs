use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HueConfig {
    pub bridge_ip: String,
    pub username: String,       // Used as "hue-application-key" in REST headers
    pub client_key: String,     // Used as PSK for DTLS encryption
    pub application_id: String, // Used as PSK Identity for DTLS (from /auth/v1)
    pub entertainment_group_id: String,
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




