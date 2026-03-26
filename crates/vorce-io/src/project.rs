//! Project I/O - High-level API
//!
//! This module provides a high-level API for saving and loading MapFlow
//! project files. It handles the application-level logic, such as version
//! validation and managing the `AppState`, while delegating the low-level
//! serialization and file I/O to the `project_format` module.

use crate::error::{IoError, Result};
use crate::project_format::{ProjectFile, PROJECT_FILE_VERSION};
use std::path::Path;
use vorce_core::AppState;

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

/// Exports the application state and media assets to a ZIP archive.
///
/// This function saves the project file and bundles it with all referenced
/// media files into a single standalone ZIP archive.
///
/// # Arguments
///
/// * `state` - A reference to the `AppState` to be exported.
/// * `path` - The file path where the exported ZIP archive will be saved.
///
/// # Returns
///
/// A `Result` indicating success or an `IoError` on failure.
pub fn export_project(state: &AppState, path: &Path) -> Result<()> {
    use std::collections::HashSet;
    use std::fs::File;
    use vorce_core::module::SourceType;
    use zip::write::FileOptions;

    let file = File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Deep clone the state so we can mutate the media paths to be relative for the export
    let mut export_state = state.clone();

    // Find all media files referenced in the project and rewrite paths to be relative
    let mut media_files = HashSet::new();

    if let Some(module_manager) = std::sync::Arc::get_mut(&mut export_state.module_manager) {
        for module in module_manager.modules.values_mut() {
            for part in &mut module.parts {
                if let vorce_core::module::ModulePartType::Source(
                    SourceType::MediaFile {
                        path: ref mut p, ..
                    }
                    | SourceType::VideoUni {
                        path: ref mut p, ..
                    }
                    | SourceType::ImageUni {
                        path: ref mut p, ..
                    },
                ) = &mut part.part_type
                {
                    if !p.is_empty() {
                        let original_path = std::path::PathBuf::from(&p);
                        media_files.insert(original_path.clone());

                        if let Some(file_name) = original_path.file_name() {
                            *p = format!("media/{}", file_name.to_string_lossy());
                        }
                    }
                }
            }
        }
    }

    // 1. Save project file to a temporary location
    let temp_dir = tempfile::tempdir()?;
    let project_path = temp_dir.path().join("project.vorce");
    save_project(&export_state, &project_path)?;

    // 2. Add project file to ZIP
    zip.start_file("project.vorce", options)
        .map_err(crate::IoError::from)?;
    let mut project_file = File::open(&project_path)?;
    std::io::copy(&mut project_file, &mut zip)?;

    // 3. Add media files to ZIP
    zip.add_directory("media/", options)
        .map_err(crate::IoError::from)?;

    for media_path in media_files {
        if media_path.exists() {
            if let Some(file_name) = media_path.file_name() {
                let zip_path = format!("media/{}", file_name.to_string_lossy());
                zip.start_file(zip_path, options)
                    .map_err(crate::IoError::from)?;
                let mut media_file = File::open(&media_path)?;
                std::io::copy(&mut media_file, &mut zip)?;
            }
        }
    }

    zip.finish().map_err(crate::IoError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use vorce_core::AppState;

    #[test]
    #[ignore]
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
    #[ignore]
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
