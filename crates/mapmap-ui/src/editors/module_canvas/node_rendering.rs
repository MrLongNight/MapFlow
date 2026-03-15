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
        inspector::effect::set_default_effect_params(effect_type, params);
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
        preview_context: &inspector::InspectorPreviewContext,
    ) {
        let interacting = ui.input(|i| i.pointer.any_pressed() || i.pointer.any_down());
        let release = ui.input(|i| i.pointer.any_released());

        if interacting && self.edit_snapshot.is_none() {
            self.edit_snapshot = Some(part.clone());
        }

        let snapshot_before = part.clone();

        inspector::render_inspector_for_part(
            self,
            mesh_editor,
            last_mesh_edit_id,
            ui,
            part,
            actions,
            module_id,
            shared_media_ids,
            preview_context,
        );

        let snapshot_after = part.clone();

        // Detect non-pointer changes (e.g., text edits or buttons that don't hold pointer state long)
        // or check if a drag just ended
        if let Some(before) = self.edit_snapshot.take() {
            if release {
                if before != snapshot_after {
                    self.undo_stack
                        .push(super::types::CanvasAction::UpdatePart {
                            part_id: part.id,
                            before: Box::new(before),
                            after: Box::new(snapshot_after),
                        });
                    self.redo_stack.clear();
                }
            } else {
                // Keep the snapshot if still dragging
                self.edit_snapshot = Some(before);
            }
        } else if snapshot_before != snapshot_after {
            // Immediate change without dragging (e.g. keypress, combo box)
            self.undo_stack
                .push(super::types::CanvasAction::UpdatePart {
                    part_id: part.id,
                    before: Box::new(snapshot_before),
                    after: Box::new(snapshot_after),
                });
            self.redo_stack.clear();
        }
    }
}
