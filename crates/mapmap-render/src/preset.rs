//! Effect Chain Preset System
//!
//! Save/Load Effect Chain configurations as JSON files.
//! Provides preset management with categories and favorites.

use mapmap_core::{EffectChain, EffectType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors that can occur when working with presets
#[derive(Error, Debug)]
pub enum PresetError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Preset not found: {0}")]
    NotFound(String),

    #[error("Invalid preset path: {0}")]
    InvalidPath(String),

    #[error("Preset directory does not exist: {0}")]
    DirectoryNotFound(PathBuf),
}

/// Result type for preset operations
pub type Result<T> = std::result::Result<T, PresetError>;

/// Metadata for a preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetMetadata {
    /// Preset name
    pub name: String,
    /// Author name
    pub author: String,
    /// Description
    pub description: String,
    /// Category (e.g. "Color", "Distortion", "Film")
    pub category: String,
    /// Tags for searching
    pub tags: Vec<String>,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Preset version
    pub version: String,
    /// Whether this is a favorite
    pub is_favorite: bool,
}

impl Default for PresetMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            name: "Untitled Preset".to_string(),
            author: String::new(),
            description: String::new(),
            category: "Uncategorized".to_string(),
            tags: Vec::new(),
            created_at: now,
            modified_at: now,
            version: "1.0".to_string(),
            is_favorite: false,
        }
    }
}

/// A complete preset file containing metadata and effect chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectPreset {
    /// Preset metadata
    pub metadata: PresetMetadata,
    /// The effect chain configuration
    pub chain: EffectChain,
}

impl EffectPreset {
    /// Create a new preset from an effect chain
    pub fn new(name: &str, chain: EffectChain) -> Self {
        let metadata = PresetMetadata {
            name: name.to_string(),
            ..Default::default()
        };

        Self { metadata, chain }
    }

    /// Create a preset with full metadata
    pub fn with_metadata(metadata: PresetMetadata, chain: EffectChain) -> Self {
        Self { metadata, chain }
    }

    /// Save preset to a JSON file
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        info!("Saved preset to {:?}", path);
        Ok(())
    }

    /// Load preset from a JSON file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let preset: EffectPreset = serde_json::from_str(&content)?;
        debug!("Loaded preset: {}", preset.metadata.name);
        Ok(preset)
    }

    /// Update the modified timestamp
    pub fn touch(&mut self) {
        self.metadata.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

/// Preset library manager
pub struct PresetLibrary {
    /// Base directory for presets
    preset_dir: PathBuf,
    /// Cached presets (path -> preset)
    cache: HashMap<PathBuf, EffectPreset>,
    /// Categories found in presets
    categories: Vec<String>,
}

impl PresetLibrary {
    /// Create a new preset library with the given base directory
    pub fn new(preset_dir: PathBuf) -> Result<Self> {
        // Create directory if it doesn't exist
        if !preset_dir.exists() {
            fs::create_dir_all(&preset_dir)?;
            info!("Created preset directory: {:?}", preset_dir);
        }

        let mut library = Self {
            preset_dir,
            cache: HashMap::new(),
            categories: Vec::new(),
        };

        // Scan for existing presets
        library.refresh()?;

        Ok(library)
    }

    /// Get the preset directory path
    pub fn preset_dir(&self) -> &Path {
        &self.preset_dir
    }

    /// Refresh the preset cache by scanning the directory
    pub fn refresh(&mut self) -> Result<()> {
        self.cache.clear();
        self.categories.clear();

        let mut categories_set = std::collections::HashSet::new();

        // Scan preset directory
        if let Ok(entries) = fs::read_dir(&self.preset_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // Scan subdirectories (categories)
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.extension().is_some_and(|ext| ext == "json") {
                                match EffectPreset::load(&sub_path) {
                                    Ok(preset) => {
                                        categories_set.insert(preset.metadata.category.clone());
                                        self.cache.insert(sub_path, preset);
                                    }
                                    Err(e) => {
                                        warn!("Failed to load preset {:?}: {}", sub_path, e);
                                    }
                                }
                            }
                        }
                    }
                } else if path.extension().is_some_and(|ext| ext == "json") {
                    // Check if it's a JSON file
                    match EffectPreset::load(&path) {
                        Ok(preset) => {
                            categories_set.insert(preset.metadata.category.clone());
                            self.cache.insert(path, preset);
                        }
                        Err(e) => {
                            warn!("Failed to load preset {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        self.categories = categories_set.into_iter().collect();
        self.categories.sort();

        info!(
            "Loaded {} presets in {} categories",
            self.cache.len(),
            self.categories.len()
        );
        Ok(())
    }

    /// Get all presets
    pub fn all_presets(&self) -> impl Iterator<Item = (&PathBuf, &EffectPreset)> {
        self.cache.iter()
    }

    /// Get presets by category
    pub fn presets_by_category(&self, category: &str) -> Vec<(&PathBuf, &EffectPreset)> {
        self.cache
            .iter()
            .filter(|(_, p)| p.metadata.category == category)
            .collect()
    }

    /// Get favorite presets
    pub fn favorites(&self) -> Vec<(&PathBuf, &EffectPreset)> {
        self.cache
            .iter()
            .filter(|(_, p)| p.metadata.is_favorite)
            .collect()
    }

    /// Search presets by name or tags
    pub fn search(&self, query: &str) -> Vec<(&PathBuf, &EffectPreset)> {
        let query_lower = query.to_lowercase();
        self.cache
            .iter()
            .filter(|(_, p)| {
                p.metadata.name.to_lowercase().contains(&query_lower)
                    || p.metadata
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
                    || p.metadata.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get all categories
    pub fn categories(&self) -> &[String] {
        &self.categories
    }

    /// Save a preset to the library
    pub fn save_preset(&mut self, preset: &EffectPreset) -> Result<PathBuf> {
        // Create category subdirectory if needed
        let category_dir = self.preset_dir.join(&preset.metadata.category);
        if !category_dir.exists() {
            fs::create_dir_all(&category_dir)?;
        }

        // Generate filename from name
        let filename = format!(
            "{}.json",
            preset
                .metadata
                .name
                .to_lowercase()
                .replace(' ', "_")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
        );

        let path = category_dir.join(filename);
        preset.save(&path)?;

        // Update cache
        self.cache.insert(path.clone(), preset.clone());

        // Update categories
        if !self.categories.contains(&preset.metadata.category) {
            self.categories.push(preset.metadata.category.clone());
            self.categories.sort();
        }

        Ok(path)
    }

    /// Delete a preset
    pub fn delete_preset(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(PresetError::NotFound(path.display().to_string()));
        }

        fs::remove_file(path)?;
        self.cache.remove(path);

        info!("Deleted preset: {:?}", path);
        Ok(())
    }

    /// Toggle favorite status for a preset
    pub fn toggle_favorite(&mut self, path: &Path) -> Result<bool> {
        if let Some(preset) = self.cache.get_mut(path) {
            preset.metadata.is_favorite = !preset.metadata.is_favorite;
            preset.touch();
            preset.save(path)?;
            Ok(preset.metadata.is_favorite)
        } else {
            Err(PresetError::NotFound(path.display().to_string()))
        }
    }

    /// Get a preset by path
    pub fn get(&self, path: &Path) -> Option<&EffectPreset> {
        self.cache.get(path)
    }

    /// Create factory presets (built-in presets)
    pub fn create_factory_presets(&mut self) -> Result<()> {
        let factory_dir = self.preset_dir.join("Factory");
        if !factory_dir.exists() {
            fs::create_dir_all(&factory_dir)?;
        }

        // Cinema Look
        let mut cinema_chain = EffectChain::new();
        let color_id = cinema_chain.add_effect(EffectType::ColorAdjust);
        let vignette_id = cinema_chain.add_effect(EffectType::Vignette);
        let grain_id = cinema_chain.add_effect(EffectType::FilmGrain);

        if let Some(effect) = cinema_chain.get_effect_mut(color_id) {
            effect.set_param("contrast", 1.2);
            effect.set_param("saturation", 0.9);
        }
        if let Some(effect) = cinema_chain.get_effect_mut(vignette_id) {
            effect.set_param("radius", 0.6);
            effect.set_param("softness", 0.4);
            effect.intensity = 0.7;
        }
        if let Some(effect) = cinema_chain.get_effect_mut(grain_id) {
            effect.set_param("amount", 0.05);
            effect.intensity = 0.5;
        }

        let cinema_preset = EffectPreset::with_metadata(
            PresetMetadata {
                name: "Cinema Look".to_string(),
                author: "MapFlow".to_string(),
                description: "Classic cinematic color grading with vignette and subtle grain"
                    .to_string(),
                category: "Factory".to_string(),
                tags: vec![
                    "cinema".to_string(),
                    "film".to_string(),
                    "color".to_string(),
                ],
                is_favorite: false,
                ..Default::default()
            },
            cinema_chain,
        );
        cinema_preset.save(&factory_dir.join("cinema_look.json"))?;

        // Retro VHS
        let mut vhs_chain = EffectChain::new();
        let chroma_id = vhs_chain.add_effect(EffectType::ChromaticAberration);
        let pixel_id = vhs_chain.add_effect(EffectType::Pixelate);
        let grain_id = vhs_chain.add_effect(EffectType::FilmGrain);

        if let Some(effect) = vhs_chain.get_effect_mut(chroma_id) {
            effect.set_param("amount", 0.02);
        }
        if let Some(effect) = vhs_chain.get_effect_mut(pixel_id) {
            effect.set_param("pixel_size", 2.0);
            effect.intensity = 0.3;
        }
        if let Some(effect) = vhs_chain.get_effect_mut(grain_id) {
            effect.set_param("amount", 0.15);
        }

        let vhs_preset = EffectPreset::with_metadata(
            PresetMetadata {
                name: "Retro VHS".to_string(),
                author: "MapFlow".to_string(),
                description: "90s VHS aesthetic with chromatic aberration and heavy grain"
                    .to_string(),
                category: "Factory".to_string(),
                tags: vec![
                    "retro".to_string(),
                    "vhs".to_string(),
                    "vintage".to_string(),
                ],
                is_favorite: false,
                ..Default::default()
            },
            vhs_chain,
        );
        vhs_preset.save(&factory_dir.join("retro_vhs.json"))?;

        // Dreamy Blur
        let mut dreamy_chain = EffectChain::new();
        let blur_id = dreamy_chain.add_effect(EffectType::Blur);
        let color_id = dreamy_chain.add_effect(EffectType::ColorAdjust);

        if let Some(effect) = dreamy_chain.get_effect_mut(blur_id) {
            effect.set_param("radius", 3.0);
            effect.intensity = 0.4;
        }
        if let Some(effect) = dreamy_chain.get_effect_mut(color_id) {
            effect.set_param("brightness", 0.1);
            effect.set_param("saturation", 1.2);
        }

        let dreamy_preset = EffectPreset::with_metadata(
            PresetMetadata {
                name: "Dreamy".to_string(),
                author: "MapFlow".to_string(),
                description: "Soft dreamy look with gentle blur and enhanced colors".to_string(),
                category: "Factory".to_string(),
                tags: vec!["dreamy".to_string(), "soft".to_string(), "blur".to_string()],
                is_favorite: false,
                ..Default::default()
            },
            dreamy_chain,
        );
        dreamy_preset.save(&factory_dir.join("dreamy.json"))?;

        // Edge Glow
        let mut edge_chain = EffectChain::new();
        let edge_id = edge_chain.add_effect(EffectType::EdgeDetect);
        let invert_id = edge_chain.add_effect(EffectType::Invert);

        if let Some(effect) = edge_chain.get_effect_mut(edge_id) {
            effect.intensity = 1.0;
        }
        if let Some(effect) = edge_chain.get_effect_mut(invert_id) {
            effect.intensity = 0.5;
        }

        let edge_preset = EffectPreset::with_metadata(
            PresetMetadata {
                name: "Neon Edges".to_string(),
                author: "MapFlow".to_string(),
                description: "Glowing edge detection for a neon-like effect".to_string(),
                category: "Factory".to_string(),
                tags: vec!["edge".to_string(), "neon".to_string(), "glow".to_string()],
                is_favorite: false,
                ..Default::default()
            },
            edge_chain,
        );
        edge_preset.save(&factory_dir.join("neon_edges.json"))?;

        // Refresh to load the new presets
        self.refresh()?;

        info!("Created factory presets");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_preset_create_and_save() {
        let mut chain = EffectChain::new();
        chain.add_effect(EffectType::Blur);
        chain.add_effect(EffectType::ColorAdjust);

        let preset = EffectPreset::new("Test Preset", chain);

        assert_eq!(preset.metadata.name, "Test Preset");
        assert_eq!(preset.chain.effects.len(), 2);
    }

    #[test]
    fn test_preset_serialization() {
        let mut chain = EffectChain::new();
        let blur_id = chain.add_effect(EffectType::Blur);
        if let Some(effect) = chain.get_effect_mut(blur_id) {
            effect.set_param("radius", 10.0);
        }

        let preset = EffectPreset::new("Serialize Test", chain);

        let json = serde_json::to_string(&preset).unwrap();
        let loaded: EffectPreset = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.metadata.name, "Serialize Test");
        assert_eq!(loaded.chain.effects.len(), 1);
        assert_eq!(loaded.chain.effects[0].get_param("radius", 0.0), 10.0);
    }

    #[test]
    fn test_preset_library() {
        let temp = temp_dir().join("MapFlow_preset_test");
        let _ = fs::remove_dir_all(&temp);

        let library = PresetLibrary::new(temp.clone()).unwrap();

        assert!(library.preset_dir().exists());
        assert_eq!(library.cache.len(), 0);

        // Cleanup
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_preset_metadata_default() {
        let meta = PresetMetadata::default();

        assert_eq!(meta.name, "Untitled Preset");
        assert_eq!(meta.category, "Uncategorized");
        assert!(!meta.is_favorite);
        assert!(meta.created_at > 0);
    }
}




