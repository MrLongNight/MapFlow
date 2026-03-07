use crate::UIAction;
use egui::Ui;
use mapmap_core::module::EffectType as ModuleEffectType;

use super::state::ModuleCanvas;
use super::{inspector, mesh};

impl ModuleCanvas {
    pub fn sync_mesh_editor_to_current_selection(
        mesh_editor: &mut crate::editors::mesh_editor::MeshEditor,
        last_mesh_edit_id: &mut Option<u64>,
        part: &mapmap_core::module::ModulePart,
    ) {
        mesh::sync_mesh_editor_to_current_selection(mesh_editor, last_mesh_edit_id, part);
    }

    pub fn apply_mesh_editor_to_selection(
        mesh_editor: &crate::editors::mesh_editor::MeshEditor,
        part: &mut mapmap_core::module::ModulePart,
    ) {
        mesh::apply_mesh_editor_to_selection(mesh_editor, part);
    }

    pub fn render_mesh_editor_ui(
        mesh_editor: &mut crate::editors::mesh_editor::MeshEditor,
        last_mesh_edit_id: &mut Option<u64>,
        ui: &mut Ui,
        mesh: &mut mapmap_core::module::MeshType,
        part_id: mapmap_core::module::ModulePartId,
        id_salt: u64,
    ) {
        mesh::render_mesh_editor_ui(mesh_editor, last_mesh_edit_id, ui, mesh, part_id, id_salt);
    }

    pub fn set_default_effect_params(
        effect_type: ModuleEffectType,
        params: &mut std::collections::HashMap<String, f32>,
    ) {
        inspector::set_default_effect_params(effect_type, params);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_inspector_for_part(
        &mut self,
        mesh_editor: &mut crate::editors::mesh_editor::MeshEditor,
        last_mesh_edit_id: &mut Option<u64>,
        ui: &mut Ui,
        part: &mut mapmap_core::module::ModulePart,
        actions: &mut Vec<UIAction>,
        module_id: mapmap_core::module::ModuleId,
        shared_media_ids: &[String],
    ) {
        inspector::render_inspector_for_part(
            self,
            mesh_editor,
            last_mesh_edit_id,
            ui,
            part,
            actions,
            module_id,
            shared_media_ids,
        );
    }
}
