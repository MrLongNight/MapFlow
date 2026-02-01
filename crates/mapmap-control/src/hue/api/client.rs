use super::error::HueError;
use crate::hue::models::HueConfig;
use serde::{Deserialize, Serialize};

pub struct HueClient;

#[derive(Serialize)]
struct RegisterBody<'a> {
    devicetype: &'a str,
    generateclientkey: bool,
}

#[derive(Deserialize)]
struct RegisterSuccess {
    username: String,
    clientkey: String,
}

#[derive(Deserialize)]
struct HueErrorResponse {
    #[serde(rename = "type")]
    error_type: i32,
    description: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RegisterResponseItem {
    Success { success: RegisterSuccess },
    Error { error: HueErrorResponse },
}

impl HueClient {
    /// Registers a new application with the Hue Bridge.
    /// Returns a HueConfig with username and client_key.
    /// Note: application_id must be fetched separately via get_application_id().
    pub async fn register_user(ip: &str, devicename: &str) -> Result<HueConfig, HueError> {
        let client = reqwest::Client::builder()
            .build()?;

        let body = RegisterBody {
            devicetype: devicename,
            generateclientkey: true,
        };

        let url = format!("https://{}/api", ip);
        let resp = client.post(&url).json(&body).send().await?;

        let items: Vec<RegisterResponseItem> = resp.json().await?;

        if let Some(item) = items.first() {
            match item {
                RegisterResponseItem::Success { success } => {
                    Ok(HueConfig {
                        bridge_ip: ip.to_string(),
                        username: success.username.clone(),
                        client_key: success.clientkey.clone(),
                        application_id: String::new(), // Must be fetched via get_application_id()
                        entertainment_group_id: String::new(),
                    })
                }
                RegisterResponseItem::Error { error } => {
                    if error.error_type == 101 {
                        Err(HueError::LinkButtonNotPressed)
                    } else {
                        Err(HueError::ApiError(error.description.clone()))
                    }
                }
            }
        } else {
            Err(HueError::ApiError(
                "Empty response from Hue Bridge".to_string(),
            ))
        }
    }

    /// Fetches the hue-application-id from the bridge.
    /// This ID is required as the PSK Identity for DTLS streaming.
    ///
    /// The bridge returns the application ID in the response header "hue-application-id"
    /// when calling GET /auth/v1 with the hue-application-key header.
    pub async fn get_application_id(ip: &str, username: &str) -> Result<String, HueError> {
        let client = reqwest::Client::builder()
            .build()?;

        let url = format!("https://{}/auth/v1", ip);
        let resp = client
            .get(&url)
            .header("hue-application-key", username)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(HueError::ApiError(format!(
                "Failed to get application ID: HTTP {}",
                resp.status()
            )));
        }

        // The application ID is in the response header
        let app_id = resp
            .headers()
            .get("hue-application-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                HueError::ApiError("Missing hue-application-id header in response".to_string())
            })?;

        Ok(app_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_parse_register_success() {
        let json = json!([{
            "success": {
                "username": "myuser",
                "clientkey": "mykey"
            }
        }]);

        let items: Vec<RegisterResponseItem> = serde_json::from_value(json).unwrap();
        if let RegisterResponseItem::Success { success } = &items[0] {
            assert_eq!(success.username, "myuser");
            assert_eq!(success.clientkey, "mykey");
        } else {
            panic!("Expected success");
        }
    }

    #[tokio::test]
    async fn test_parse_register_error_101() {
        let json = json!([{
            "error": {
                "type": 101,
                "address": "",
                "description": "link button not pressed"
            }
        }]);

        let items: Vec<RegisterResponseItem> = serde_json::from_value(json).unwrap();
        if let RegisterResponseItem::Error { error } = &items[0] {
            assert_eq!(error.error_type, 101);
        } else {
            panic!("Expected error");
        }
    }
}
