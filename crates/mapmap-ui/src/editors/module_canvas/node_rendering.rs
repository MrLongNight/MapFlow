use crate::UIAction;
use egui::Ui;
use mapmap_core::module::{EffectType as ModuleEffectType, ModulePartId};

use super::state::ModuleCanvas;
use super::{inspector, mesh};

impl ModuleCanvas {
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
}
