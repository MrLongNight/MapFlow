//! UI modules for MapFlow.
//!
//! This module contains the user interface components extracted from the main application loop.

/// Settings and dialog windows.
pub mod dialogs;
/// Functional panels and sidebars.
pub mod panels;

/// Re-export settings for backward compatibility
pub use dialogs::settings;
