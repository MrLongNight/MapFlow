use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaType {
    Video,
    Image,
    Audio,
    Unknown,
}

impl MediaType {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub duration: Option<Duration>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub path: PathBuf,
    pub name: String,
    pub media_type: MediaType,
    pub metadata: Option<MediaMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub items: Vec<PathBuf>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MediaLibrary {
    pub items: HashMap<PathBuf, MediaItem>,
    pub playlists: Vec<Playlist>,
    pub scanned_paths: Vec<PathBuf>,
}

impl MediaLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_scan_path(&mut self, path: PathBuf) {
        if !self.scanned_paths.contains(&path) {
            self.scanned_paths.push(path);
        }
    }

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

    pub fn get_items(&self) -> Vec<&MediaItem> {
        self.items.values().collect()
    }

    pub fn create_playlist(&mut self, name: String) {
        self.playlists.push(Playlist {
            name,
            items: Vec::new(),
        });
    }

    pub fn remove_playlist(&mut self, name: &str) {
        self.playlists.retain(|p| p.name != name);
    }

    pub fn add_to_playlist(&mut self, playlist_name: &str, path: PathBuf) {
        if let Some(playlist) = self.playlists.iter_mut().find(|p| p.name == playlist_name) {
            if !playlist.items.contains(&path) {
                playlist.items.push(path);
            }
        }
    }

    pub fn remove_from_playlist(&mut self, playlist_name: &str, path: &Path) {
        if let Some(playlist) = self.playlists.iter_mut().find(|p| p.name == playlist_name) {
            playlist.items.retain(|p| p != path);
        }
    }
}
