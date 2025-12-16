//! Project I/O
//!
//! Handles saving and loading of project files.
//! Supported formats: RON (Rusty Object Notation) and JSON.

use mapmap_core::AppState;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;

/// Project I/O errors
#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] ron::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] ron::error::SpannedError),

    #[error("JSON Serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("Format not supported: {0}")]
    UnsupportedFormat(String),
}

/// Save project to file
///
/// Uses RON format by default for .mapmap files, or JSON if extension is .json
pub fn save_project(state: &AppState, path: &Path) -> Result<(), ProjectError> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("ron");

    match extension {
        "json" => {
            let file = File::create(path)?;
            serde_json::to_writer_pretty(file, state)?;
        }
        "ron" | "mapmap" => {
            let config = ron::ser::PrettyConfig::default();
            let s = ron::ser::to_string_pretty(state, config)?;
            let mut file = File::create(path)?;
            file.write_all(s.as_bytes())?;
        }
        _ => return Err(ProjectError::UnsupportedFormat(extension.to_string())),
    }

    Ok(())
}

/// Load project from file
pub fn load_project(path: &Path) -> Result<AppState, ProjectError> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("ron");
    let mut file = File::open(path)?;

    match extension {
        "json" => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let state: AppState = serde_json::from_str(&content)?;
            Ok(state)
        }
        "ron" | "mapmap" => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let state: AppState = ron::from_str(&content)?;
            Ok(state)
        }
        _ => Err(ProjectError::UnsupportedFormat(extension.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_load_project() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_project.mapmap");

        let mut state = AppState::default();
        state.name = "My Test Project".to_string();

        save_project(&state, &file_path).unwrap();

        let loaded_state = load_project(&file_path).unwrap();
        assert_eq!(loaded_state.name, "My Test Project");
    }
}
