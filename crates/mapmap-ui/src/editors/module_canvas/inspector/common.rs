use super::super::state::ModuleCanvas;
use super::super::types::MediaPlaybackCommand;
use crate::widgets::{styled_drag_value, styled_slider};
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::module::{BlendModeType, ModulePartId};

use super::super::state::LayerInspectorViewMode;
use super::InspectorPreviewContext;
use egui::ProgressBar;
use mapmap_core::module::ModuleId;

/// Renders the common transform and color correction controls for a media source.
#[allow(clippy::too_many_arguments)]
pub fn render_common_controls(
    ui: &mut Ui,
    opacity: &mut f32,
    blend_mode: &mut Option<BlendModeType>,
    brightness: &mut f32,
    contrast: &mut f32,
    saturation: &mut f32,
    hue_shift: &mut f32,
    scale_x: &mut f32,
    scale_y: &mut f32,
    rotation: &mut f32,
    offset_x: &mut f32,
    offset_y: &mut f32,
    flip_horizontal: &mut bool,
    flip_vertical: &mut bool,
) {
    // === APPEARANCE ===
    ui.collapsing("\u{1F3A8} Appearance", |ui| {
        egui::Grid::new("appearance_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Opacity:");
                styled_slider(ui, opacity, 0.0..=1.0, 1.0);
                ui.end_row();

                ui.label("Blend Mode:");
                egui::ComboBox::from_id_salt("blend_mode_selector")
                    .selected_text(match blend_mode {
                        Some(BlendModeType::Normal) => "Normal",
                        Some(BlendModeType::Add) => "Add",
                        Some(BlendModeType::Multiply) => "Multiply",
                        Some(BlendModeType::Screen) => "Screen",
                        Some(BlendModeType::Overlay) => "Overlay",
                        Some(BlendModeType::Difference) => "Difference",
                        Some(BlendModeType::Exclusion) => "Exclusion",
                        None => "Normal",
                    })
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(blend_mode.is_none(), "Normal")
                            .clicked()
                        {
                            *blend_mode = None;
                        }
                        if ui
                            .selectable_label(*blend_mode == Some(BlendModeType::Add), "Add")
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Add);
                        }
                        if ui
                            .selectable_label(
                                *blend_mode == Some(BlendModeType::Multiply),
                                "Multiply",
                            )
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Multiply);
                        }
                        if ui
                            .selectable_label(*blend_mode == Some(BlendModeType::Screen), "Screen")
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Screen);
                        }
                        if ui
                            .selectable_label(
                                *blend_mode == Some(BlendModeType::Overlay),
                                "Overlay",
                            )
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Overlay);
                        }
                        if ui
                            .selectable_label(
                                *blend_mode == Some(BlendModeType::Difference),
                                "Difference",
                            )
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Difference);
                        }
                        if ui
                            .selectable_label(
                                *blend_mode == Some(BlendModeType::Exclusion),
                                "Exclusion",
                            )
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Exclusion);
                        }
                    });
                ui.end_row();
            });
    });

    // === COLOR CORRECTION ===
    if crate::widgets::collapsing_header_with_reset(ui, "\u{1F308} Color Correction", false, |ui| {
        egui::Grid::new("color_correction_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Brightness:");
                styled_slider(ui, brightness, -1.0..=1.0, 0.0);
                ui.end_row();

                ui.label("Contrast:");
                styled_slider(ui, contrast, 0.0..=2.0, 1.0);
                ui.end_row();

                ui.label("Saturation:");
                styled_slider(ui, saturation, 0.0..=2.0, 1.0);
                ui.end_row();

                ui.label("Hue Shift:");
                styled_slider(ui, hue_shift, -180.0..=180.0, 0.0);
                ui.end_row();
            });
    }) {
        *brightness = 0.0;
        *contrast = 1.0;
        *saturation = 1.0;
        *hue_shift = 0.0;
    }

    // === TRANSFORM ===
    if crate::widgets::collapsing_header_with_reset(ui, "📐 Transform", false, |ui| {
        egui::Grid::new("transform_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Scale:");
                ui.horizontal(|ui| {
                    styled_drag_value(ui, scale_x, 0.01, 0.0..=10.0, 1.0, "X: ", "");
                    styled_drag_value(ui, scale_y, 0.01, 0.0..=10.0, 1.0, "Y: ", "");
                });
                ui.end_row();

                ui.label("Offset:");
                ui.horizontal(|ui| {
                    styled_drag_value(ui, offset_x, 1.0, -2000.0..=2000.0, 0.0, "X: ", "px");
                    styled_drag_value(ui, offset_y, 1.0, -2000.0..=2000.0, 0.0, "Y: ", "px");
                });
                ui.end_row();

                ui.label("Rotation:");
                styled_slider(ui, rotation, -180.0..=180.0, 0.0);
                ui.end_row();

                ui.label("Mirror:");
                ui.horizontal(|ui| {
                    ui.checkbox(flip_horizontal, "X");
                    ui.checkbox(flip_vertical, "Y");
                });
                ui.end_row();
            });
    }) {
        *scale_x = 1.0;
        *scale_y = 1.0;
        *rotation = 0.0;
        *offset_x = 0.0;
        *offset_y = 0.0;
        *flip_horizontal = false;
        *flip_vertical = false;
    }
}

/// Renders the transport controls for media playback (play, pause, stop, loop, reverse).
pub fn render_transport_controls(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    part_id: ModulePartId,
    is_playing: bool,
    current_pos: f32,
    loop_enabled: &mut bool,
    reverse_playback: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing.x = 8.0;
        let button_height = 42.0;
        let big_btn_size = Vec2::new(70.0, button_height);
        let small_btn_size = Vec2::new(40.0, button_height);

        // PLAY (Primary Action - Green)
        let play_btn = egui::Button::new(egui::RichText::new("\u{25B6}").size(24.0))
            .min_size(big_btn_size)
            .fill(if is_playing {
                Color32::from_rgb(40, 180, 60)
            } else {
                Color32::from_gray(50)
            });
        if ui.add(play_btn).on_hover_text("Play").clicked() {
            canvas
                .pending_playback_commands
                .push((part_id, MediaPlaybackCommand::Play));
        }

        // PAUSE (Secondary Action - Yellow)
        let pause_btn = egui::Button::new(egui::RichText::new("⏸").size(24.0))
            .min_size(big_btn_size)
            .fill(if !is_playing && current_pos > 0.1 {
                Color32::from_rgb(200, 160, 40)
            } else {
                Color32::from_gray(50)
            });
        if ui.add(pause_btn).on_hover_text("Pause").clicked() {
            canvas
                .pending_playback_commands
                .push((part_id, MediaPlaybackCommand::Pause));
        }

        // Safety Spacer
        ui.add_space(24.0);
        ui.separator();
        ui.add_space(8.0);

        // STOP (Destructive Action - Separated)
        if crate::widgets::hold_to_action_button(ui, "⏹", Color32::from_rgb(255, 80, 80)) {
            canvas
                .pending_playback_commands
                .push((part_id, MediaPlaybackCommand::Stop));
        }

        // LOOP
        let loop_color = if *loop_enabled {
            Color32::from_rgb(80, 150, 255)
        } else {
            Color32::from_gray(45)
        };
        if ui
            .add(
                egui::Button::new(egui::RichText::new("🔁").size(18.0))
                    .min_size(small_btn_size)
                    .fill(loop_color),
            )
            .on_hover_text("Toggle Loop")
            .clicked()
        {
            *loop_enabled = !*loop_enabled;
            canvas
                .pending_playback_commands
                .push((part_id, MediaPlaybackCommand::SetLoop(*loop_enabled)));
        }

        // REVERSE
        let rev_color = if *reverse_playback {
            Color32::from_rgb(200, 80, 80)
        } else {
            Color32::from_gray(45)
        };
        if ui
            .add(
                egui::Button::new(egui::RichText::new("⏪").size(18.0))
                    .min_size(small_btn_size)
                    .fill(rev_color),
            )
            .on_hover_text("Toggle Reverse Playback")
            .clicked()
        {
            *reverse_playback = !*reverse_playback;
        }
    });
}

/// Renders an interactive timeline for seeking within a media clip.
pub fn render_timeline(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    part_id: ModulePartId,
    video_duration: f32,
    current_pos: f32,
    start_time: &mut f32,
    end_time: &mut f32,
) {
    let (response, painter) = ui.allocate_painter(
        Vec2::new(ui.available_width(), 32.0),
        Sense::click_and_drag(),
    );
    let rect = response.rect;

    // Background (Full Track)
    painter.rect_filled(rect, 0.0, Color32::from_gray(30));
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0 * canvas.zoom, Color32::from_gray(60)),
        egui::StrokeKind::Middle,
    );

    // Data normalization
    let effective_end = if *end_time > 0.0 {
        *end_time
    } else {
        video_duration
    };
    let start_x = rect.min.x + (*start_time / video_duration).clamp(0.0, 1.0) * rect.width();
    let end_x = rect.min.x + (effective_end / video_duration).clamp(0.0, 1.0) * rect.width();

    // Active Region Highlight
    let region_rect =
        Rect::from_min_max(Pos2::new(start_x, rect.min.y), Pos2::new(end_x, rect.max.y));
    painter.rect_filled(
        region_rect,
        0.0,
        Color32::from_rgba_unmultiplied(60, 180, 100, 80),
    );
    painter.rect_stroke(
        region_rect,
        0.0,
        Stroke::new(1.0, Color32::from_rgb(60, 180, 100)),
        egui::StrokeKind::Middle,
    );

    // INTERACTION LOGIC
    let mut handled = false;

    // 1. Handles (Prioritize resizing)
    let handle_width = 8.0;
    let start_handle_rect = Rect::from_center_size(
        Pos2::new(start_x, rect.center().y),
        Vec2::new(handle_width, rect.height()),
    );
    let end_handle_rect = Rect::from_center_size(
        Pos2::new(end_x, rect.center().y),
        Vec2::new(handle_width, rect.height()),
    );

    let start_resp = ui.interact(start_handle_rect, response.id.with("start"), Sense::drag());
    let end_resp = ui.interact(end_handle_rect, response.id.with("end"), Sense::drag());

    if start_resp.hovered() || end_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }

    if start_resp.dragged() {
        let delta_s = (start_resp.drag_delta().x / rect.width()) * video_duration;
        *start_time = (*start_time + delta_s).clamp(0.0, effective_end - 0.1);
        handled = true;
    } else if end_resp.dragged() {
        let delta_s = (end_resp.drag_delta().x / rect.width()) * video_duration;
        let mut new_end = (effective_end + delta_s).clamp(*start_time + 0.1, video_duration);
        // Snap to end (0.0) if close
        if (video_duration - new_end).abs() < 0.1 {
            new_end = 0.0;
        }
        *end_time = new_end;
        handled = true;
    }

    // 2. Body Interaction (Slide or Seek)
    if !handled && response.hovered() {
        if ui.input(|i| i.modifiers.shift)
            && region_rect.contains(response.hover_pos().unwrap_or_default())
        {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    if !handled && response.dragged() {
        if ui.input(|i| i.modifiers.shift) {
            // Slide Region
            let delta_s = (response.drag_delta().x / rect.width()) * video_duration;
            let duration_s = effective_end - *start_time;

            let new_start = (*start_time + delta_s).clamp(0.0, video_duration - duration_s);
            let new_end = new_start + duration_s;

            *start_time = new_start;
            *end_time = if (video_duration - new_end).abs() < 0.1 {
                0.0
            } else {
                new_end
            };
        } else {
            // Seek
            if let Some(pos) = response.interact_pointer_pos() {
                let seek_norm = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                let seek_s = seek_norm * video_duration;
                canvas
                    .pending_playback_commands
                    .push((part_id, MediaPlaybackCommand::Seek(seek_s as f64)));
            }
        }
    }

    // Draw Handles
    painter.rect_filled(start_handle_rect.shrink(2.0), 2.0, Color32::WHITE);
    painter.rect_filled(end_handle_rect.shrink(2.0), 2.0, Color32::WHITE);

    // Draw Playhead
    let cursor_norm = (current_pos / video_duration).clamp(0.0, 1.0);
    let cursor_x = rect.min.x + cursor_norm * rect.width();
    painter.line_segment(
        [
            Pos2::new(cursor_x, rect.min.y),
            Pos2::new(cursor_x, rect.max.y),
        ],
        Stroke::new(2.0, Color32::from_rgb(255, 200, 50)),
    );
    // Playhead triangle top
    let tri_size = 6.0;
    painter.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(cursor_x - tri_size, rect.min.y),
            Pos2::new(cursor_x + tri_size, rect.min.y),
            Pos2::new(cursor_x, rect.min.y + tri_size * 1.5),
        ],
        Color32::from_rgb(255, 200, 50),
        Stroke::NONE,
    ));
}

pub fn render_inspector_preview_toggle(canvas: &mut ModuleCanvas, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.heading("Inspector Preview");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(&mut canvas.show_inspector_previews, "Enabled");
        });
    });
}

pub fn render_fixed_timer_preview(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    part_id: ModulePartId,
    interval_ms: u32,
    offset_ms: u32,
) {
    if !canvas.show_inspector_previews {
        return;
    }

    let now_ms = (ui.input(|input| input.time) * 1000.0) as u32;
    let cycle_ms = interval_ms.max(1);
    let phase_ms = now_ms.wrapping_add(offset_ms) % cycle_ms;
    let progress = phase_ms as f32 / cycle_ms as f32;
    let live_value = canvas
        .last_trigger_values
        .get(&part_id)
        .copied()
        .unwrap_or(0.0);
    let is_live = live_value > 0.1;
    let next_pulse_ms = cycle_ms.saturating_sub(phase_ms) % cycle_ms;

    ui.ctx().request_repaint();
    ui.separator();
    render_inspector_preview_toggle(canvas, ui);
    ui.group(|ui| {
        ui.label("Fixed timer cadence");
        ui.add(
            ProgressBar::new(progress)
                .desired_width(ui.available_width())
                .text(format!("cycle {} ms", cycle_ms)),
        );
        ui.horizontal(|ui| {
            let status = if is_live { "LIVE pulse" } else { "Waiting" };
            let color = if is_live {
                Color32::from_rgb(110, 235, 150)
            } else {
                Color32::from_rgb(180, 180, 180)
            };
            ui.colored_label(color, status);
            ui.label(format!("Next pulse in {} ms", next_pulse_ms));
        });
        ui.label(format!("Offset {} ms", offset_ms));
        ui.label(format!("Current trigger value {:.2}", live_value));
    });
}

pub fn render_preview_texture(ui: &mut Ui, texture_id: egui::TextureId, caption: &str) {
    let width = ui.available_width().max(160.0);
    let size = Vec2::new(width, width * 9.0 / 16.0);
    ui.image((texture_id, size));
    ui.small(caption);
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
        ui.label(
            egui::RichText::new("No preview available yet.")
                .weak()
                .italics(),
        );
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
            ui.label(
                egui::RichText::new("No upstream source node was found for this layer.")
                    .weak()
                    .italics(),
            );
        } else {
            ui.label(
                egui::RichText::new(
                    "Upstream source exists, but no preview texture reached the inspector.",
                )
                .weak()
                .italics(),
            );
        }
    });
}
