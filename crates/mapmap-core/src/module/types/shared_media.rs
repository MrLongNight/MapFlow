use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of shared media
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SharedMediaType {
    /// Enumeration variant.
    Video,
    /// Enumeration variant.
    Image,
}

/// A shared media resource entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedMediaItem {
    /// Unique identifier for this entity.
    pub id: String,
    /// File path to asset.
    pub path: String,
    /// Component property or field.
    pub media_type: SharedMediaType,
}

/// Registry for shared media resources
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SharedMediaState {
    /// Component property or field.
    pub items: HashMap<String, SharedMediaItem>,
}

impl SharedMediaState {
    /// Associated function.
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Method implementation.
    pub fn register(&mut self, id: String, path: String, media_type: SharedMediaType) {
        self.items.insert(
            id.clone(),
            SharedMediaItem {
                id,
                path,
                media_type,
            },
        );
    }

    /// Method implementation.
    pub fn get(&self, id: &str) -> Option<&SharedMediaItem> {
        self.items.get(id)
    }

    /// Method implementation.
    pub fn unregister(&mut self, id: &str) {
        self.items.remove(id);
    }
}
