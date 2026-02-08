//! Authentication for web API
//!
//! Provides optional API key authentication for the web control interface.

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use subtle::ConstantTimeEq;

/// Authentication configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,
    /// Stored API key hashes (SHA-256)
    ///
    /// Custom deserializer ensures plain text keys in config files are hashed on load
    #[serde(deserialize_with = "deserialize_keys_hashed")]
    pub api_keys: HashSet<String>,
}

/// Custom deserializer to hash keys on load
fn deserialize_keys_hashed<'de, D>(deserializer: D) -> Result<HashSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let keys: HashSet<String> = HashSet::deserialize(deserializer)?;
    let mut hashed_keys = HashSet::new();
    for key in keys {
        // If it looks like a hash (64 hex chars), assume it's already hashed.
        // Otherwise hash it. Ideally we'd have a flag, but this heuristic supports legacy configs.
        if key.len() == 64 && key.chars().all(|c| c.is_ascii_hexdigit()) {
            hashed_keys.insert(key);
        } else {
            hashed_keys.insert(AuthConfig::hash_key(&key));
        }
    }
    Ok(hashed_keys)
}

impl AuthConfig {
    /// Create a new auth config with authentication disabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an auth config with authentication enabled and provided keys
    pub fn with_keys(keys: Vec<String>) -> Self {
        let mut config = Self {
            enabled: true,
            api_keys: HashSet::new(),
        };
        for key in keys {
            config.add_key(key);
        }
        config
    }

    /// Add an API key (hashes it before storing)
    pub fn add_key(&mut self, key: String) {
        let hash = Self::hash_key(&key);
        self.api_keys.insert(hash);
        self.enabled = true;
    }

    /// Remove an API key (expects the raw key to remove)
    pub fn remove_key(&mut self, key: &str) -> bool {
        let hash = Self::hash_key(key);
        self.api_keys.remove(&hash)
    }

    /// Validate an API key
    pub fn validate(&self, key: &str) -> bool {
        if !self.enabled {
            return true; // No auth required
        }

        let input_hash = Self::hash_key(key);

        // Use constant-time comparison to prevent timing attacks
        let mut is_valid = false;
        for stored_hash in &self.api_keys {
            // Both hashes are hex-encoded SHA-256 (64 chars), so lengths should match.
            // Using subtle::ConstantTimeEq ensures safe comparison.
            if stored_hash.as_bytes().ct_eq(input_hash.as_bytes()).into() {
                is_valid = true;
            }
        }
        is_valid
    }

    /// Check if authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Helper to hash a key
    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Extract API key from various sources
///
/// checks:
/// 1. Authorization header (Bearer token)
/// 2. X-API-Key header
///
/// Query parameters are explicitly NOT supported for security reasons
/// (to prevent API keys from appearing in server logs/browser history).
pub fn extract_api_key(headers: &http::HeaderMap, _query: Option<&str>) -> Option<String> {
    // Try Authorization header first (Bearer token)
    if let Some(auth_header) = headers.get(http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    // Try X-API-Key header
    if let Some(api_key_header) = headers.get("X-API-Key") {
        if let Ok(key) = api_key_header.to_str() {
            return Some(key.to_string());
        }
    }

    // Try Sec-WebSocket-Protocol header (for browser-based WebSockets)
    // Browsers cannot set custom headers on WebSocket upgrade requests,
    // so we use the subprotocol header as a transport for the API key.
    // Format: "other-protocol, mapmap.auth.<TOKEN>"
    if let Some(protocol_header) = headers.get(http::header::SEC_WEBSOCKET_PROTOCOL) {
        if let Ok(protocols) = protocol_header.to_str() {
            for protocol in protocols.split(',') {
                let protocol = protocol.trim();
                if let Some(token) = protocol.strip_prefix("mapmap.auth.") {
                    return Some(token.to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config() {
        let mut config = AuthConfig::new();
        assert!(!config.is_enabled());
        assert!(config.validate("any_key"));

        config.add_key("test_key".to_string());
        assert!(config.is_enabled());
        assert!(config.validate("test_key"));
        assert!(!config.validate("wrong_key"));

        // Verify key is hashed
        assert!(!config.api_keys.contains("test_key"));
        assert!(config.api_keys.iter().any(|h| h.len() == 64)); // SHA-256 hex length
    }

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::AUTHORIZATION,
            "Bearer test_token".parse().unwrap(),
        );

        let key = extract_api_key(&headers, None);
        assert_eq!(key, Some("test_token".to_string()));
    }

    #[test]
    fn test_extract_api_key_header() {
        let mut headers = http::HeaderMap::new();
        headers.insert("X-API-Key", "test_key".parse().unwrap());

        let key = extract_api_key(&headers, None);
        assert_eq!(key, Some("test_key".to_string()));
    }

    #[test]
    fn test_extract_query_param_disabled() {
        let headers = http::HeaderMap::new();
        // Query param extraction should be disabled for security
        let key = extract_api_key(&headers, Some("foo=bar&api_key=test_key"));
        assert_eq!(key, None);
    }

    #[test]
    fn test_extract_websocket_protocol() {
        let mut headers = http::HeaderMap::new();
        // Simulate WebSocket connection from browser: new WebSocket("url", ["mapmap.auth.test_token"])
        headers.insert("Sec-WebSocket-Protocol", "mapmap.auth.test_token".parse().unwrap());

        let key = extract_api_key(&headers, None);
        assert_eq!(key, Some("test_token".to_string()));
    }

    #[test]
    fn test_extract_websocket_protocol_multiple() {
        let mut headers = http::HeaderMap::new();
        // Multiple subprotocols: "mqtt, chat, mapmap.auth.test_token"
        headers.insert("Sec-WebSocket-Protocol", "mqtt, chat, mapmap.auth.test_token".parse().unwrap());

        let key = extract_api_key(&headers, None);
        assert_eq!(key, Some("test_token".to_string()));
    }

    #[test]
    fn test_legacy_config_deserialization() {
        // Simulate a legacy JSON config with plain text keys
        let json = r#"
        {
            "enabled": true,
            "api_keys": ["my_secret_key", "another_key"]
        }
        "#;

        let config: AuthConfig =
            serde_json::from_str(json).expect("Failed to deserialize legacy config");

        // Validation should work against the PLAIN TEXT key (because it was hashed on load)
        assert!(config.validate("my_secret_key"));
        assert!(config.validate("another_key"));
        assert!(!config.validate("wrong_key"));

        // Internal storage should be hashed
        assert!(!config.api_keys.contains("my_secret_key"));
        assert!(config.api_keys.iter().any(|k| k.len() == 64));
    }

    #[test]
    fn test_hashed_config_deserialization() {
        // Simulate a config that already has hashed keys
        let secret = "my_secret_key";
        let hash = AuthConfig::hash_key(secret);
        let json = format!(
            r#"
        {{
            "enabled": true,
            "api_keys": ["{}"]
        }}
        "#,
            hash
        );

        let config: AuthConfig =
            serde_json::from_str(&json).expect("Failed to deserialize hashed config");

        // Validation should still work
        assert!(config.validate(secret));

        // Internal storage should match the input hash (not double-hashed)
        assert!(config.api_keys.contains(&hash));
    }
}
