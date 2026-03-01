use super::mesh;
use super::state::ModuleCanvas;
use crate::UIAction;
use egui::Ui;
use mapmap_core::module::{
    ModuleId, ModulePart, ModulePartType,
};

pub mod common;
pub mod trigger;
pub mod source;
pub mod effect;
pub mod output;
pub mod layer;

pub use common::{render_common_controls, render_transport_controls, render_timeline};
pub use effect::set_default_effect_params;

/// Renders the inspector UI for the selected module part.
pub fn render_inspector_for_part(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    part: &mut ModulePart,
    actions: &mut Vec<UIAction>,
    module_id: ModuleId,
    shared_media_ids: &[String],
) {
    // Sync mesh editor state if needed
    mesh::sync_mesh_editor_to_current_selection(canvas, part);

    let part_id = part.id;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // --- Input Configuration ---
            trigger::render_trigger_config_ui(canvas, ui, part);
            ui.separator();

            match &mut part.part_type {
                ModulePartType::Trigger(trigger) => {
                    trigger::render_trigger_ui(canvas, ui, trigger, part_id);
                }
                ModulePartType::Source(source) => {
                    source::render_source_ui(canvas, ui, source, part_id, module_id, shared_media_ids, actions);
                }
                ModulePartType::Mask(mask) => {
                    layer::render_mask_ui(ui, mask);
                }
                ModulePartType::Modulizer(mod_type) => {
                    effect::render_effect_ui(ui, mod_type, part_id);
                }
                ModulePartType::Layer(layer) => {
                    layer::render_layer_ui(canvas, ui, layer, part_id);
                }
                ModulePartType::Mesh(mesh_data) => {
                    ui.label("🕸️ Mesh Node");
                    ui.separator();
                    mesh::render_mesh_editor_ui(canvas, ui, mesh_data, part_id, part_id);
                }
                ModulePartType::Output(output_type) => {
                    output::render_output_ui(canvas, ui, output_type, part_id);
                }
                ModulePartType::Hue(_) => {
                    ui.label("Hue Node Configuration");
                }
            }
        });
}
