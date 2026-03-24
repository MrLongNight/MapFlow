use crate::i18n::LocaleManager;
use egui::Ui;
use vorce_core::module::ModuleManager;

use super::state::ModuleCanvas;
use super::{renderer, utils, ModuleCanvasRenderOptions};

impl ModuleCanvas {
    pub fn ensure_icons_loaded(&mut self, ctx: &egui::Context) {
        utils::ensure_icons_loaded(&mut self.plug_icons, ctx);
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        manager: &mut ModuleManager,
        locale: &LocaleManager,
        actions: &mut Vec<crate::UIAction>,
        options: ModuleCanvasRenderOptions,
    ) {
        renderer::show(self, ui, manager, locale, actions, options);
    }
}
