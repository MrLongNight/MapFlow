//! View module orchestration

pub mod dashboard;
pub mod media_browser;
pub mod media_manager_wrapper;
#[path = "menu_bar/mod.rs"]
pub mod menu_bar;
pub mod module_sidebar;

pub use dashboard::*;
pub use media_browser::*;
pub use media_manager_wrapper::*;
pub use menu_bar::*;
pub use module_sidebar::*;
