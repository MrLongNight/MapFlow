//! Effect Recent Configs System
//!
//! Automatically stores the last N configurations for each effect type,
//! allowing quick access to recently used settings like GIMP's preset system.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use tracing::{debug, info};

/// Maximum number of recent configs to keep per effect
pub const MAX_RECENT_CONFIGS: usize = 5;

/// A single effect configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectConfig {
    /// Timestamp when this config was used
    pub timestamp: u64,
    /// Human-readable name (auto-generated or user-set)
    pub name: String,
    /// Effect parameters as key-value pairs
    pub params: HashMap<String, EffectParamValue>,
}

/// Parameter value types for effects
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EffectParamValue {
    /// Float value
    Float(f32),
    /// Integer value
    Int(i32),
    /// Boolean value
    Bool(bool),
    /// vector2 value
    Vec2([f32; 2]),
    /// vector3 value
    Vec3([f32; 3]),
    /// vector4 value
    Vec4([f32; 4]),
    /// Color value (RGBA)
    Color([f32; 4]),
    /// String value
    String(String),
}

impl EffectConfig {
    /// Create a new config with current timestamp
    pub fn new(params: HashMap<String, EffectParamValue>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Generate a descriptive name from parameters
        let name = Self::generate_name(&params);

        Self {
            timestamp,
            name,
            params,
        }
    }

    /// Create with a custom name
    pub fn with_name(name: String, params: HashMap<String, EffectParamValue>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            timestamp,
            name,
            params,
        }
    }

    /// Generate a descriptive name from key parameters
    fn generate_name(params: &HashMap<String, EffectParamValue>) -> String {
        // Try to create a meaningful name from the first few params
        let mut parts: Vec<String> = Vec::new();

        for (key, value) in params.iter().take(3) {
            let short_key = if key.len() > 6 { &key[..6] } else { key };
            let val_str = match value {
                EffectParamValue::Float(v) => format!("{:.1}", v),
                EffectParamValue::Int(v) => format!("{}", v),
                EffectParamValue::Bool(v) => if *v { "on" } else { "off" }.to_string(),
                EffectParamValue::Color(c) => format!(
                    "#{:02x}{:02x}{:02x}",
                    (c[0] * 255.0) as u8,
                    (c[1] * 255.0) as u8,
                    (c[2] * 255.0) as u8
                ),
                _ => "...".to_string(),
            };
            parts.push(format!("{}:{}", short_key, val_str));
        }

        if parts.is_empty() {
            "Default".to_string()
        } else {
            parts.join(" ")
        }
    }

    /// Check if this config matches another (same parameters)
    pub fn matches(&self, other: &EffectConfig) -> bool {
        self.params == other.params
    }
}

/// Recent configs manager for a single effect type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentConfigQueue {
    /// Queue of recent configurations (newest first)
    configs: VecDeque<EffectConfig>,
}

impl RecentConfigQueue {
    /// Create a new empty queue
    pub fn new() -> Self {
        Self {
            configs: VecDeque::new(),
        }
    }

    /// Add a new configuration to the recent list
    ///
    /// If the config already exists (same params), it moves to the front.
    /// Otherwise, adds to front and removes oldest if > MAX_RECENT_CONFIGS.
    pub fn add(&mut self, config: EffectConfig) {
        // Check if this config already exists
        if let Some(pos) = self.configs.iter().position(|c| c.matches(&config)) {
            // Move to front
            self.configs.remove(pos);
        }

        // Add to front
        self.configs.push_front(config);

        // Trim to max size
        while self.configs.len() > MAX_RECENT_CONFIGS {
            self.configs.pop_back();
        }
    }

    /// Get all recent configs (newest first)
    pub fn get_all(&self) -> &VecDeque<EffectConfig> {
        &self.configs
    }

    /// Get a config by index (0 = most recent)
    pub fn get(&self, index: usize) -> Option<&EffectConfig> {
        self.configs.get(index)
    }

    /// Get the most recent config
    pub fn most_recent(&self) -> Option<&EffectConfig> {
        self.configs.front()
    }

    /// Clear all recent configs
    pub fn clear(&mut self) {
        self.configs.clear();
    }

    /// Number of stored configs
    pub fn len(&self) -> usize {
        self.configs.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.configs.is_empty()
    }
}

/// Global recent configs manager for all effect types
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentEffectConfigs {
    /// Recent configs per effect type (keyed by effect type name)
    configs: HashMap<String, RecentConfigQueue>,
    /// Path to persist configs
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

impl RecentEffectConfigs {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            config_path: None,
        }
    }

    /// Create with persistence path
    pub fn with_persistence(path: PathBuf) -> Self {
        let mut manager = Self::load_from_path(&path).unwrap_or_default();
        manager.config_path = Some(path);
        manager
    }

    /// Add a config for an effect type
    pub fn add_config(&mut self, effect_type: &str, config: EffectConfig) {
        debug!("Adding recent config for effect: {}", effect_type);

        self.configs
            .entry(effect_type.to_string())
            .or_default()
            .add(config);

        // Auto-save if persistence is enabled
        if let Some(path) = &self.config_path {
            let _ = self.save_to_path(path);
        }
    }

    /// Add a config using only float parameters (for UI compatibility)
    pub fn add_float_config(&mut self, effect_type: &str, params: HashMap<String, f32>) {
        let mut converted = HashMap::new();
        for (k, v) in params {
            converted.insert(k, EffectParamValue::Float(v));
        }
        // Need a dummy config or generate one
        let config = EffectConfig::new(converted);
        self.add_config(effect_type, config);
    }

    /// Get recent configs for an effect type
    pub fn get_configs(&self, effect_type: &str) -> Option<&RecentConfigQueue> {
        self.configs.get(effect_type)
    }

    /// Get the most recent config for an effect type
    pub fn get_most_recent(&self, effect_type: &str) -> Option<&EffectConfig> {
        self.configs.get(effect_type)?.most_recent()
    }

    /// Get all recent configs for an effect type
    pub fn get_recent(&self, effect_type: &str) -> Vec<EffectConfig> {
        if let Some(queue) = self.configs.get(effect_type) {
            queue.get_all().iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Apply a recent config by index
    pub fn get_by_index(&self, effect_type: &str, index: usize) -> Option<&EffectConfig> {
        self.configs.get(effect_type)?.get(index)
    }

    /// List all effect types with recent configs
    pub fn effect_types(&self) -> Vec<&str> {
        self.configs.keys().map(|s| s.as_str()).collect()
    }

    /// Clear all configs for an effect type
    pub fn clear_effect(&mut self, effect_type: &str) {
        self.configs.remove(effect_type);
    }

    /// Clear all recent configs
    pub fn clear_all(&mut self) {
        self.configs.clear();
    }

    /// Load from a JSON file
    pub fn load_from_path(path: &PathBuf) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save to a JSON file
    pub fn save_to_path(&self, path: &PathBuf) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;
        info!("Saved recent effect configs to {:?}", path);
        Ok(())
    }

    /// Save to the persisted path if set
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(path) = &self.config_path {
            self.save_to_path(path)
        } else {
            Ok(())
        }
    }

    /// Get the default config path (in user data directory)
    pub fn default_config_path() -> Option<PathBuf> {
        dirs::data_dir().map(|p| p.join("MapFlow").join("recent_effect_configs.json"))
    }
}

/// Helper to create a config from common effect parameters
pub fn create_blur_config(radius: f32, sigma: f32) -> EffectConfig {
    let mut params = HashMap::new();
    params.insert("radius".to_string(), EffectParamValue::Float(radius));
    params.insert("sigma".to_string(), EffectParamValue::Float(sigma));
    EffectConfig::new(params)
}

/// Helper to create a color config
pub fn create_color_config(hue: f32, saturation: f32, brightness: f32) -> EffectConfig {
    let mut params = HashMap::new();
    params.insert("hue".to_string(), EffectParamValue::Float(hue));
    params.insert(
        "saturation".to_string(),
        EffectParamValue::Float(saturation),
    );
    params.insert(
        "brightness".to_string(),
        EffectParamValue::Float(brightness),
    );
    EffectConfig::new(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_config_queue() {
        let mut queue = RecentConfigQueue::new();

        // Add configs
        for i in 0..7 {
            let mut params = HashMap::new();
            params.insert("value".to_string(), EffectParamValue::Float(i as f32));
            queue.add(EffectConfig::new(params));
        }

        // Should only keep MAX_RECENT_CONFIGS
        assert_eq!(queue.len(), MAX_RECENT_CONFIGS);

        // Most recent should be the last added (6.0)
        let recent = queue.most_recent().unwrap();
        assert_eq!(
            recent.params.get("value"),
            Some(&EffectParamValue::Float(6.0))
        );
    }

    #[test]
    fn test_duplicate_detection() {
        let mut queue = RecentConfigQueue::new();

        let mut params = HashMap::new();
        params.insert("value".to_string(), EffectParamValue::Float(1.0));

        let config1 = EffectConfig::new(params.clone());
        let config2 = EffectConfig::new(params.clone());

        queue.add(config1);
        queue.add(config2);

        // Should still be length 1 (duplicate moved to front)
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_effect_configs_manager() {
        let mut manager = RecentEffectConfigs::new();

        manager.add_config("blur", create_blur_config(5.0, 2.0));
        manager.add_config("blur", create_blur_config(10.0, 3.0));
        manager.add_config("color", create_color_config(0.5, 1.0, 0.8));

        assert_eq!(manager.get_configs("blur").unwrap().len(), 2);
        assert_eq!(manager.get_configs("color").unwrap().len(), 1);

        // Most recent blur should be radius=10
        let recent = manager.get_most_recent("blur").unwrap();
        assert_eq!(
            recent.params.get("radius"),
            Some(&EffectParamValue::Float(10.0))
        );
    }
}




