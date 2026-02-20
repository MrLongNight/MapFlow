//! Inspector Panel - Context-sensitive property inspector
//!
//! Shows different content based on current selection:
//! - Layer selected: Transform, Effects, Blend Mode
//! - Output selected: Edge Blend, Calibration, Resolution
//! - Nothing selected: Project Settings summary

use egui::{Color32, Ui};

use crate::i18n::LocaleManager;
use crate::icons::IconManager;
use crate::theme::colors;
use crate::transform_panel::TransformPanel;
use crate::widgets::custom::styled_slider;
use crate::widgets::panel::{cyber_panel_frame, render_panel_header};
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
        render_panel_header(ui, &format!("📦 {}", layer.name), |_| {});
        ui.add_space(8.0);

        // Transform section
        inspector_section(ui, "Transform", true, |ui| {
            inspector_row(ui, "Position", |ui| {
                inspector_value(
                    ui,
                    &format!("({:.1}, {:.1})", transform.position.x, transform.position.y),
                );
            });

            inspector_row(ui, "Scale", |ui| {
                inspector_value(
                    ui,
                    &format!("({:.2}, {:.2})", transform.scale.x, transform.scale.y),
                );
            });

            inspector_row(ui, "Rotation", |ui| {
                inspector_value(ui, &format!("{:.1}°", transform.rotation.z.to_degrees()));
            });
        });

        ui.add_space(4.0);

        // Blending section
        inspector_section(ui, "Blending", true, |ui| {
            inspector_row(ui, "Opacity", |ui| {
                let mut opacity = layer.opacity;
                let response = styled_slider(ui, &mut opacity, 0.0..=1.0, 1.0);
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

            inspector_row(ui, "Blend Mode", |ui| {
                inspector_value(ui, &format!("{:?}", layer.blend_mode));
            });
        });

        ui.add_space(4.0);

        // Layer state
        inspector_section(ui, "State", true, |ui| {
            inspector_row(ui, "Visible", |ui| {
                let (text, color) = if layer.visible {
                    ("Visible", colors::CYAN_ACCENT)
                } else {
                    ("Hidden", Color32::GRAY)
                };
                ui.label(egui::RichText::new(text).color(color).strong());
            });

            inspector_row(ui, "Solo", |ui| {
                if layer.solo {
                    ui.label(
                        egui::RichText::new("ACTIVE")
                            .color(colors::MINT_ACCENT)
                            .strong(),
                    );
                } else {
                    ui.label(egui::RichText::new("—").color(Color32::GRAY));
                }
            });

            inspector_row(ui, "Bypass", |ui| {
                if layer.bypass {
                    ui.label(
                        egui::RichText::new("ACTIVE")
                            .color(colors::WARN_COLOR)
                            .strong(),
                    );
                } else {
                    ui.label(egui::RichText::new("—").color(Color32::GRAY));
                }
            });
        });

        action
    }

    /// Show output properties inspector
    fn show_output_inspector(&self, ui: &mut Ui, output: &OutputConfig, _i18n: &LocaleManager) {
        // Header
        render_panel_header(ui, &format!("🖥 {}", output.name), |_| {});
        ui.add_space(8.0);

        // Resolution section
        inspector_section(ui, "Resolution", true, |ui| {
            inspector_row(ui, "Size", |ui| {
                inspector_value(
                    ui,
                    &format!("{}x{}", output.resolution.0, output.resolution.1),
                );
            });
        });

        ui.add_space(4.0);

        // Canvas Region section
        inspector_section(ui, "Canvas Region", true, |ui| {
            let region = &output.canvas_region;
            inspector_row(ui, "Position", |ui| {
                inspector_value(ui, &format!("({:.0}, {:.0})", region.x, region.y));
            });

            inspector_row(ui, "Size", |ui| {
                inspector_value(ui, &format!("{:.0}x{:.0}", region.width, region.height));
            });
        });

        ui.add_space(4.0);

        // Edge Blend indicator
        inspector_section(ui, "Edge Blend", false, |ui| {
            let eb = &output.edge_blend;
            inspector_row(ui, "Left", |ui| {
                inspector_value(ui, &format!("{:.0}px", eb.left.width * 100.0));
            });
            inspector_row(ui, "Right", |ui| {
                inspector_value(ui, &format!("{:.0}px", eb.right.width * 100.0));
            });
            inspector_row(ui, "Top", |ui| {
                inspector_value(ui, &format!("{:.0}px", eb.top.width * 100.0));
            });
            inspector_row(ui, "Bottom", |ui| {
                inspector_value(ui, &format!("{:.0}px", eb.bottom.width * 100.0));
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

// --- Visual Helpers ---

fn inspector_section(
    ui: &mut Ui,
    title: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut Ui),
) {
    egui::CollapsingHeader::new(title)
        .default_open(default_open)
        .show(ui, |ui| {
            egui::Frame::NONE
                .fill(colors::LIGHTER_GREY)
                .inner_margin(8.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    add_contents(ui);
                });
        });
}

fn inspector_row(ui: &mut Ui, label: &str, add_contents: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            add_contents(ui);
        });
    });
}

fn inspector_value(ui: &mut Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(Color32::WHITE).size(12.0));
}
