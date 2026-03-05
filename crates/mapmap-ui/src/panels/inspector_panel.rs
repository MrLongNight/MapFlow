//! Inspector Panel - Context-sensitive property inspector
//!
//! Shows different content based on current selection:
//! - Layer selected: Transform, Effects, Blend Mode
//! - Output selected: Edge Blend, Calibration, Resolution
//! - Nothing selected: Project Settings summary

use egui::Ui;

use crate::i18n::LocaleManager;
use crate::icons::IconManager;
use crate::theme::colors;
use crate::transform_panel::TransformPanel;
use crate::widgets::panel::{cyber_panel_frame, render_panel_header};

// Re-export types from the new inspector module
pub use crate::panels::inspector::{InspectorAction, InspectorContext};
use crate::panels::inspector::layer::show_layer_inspector;
use crate::panels::inspector::module::show_module_inspector;
use crate::panels::inspector::output::show_output_inspector;

/// The Inspector Panel provides context-sensitive property editing
pub struct InspectorPanel {
    /// Whether the inspector is visible
    pub visible: bool,
    /// Internal transform panel for layer properties
    #[allow(dead_code)]
    transform_panel: TransformPanel,
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self {
            visible: true,
            transform_panel: TransformPanel::default(),
        }
    }
}

impl InspectorPanel {
    /// Show the inspector panel as a right side panel
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        context: InspectorContext<'_>,
        i18n: &LocaleManager,
        _icon_manager: Option<&IconManager>,
        // New params for interactivity & MIDI Learn
        is_learning: bool,
        last_active_element: Option<&String>,
        last_active_time: Option<std::time::Instant>,
        global_actions: &mut Vec<crate::UIAction>,
    ) -> Option<InspectorAction> {
        if !self.visible {
            return None;
        }

        let mut action = None;

        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(400.0)
            .min_width(320.0)
            .max_width(600.0)
            .frame(cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                // Cyber Header
                render_panel_header(ui, &i18n.t("panel-inspector"), |ui| {
                    if ui.button("✕").clicked() {
                        self.visible = false;
                    }
                });

                ui.add_space(8.0);

                // Context-sensitive content
                match context {
                    InspectorContext::None => {
                        self.show_no_selection(ui, i18n);
                    }
                    InspectorContext::Layer {
                        layer,
                        transform,
                        index,
                    } => {
                        action = show_layer_inspector(
                            ui,
                            layer,
                            transform,
                            index,
                            is_learning,
                            last_active_element,
                            last_active_time,
                            global_actions,
                        );
                    }
                    InspectorContext::Output(output) => {
                        show_output_inspector(ui, output);
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
                            module,
                            part_id,
                            &shared_media_ids,
                            global_actions,
                        );
                    }
                }
            });

        action
    }

    /// Show placeholder when nothing is selected
    fn show_no_selection(&self, ui: &mut Ui, _i18n: &LocaleManager) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(
                egui::RichText::new("∅")
                    .size(48.0)
                    .color(colors::CYAN_ACCENT.linear_multiply(0.3)),
            );
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("No Selection")
                    .size(20.0)
                    .strong()
                    .color(colors::CYAN_ACCENT),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Select a layer or output\nto view properties")
                    .size(12.0)
                    .color(egui::Color32::WHITE.linear_multiply(0.5)),
            );
        });
    }
}
