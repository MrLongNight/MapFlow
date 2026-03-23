#[allow(unused_imports)]
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, TextureHandle, Ui, Vec2};

#[allow(unused_imports)]
use crate::config::{MidiAssignment, MidiAssignmentTarget, UserConfig};

#[cfg(feature = "midi")]
use mapmap_control::midi::{ControllerElement, ElementState, ElementType};
use mapmap_core::runtime_paths;
#[allow(unused_imports)]
use std::collections::{HashMap, HashSet};

use super::panel::ControllerOverlayPanel;

impl ControllerOverlayPanel {
    /// Load resources (background and assets)
    pub(crate) fn ensure_resources_loaded(&mut self, ctx: &egui::Context) {
        // Load Background
        if self.background_texture.is_none() {
            let bg_paths = [
                runtime_paths::resource_path("controllers/ecler_nuo4/background.png"),
                runtime_paths::resource_path("controllers/ecler_nuo4/background.jpg"),
            ];
            if let Some(tex) = self.load_texture_from_candidates(ctx, &bg_paths, "mixer_background")
            {
                self.background_texture = Some(tex);
                tracing::info!("Loaded mixer background");
            }
        }

        // Load Assets
        #[cfg(feature = "midi")]
        if let Some(elements) = &self.elements {
            let mut needed = HashSet::new();
            for el in &elements.elements {
                if let Some(asset) = &el.asset {
                    needed.insert(asset.clone());
                }
            }

            for asset_name in needed {
                if !self.assets.contains_key(&asset_name) {
                    let paths = [runtime_paths::resource_path(
                        std::path::Path::new("controllers/ecler_nuo4").join(&asset_name),
                    )];
                    if let Some(tex) = self.load_texture_from_candidates(ctx, &paths, &asset_name) {
                        self.assets.insert(asset_name, tex);
                    }
                }
            }
        }
    }
    #[cfg(feature = "midi")]
    pub(crate) fn draw_asset(
        painter: &egui::Painter,
        assets: &HashMap<String, TextureHandle>,
        scale: f32,
        elem_rect: Rect,
        element: &ControllerElement,
        value: f32,
    ) {
        if let Some(asset_name) = &element.asset {
            if let Some(texture) = assets.get(asset_name) {
                match element.element_type {
                    ElementType::Fader => {
                        let asset_size = texture.size_vec2() * scale;
                        let travel = elem_rect.height() - asset_size.y;
                        // Avoid negative travel
                        let travel = travel.max(0.0);
                        let y = elem_rect.max.y - asset_size.y - (value * travel);

                        let cap_rect = Rect::from_min_size(
                            Pos2::new(elem_rect.center().x - asset_size.x * 0.5, y),
                            asset_size,
                        );

                        painter.image(
                            texture.id(),
                            cap_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    ElementType::Knob | ElementType::Encoder => {
                        let asset_size = texture.size_vec2() * scale;
                        let knob_rect = Rect::from_center_size(elem_rect.center(), asset_size);
                        painter.image(
                            texture.id(),
                            knob_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    _ => {}
                }
            }
        }
    }
    #[allow(unused_variables)]
    pub(crate) fn show_overlay_view(&mut self, ui: &mut Ui, assignments: &[MidiAssignment]) {
        let (base_w, base_h) = if let Some(tex) = &self.background_texture {
            let size = tex.size();
            (size[0] as f32, size[1] as f32)
        } else {
            (MAX_WIDTH, MAX_HEIGHT)
        };

        let panel_width = base_w * self.scale;
        let panel_height = base_h * self.scale;

        // Allocate space for the overlay
        let (response, painter) = ui.allocate_painter(
            Vec2::new(panel_width, panel_height),
            Sense::click_and_drag(),
        );

        let rect = response.rect;

        // Draw background image
        if let Some(texture) = &self.background_texture {
            painter.image(
                texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            // Fallback: dark background
            let bg_color = Color32::from_rgb(30, 30, 35);
            painter.rect_filled(rect, 4.0, bg_color);
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(2.0, Color32::from_rgb(80, 80, 80)),
                egui::StrokeKind::Middle,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Hintergrundbild wird geladen...",
                egui::FontId::default(),
                Color32::WHITE,
            );
        }

        // Draw elements with frames
        #[cfg(feature = "midi")]
        if self.is_edit_mode {
            // Local state to avoid borrow conflicts
            let mut next_selection = self.selected_element.clone();
            let mut next_clipboard = self.clipboard_size;

            if let Some(elements) = &mut self.elements {
                for element in &mut elements.elements {
                    let elem_rect = Rect::from_min_size(
                        Pos2::new(
                            rect.min.x + element.position.x * rect.width(),
                            rect.min.y + element.position.y * rect.height(),
                        ),
                        Vec2::new(
                            element.position.width * rect.width(),
                            element.position.height * rect.height(),
                        ),
                    );

                    // Resize Handle Rect (Bottom-Right)
                    let handle_size = 10.0;
                    let handle_rect = Rect::from_min_size(
                        elem_rect.max - Vec2::splat(handle_size),
                        Vec2::splat(handle_size),
                    );

                    // Interactions
                    let id = ui.make_persistent_id(&element.id);
                    let handle_id = id.with("resize");

                    // Define Move interaction (click_and_drag for selection)
                    let move_interact = ui.interact(elem_rect, id, Sense::click_and_drag());
                    // Define Handle interaction
                    let handle_interact = ui.interact(handle_rect, handle_id, Sense::drag());

                    // Handle Selection
                    if move_interact.clicked() || move_interact.dragged() {
                        next_selection = Some(element.id.clone());
                    }

                    let is_selected = next_selection.as_ref() == Some(&element.id);
                    let mut anim_handle_dragged = false;

                    // Animation Handles (Fader only)
                    if is_selected && element.element_type == ElementType::Fader {
                        let mut range = element.animation_range.unwrap_or([0.0, 1.0]);
                        let mut changed = false;

                        let h = elem_rect.height();
                        let y_top = elem_rect.min.y + range[0] * h;
                        let y_bot = elem_rect.min.y + range[1] * h;

                        // Handles
                        let h_top_rect = Rect::from_center_size(
                            Pos2::new(elem_rect.center().x, y_top),
                            Vec2::new(elem_rect.width(), 8.0),
                        );
                        let h_bot_rect = Rect::from_center_size(
                            Pos2::new(elem_rect.center().x, y_bot),
                            Vec2::new(elem_rect.width(), 8.0),
                        );

                        let h_top_res = ui.interact(h_top_rect, id.with("h_top"), Sense::drag());
                        let h_bot_res = ui.interact(h_bot_rect, id.with("h_bot"), Sense::drag());

                        if h_top_res.dragged() {
                            range[0] += h_top_res.drag_delta().y / h;
                            changed = true;
                            anim_handle_dragged = true;
                        }
                        if h_bot_res.dragged() {
                            range[1] += h_bot_res.drag_delta().y / h;
                            changed = true;
                            anim_handle_dragged = true;
                        }

                        if changed {
                            element.animation_range = Some(range);
                        }

                        // Draw Handles
                        painter.line_segment(
                            [
                                Pos2::new(elem_rect.min.x, y_top),
                                Pos2::new(elem_rect.max.x, y_top),
                            ],
                            Stroke::new(1.0, Color32::RED),
                        );
                        painter.line_segment(
                            [
                                Pos2::new(elem_rect.min.x, y_bot),
                                Pos2::new(elem_rect.max.x, y_bot),
                            ],
                            Stroke::new(1.0, Color32::RED),
                        );

                        painter.text(
                            Pos2::new(elem_rect.max.x + 2.0, y_top),
                            egui::Align2::LEFT_CENTER,
                            "Top",
                            egui::FontId::proportional(12.0),
                            Color32::RED,
                        );
                        painter.text(
                            Pos2::new(elem_rect.max.x + 2.0, y_bot),
                            egui::Align2::LEFT_CENTER,
                            "Bot",
                            egui::FontId::proportional(12.0),
                            Color32::RED,
                        );
                    }

                    // Logic
                    if handle_interact.dragged() {
                        let delta = handle_interact.drag_delta();
                        if rect.width() > 0.0 && rect.height() > 0.0 {
                            element.position.width += delta.x / rect.width();
                            element.position.height += delta.y / rect.height();
                            element.position.width = element.position.width.max(0.01);
                            element.position.height = element.position.height.max(0.01);
                        }
                    } else if anim_handle_dragged {
                        // Consumed by animation handles
                    } else if move_interact.dragged() {
                        let delta = move_interact.drag_delta();
                        if rect.width() > 0.0 && rect.height() > 0.0 {
                            element.position.x += delta.x / rect.width();
                            element.position.y += delta.y / rect.height();
                        }
                    }

                    // Keyboard Actions (only if selected)
                    let is_selected = next_selection.as_ref() == Some(&element.id);
                    if is_selected {
                        let mut delta_key = Vec2::ZERO;
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                            delta_key.x -= 1.0;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                            delta_key.x += 1.0;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            delta_key.y -= 1.0;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            delta_key.y += 1.0;
                        }

                        if delta_key != Vec2::ZERO && rect.width() > 0.0 && rect.height() > 0.0 {
                            element.position.x += delta_key.x / rect.width();
                            element.position.y += delta_key.y / rect.height();
                        }

                        // Copy / Paste Size (Ctrl+C / Ctrl+V)
                        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C)) {
                            next_clipboard =
                                Some([element.position.width, element.position.height]);
                        }
                        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::V)) {
                            if let Some(size) = next_clipboard {
                                element.position.width = size[0];
                                element.position.height = size[1];
                            }
                        }
                    }

                    // Draw Asset (in background of frame)
                    let current_val = self
                        .state_manager
                        .get(&element.id)
                        .map(|s| s.value as f32 / 127.0)
                        .unwrap_or(0.0);
                    Self::draw_asset(
                        &painter,
                        &self.assets,
                        self.scale,
                        elem_rect,
                        element,
                        current_val,
                    );

                    // Draw edit frame
                    let base_color = if is_selected {
                        Color32::from_rgb(255, 0, 255)
                    } else {
                        Color32::YELLOW
                    }; // Magenta for selected
                    let stroke = Stroke::new(if is_selected { 2.0 } else { 1.0 }, base_color);

                    match element.element_type {
                        ElementType::Knob | ElementType::Encoder => {
                            let radius = elem_rect.width().min(elem_rect.height()) / 2.0;
                            painter.circle_stroke(elem_rect.center(), radius, stroke);
                        }
                        _ => {
                            painter.rect_stroke(elem_rect, 0.0, stroke, egui::StrokeKind::Middle);
                        }
                    }

                    // Draw resize handle
                    painter.rect_filled(handle_rect, 2.0, Color32::from_rgb(0, 255, 255));
                }
            }

            // Apply state updates
            self.selected_element = next_selection;
            self.clipboard_size = next_clipboard;
        } else if let Some(elements) = self.elements.clone() {
            for element in &elements.elements {
                self.draw_element_with_frame(&painter, rect, element, &response, assignments);
            }
        }
    }
    #[cfg(feature = "midi")]
    pub(crate) fn draw_element_with_frame(
        &mut self,
        painter: &egui::Painter,
        container: Rect,
        element: &ControllerElement,
        response: &Response,
        assignments: &[MidiAssignment],
    ) {
        // Calculate element rect based on relative position
        let elem_rect = Rect::from_min_size(
            Pos2::new(
                container.min.x + element.position.x * container.width(),
                container.min.y + element.position.y * container.height(),
            ),
            Vec2::new(
                element.position.width * container.width(),
                element.position.height * container.height(),
            ),
        );

        // Check states
        let state = self.state_manager.get(&element.id);

        // Draw Asset
        let val = state.map(|s| s.value as f32 / 127.0).unwrap_or(0.0);
        Self::draw_asset(painter, &self.assets, self.scale, elem_rect, element, val);

        let is_hovered = response
            .hover_pos()
            .map(|pos| elem_rect.contains(pos))
            .unwrap_or(false);
        let is_selected = self.selected_element.as_ref() == Some(&element.id);
        let is_learning = self.learn_manager.is_learning()
            && self.learn_manager.state().target_element() == Some(element.id.as_str());
        // Check if element was recently updated (within last 200ms)
        let is_active = state
            .map(|s: &ElementState| s.last_update.elapsed().as_millis() < 200)
            .unwrap_or(false);

        // Determine frame color based on state
        let frame_color = if is_learning {
            // Pulsing yellow for learn mode
            let t = (ui_time_seconds() * 3.0).sin() * 0.5 + 0.5;
            Color32::from_rgba_unmultiplied(255, 220, 0, (128.0 + 127.0 * t as f32) as u8)
        } else if is_active {
            Color32::GREEN
        } else if is_selected {
            Color32::from_rgb(100, 149, 237) // Cornflower blue
        } else if is_hovered {
            Color32::WHITE
        } else {
            Color32::TRANSPARENT
        };

        // Override colors assignments view is active
        let frame_color = if self.show_assignment_colors {
            let assignment = assignments.iter().find(|a| a.element_id == element.id);
            match assignment {
                Some(a) => match &a.target {
                    MidiAssignmentTarget::MapFlow(_) => Color32::from_rgb(0, 150, 255), // Blue
                    MidiAssignmentTarget::StreamerBot(_) => Color32::from_rgb(180, 0, 255), // Purple
                    MidiAssignmentTarget::Mixxx(_) => Color32::from_rgb(255, 128, 0), // Orange
                },
                None => Color32::GREEN, // Green for free elements
            }
        } else {
            frame_color
        };

        // Draw frame
        if frame_color != Color32::TRANSPARENT {
            let stroke_width = if is_learning { 3.0 } else { 2.0 };
            let stroke = Stroke::new(stroke_width, frame_color);
            match element.element_type {
                ElementType::Knob | ElementType::Encoder => {
                    let radius = elem_rect.width().min(elem_rect.height()) / 2.0;
                    painter.circle_stroke(elem_rect.center(), radius, stroke);
                }
                _ => {
                    painter.rect_stroke(elem_rect, 4.0, stroke, egui::StrokeKind::Middle);
                }
            }
        }

        // Update hovered element for tooltip
        if is_hovered {
            self.hovered_element = Some(element.id.clone());
        }

        // Handle click for MIDI learn
        if response.clicked() && is_hovered {
            if let Some(target) = &self.learn_target {
                self.learn_manager.start_learning(&element.id);
                tracing::info!(
                    "Started MIDI learn for {} with target {:?}",
                    element.id,
                    target
                );
            } else {
                self.selected_element = Some(element.id.clone());
            }
        }

        // Show tooltip on hover
        if is_hovered {
            #[allow(deprecated)]
            egui::show_tooltip(
                painter.ctx(),
                egui::LayerId::background(),
                egui::Id::new(&element.id),
                |ui| {
                    ui.strong(&element.label);
                    ui.label(format!("ID: {}", element.id));
                    ui.label(format!("Typ: {:?}", element.element_type));
                    if let Some(midi) = &element.midi {
                        ui.label(format!("MIDI: {:?}", midi));
                    }
                    if let Some(state) = state {
                        ui.label(format!("Wert: {:.2}", state.value));
                    }

                    // Show assignment info
                    let assignment = assignments.iter().find(|a| a.element_id == element.id);
                    if let Some(assign) = assignment {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Zuweisung:");
                            ui.colored_label(Color32::YELLOW, assign.target.to_string());
                        });
                        crate::widgets::custom::render_info_label_with_size(
                            ui,
                            "(Klick für Details in Liste)",
                            10.0,
                        );
                    } else {
                        ui.separator();
                        crate::widgets::custom::render_info_label(ui, "Nicht zugewiesen");
                    }
                },
            );
        }
    }
}

#[allow(dead_code)]
fn ui_time_seconds() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
/// Maximum size of the overlay (matches mixer photo resolution)
pub(crate) const MAX_WIDTH: f32 = 841.0;
pub(crate) const MAX_HEIGHT: f32 = 1024.0;
