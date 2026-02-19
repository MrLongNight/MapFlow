use super::error::HueError;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone)]
pub struct DiscoveredBridge {
    #[serde(rename = "internalipaddress")]
    pub ip: String,
    pub id: String,
}

/// Discover Hue Bridges using the meethue.com N-UPnP API
/// Returns all discovered bridges, sorted by reachability
pub async fn discover_bridges() -> Result<Vec<DiscoveredBridge>, HueError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(HueError::Network)?;

    let resp = client.get("https://discovery.meethue.com").send().await?;

    let devices: Vec<DiscoveredBridge> = resp.json().await?;

    if devices.is_empty() {
        return Err(HueError::DiscoveryFailed);
    }

    // Verify which bridges are actually reachable
    let mut reachable = Vec::new();
    let mut unreachable = Vec::new();

    for device in devices {
        if is_bridge_reachable(&device.ip).await {
            reachable.push(device);
        } else {
            unreachable.push(device);
        }
    }

    // Return reachable bridges first, then unreachable ones
    reachable.extend(unreachable);
    Ok(reachable)
}

/// Check if a bridge is reachable by making a simple HTTP request
async fn is_bridge_reachable(ip: &str) -> bool {
    let client = match Client::builder()
        .timeout(Duration::from_secs(3))
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Try to reach the bridge's description endpoint
    let url = format!("http://{}/api/0/config", ip);
    client.get(&url).send().await.is_ok()
}

/// Legacy function for backwards compatibility - returns first reachable bridge
pub async fn discover_bridge() -> Result<String, HueError> {
    let bridges = discover_bridges().await?;

    // Return the first bridge (which should be the first reachable one)
    bridges
        .first()
        .map(|b| b.ip.clone())
        .ok_or(HueError::DiscoveryFailed)
}
