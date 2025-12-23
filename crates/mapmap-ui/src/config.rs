//! User configuration management
//!
//! Handles saving and loading user preferences including language settings.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// User configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// Preferred language code (e.g., "en", "de")
    pub language: String,
    /// Last opened project path
    #[serde(default)]
    pub last_project: Option<String>,
    /// Recently opened files
    #[serde(default)]
    pub recent_files: Vec<String>,
    /// Last used UI scale
    #[serde(default)]
    pub ui_scale: Option<f32>,
    /// Last used theme
    #[serde(default)]
    pub theme: Option<crate::theme::Theme>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            last_project: None,
            recent_files: Vec::new(),
            ui_scale: Some(1.0),
            theme: Some(crate::theme::Theme::Dark),
        }
    }
}

impl UserConfig {
    /// Get the config file path
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("MapFlow");
            p.push("config.json");
            p
        })
    }

    /// Load configuration from disk
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| {
                if path.exists() {
                    fs::read_to_string(&path).ok()
                } else {
                    None
                }
            })
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(&path, content)?;
        }
        Ok(())
    }

    /// Update language and save
    pub fn set_language(&mut self, lang: &str) {
        self.language = lang.to_string();
        if let Err(e) = self.save() {
            tracing::error!("Failed to save config: {}", e);
        }
    }

    /// Add a file to recent files list
    pub fn add_recent_file(&mut self, path: &str) {
        // Remove if already exists
        self.recent_files.retain(|p| p != path);
        // Add to front
        self.recent_files.insert(0, path.to_string());
        // Keep max 10 recent files
        self.recent_files.truncate(10);
        if let Err(e) = self.save() {
            tracing::error!("Failed to save config: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UserConfig::default();
        assert_eq!(config.language, "en");
        assert!(config.recent_files.is_empty());
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = UserConfig {
            language: "de".to_string(),
            last_project: Some("/path/to/project.MapFlow".to_string()),
            recent_files: vec!["file1.mp4".to_string(), "file2.mp4".to_string()],
            ui_scale: Some(1.2),
            theme: Some(crate::theme::Theme::Light),
        };

        let json = serde_json::to_string(&config).unwrap();
        let loaded: UserConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.language, "de");
        assert_eq!(loaded.recent_files.len(), 2);
    }
}
