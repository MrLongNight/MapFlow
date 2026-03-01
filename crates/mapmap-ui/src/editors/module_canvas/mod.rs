use crate::i18n::LocaleManager;
use crate::UIAction;
use egui::Ui;
use mapmap_core::module::{EffectType as ModuleEffectType, ModuleManager, ModulePartId};

pub mod controller;
pub mod diagnostics;
pub mod draw;
pub mod geometry;
pub mod inspector;
pub mod mesh;
pub mod renderer;
pub mod state;
pub mod types;
pub mod utils;

pub use state::ModuleCanvas;
use types::*;

impl ModuleCanvas {
    pub fn ensure_icons_loaded(&mut self, ctx: &egui::Context) {
        utils::ensure_icons_loaded(&mut self.plug_icons, ctx);
    }

    pub fn sync_mesh_editor_to_current_selection(
        &mut self,
        part: &mapmap_core::module::ModulePart,
    ) {
        mesh::sync_mesh_editor_to_current_selection(self, part);
    }

    pub fn apply_mesh_editor_to_selection(&mut self, part: &mut mapmap_core::module::ModulePart) {
        mesh::apply_mesh_editor_to_selection(self, part);
    }

    pub fn render_mesh_editor_ui(
        &mut self,
        ui: &mut Ui,
        mesh: &mut mapmap_core::module::MeshType,
        part_id: ModulePartId,
        id_salt: u64,
    ) {
        mesh::render_mesh_editor_ui(self, ui, mesh, part_id, id_salt);
    }

    pub fn take_playback_commands(&mut self) -> Vec<(ModulePartId, MediaPlaybackCommand)> {
        std::mem::take(&mut self.pending_playback_commands)
    }

    pub fn get_selected_part_id(&self) -> Option<ModulePartId> {
        self.selected_parts.last().copied()
    }

    pub fn set_default_effect_params(
        effect_type: ModuleEffectType,
        params: &mut std::collections::HashMap<String, f32>,
    ) {
        inspector::set_default_effect_params(effect_type, params);
    }

    pub fn render_inspector_for_part(
        &mut self,
        ui: &mut Ui,
        part: &mut mapmap_core::module::ModulePart,
        actions: &mut Vec<UIAction>,
        module_id: mapmap_core::module::ModuleId,
        shared_media_ids: &[String],
    ) {
        inspector::render_inspector_for_part(self, ui, part, actions, module_id, shared_media_ids);
    }

    pub fn set_active_module(&mut self, module_id: Option<u64>) {
        self.active_module_id = module_id;
        // Also clear selection when switching modules
        self.selected_parts.clear();
        self.dragging_part = None;
        self.creating_connection = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn active_module_id(&self) -> Option<u64> {
        self.active_module_id
    }

    pub fn set_audio_data(&mut self, data: mapmap_core::audio_reactive::AudioTriggerData) {
        self.audio_trigger_data = data;
    }

    pub fn get_audio_trigger_data(&self) -> Option<&mapmap_core::audio_reactive::AudioTriggerData> {
        Some(&self.audio_trigger_data)
    }

    pub fn get_rms_volume(&self) -> f32 {
        self.audio_trigger_data.rms_volume
    }

    pub fn is_beat_detected(&self) -> bool {
        self.audio_trigger_data.beat_detected
    }

    #[cfg(feature = "midi")]
    pub fn process_midi_message(&mut self, message: mapmap_control::midi::MidiMessage) {
        controller::process_midi_message(self, message);
    }

    #[cfg(not(feature = "midi"))]
    pub fn process_midi_message(&mut self, message: ()) {
        controller::process_midi_message(self, message);
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        manager: &mut ModuleManager,
        locale: &LocaleManager,
        actions: &mut Vec<crate::UIAction>,
    ) {
        renderer::show(self, ui, manager, locale, actions);
    }
}
