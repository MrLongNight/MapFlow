//! Effect Chain UI Panel
//!
//! egui-based panel for managing effect chains with drag & drop reordering,
//! parameter sliders, and preset browser.

mod components;
mod models;
mod panel;
mod types;

#[cfg(test)]
mod tests;

pub use models::{EffectChainAction, PresetEntry, UIEffect, UIEffectChain};
pub use panel::EffectChainPanel;
pub use types::EffectType;
