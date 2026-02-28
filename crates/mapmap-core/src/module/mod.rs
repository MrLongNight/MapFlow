//! Module System - Visual Programming Graph
//!
//! This module defines the graph structure used for MapFlow scenes.
//! It is split into submodules for better maintainability.

pub mod config;
pub mod manager;
pub mod types;

// Re-export core types for backward compatibility
pub use manager::ModuleManager;
pub use types::*;