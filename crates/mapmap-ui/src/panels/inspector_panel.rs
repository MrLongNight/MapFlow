//! Inspector Panel - Context-sensitive property inspector
//!
//! Shows different content based on current selection:
//! - Layer selected: Transform, Effects, Blend Mode
//! - Output selected: Edge Blend, Calibration, Resolution
//! - Nothing selected: Project Settings summary

use egui::Ui;

use crate::i18n::LocaleManager;
use crate::icons::IconManager;
use crate::transform_panel::TransformPanel;
use crate::widgets;
use mapmap_core::{Layer, OutputConfig, Transform};

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

/// Represents the current selection context for the inspector
pub enum InspectorContext<'a> {
    /// No selection
    None,
    /// A layer is selected
    Layer {
        layer: &'a Layer,
        transform: &'a Transform,
        index: usize,
    },
    /// An output is selected
    Output(&'a OutputConfig),
    /// A module part is selected
    Module {
        canvas: &'a mut crate::ModuleCanvas,
        module: &'a mut mapmap_core::module::MapFlowModule,
        part_id: mapmap_core::module::ModulePartId,
        shared_media_ids: Vec<String>,
    },
}

impl InspectorPanel {
    /// Show the inspector panel as a right side panel
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        context: InspectorContext<'_>,
        i18n: &LocaleManager,
        icon_manager: Option<&IconManager>,
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
            .default_width(300.0)
            .min_width(250.0)
            .max_width(450.0)
            .show(ctx, |ui| {
                // Header
                widgets::render_panel_header(
                    ui,
                    &i18n.t("panel-inspector"),
                    |ui| {
                        if ui.button("✕").clicked() {
                            self.visible = false;
                        }
                    },
                );

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
                        action = self.show_layer_inspector(
                            ui,
                            layer,
                            transform,
                            index,
                            i18n,
                            icon_manager,
                            is_learning,
                            last_active_element,
                            last_active_time,
                            global_actions,
                        );
                    }
                    InspectorContext::Output(output) => {
                        self.show_output_inspector(ui, output, i18n);
                    }
                    InspectorContext::Module {
                        canvas,
                        module,
                        part_id,
                        shared_media_ids,
                    } => {
                        if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                            canvas.render_inspector_for_part(
                                ui,
                                part,
                                global_actions,
                                module.id,
                                &shared_media_ids,
                            );
                        } else {
                            self.show_no_selection(ui, i18n);
                        }
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
                egui::RichText::new("No Selection")
                    .size(16.0)
                    .color(egui::Color32::from_rgb(120, 120, 140)),
            );
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Select a layer or output to view properties")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(100, 100, 120)),
            );
        });
    }

    /// Show layer properties inspector
    #[allow(clippy::too_many_arguments)]
    fn show_layer_inspector(
        &mut self,
        ui: &mut Ui,
        layer: &Layer,
        transform: &Transform,
        index: usize,
        _i18n: &LocaleManager,
        _icon_manager: Option<&IconManager>,
        is_learning: bool,
        last_active_element: Option<&String>,
        last_active_time: Option<std::time::Instant>,
        global_actions: &mut Vec<crate::UIAction>,
    ) -> Option<InspectorAction> {
        let mut action = None;

        // Layer header with icon
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📦").size(18.0));
            ui.label(egui::RichText::new(&layer.name).size(14.0).strong());
        });
        ui.separator();

        // Transform section
        egui::CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                // Position
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.label(format!(
                        "({:.1}, {:.1})",
                        transform.position.x, transform.position.y
                    ));
                });

                // Scale
                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    ui.label(format!(
                        "({:.2}, {:.2})",
                        transform.scale.x, transform.scale.y
                    ));
                });

                // Rotation
                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    ui.label(format!("{:.1}°", transform.rotation.z.to_degrees()));
                });
            });

        // Blending section
        egui::CollapsingHeader::new("Blending")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Opacity:");

                    let mut opacity = layer.opacity;
                    let response = ui.add(egui::Slider::new(&mut opacity, 0.0..=1.0));
                    if response.changed() {
                        action = Some(InspectorAction::UpdateOpacity(layer.id, opacity));
                    }

                    // MIDI Learn
                    use mapmap_control::target::ControlTarget;
                    crate::AppUI::midi_learn_helper(
                        ui,
                        &response,
                        ControlTarget::LayerOpacity(index as u32),
                        is_learning,
                        last_active_element,
                        last_active_time,
                        global_actions,
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Blend Mode:");
                    ui.label(format!("{:?}", layer.blend_mode));
                });
            });

        // Layer state
        egui::CollapsingHeader::new("State")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Visible:");
                    ui.label(if layer.visible { "✅" } else { "❌" });
                });

                ui.horizontal(|ui| {
                    ui.label("Solo:");
                    ui.label(if layer.solo { "🔊" } else { "—" });
                });

                ui.horizontal(|ui| {
                    ui.label("Bypass:");
                    ui.label(if layer.bypass { "⏸" } else { "—" });
                });
            });

        action
    }

    /// Show output properties inspector
    fn show_output_inspector(&self, ui: &mut Ui, output: &OutputConfig, _i18n: &LocaleManager) {
        // Output header
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🖥").size(18.0));
            ui.label(egui::RichText::new(&output.name).size(14.0).strong());
        });
        ui.separator();

        // Resolution section
        egui::CollapsingHeader::new("Resolution")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.label(format!("{}x{}", output.resolution.0, output.resolution.1));
                });
            });

        // Canvas Region section
        egui::CollapsingHeader::new("Canvas Region")
            .default_open(true)
            .show(ui, |ui| {
                let region = &output.canvas_region;
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.label(format!("({:.0}, {:.0})", region.x, region.y));
                });
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.label(format!("{:.0}x{:.0}", region.width, region.height));
                });
            });

        // Edge Blend indicator
        egui::CollapsingHeader::new("Edge Blend")
            .default_open(false)
            .show(ui, |ui| {
                let eb = &output.edge_blend;
                ui.horizontal(|ui| {
                    ui.label("Left:");
                    ui.label(format!("{:.0}px", eb.left.width * 100.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Right:");
                    ui.label(format!("{:.0}px", eb.right.width * 100.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Top:");
                    ui.label(format!("{:.0}px", eb.top.width * 100.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Bottom:");
                    ui.label(format!("{:.0}px", eb.bottom.width * 100.0));
                });
            });
    }
}

/// Actions that can be triggered from the Inspector
#[derive(Debug, Clone)]
pub enum InspectorAction {
    /// Update layer transform
    UpdateTransform(u64, Transform),
    /// Update layer opacity
    UpdateOpacity(u64, f32),
}
