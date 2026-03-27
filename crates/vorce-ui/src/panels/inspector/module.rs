use crate::editors::module_canvas::state::ModuleCanvas;
use crate::UIAction;
use egui::Ui;
use vorce_core::module::{ModulePartId, VorceModule};

/// Show module properties inspector
#[allow(clippy::too_many_arguments)]
pub fn show_module_inspector(
    ui: &mut Ui,
    canvas: &mut ModuleCanvas,
    mesh_editor: &mut crate::editors::mesh_editor::MeshEditor,
    last_mesh_edit_id: &mut Option<u64>,
    module: &mut VorceModule,
    part_id: ModulePartId,
    shared_media_ids: &[String],
    global_actions: &mut Vec<UIAction>,
) {
    let preview_context =
        crate::editors::module_canvas::inspector::build_preview_context(module, part_id);

    if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
        canvas.render_inspector_for_part(
            mesh_editor,
            last_mesh_edit_id,
            ui,
            part,
            global_actions,
            module.id,
            shared_media_ids,
            &preview_context,
        );
    }
}
