pub mod capabilities;
pub mod common;
pub mod effect;
pub mod layer;
pub mod output;
pub mod source;
pub mod trigger;

pub use effect::set_default_effect_params;

use super::mesh;
use super::state::{LayerInspectorViewMode, ModuleCanvas};
use crate::UIAction;
use egui::{Ui, Vec2};
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

pub fn render_inspector_preview_toggle(canvas: &mut ModuleCanvas, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.heading("Inspector Preview");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(&mut canvas.show_inspector_previews, "Enabled");
        });
    });
}

pub fn render_trigger_preview(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    part_id: ModulePartId,
    extra_ui: impl FnOnce(&mut Ui, f32, bool),
) {
    render_inspector_preview_toggle(canvas, ui);
    if canvas.show_inspector_previews {
        let live_value = canvas
            .last_trigger_values
            .get(&part_id)
            .copied()
            .unwrap_or(0.0);
        let is_live = live_value > 0.1;
        ui.ctx().request_repaint();

        ui.group(|ui| {
            ui.label("Live Trigger Preview");
            ui.add(
                egui::ProgressBar::new(live_value.clamp(0.0, 1.0))
                    .desired_width(ui.available_width())
                    .text(format!("{:.2}", live_value)),
            );

            let status = if is_live { "LIVE pulse" } else { "Waiting" };
            let color = if is_live {
                egui::Color32::from_rgb(110, 235, 150)
            } else {
                egui::Color32::from_rgb(180, 180, 180)
            };
            ui.colored_label(color, status);

            extra_ui(ui, live_value, is_live);
        });
    }
}

pub fn render_preview_texture(ui: &mut Ui, texture_id: egui::TextureId, caption: &str) {
    let width = ui.available_width().max(160.0);
    let size = Vec2::new(width, width * 9.0 / 16.0);
    ui.image((texture_id, size));
    ui.small(caption);
}

pub fn render_standard_texture_preview(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    module_id: ModuleId,
    part_id: ModulePartId,
) {
    render_inspector_preview_toggle(canvas, ui);
    if !canvas.show_inspector_previews {
        return;
    }

    ui.add_space(6.0);
    if let Some(&texture_id) = canvas.node_previews.get(&(module_id, part_id)) {
        render_preview_texture(ui, texture_id, "Live node preview");
    } else {
        crate::widgets::custom::render_missing_preview_banner(ui);
    }
}

pub fn render_output_texture_preview(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    preview_context: &InspectorPreviewContext,
) {
    render_inspector_preview_toggle(canvas, ui);
    if !canvas.show_inspector_previews {
        return;
    }

    ui.add_space(6.0);

    let mut preview_found = false;
    for output_id in &preview_context.output_ids {
        if let Some(&texture_id) = canvas.output_previews.get(output_id) {
            render_preview_texture(
                ui,
                texture_id,
                &format!("Linked output preview (Output {})", output_id),
            );
            preview_found = true;
        }
    }

    if !preview_found {
        crate::widgets::custom::render_missing_preview_banner(ui);
    }
}

pub fn render_layer_preview_panel(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    module_id: ModuleId,
    part_id: ModulePartId,
    preview_context: &InspectorPreviewContext,
) {
    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut canvas.layer_inspector_view_mode,
            LayerInspectorViewMode::Preview,
            "Preview",
        );
        ui.selectable_value(
            &mut canvas.layer_inspector_view_mode,
            LayerInspectorViewMode::MeshEditor,
            "Mesh Editor",
        );
    });

    if canvas.layer_inspector_view_mode != LayerInspectorViewMode::Preview {
        return;
    }

    if !canvas.show_inspector_previews {
        ui.label("Inspector preview is disabled.");
        return;
    }

    ui.add_space(6.0);
    if let Some(&texture_id) = canvas.node_previews.get(&(module_id, part_id)) {
        render_preview_texture(ui, texture_id, "Direct layer preview");
        return;
    }

    for output_id in &preview_context.output_ids {
        if let Some(&texture_id) = canvas.output_previews.get(output_id) {
            render_preview_texture(
                ui,
                texture_id,
                &format!("Linked output preview (Output {})", output_id),
            );
            return;
        }
    }

    for source_part_id in &preview_context.upstream_source_part_ids {
        if let Some(&texture_id) = canvas.node_previews.get(&(module_id, *source_part_id)) {
            render_preview_texture(ui, texture_id, "Fallback: upstream source preview");
            ui.small(
                "The layer preview is falling back to the source texture. If the output stays black, the issue is after the source stage.",
            );
            return;
        }
    }

    ui.group(|ui| {
        crate::widgets::custom::render_info_label(ui, "No preview available yet.");
        if preview_context.output_ids.is_empty() {
            ui.small("This layer is not linked to a projector output yet.");
        } else {
            ui.small(format!(
                "Expected linked output preview for Output {}.",
                preview_context
                    .output_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if preview_context.upstream_source_part_ids.is_empty() {
            crate::widgets::custom::render_info_label(
                ui,
                "No upstream source node was found for this layer.",
            );
        } else {
            crate::widgets::custom::render_info_label(
                ui,
                "Upstream source exists, but no preview texture reached the inspector.",
            );
        }
    });
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
    preview_context: &InspectorPreviewContext,
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
                    render_standard_texture_preview(canvas, ui, module_id, part_id);
                    ui.separator();
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
                    render_standard_texture_preview(canvas, ui, module_id, part_id);
                    ui.separator();
                    layer::render_mask_ui(ui, mask);
                }
                ModulePartType::Modulizer(mod_type) => {
                    render_standard_texture_preview(canvas, ui, module_id, part_id);
                    ui.separator();
                    effect::render_effect_ui(ui, mod_type, part_id);
                }
                ModulePartType::Layer(layer) => {
                    render_inspector_preview_toggle(canvas, ui);
                    render_layer_preview_panel(canvas, ui, module_id, part_id, preview_context);
                    layer::render_layer_ui(canvas, mesh_editor, last_mesh_edit_id, ui, layer, part_id);
                }
                ModulePartType::Mesh(mesh) => {
                    ui.label("🕸️ Mesh Node");
                    ui.separator();
                    crate::widgets::custom::render_info_label(
                        ui,
                        "Live texture preview not applicable. Use the Mesh Editor below.",
                    );
                    ui.separator();
                    mesh::render_mesh_editor_ui(
                        mesh_editor,
                        last_mesh_edit_id,
                        ui,
                        mesh,
                        part_id,
                        part_id,
                        true,
                    );
                }
                ModulePartType::Output(output) => {
                    render_output_texture_preview(canvas, ui, preview_context);
                    ui.separator();
                    output::render_output_ui(canvas, ui, output, part_id);
                }
                ModulePartType::Hue(_) => {
                    ui.label("Hue Node Configuration");
                    ui.separator();
                    crate::widgets::custom::render_info_label(
                        ui,
                        "Live visual preview not available for hardware outputs. Check spatial editor or physical lamps.",
                    );
                }
            }
        });
}
