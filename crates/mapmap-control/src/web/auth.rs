//! Authentication for web API
//!
//! Provides optional API key authentication for the web control interface.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

const HASH_PREFIX: &str = "$sha256$";

/// Authentication configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,
    /// API keys (stored as SHA-256 hashes with $sha256$ prefix)
    #[serde(
        serialize_with = "serialize_api_keys",
        deserialize_with = "deserialize_api_keys"
    )]
    pub api_keys: HashSet<String>,
}

impl AuthConfig {
    /// Create a new auth config with authentication disabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an auth config with authentication enabled
    pub fn with_keys(keys: Vec<String>) -> Self {
        let hashed_keys = keys.iter().map(|k| Self::hash_key(k)).collect();
        Self {
            enabled: true,
            api_keys: hashed_keys,
        }
    }

    /// Add an API key (will be hashed before storage)
    pub fn add_key(&mut self, key: String) {
        self.api_keys.insert(Self::hash_key(&key));
        self.enabled = true;
    }

    /// Remove an API key
    pub fn remove_key(&mut self, key: &str) -> bool {
        self.api_keys.remove(&Self::hash_key(key))
    }

    /// Validate an API key
    pub fn validate(&self, key: &str) -> bool {
        if !self.enabled {
            return true; // No auth required
        }

        self.api_keys.contains(&Self::hash_key(key))
    }

    /// Check if authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Compute SHA-256 hash of the key and prepend prefix
    ///
    /// This ALWAYS hashes the input. It does NOT check for existing prefixes.
    /// This prevents pass-the-hash attacks where an attacker sends the hash as the password.
    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{}{}", HASH_PREFIX, hex::encode(hasher.finalize()))
    }
}

/// Custom serializer for API keys (just standard serialization)
fn serialize_api_keys<S>(keys: &HashSet<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // We just serialize the set as is, since it contains the hashes
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(keys.len()))?;
    for key in keys {
        seq.serialize_element(key)?;
    }
    seq.end()
}

/// Custom deserializer to handle migration from plaintext to hash
fn deserialize_api_keys<'de, D>(deserializer: D) -> Result<HashSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_keys: Vec<String> = Vec::deserialize(deserializer)?;
    let mut processed_keys = HashSet::new();

    for key in raw_keys {
        if key.starts_with(HASH_PREFIX) {
            // Already hashed - keep as is
            processed_keys.insert(key);
        } else {
            // Legacy plaintext - migrate to hash
            // We duplicate logic here intentionally to avoid exposing a "conditional hash" method
            // that could be misused in validation.
            let mut hasher = Sha256::new();
            hasher.update(key.as_bytes());
            let hash = format!("{}{}", HASH_PREFIX, hex::encode(hasher.finalize()));
            processed_keys.insert(hash);
        }
    }

    Ok(processed_keys)
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
    }

    #[test]
    fn test_auth_config_with_keys() {
        let config = AuthConfig::with_keys(vec!["key1".to_string(), "key2".to_string()]);
        assert!(config.is_enabled());
        assert!(config.validate("key1"));
        assert!(config.validate("key2"));
        assert!(!config.validate("key3"));
    }

    #[test]
    fn test_remove_key() {
        let mut config = AuthConfig::new();
        config.add_key("test_key".to_string());
        assert!(config.validate("test_key"));

        let removed = config.remove_key("test_key");
        assert!(removed);
        assert!(!config.validate("test_key"));
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
    fn test_hashing_format() {
        let key = "secret_password";
        let hash = AuthConfig::hash_key(key);
        assert!(hash.starts_with(HASH_PREFIX));
        assert_eq!(hash.len(), HASH_PREFIX.len() + 64);
    }

    #[test]
    fn test_pass_the_hash_prevention() {
        // Test that sending the hash itself as the password fails
        let mut config = AuthConfig::new();
        let secret = "my_secret_password";
        config.add_key(secret.to_string());

        // Get the actual stored hash
        let stored_hash = AuthConfig::hash_key(secret);

        // Authenticating with the secret should work
        assert!(config.validate(secret));

        // Authenticating with the hash should FAIL
        // If the code was vulnerable, this would pass because hash_key would return stored_hash unchanged
        assert!(!config.validate(&stored_hash));
    }

    #[test]
    fn test_migration_deserialization() {
        // Simulate legacy JSON with plaintext keys
        let json = r#"
        {
            "enabled": true,
            "api_keys": ["legacy_secret", "another_secret"]
        }
        "#;

        let config: AuthConfig = serde_json::from_str(json).unwrap();

        // Validation should work because deserializer migrated them
        assert!(config.validate("legacy_secret"));
        assert!(config.validate("another_secret"));
        assert!(!config.validate("wrong_secret"));

        // Check internal storage is hashed
        for key in &config.api_keys {
            assert!(key.starts_with(HASH_PREFIX));
        }
    }

    #[test]
    fn test_hashed_deserialization() {
        // Create a config, serialize it, then deserialize it
        let mut original = AuthConfig::new();
        original.add_key("my_secret".to_string());

        let json = serde_json::to_string(&original).unwrap();

        // Ensure serialization stored the hash (and prefix)
        assert!(json.contains(HASH_PREFIX));

        let loaded: AuthConfig = serde_json::from_str(&json).unwrap();
        assert!(loaded.validate("my_secret"));

        // Ensure no double hashing happened
        assert_eq!(original.api_keys, loaded.api_keys);
    }
}
