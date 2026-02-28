//! OSC Address Mapping
//!
//! Provides persistent mapping between OSC addresses and ControlTargets.
//! Uses a simple HashMap-based approach with JSON serialization.

use crate::target::ControlTarget;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{error, info};

/// Mapping from OSC addresses to ControlTargets
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OscMapping {
    /// The mapping storage
    pub map: HashMap<String, ControlTarget>,
}

impl OscMapping {
    /// Create a new empty mapping
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a mapping
    pub fn set_mapping(&mut self, address: String, target: ControlTarget) {
        self.map.insert(address, target);
    }

    /// Remove a mapping
    pub fn remove_mapping(&mut self, address: &str) {
        self.map.remove(address);
    }

    /// Get target for address
    pub fn get(&self, address: &str) -> Option<&ControlTarget> {
        self.map.get(address)
    }

    /// Clear all mappings
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Load from JSON file (robust error handling)
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            info!("OSC mapping file not found at {:?}, using defaults", path);
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        match serde_json::from_str::<OscMapping>(&content) {
            Ok(loaded) => {
                self.map = loaded.map;
                info!("Loaded {} OSC mappings from {:?}", self.map.len(), path);
                Ok(())
            }
            Err(e) => {
                error!("Failed to parse OSC mapping file {:?}: {}", path, e);
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            }
        }
    }

    /// Save to JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_operations() {
        let mut mapping = OscMapping::new();
        let addr = "/test/1".to_string();
        let target = ControlTarget::MasterOpacity;

        mapping.set_mapping(addr.clone(), target.clone());
        assert_eq!(mapping.get(&addr), Some(&target));

        mapping.remove_mapping(&addr);
        assert_eq!(mapping.get(&addr), None);
    }

    #[test]
    fn test_serialization() {
        let mut mapping = OscMapping::new();
        mapping.set_mapping("/a".into(), ControlTarget::LayerOpacity(0));
        mapping.set_mapping("/b".into(), ControlTarget::PlaybackSpeed(None));

        let json = serde_json::to_string(&mapping).unwrap();
        let loaded: OscMapping = serde_json::from_str(&json).unwrap();

        assert_eq!(mapping, loaded);
    }

    #[test]
    fn test_save_load_file() {
        let mut mapping = OscMapping::new();
        mapping.set_mapping("/test/save".into(), ControlTarget::MasterBlackout);

        let mut path = std::env::temp_dir();
        path.push("mapmap_test_osc_mapping.json");

        // Ensure cleanup from previous runs
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }

        assert!(mapping.save(&path).is_ok());

        let mut loaded = OscMapping::new();
        assert!(loaded.load(&path).is_ok());

        assert_eq!(mapping, loaded);

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }
}
