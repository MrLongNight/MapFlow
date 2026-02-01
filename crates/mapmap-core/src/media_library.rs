use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

/// Represents the type of media file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaType {
    /// Video file (mp4, mov, avi, etc.)
    Video,
    /// Image file (png, jpg, etc.)
    Image,
    /// Audio file (mp3, wav, etc.)
    Audio,
    /// Unknown or unsupported file type
    Unknown,
}

impl MediaType {
    /// Determines the media type from the file extension.
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

/// Metadata associated with a media item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// Duration of the media (if applicable)
    pub duration: Option<Duration>,
    /// Width in pixels (if applicable)
    pub width: Option<u32>,
    /// Height in pixels (if applicable)
    pub height: Option<u32>,
    /// File size in bytes
    pub file_size: u64,
}

/// Represents a single media file in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    /// Absolute path to the file
    pub path: PathBuf,
    /// Display name (usually filename)
    pub name: String,
    /// Type of media
    pub media_type: MediaType,
    /// Cached metadata
    pub metadata: Option<MediaMetadata>,
}

/// A named collection of media items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    /// Name of the playlist
    pub name: String,
    /// Paths to items in the playlist
    pub items: Vec<PathBuf>,
}

/// Manages media files, playlists, and scanned directories.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MediaLibrary {
    /// Map of paths to media items
    pub items: HashMap<PathBuf, MediaItem>,
    /// List of playlists
    pub playlists: Vec<Playlist>,
    /// List of directories to scan for media
    pub scanned_paths: Vec<PathBuf>,
}

impl MediaLibrary {
    /// Creates a new empty media library.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a directory path to be scanned for media.
    pub fn add_scan_path(&mut self, path: PathBuf) {
        if !self.scanned_paths.contains(&path) {
            self.scanned_paths.push(path);
        }
    }

    /// Rescans all registered paths and updates the library.
    pub fn refresh(&mut self) {
        for root in self.scanned_paths.clone() {
            for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
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

    /// Returns a list of all media items in the library.
    pub fn get_items(&self) -> Vec<&MediaItem> {
        self.items.values().collect()
    }

    /// Creates a new empty playlist.
    pub fn create_playlist(&mut self, name: String) {
        self.playlists.push(Playlist {
            name,
            items: Vec::new(),
        });
    }

    /// Removes a playlist by name.
    pub fn remove_playlist(&mut self, name: &str) {
        self.playlists.retain(|p| p.name != name);
    }

    /// Adds a media item path to a playlist.
    pub fn add_to_playlist(&mut self, playlist_name: &str, path: PathBuf) {
        if let Some(playlist) = self.playlists.iter_mut().find(|p| p.name == playlist_name) {
            if !playlist.items.contains(&path) {
                playlist.items.push(path);
            }
        }
    }

    /// Removes a media item path from a playlist.
    pub fn remove_from_playlist(&mut self, playlist_name: &str, path: &Path) {
        if let Some(playlist) = self.playlists.iter_mut().find(|p| p.name == playlist_name) {
            playlist.items.retain(|p| p != path);
        }
    }
}
