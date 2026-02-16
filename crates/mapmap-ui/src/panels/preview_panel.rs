//! Preview Panel
//!
//! A collapsible panel that displays output previews in the main UI.
//! Shows thumbnails of configured outputs that have `show_in_preview_panel` enabled.

use egui::{Rect, Ui, Vec2};

/// Output preview information
#[derive(Debug, Clone)]
pub struct OutputPreviewInfo {
    /// Output ID
    pub id: u64,
    /// Output name
    pub name: String,
    /// Whether to show in preview panel
    pub show_in_panel: bool,
    /// Texture handle name (for looking up in texture pool)
    pub texture_name: Option<String>,
    /// Registered egui texture ID for rendering
    pub texture_id: Option<egui::TextureId>,
}

/// Preview Panel for displaying output thumbnails
pub struct PreviewPanel {
    /// Whether the panel is expanded
    pub expanded: bool,
    /// Panel height when expanded
    pub panel_height: f32,
    /// Configured output previews
    pub outputs: Vec<OutputPreviewInfo>,
    /// Selected output for detail view
    pub selected_output: Option<u64>,
}

impl Default for PreviewPanel {
    fn default() -> Self {
        Self {
            expanded: true,
            panel_height: 150.0,
            outputs: Vec::new(),
            selected_output: None,
        }
    }
}

impl PreviewPanel {
    /// Create a new preview panel
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the list of outputs to preview
    pub fn update_outputs(&mut self, outputs: Vec<OutputPreviewInfo>) {
        // Deduplicate: If multiple entries for the same name/id exist, prefer the one with texture_id
        use std::collections::HashMap;
        let mut best_previews: HashMap<u64, OutputPreviewInfo> = HashMap::new();

        for out in outputs {
            if let Some(existing) = best_previews.get(&out.id) {
                // If existing has no texture but current has, replace it
                if existing.texture_id.is_none() && out.texture_id.is_some() {
                    best_previews.insert(out.id, out);
                }
            } else {
                best_previews.insert(out.id, out);
            }
        }

        let mut final_outputs: Vec<_> = best_previews.into_values().collect();
        final_outputs.sort_by(|a, b| a.id.cmp(&b.id));
        self.outputs = final_outputs;
    }

    /// Show the preview panel UI
    /// Returns the rect used by the panel for layout purposes
    pub fn show(&mut self, ui: &mut Ui) -> Rect {
        let panel_rect = ui.available_rect_before_wrap();

        // Header with collapse toggle
        ui.horizontal(|ui| {
            let icon = if self.expanded { "▼" } else { "▶" };
            if ui.button(format!("{} Preview", icon)).clicked() {
                self.expanded = !self.expanded;
            }

            if self.expanded {
                ui.separator();
                ui.label(format!(
                    "{} outputs",
                    self.outputs.iter().filter(|o| o.show_in_panel).count()
                ));

                // Height adjustment
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("−").clicked() && self.panel_height > 80.0 {
                        self.panel_height -= 20.0;
                    }
                    if ui.button("+").clicked() && self.panel_height < 300.0 {
                        self.panel_height += 20.0;
                    }
                });
            }
        });

        if self.expanded {
            ui.add_space(4.0);

            // Preview content area
            let preview_area = egui::Frame::default()
                .fill(ui.style().visuals.extreme_bg_color)
                .inner_margin(8.0)
                .corner_radius(4);

            preview_area.show(ui, |ui| {
                ui.set_min_height(self.panel_height - 40.0);

                // Get outputs that should be shown
                let visible_outputs: Vec<_> = self.outputs.iter()
                    .filter(|o| o.show_in_panel)
                    .collect();

                if visible_outputs.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("No outputs configured for preview.\nAdd a Projector Output node and enable 'Show in Preview Panel'.");
                    });
                } else {
                    // Calculate preview thumbnail size
                    let available_width = ui.available_width();
                    let num_outputs = visible_outputs.len();
                    let spacing = 8.0;
                    let thumbnail_width = ((available_width - spacing * (num_outputs as f32 - 1.0))
                        / num_outputs as f32)
                        .clamp(80.0, 200.0);
                    let thumbnail_height = thumbnail_width * 9.0 / 16.0; // 16:9 aspect ratio

                    ui.horizontal_wrapped(|ui| {
                        for output in visible_outputs {
                            let is_selected = self.selected_output == Some(output.id);

                            // Preview thumbnail frame
                            let response = egui::Frame::default()
                                .fill(if is_selected {
                                    ui.style().visuals.selection.bg_fill
                                } else {
                                    ui.style().visuals.widgets.noninteractive.bg_fill
                                })
                                .stroke(if is_selected {
                                    egui::Stroke::new(2.0, ui.style().visuals.selection.stroke.color)
                                } else {
                                    egui::Stroke::NONE
                                })
                                .corner_radius(4)
                                .inner_margin(4.0)
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        // Allocate space for preview
                                        let (rect, response) = ui.allocate_exact_size(
                                            Vec2::new(thumbnail_width, thumbnail_height),
                                            egui::Sense::click(),
                                        );

                                        // Render texture or placeholder
                                        if let Some(tex_id) = output.texture_id {
                                            // Render the actual GPU texture
                                            ui.painter().image(
                                                tex_id,
                                                rect,
                                                egui::Rect::from_min_max(
                                                    egui::pos2(0.0, 0.0),
                                                    egui::pos2(1.0, 1.0),
                                                ),
                                                egui::Color32::WHITE,
                                            );
                                        } else {
                                            // Draw placeholder background
                                            ui.painter().rect_filled(
                                                rect,
                                                2.0,
                                                egui::Color32::from_gray(40),
                                            );

                                            // Draw "no signal" text
                                            ui.painter().text(
                                                rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                "No Signal",
                                                egui::FontId::proportional(12.0),
                                                egui::Color32::GRAY,
                                            );
                                        }

                                        // Output label
                                        ui.label(&output.name);

                                        response
                                    }).inner
                                });

                            if response.inner.clicked() {
                                self.selected_output = Some(output.id);
                            }
                        }
                    });
                }
            });
        }

        panel_rect
    }

    /// Get the current panel height (for layout calculations)
    pub fn current_height(&self) -> f32 {
        if self.expanded {
            self.panel_height + 30.0 // Header + content
        } else {
            24.0 // Just header
        }
    }
}
