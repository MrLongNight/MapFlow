//! Inspector sub-modules for different contexts

pub mod layer;
pub mod module;
pub mod output;
mod panel;
pub mod types;
pub mod ui;

pub use panel::InspectorPanel;
pub use types::{InspectorAction, InspectorContext};
pub use ui::{inspector_row, inspector_section, inspector_value};
