use crate::i18n::LocaleManager;
use crate::panels::transform_panel::TransformPanel;
use crate::widgets::panel::render_panel_header;

use super::layer::render_layer_inspector;
use super::module::show_module_inspector;
use super::output::show_output_inspector;
use super::types::{InspectorAction, InspectorContext};

use crate::editors::mesh_editor::MeshEditor;

/// The Inspector Panel provides context-sensitive property editing
#[derive(Default)]
pub struct InspectorPanel {
    /// Internal transform panel state
    pub transform_panel: TransformPanel,
    /// Shared Mesh Editor state
    pub mesh_editor: MeshEditor,
    /// Track last edited ID to avoid continuous resetting
    pub last_mesh_edit_id: Option<u64>,
}

impl InspectorPanel {
    /// Render the inspector UI based on the current context
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        context: InspectorContext,
        i18n: &LocaleManager,
        actions: &mut Vec<crate::UIAction>,
    ) -> Option<InspectorAction> {
        let mut action = None;

        // Cyber Header
        render_panel_header(ui, &i18n.t("panel-inspector"), |ui| {
            if ui.button("✕").on_hover_text("Close Inspector").clicked() {
                action = Some(InspectorAction::RequestClose);
            }
        });

        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| match context {
            InspectorContext::None => {
                ui.centered_and_justified(|ui| {
                    crate::widgets::custom::render_info_label(ui, "No selection");
                });
            }
            InspectorContext::Layer {
                layer,
                transform,
                index,
                first_mapping,
            } => {
                action = render_layer_inspector(
                    &mut self.mesh_editor,
                    &mut self.last_mesh_edit_id,
                    ui,
                    layer,
                    transform,
                    index,
                    first_mapping,
                    i18n,
                );
            }
            InspectorContext::Module {
                canvas,
                module,
                part_id,
                shared_media_ids,
            } => {
                show_module_inspector(
                    ui,
                    canvas,
                    &mut self.mesh_editor,
                    &mut self.last_mesh_edit_id,
                    module,
                    part_id,
                    &shared_media_ids,
                    actions,
                );
            }
            InspectorContext::Output(config) => {
                show_output_inspector(ui, config);
            }
        });

        action
    }
}
