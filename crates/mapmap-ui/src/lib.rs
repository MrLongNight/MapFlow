//! MapFlow UI - ImGui and egui Integration
//!
//! This crate provides the user interface layer using ImGui (legacy) and egui (Phase 6+), including:
//! - ImGui context setup (Phase 0-5)
//! - egui integration (Phase 6+)
//! - Window management
//! - Control panels
//! - Advanced authoring UI (Phase 6)
//! - Effect Chain Panel (Phase 3)
//! - Controller Overlay Panel (MIDI visualization)

#![warn(missing_docs)]

// Categorized modules
#[allow(missing_docs)]
pub mod core;
#[allow(missing_docs)]
pub mod editors;
#[allow(missing_docs)]
pub mod panels;
#[allow(missing_docs)]
pub mod view;
#[allow(missing_docs)]
pub mod widgets;

#[allow(missing_docs)]
pub mod action;
#[allow(missing_docs)]
pub mod app_ui;

// Re-export categorized modules to maintain API compatibility
pub use crate::core::*;
pub use crate::editors::{
    mesh_editor::*, module_canvas::*, node_editor::*, shortcut_editor::*, timeline_v2::*,
};
// Re-export panel types directly to avoid ambiguous glob re-exports
pub use crate::panels::{
    assignment_panel::*, audio_panel::*, controller_overlay_panel::*, cue_panel::*,
    edge_blend_panel::*, effect_chain::*, inspector_panel::*, layer_panel::*, mapping_panel::*,
    osc_panel::*, oscillator_panel::*, output_panel::*, paint_panel::*, preview_panel::*,
    shortcuts_panel::*, transform_panel::*,
};
pub use crate::view::*;
pub use crate::widgets::*;

pub use crate::action::UIAction;
pub use crate::app_ui::AppUI;

/// Re-export types for public use
pub mod types {
    pub use crate::editors::module_canvas::types::*;
}
