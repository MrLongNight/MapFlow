//! Authentication for web API
//!
//! Provides optional API key authentication for the web control interface.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Authentication configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,
    /// API keys (plain text for simplicity; use hashed keys in production)
    pub api_keys: HashSet<String>,
}

impl AuthConfig {
    /// Create a new auth config with authentication disabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an auth config with authentication enabled
    pub fn with_keys(keys: Vec<String>) -> Self {
        Self {
            enabled: true,
            api_keys: keys.into_iter().collect(),
        }
    }

    /// Add an API key
    pub fn add_key(&mut self, key: String) {
        self.api_keys.insert(key);
        self.enabled = true;
    }

    /// Remove an API key
    pub fn remove_key(&mut self, key: &str) -> bool {
        self.api_keys.remove(key)
    }

    /// Validate an API key
    pub fn validate(&self, key: &str) -> bool {
        if !self.enabled {
            return true; // No auth required
        }

        // Use constant-time comparison to prevent timing attacks
        let mut is_valid = false;
        for stored_key in &self.api_keys {
            if constant_time_eq(stored_key, key) {
                is_valid = true;
            }
        }
        is_valid
    }

    /// Check if authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
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

    None
}

/// Constant-time string comparison to mitigate timing attacks
///
/// Returns true if the two strings are identical.
/// Note: This comparison runs in time proportional to the length of `a` (the stored secret).
/// This prevents attackers from guessing the length of the secret by observing the time taken
/// to verify an invalid key of a certain length.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    // Initialize result based on length comparison.
    // If lengths differ, start with 1 (fail).
    // Note: We deliberately do NOT return early here to ensure constant execution time
    // dependent only on the stored key length.
    let mut result = if a.len() != b.len() { 1u8 } else { 0u8 };

    for (i, &byte) in a_bytes.iter().enumerate() {
        // Safe access to b's bytes. If b is shorter, use 0 (which likely mismatches).
        // This keeps the loop running for exactly a.len() iterations regardless of b's length.
        let b_byte = *b_bytes.get(i).unwrap_or(&0);
        result |= byte ^ b_byte;
    }

    result == 0
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
    fn test_constant_time_eq() {
        assert!(constant_time_eq("secret", "secret"));
        assert!(!constant_time_eq("secret", "secreT"));
        assert!(!constant_time_eq("secret", "public"));
        assert!(!constant_time_eq("secret", "secret1"));
        assert!(!constant_time_eq("secret", "secre"));
        // New case: b is longer
        assert!(!constant_time_eq("secret", "secret_long"));
        assert!(!constant_time_eq("", "secret"));
        assert!(constant_time_eq("", ""));
    }
}
