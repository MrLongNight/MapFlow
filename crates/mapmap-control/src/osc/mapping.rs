//! OSC Address Mapping
//!
//! This module provides a persistent mapping between OSC addresses and `ControlTarget`s.

use crate::target::ControlTarget;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A map from OSC addresses to `ControlTarget`s.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OscMapping {
    pub map: HashMap<String, ControlTarget>,
}

impl OscMapping {
    /// Create a new, empty `OscMapping`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new mapping.
    pub fn add_mapping(&mut self, address: String, target: ControlTarget) {
        self.map.insert(address, target);
    }

    /// Remove a mapping.
    pub fn remove_mapping(&mut self, address: &str) {
        self.map.remove(address);
    }

    /// Get the `ControlTarget` for a given OSC address.
    pub fn get_target(&self, address: &str) -> Option<&ControlTarget> {
        self.map.get(address)
    }

    /// Load mappings from a JSON file.
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let data = fs::read_to_string(path)?;
        let loaded_map: OscMapping = serde_json::from_str(&data)?;
        self.map = loaded_map.map;
        Ok(())
    }

    /// Save mappings to a JSON file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osc_mapping_add_remove() {
        let mut mapping = OscMapping::new();
        let address = "/mapmap/master/opacity".to_string();
        let target = ControlTarget::MasterOpacity;

        mapping.add_mapping(address.clone(), target.clone());
        assert_eq!(mapping.get_target(&address), Some(&target));

        mapping.remove_mapping(&address);
        assert_eq!(mapping.get_target(&address), None);
    }

    #[test]
    fn test_osc_mapping_save_load() {
        let mut mapping = OscMapping::new();
        let address = "/mapmap/master/opacity".to_string();
        let target = ControlTarget::MasterOpacity;
        mapping.add_mapping(address, target);

        let path = std::env::temp_dir().join("osc_mappings.json");
        mapping.save_to_file(&path).unwrap();

        let mut new_mapping = OscMapping::new();
        new_mapping.load_from_file(&path).unwrap();

        assert_eq!(mapping.map, new_mapping.map);
    }
}
