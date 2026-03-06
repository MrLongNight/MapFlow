use crate::editors::module_canvas::state::ModuleCanvas;
use crate::UIAction;
use egui::Ui;
use mapmap_core::module::{MapFlowModule, ModulePartId};

/// Show module properties inspector
pub fn show_module_inspector(
    ui: &mut Ui,
    canvas: &mut ModuleCanvas,
    module: &mut MapFlowModule,
    part_id: ModulePartId,
    shared_media_ids: &[String],
    global_actions: &mut Vec<UIAction>,
) {
    if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
        canvas.render_inspector_for_part(ui, part, global_actions, module.id, shared_media_ids);
    }
}
