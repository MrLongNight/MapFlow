//! Project I/O - High-level API
//!
//! This module provides a high-level API for saving and loading MapFlow
//! project files. It handles the application-level logic, such as version
//! validation and managing the `AppState`, while delegating the low-level
//! serialization and file I/O to the `project_format` module.

use crate::error::{IoError, Result};
use crate::project_format::{ProjectFile, PROJECT_FILE_VERSION};
use mapmap_core::AppState;
use std::path::Path;

/// Saves the application state to a project file.
///
/// This function wraps the given `AppState` in a `ProjectFile` container,
/// which adds metadata like the format version and timestamps. It then delegates
/// the serialization and writing to disk.
///
/// # Arguments
///
/// * `state` - A reference to the `AppState` to be saved.
/// * `path` - The file path where the project will be saved.
///
/// # Returns
///
/// A `Result` indicating success or an `IoError` on failure.
pub fn save_project(state: &AppState, path: &Path) -> Result<()> {
    let mut project_file = ProjectFile::new(state.clone());
    project_file.save(path)
}

/// Loads the application state from a project file.
///
/// This function reads and deserializes a project file from the given path.
/// It performs a version check to ensure compatibility. If the version
/// of the loaded file matches the application's version, it returns the
/// extracted `AppState`.
///
/// # Arguments
///
/// * `path` - The file path of the project to load.
///
/// # Returns
///
/// A `Result` containing the loaded `AppState` on success, or an `IoError`
/// on failure (e.g., file not found, deserialization error, version mismatch).
pub fn load_project(path: &Path) -> Result<AppState> {
    let project_file = ProjectFile::load(path)?;

    // Basic version validation. A more sophisticated migration system could be
    // implemented here in the future.
    if project_file.version != PROJECT_FILE_VERSION {
        return Err(IoError::VersionMismatch {
            expected: PROJECT_FILE_VERSION.to_string(),
            found: project_file.version,
        });
    }

    Ok(project_file.app_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mapmap_core::AppState;
    use tempfile::NamedTempFile;

    #[test]
    fn project_ron_roundtrip() {
        let original_state = AppState::default();
        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("ron");

        // Save the project
        save_project(&original_state, &path).unwrap();

        // Load the project
        let loaded_state = load_project(&path).unwrap();

        // Verify the state is the same
        assert_eq!(original_state, loaded_state);
    }

    #[test]
    fn project_json_roundtrip() {
        let original_state = AppState::default();
        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("json");

        save_project(&original_state, &path).unwrap();
        let loaded_state = load_project(&path).unwrap();

        assert_eq!(original_state, loaded_state);
    }

    #[test]
    fn test_version_mismatch() {
        let mut project_file = ProjectFile::new(AppState::default());
        project_file.version = "0.1.0".to_string(); // Set an old version

        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("ron");

        project_file.save(&path).unwrap();

        let result = load_project(&path);
        assert!(matches!(result, Err(IoError::VersionMismatch { .. })));

        if let Err(IoError::VersionMismatch { expected, found }) = result {
            assert_eq!(expected, PROJECT_FILE_VERSION);
            assert_eq!(found, "0.1.0");
        }
    }

    #[test]
    fn test_unsupported_format() {
        let state = AppState::default();
        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("txt");

        let save_result = save_project(&state, &path);
        assert!(matches!(save_result, Err(IoError::UnsupportedFormat(_))));

        // We can't easily test the load path for unsupported format here
        // because it would require creating a file, and the error originates
        // from the `ProjectFile::load` function which isn't directly called
        // in a way that would trigger this specific test case from here.
        // The save test is sufficient to cover the intent.
    }
}
