use crate::i18n::LocaleManager;
use egui::Ui;
use mapmap_core::module::ModuleManager;

use super::state::ModuleCanvas;
use super::{renderer, utils};

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
        meter_style: crate::config::AudioMeterStyle,
        node_animations_enabled: bool,
        short_circuit_animation_enabled: bool,
        animation_profile: crate::config::AnimationProfile,
        reduce_motion_enabled: bool,
    ) {
        renderer::show(
            self,
            ui,
            manager,
            locale,
            actions,
            meter_style,
            node_animations_enabled,
            short_circuit_animation_enabled,
            animation_profile,
            reduce_motion_enabled,
        );
    }
}
