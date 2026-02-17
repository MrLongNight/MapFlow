//! Modular Media Manager orchestration.

use crate::media_manager_ui::MediaManagerUI;
use egui::Context;
use mapmap_core::media_library::MediaLibrary;

/// Context required to render the media manager.
pub struct MediaManagerContext<'a> {
    /// Reference to the media manager UI state.
    pub ui: &'a mut MediaManagerUI,
    /// Reference to the media library.
    pub library: &'a mut MediaLibrary,
}

/// Renders the media manager panel.
pub fn show(ctx: &Context, context: MediaManagerContext) {
    context.ui.ui(ctx, context.library);
}




