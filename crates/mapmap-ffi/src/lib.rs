//! MapFlow FFI - Foreign Function Interface Bridge
//!
//! This crate provides FFI bindings to external SDKs including:
//! - NDI (Network Device Interface)
//! - DeckLink SDI
//! - Spout (Windows)
//! - Syphon (macOS)
//!
//! NOTE: This is a placeholder for Phase 0.
//! Full implementation will be completed in Phase 5.

use thiserror::Error;

/// FFI errors
#[derive(Error, Debug)]
pub enum FfiError {
    #[error("NDI error: {0}")]
/// Error: NDI error.
    /// Error: NDI error.
    NdiError(String),

    #[error("DeckLink error: {0}")]
/// Error: DeckLink error.
    /// Error: DeckLink error.
    DeckLinkError(String),

    #[error("Spout error: {0}")]
/// Error: Spout error.
    /// Error: Spout error.
    SpoutError(String),

    #[error("Syphon error: {0}")]
/// Error: Syphon error.
    /// Error: Syphon error.
    SyphonError(String),
}

/// Result type for FFI operations
pub type Result<T> = std::result::Result<T, FfiError>;

/// C-ABI plugin interface (placeholder)
#[repr(C)]
pub struct PluginApi {
    /// Version number for API or plugin compatibility.
    pub version: u32,
}

impl Default for PluginApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginApi {
    /// The current architectural version of the API or plugin.
    pub const VERSION: u32 = 1;

    /// Creates a new, uninitialized instance with default settings.
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_api() {
        let api = PluginApi::new();
        assert_eq!(api.version, PluginApi::VERSION);
    }
}