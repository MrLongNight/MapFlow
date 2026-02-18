//! Keyboard shortcuts and macro system

mod bindings;
mod macros;
#[allow(clippy::module_inception)]
mod shortcuts;

pub use bindings::*;
pub use macros::*;
pub use shortcuts::*;
