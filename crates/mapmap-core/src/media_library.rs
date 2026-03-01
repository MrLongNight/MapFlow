use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::warn;
use walkdir::WalkDir;

#[cfg(not(test))]
const MAX_MEDIA_ITEMS: usize = 100_000;
#[cfg(test)]
const MAX_MEDIA_ITEMS: usize = 100;

/// Type of media file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaType {
    /// Video file
    Video,
    /// Image file
    Image,
    /// Audio file
    Audio,
    /// Unknown or unsupported format
    Unknown,
}

impl MediaType {
    /// Determine media type from file extension
    pub fn from_path(path: &Path) -> Self {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "mp4" | "mov" | "avi" | "mkv" | "webm" => MediaType::Video,
                "png" | "jpg" | "jpeg" | "gif" | "bmp" => MediaType::Image,
                "mp3" | "wav" | "ogg" | "flac" => MediaType::Audio,
                _ => MediaType::Unknown,
            }
        } else {
            MediaType::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_library_limit() {
        use std::fs::File;
        use std::path::PathBuf;

        struct TempDir(PathBuf);
        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        let path = std::env::temp_dir().join(format!("mapmap_test_limit_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        let _guard = TempDir(path.clone()); // Ensures cleanup on panic

        // Create MAX_MEDIA_ITEMS + 10 files
        // MAX_MEDIA_ITEMS is 100 in test
        for i in 0..(MAX_MEDIA_ITEMS + 10) {
            let file_path = path.join(format!("test_{}.jpg", i));
            File::create(&file_path).unwrap();
        }

        let mut library = MediaLibrary::new();
        library.add_scan_path(path);
        library.refresh();

        assert_eq!(library.items.len(), MAX_MEDIA_ITEMS);
    }
}

/// Metadata for a media item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// Duration if applicable
    pub duration: Option<Duration>,
    /// Width in pixels
    pub width: Option<u32>,
    /// Height in pixels
    pub height: Option<u32>,
    /// File size in bytes
    pub file_size: u64,
}

/// A media item in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    /// Absolute path to the file
    pub path: PathBuf,
    /// Display name
    pub name: String,
    /// Category of the media
    pub media_type: MediaType,
    /// Optional metadata
    pub metadata: Option<MediaMetadata>,
}

/// A collection of media items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    /// Name of the playlist
    pub name: String,
    /// List of paths to media items
    pub items: Vec<PathBuf>,
}

/// Central library for managing media assets
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MediaLibrary {
    /// Map of all discovered media items
    pub items: HashMap<PathBuf, MediaItem>,
    /// List of user-created playlists
    pub playlists: Vec<Playlist>,
    /// Directories being monitored for media
    pub scanned_paths: Vec<PathBuf>,
}

impl MediaLibrary {
    /// Create a new empty media library
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a directory to be scanned for media
    pub fn add_scan_path(&mut self, path: PathBuf) {
        if !self.scanned_paths.contains(&path) {
            self.scanned_paths.push(path);
        }
    }

    /// Refresh the library by re-scanning all paths
    pub fn refresh(&mut self) {
        'outer: for root in self.scanned_paths.clone() {
            for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
                if self.items.len() >= MAX_MEDIA_ITEMS {
                    warn!(
                        "Media library limit reached ({}) - stopping scan",
                        MAX_MEDIA_ITEMS
                    );
                    break 'outer;
                }

                let path = entry.path();
                if path.is_file() {
                    let media_type = MediaType::from_path(path);
                    if media_type != MediaType::Unknown {
                        let metadata = std::fs::metadata(path).ok();
                        let size = metadata.map(|m| m.len()).unwrap_or(0);

                        let item = MediaItem {
                            path: path.to_path_buf(),
                            name: path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            media_type,
                            metadata: Some(MediaMetadata {
                                duration: None, // Requires FFmpeg
                                width: None,    // Requires FFmpeg/Image
                                height: None,   // Requires FFmpeg/Image
                                file_size: size,
                            }),
                        };
                        self.items.insert(path.to_path_buf(), item);
                    }
                }
            }
        }
    }

    /// Get all media items currently in the library
    pub fn get_items(&self) -> Vec<&MediaItem> {
        self.items.values().collect()
    }

    /// Create a new empty playlist
    pub fn create_playlist(&mut self, name: String) {
        self.playlists.push(Playlist {
            name,
            items: Vec::new(),
        });
    }

    /// Remove a playlist by name
    pub fn remove_playlist(&mut self, name: &str) {
        self.playlists.retain(|p| p.name != name);
    }

    /// Add a media item to a playlist
    pub fn add_to_playlist(&mut self, playlist_name: &str, path: PathBuf) {
        if let Some(playlist) = self.playlists.iter_mut().find(|p| p.name == playlist_name) {
            if !playlist.items.contains(&path) {
                playlist.items.push(path);
            }
        }
    }

    /// Remove a media item from a playlist
    pub fn remove_from_playlist(&mut self, playlist_name: &str, path: &Path) {
        if let Some(playlist) = self.playlists.iter_mut().find(|p| p.name == playlist_name) {
            playlist.items.retain(|p| p != path);
        }
    }
}