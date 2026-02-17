//! Shader Hot-Reload System
//!
//! Watches WGSL shader files for changes and triggers recompilation.
//! Provides graceful error handling with fallback to previous working shader.

use crossbeam_channel::{Receiver, Sender};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Errors related to hot-reload
#[derive(Error, Debug)]
pub enum HotReloadError {
    #[error("Watcher error: {0}")]
    WatcherError(#[from] notify::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Shader compilation error: {0}")]
    ShaderCompileError(String),

    #[error("Channel closed")]
    ChannelClosed,
}

/// Result type for hot-reload operations
pub type Result<T> = std::result::Result<T, HotReloadError>;

/// Event sent when a shader file changes
#[derive(Debug, Clone)]
pub struct ShaderChangeEvent {
    /// Path to the changed shader file
    pub path: PathBuf,
    /// The new shader source code
    pub source: String,
    /// Timestamp of the change
    pub timestamp: Instant,
}

/// Status of a watched shader
#[derive(Debug, Clone)]
pub struct ShaderStatus {
    /// Path to the shader file
    pub path: PathBuf,
    /// Current source code
    pub current_source: String,
    /// Whether the shader is valid (compiled successfully)
    pub is_valid: bool,
    /// Last error message if compilation failed
    pub last_error: Option<String>,
    /// Last successful source (for fallback)
    pub fallback_source: Option<String>,
    /// Last modification time
    pub last_modified: Instant,
}

/// Shader Hot-Reload Watcher
pub struct ShaderHotReload {
    /// Directory being watched
    watch_dir: PathBuf,

    /// File watcher handle
    _watcher: RecommendedWatcher,

    /// Receiver for shader change events
    change_receiver: Receiver<ShaderChangeEvent>,

    /// Status of each watched shader
    shader_status: HashMap<PathBuf, ShaderStatus>,

    /// Debounce duration to prevent multiple rapid reloads
    debounce_duration: Duration,

    /// Last event time per file (for debouncing)
    last_event_time: HashMap<PathBuf, Instant>,
}

impl ShaderHotReload {
    /// Create a new hot-reload watcher for the given shader directory
    pub fn new(watch_dir: PathBuf) -> Result<Self> {
        info!("Starting shader hot-reload for: {:?}", watch_dir);

        // Create channel for events
        let (tx, rx) = crossbeam_channel::unbounded();

        // Create file watcher
        let watcher = Self::create_watcher(watch_dir.clone(), tx)?;

        // Initial scan of existing shaders
        let mut shader_status = HashMap::new();
        if watch_dir.exists() {
            Self::scan_directory(&watch_dir, &mut shader_status)?;
        }

        Ok(Self {
            watch_dir,
            _watcher: watcher,
            change_receiver: rx,
            shader_status,
            debounce_duration: Duration::from_millis(100),
            last_event_time: HashMap::new(),
        })
    }

    /// Create the file system watcher
    fn create_watcher(
        watch_dir: PathBuf,
        tx: Sender<ShaderChangeEvent>,
    ) -> Result<RecommendedWatcher> {
        let mut watcher = RecommendedWatcher::new(
            move |res: std::result::Result<Event, notify::Error>| match res {
                Ok(event) => {
                    // Only process modify and create events
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in event.paths {
                            // Only watch .wgsl files
                            if path.extension().is_some_and(|ext| ext == "wgsl") {
                                debug!("Shader file changed: {:?}", path);

                                // Try to read the new content
                                if let Ok(source) = fs::read_to_string(&path) {
                                    let change_event = ShaderChangeEvent {
                                        path,
                                        source,
                                        timestamp: Instant::now(),
                                    };
                                    if tx.send(change_event).is_err() {
                                        error!("Failed to send shader change event");
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("File watcher error: {}", e);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;

        // Start watching the directory
        if watch_dir.exists() {
            watcher.watch(&watch_dir, RecursiveMode::Recursive)?;
            info!("Watching shader directory: {:?}", watch_dir);
        } else {
            warn!("Shader directory does not exist: {:?}", watch_dir);
        }

        Ok(watcher)
    }

    /// Scan directory for existing shader files
    fn scan_directory(
        dir: &Path,
        shader_status: &mut HashMap<PathBuf, ShaderStatus>,
    ) -> Result<()> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() && path.extension().is_some_and(|ext| ext == "wgsl") {
                    if let Ok(source) = fs::read_to_string(&path) {
                        shader_status.insert(
                            path.clone(),
                            ShaderStatus {
                                path: path.clone(),
                                current_source: source.clone(),
                                is_valid: true, // Assume valid initially
                                last_error: None,
                                fallback_source: Some(source),
                                last_modified: Instant::now(),
                            },
                        );
                        debug!("Found shader: {:?}", path);
                    }
                } else if path.is_dir() {
                    Self::scan_directory(&path, shader_status)?;
                }
            }
        }
        Ok(())
    }

    /// Poll for shader changes (non-blocking)
    ///
    /// Returns a list of changed shaders that should be recompiled
    pub fn poll_changes(&mut self) -> Vec<ShaderChangeEvent> {
        let mut changes = Vec::new();
        let now = Instant::now();

        // Drain the channel
        while let Ok(event) = self.change_receiver.try_recv() {
            // Debounce: only process if enough time has passed
            let should_process = self
                .last_event_time
                .get(&event.path)
                .map(|last| now.duration_since(*last) > self.debounce_duration)
                .unwrap_or(true);

            if should_process {
                self.last_event_time.insert(event.path.clone(), now);

                // Update status
                if let Some(status) = self.shader_status.get_mut(&event.path) {
                    status.current_source = event.source.clone();
                    status.last_modified = event.timestamp;
                } else {
                    // New shader file
                    self.shader_status.insert(
                        event.path.clone(),
                        ShaderStatus {
                            path: event.path.clone(),
                            current_source: event.source.clone(),
                            is_valid: true,
                            last_error: None,
                            fallback_source: None,
                            last_modified: event.timestamp,
                        },
                    );
                }

                changes.push(event);
            }
        }

        changes
    }

    /// Report a successful shader compilation
    pub fn report_success(&mut self, path: &Path) {
        if let Some(status) = self.shader_status.get_mut(path) {
            status.is_valid = true;
            status.last_error = None;
            status.fallback_source = Some(status.current_source.clone());
            info!("Shader compiled successfully: {:?}", path);
        }
    }

    /// Report a failed shader compilation
    pub fn report_error(&mut self, path: &Path, error: &str) {
        if let Some(status) = self.shader_status.get_mut(path) {
            status.is_valid = false;
            status.last_error = Some(error.to_string());
            warn!("Shader compilation failed: {:?} - {}", path, error);
        }
    }

    /// Get the fallback source for a shader (last known working version)
    pub fn get_fallback(&self, path: &Path) -> Option<&str> {
        self.shader_status
            .get(path)
            .and_then(|s| s.fallback_source.as_deref())
    }

    /// Get the current source for a shader
    pub fn get_source(&self, path: &Path) -> Option<&str> {
        self.shader_status
            .get(path)
            .map(|s| s.current_source.as_str())
    }

    /// Get status for all watched shaders
    pub fn all_status(&self) -> impl Iterator<Item = &ShaderStatus> {
        self.shader_status.values()
    }

    /// Get status for a specific shader
    pub fn get_status(&self, path: &Path) -> Option<&ShaderStatus> {
        self.shader_status.get(path)
    }

    /// Get the watch directory
    pub fn watch_dir(&self) -> &Path {
        &self.watch_dir
    }

    /// Check if any shaders have errors
    pub fn has_errors(&self) -> bool {
        self.shader_status.values().any(|s| !s.is_valid)
    }

    /// Get all shaders with errors
    pub fn errors(&self) -> Vec<&ShaderStatus> {
        self.shader_status
            .values()
            .filter(|s| !s.is_valid)
            .collect()
    }
}

/// Integration helper for using hot-reload with the EffectChainRenderer
pub struct HotReloadIntegration {
    hot_reload: ShaderHotReload,
    pending_reloads: Vec<PathBuf>,
}

impl HotReloadIntegration {
    /// Create a new hot-reload integration
    pub fn new(shader_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            hot_reload: ShaderHotReload::new(shader_dir)?,
            pending_reloads: Vec::new(),
        })
    }

    /// Update and collect pending shader reloads
    pub fn update(&mut self) -> Vec<ShaderChangeEvent> {
        let changes = self.hot_reload.poll_changes();

        for change in &changes {
            self.pending_reloads.push(change.path.clone());
        }

        changes
    }

    /// Attempt to compile a shader and report result
    pub fn try_compile(
        &mut self,
        device: &wgpu::Device,
        path: &Path,
        source: &str,
    ) -> std::result::Result<wgpu::ShaderModule, String> {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path.to_str().unwrap_or("shader")),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        self.hot_reload.report_success(path);
        Ok(module)
    }

    /// Get inner hot-reload instance
    pub fn hot_reload(&self) -> &ShaderHotReload {
        &self.hot_reload
    }

    /// Get mutable hot-reload instance
    pub fn hot_reload_mut(&mut self) -> &mut ShaderHotReload {
        &mut self.hot_reload
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_status_initial() {
        let status = ShaderStatus {
            path: PathBuf::from("test.wgsl"),
            current_source: "// test".to_string(),
            is_valid: true,
            last_error: None,
            fallback_source: None,
            last_modified: Instant::now(),
        };

        assert!(status.is_valid);
        assert!(status.last_error.is_none());
    }

    #[test]
    fn test_shader_change_event() {
        let event = ShaderChangeEvent {
            path: PathBuf::from("effect.wgsl"),
            source: "@fragment fn main() {}".to_string(),
            timestamp: Instant::now(),
        };

        assert!(event.path.ends_with("effect.wgsl"));
        assert!(event.source.contains("@fragment"));
    }

    // Note: Full hot-reload tests require a real filesystem and would be integration tests
}




