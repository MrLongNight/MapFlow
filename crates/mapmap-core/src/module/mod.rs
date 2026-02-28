//! Module System - Visual Programming Graph
//!
//! This module defines the graph structure used for MapFlow scenes.
//! It is split into submodules for better maintainability.

/// config module
pub mod config;
/// manager module
pub mod manager;
/// types module
pub mod types;

// Re-export core types for backward compatibility
pub use manager::ModuleManager;
pub use types::*;
