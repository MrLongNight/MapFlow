//! Defines the on-disk project file format for MapFlow.
//!
//! This module specifies the structure of the project file, which is serialized
//! to and from RON or JSON. It includes metadata and the core application state.

use crate::error::{IoError, Result};
use chrono::{DateTime, Utc};
use mapmap_core::AppState;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// The current version of the project file format.
///
/// This constant is used to stamp saved project files. It follows semantic
/// versioning (MAJOR.MINOR.PATCH) and should be incremented when breaking
/// changes are made to the `ProjectFile` struct or its children.
pub const PROJECT_FILE_VERSION: &str = "1.0.0";

/// Maximum allowed project file size (50 MB).
///
/// This limit prevents unbounded resource consumption (DoS) when loading project files.
pub const MAX_PROJECT_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Represents the top-level structure of a saved MapFlow project file.
///
/// This struct is what gets serialized to/from RON or JSON. It wraps the
/// main `AppState` with metadata for versioning and tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectFile {
    /// The version of the project file format.
    pub version: String,
    /// Metadata about the project.
    pub metadata: ProjectMetadata,
    /// The core application state.
    pub app_state: AppState,
}

impl ProjectFile {
    /// Creates a new `ProjectFile` from an `AppState`, setting creation
    /// and modification times to now.
    pub fn new(app_state: AppState) -> Self {
        let now = Utc::now();
        Self {
            version: PROJECT_FILE_VERSION.to_string(),
            metadata: ProjectMetadata {
                created_at: now,
                modified_at: now,
            },
            app_state,
        }
    }

    /// Loads a `ProjectFile` from the given path.
    ///
    /// This function handles the low-level deserialization from either RON or JSON,
    /// depending on the file extension.
    pub fn load(path: &Path) -> Result<Self> {
        Self::load_with_limit(path, MAX_PROJECT_FILE_SIZE)
    }

    /// Loads a `ProjectFile` with a specific file size limit.
    fn load_with_limit(path: &Path, limit: u64) -> Result<Self> {
        // Check file size first
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        if size > limit {
            return Err(IoError::FileTooLarge { size, limit });
        }

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("ron");

        match extension {
            "json" => {
                let mut content = String::new();
                File::open(path)?.read_to_string(&mut content)?;
                let file: ProjectFile = serde_json::from_str(&content)?;
                Ok(file)
            }
            "ron" | "mapmap" | "mflow" => {
                let mut content = String::new();
                File::open(path)?.read_to_string(&mut content)?;
                let file: ProjectFile = ron::from_str(&content)?;
                Ok(file)
            }
            _ => Err(IoError::UnsupportedFormat(extension.to_string())),
        }
    }

    /// Saves the `ProjectFile` to the given path.
    ///
    /// This function handles the low-level serialization to either RON or JSON,
    /// depending on the file extension. It also updates the `modified_at` timestamp.
    pub fn save(&mut self, path: &Path) -> Result<()> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("ron");

        // Update the modification timestamp
        self.metadata.modified_at = Utc::now();

        match extension {
            "json" => {
                let file = File::create(path)?;
                serde_json::to_writer_pretty(file, self)?;
            }
            "ron" | "mapmap" | "mflow" => {
                let config = ron::ser::PrettyConfig::default();
                let s = ron::ser::to_string_pretty(self, config)?;
                let mut file = File::create(path)?;
                file.write_all(s.as_bytes())?;
            }
            _ => return Err(IoError::UnsupportedFormat(extension.to_string())),
        }

        Ok(())
    }
}

/// Metadata associated with a project file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectMetadata {
    /// Timestamp of when the project was first created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last modification.
    pub modified_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mapmap_core::AppState;
    use tempfile::NamedTempFile;

    #[test]
    fn project_file_ron_roundtrip() {
        let app_state = AppState::default();
        let mut project_file = ProjectFile::new(app_state);

        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("ron");

        // Save and load
        project_file.save(&path).unwrap();
        let loaded_project_file = ProjectFile::load(&path).unwrap();

        // Check top-level fields
        assert_eq!(project_file.version, loaded_project_file.version);
        assert_eq!(project_file.app_state, loaded_project_file.app_state);

        // Check timestamps - modified_at should be different, created_at should be the same
        assert_eq!(
            project_file.metadata.created_at,
            loaded_project_file.metadata.created_at
        );
        assert!(project_file.metadata.modified_at <= loaded_project_file.metadata.modified_at);
    }

    #[test]
    fn project_file_json_roundtrip() {
        let app_state = AppState::default();
        let mut project_file = ProjectFile::new(app_state);

        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("json");

        // Save and load
        project_file.save(&path).unwrap();
        let loaded_project_file = ProjectFile::load(&path).unwrap();

        assert_eq!(project_file.version, loaded_project_file.version);
        assert_eq!(project_file.app_state, loaded_project_file.app_state);
    }

    #[test]
    fn test_modified_at_updates_on_save() {
        let app_state = AppState::default();
        let mut project_file = ProjectFile::new(app_state);

        let first_modified_at = project_file.metadata.modified_at;

        // Wait a moment to ensure the timestamp will be different
        std::thread::sleep(std::time::Duration::from_millis(10));

        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("ron");
        project_file.save(&path).unwrap();

        let second_modified_at = project_file.metadata.modified_at;

        assert!(second_modified_at > first_modified_at);
    }

    #[test]
    fn test_load_file_too_large() {
        use std::io::Write;

        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("ron");
        let mut file = File::create(&path).unwrap();

        // Write 1KB of data
        let data = vec![b' '; 1024];
        file.write_all(&data).unwrap();

        // Try to load with a small limit (500 bytes)
        let result = ProjectFile::load_with_limit(&path, 500);

        assert!(matches!(result, Err(IoError::FileTooLarge { .. })));

        if let Err(IoError::FileTooLarge { size, limit }) = result {
            assert_eq!(size, 1024);
            assert_eq!(limit, 500);
        }
    }
}
