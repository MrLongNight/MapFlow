use super::error::HueError;
use crate::hue::models::{HueConfig, LightNode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub id: String, // v2 API UUID (for stream activation and DTLS streaming)
    pub name: String,
    pub lights: Vec<LightNode>,
}

// V2 API structures
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct V2Response<T> {
    data: Vec<T>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct V2EntertainmentConfig {
    id: String,
    metadata: V2Metadata,
    channels: Vec<V2Channel>,
    #[serde(default)]
    status: String,
}

#[derive(Deserialize, Debug)]
struct V2Metadata {
    name: String,
}

#[derive(Deserialize, Debug)]
struct V2Channel {
    channel_id: u8,
    position: V2Position,
    #[serde(default)]
    members: Vec<V2ChannelMember>,
}

#[derive(Deserialize, Debug)]
struct V2Position {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Deserialize, Debug, Default)]
struct V2ChannelMember {
    service: Option<V2ServiceRef>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct V2ServiceRef {
    rid: String,
    rtype: String,
}

#[derive(Serialize)]
struct StreamAction {
    action: String,
}

// Helper to build a client for communicating with the Hue Bridge
fn build_client() -> Result<reqwest::Client, HueError> {
    reqwest::Client::builder()
        .build()
        .map_err(HueError::Network)
}

/// Fetches entertainment configurations from the v2 API.
/// Returns groups with proper channel_id mapping for streaming.
pub async fn get_entertainment_groups(config: &HueConfig) -> Result<Vec<GroupInfo>, HueError> {
    let client = build_client()?;

    // Use v2 API to get entertainment configurations with channels
    let url = format!(
        "https://{}/clip/v2/resource/entertainment_configuration",
        config.bridge_ip
    );

    let resp = client
        .get(&url)
        .header("hue-application-key", &config.username)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(HueError::ApiError(format!(
            "Failed to get entertainment configurations: HTTP {}",
            resp.status()
        )));
    }

    let v2_response: V2Response<V2EntertainmentConfig> = resp.json().await?;

    let mut result = Vec::new();

    for cfg in v2_response.data {
        let mut lights = Vec::new();

        for channel in &cfg.channels {
            // Get light ID from channel members if available
            let light_id = channel
                .members
                .first()
                .and_then(|m| m.service.as_ref())
                .map(|s| s.rid.clone())
                .unwrap_or_else(|| format!("channel_{}", channel.channel_id));

            lights.push(LightNode {
                id: light_id,
                channel_id: channel.channel_id,
                x: channel.position.x,
                y: channel.position.y,
                z: channel.position.z,
            });
        }

        result.push(GroupInfo {
            id: cfg.id,
            name: cfg.metadata.name,
            lights,
        });
    }

    Ok(result)
}

/// Activates or deactivates streaming for an entertainment configuration.
/// Uses the v2 API with {"action": "start"} or {"action": "stop"}.
pub async fn set_stream_active(
    config: &HueConfig,
    entertainment_config_id: &str,
    active: bool,
) -> Result<(), HueError> {
    let client = build_client()?;

    let url = format!(
        "https://{}/clip/v2/resource/entertainment_configuration/{}",
        config.bridge_ip, entertainment_config_id
    );

    let body = StreamAction {
        action: if active {
            "start".to_string()
        } else {
            "stop".to_string()
        },
    };

    let resp = client
        .put(&url)
        .header("hue-application-key", &config.username)
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let response_text = resp.text().await?;

    if !status.is_success() {
        return Err(HueError::ApiError(format!(
            "Failed to {} stream: HTTP {} - {}",
            if active { "start" } else { "stop" },
            status,
            response_text
        )));
    }

    // Check for error in response body
    if response_text.contains("\"error\"") {
        return Err(HueError::ApiError(format!(
            "Failed to {} stream: {}",
            if active { "start" } else { "stop" },
            response_text
        )));
    }

    Ok(())
}

/// Flash a light using the v1 API (for testing connectivity)
pub async fn flash_light(config: &HueConfig, light_id: &str) -> Result<(), HueError> {
    let client = build_client()?;
    let url = format!("https://{}/api/lights/{}/state", config.bridge_ip, light_id);

    let body = serde_json::json!({
        "alert": "select"
    });

    let resp = client
        .put(&url)
        .header("X-Hue-Username", &config.username)
        .json(&body)
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(HueError::ApiError(format!(
            "Failed to flash light: {}",
            resp.status()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_v2_entertainment_config() {
        let json = json!({
            "data": [{
                "id": "1a8d99cc-967b-44f2-9202-43f976c0fa6b",
                "type": "entertainment_configuration",
                "metadata": { "name": "Entertainment area 1" },
                "configuration_type": "screen",
                "status": "inactive",
                "channels": [
                    {
                        "channel_id": 0,
                        "position": { "x": -0.6, "y": 0.8, "z": 0.0 },
                        "members": []
                    },
                    {
                        "channel_id": 1,
                        "position": { "x": 0.6, "y": 0.8, "z": 0.0 },
                        "members": []
                    }
                ]
            }]
        });

        let response: V2Response<V2EntertainmentConfig> = serde_json::from_value(json).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].id, "1a8d99cc-967b-44f2-9202-43f976c0fa6b");
        assert_eq!(response.data[0].channels.len(), 2);
        assert_eq!(response.data[0].channels[0].channel_id, 0);
        assert_eq!(response.data[0].channels[1].channel_id, 1);
    }
}
