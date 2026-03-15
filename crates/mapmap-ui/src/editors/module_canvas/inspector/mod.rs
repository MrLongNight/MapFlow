pub mod common;
pub mod effect;
pub mod layer;
pub mod output;
pub mod source;
pub mod trigger;
use super::mesh;
use super::state::ModuleCanvas;
use crate::UIAction;
use egui::Ui;
use mapmap_core::module::{
    MapFlowModule, ModuleId, ModulePart, ModulePartId, ModulePartType, OutputType,
};
use std::collections::HashSet;
#[derive(Debug, Clone, Default)]
pub struct InspectorPreviewContext {
    pub output_ids: Vec<u64>,
    pub upstream_source_part_ids: Vec<ModulePartId>,
}
pub fn build_preview_context(
    module: &MapFlowModule,
    part_id: ModulePartId,
) -> InspectorPreviewContext {
    let mut output_ids = Vec::new();
    let mut source_ids = Vec::new();
    collect_downstream_output_ids(module, part_id, &mut HashSet::new(), &mut output_ids);
    collect_upstream_source_ids(module, part_id, &mut HashSet::new(), &mut source_ids);
    output_ids.sort_unstable();
    output_ids.dedup();
    source_ids.sort_unstable();
    source_ids.dedup();
    InspectorPreviewContext {
        output_ids,
        upstream_source_part_ids: source_ids,
    }
}
fn collect_downstream_output_ids(
    module: &MapFlowModule,
    part_id: ModulePartId,
    visited: &mut HashSet<ModulePartId>,
    output_ids: &mut Vec<u64>,
) {
    if !visited.insert(part_id) {
        return;
    }
    for connection in module
        .connections
        .iter()
        .filter(|conn| conn.from_part == part_id)
    {
        if let Some(next_part) = module
            .parts
            .iter()
            .find(|part| part.id == connection.to_part)
        {
            match &next_part.part_type {
                ModulePartType::Output(OutputType::Projector { id, .. }) => output_ids.push(*id),
                _ => collect_downstream_output_ids(module, next_part.id, visited, output_ids),
            }
        }
    }
}
fn collect_upstream_source_ids(
    module: &MapFlowModule,
    part_id: ModulePartId,
    visited: &mut HashSet<ModulePartId>,
    source_ids: &mut Vec<ModulePartId>,
) {
    if !visited.insert(part_id) {
        return;
    }
    if let Some(part) = module.parts.iter().find(|part| part.id == part_id) {
        if matches!(part.part_type, ModulePartType::Source(_)) {
            source_ids.push(part_id);
            return;
        }
    }
    for connection in module
        .connections
        .iter()
        .filter(|conn| conn.to_part == part_id)
    {
        collect_upstream_source_ids(module, connection.from_part, visited, source_ids);
    }
}
#[allow(clippy::too_many_arguments)]
pub fn render_inspector_for_part(
    canvas: &mut ModuleCanvas,
    mesh_editor: &mut crate::editors::mesh_editor::MeshEditor,
    last_mesh_edit_id: &mut Option<u64>,
    ui: &mut Ui,
    part: &mut ModulePart,
    actions: &mut Vec<UIAction>,
    module_id: ModuleId,
    shared_media_ids: &[String],
    _preview_context: &InspectorPreviewContext,
) {
    // Sync mesh editor state if needed
    mesh::sync_mesh_editor_to_current_selection(mesh_editor, last_mesh_edit_id, part);
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
                    source::render_source_ui(
                        canvas,
                        ui,
                        source,
                        part_id,
                        module_id,
                        shared_media_ids,
                        actions,
                    );
                }
                ModulePartType::Mask(mask) => {
                    layer::render_mask_ui(ui, mask);
                }
                ModulePartType::Modulizer(mod_type) => {
                    effect::render_effect_ui(ui, mod_type, part_id);
                }
                ModulePartType::Layer(layer) => {
                    layer::render_layer_ui(
                        canvas,
                        mesh_editor,
                        last_mesh_edit_id,
                        ui,
                        layer,
                        part_id,
                    );
                }
                ModulePartType::Mesh(mesh) => {
                    ui.label("🕸️ Mesh Node");
                    ui.separator();
                    mesh::render_mesh_editor_ui(
                        mesh_editor,
                        last_mesh_edit_id,
                        ui,
                        mesh,
                        part_id,
                        part_id,
                    );
                }
                ModulePartType::Output(output) => {
                    output::render_output_ui(canvas, ui, output, part_id);
                }
                ModulePartType::Hue(_) => {
                    ui.label("Hue Node Configuration");
                }
            }
        });
}
