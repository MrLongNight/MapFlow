use super::super::state::ModuleCanvas;
use super::capabilities;
use crate::widgets::{styled_drag_value, styled_slider};
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use vorce_core::module::{BlendModeType, ModulePartId};

/// Standardized informational label, used as an explicit fallback when no active preview is available.
pub fn render_info_label(ui: &mut Ui, text: &str) {
    crate::widgets::custom::render_info_label(ui, text);
}

/// Standardized missing preview banner.
pub fn render_missing_preview_banner(ui: &mut Ui, text: &str) {
    ui.group(|ui| {
        crate::widgets::custom::render_info_label(ui, text);
    });
}

pub fn render_transport_controls(
    _canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    _part_id: ModulePartId,
    is_playing: bool,
    _current_pos: f32,
    _loop_enabled: bool,
    _reverse_playback: bool,
) {
    ui.horizontal(|ui| {
        let _play_btn = if is_playing {
            ui.button("⏸ Pause")
        } else {
            ui.button("▶ Play")
        };

        if ui.button("⏮").clicked() {
            // Seek to start
        }
    });
}

pub fn render_timeline(
    _canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    _part_id: ModulePartId,
    duration: f32,
    current_pos: f32,
    _start_time: &mut f32,
    _end_time: &mut f32,
) {
    ui.horizontal(|ui| {
        ui.add(
            egui::ProgressBar::new(current_pos / duration)
                .desired_width(ui.available_width() - 60.0),
        );
    });
}

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
    ui.collapsing("🌓 Opacity & Blend", |ui| {
        ui.horizontal(|ui| {
            ui.label("Opacity:");
            styled_slider(ui, opacity, 0.0..=1.0, 1.0);
        });

        ui.horizontal(|ui| {
            ui.label("Blend Mode:");
            let supported =
                capabilities::is_blend_mode_supported(&blend_mode.unwrap_or(BlendModeType::Normal));
            if !supported {
                capabilities::render_unsupported_warning(ui, "Blend modes partially supported");
            }
            egui::ComboBox::from_id_salt("blend_mode_select")
                .selected_text(match blend_mode {
                    Some(m) => m.name(),
                    None => "None",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(blend_mode, None, "None");
                    for mode in BlendModeType::all() {
                        ui.selectable_value(blend_mode, Some(*mode), mode.name());
                    }
                });
        });
    });

    ui.collapsing("🎨 Color Adjust", |ui| {
        egui::Grid::new("color_grid")
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
                styled_slider(ui, hue_shift, 0.0..=1.0, 0.0);
                ui.end_row();
            });
    });

    ui.collapsing("📐 Transform", |ui| {
        let supported = capabilities::is_transform_supported();
        if !supported {
            capabilities::render_unsupported_warning(
                ui,
                "Transform properties currently not supported in render pipeline.",
            );
        }
        ui.add_enabled_ui(supported, |ui| {
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

                    ui.label("Rotation:");
                    styled_slider(ui, rotation, 0.0..=360.0, 0.0);
                    ui.end_row();

                    ui.label("Offset:");
                    ui.horizontal(|ui| {
                        styled_drag_value(ui, offset_x, 0.01, -1.0..=1.0, 0.0, "X: ", "");
                        styled_drag_value(ui, offset_y, 0.01, -1.0..=1.0, 0.0, "Y: ", "");
                    });
                    ui.end_row();

                    ui.label("Flip:");
                    ui.horizontal(|ui| {
                        ui.checkbox(flip_horizontal, "Horizontal");
                        ui.checkbox(flip_vertical, "Vertical");
                    });
                    ui.end_row();
                });
        });
    });
}

/// Renders a 2D spatial editor for Philips Hue entertainment areas.
/// Allows positioning lamps relative to the screen area.
pub fn render_hue_spatial_editor(
    ui: &mut Ui,
    lamp_positions: &mut std::collections::HashMap<String, [f32; 2]>,
) {
    let size = ui.available_width().min(300.0);
    let (rect, response) = ui.allocate_at_least(Vec2::splat(size), Sense::drag());

    // Draw background grid
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, Color32::from_black_alpha(100));

    // Draw reference screen
    let screen_rect = Rect::from_center_size(rect.center(), Vec2::new(size * 0.6, size * 0.4));
    painter.rect_stroke(
        screen_rect,
        2.0,
        Stroke::new(1.0, Color32::WHITE),
        egui::StrokeKind::Inside,
    );
    painter.text(
        screen_rect.center(),
        egui::Align2::CENTER_CENTER,
        "SCREEN",
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );

    // Draw and handle lamps
    for (id, pos) in lamp_positions.iter_mut() {
        // Convert normalized (-1..1) to screen space
        let mut screen_pos = Pos2::new(
            rect.center().x + pos[0] * (size * 0.4),
            rect.center().y + pos[1] * (size * 0.4),
        );

        let lamp_resp = ui.interact(
            Rect::from_center_size(screen_pos, Vec2::splat(20.0)),
            ui.id().with(id),
            Sense::drag(),
        );

        if lamp_resp.dragged() {
            screen_pos += lamp_resp.drag_delta();
            // Convert back to normalized
            pos[0] = (screen_pos.x - rect.center().x) / (size * 0.4);
            pos[1] = (screen_pos.y - rect.center().y) / (size * 0.4);

            // Clamp to bounds
            pos[0] = pos[0].clamp(-1.0, 1.0);
            pos[1] = pos[1].clamp(-1.0, 1.0);
        }

        // Visual representation of the lamp
        painter.circle_filled(
            screen_pos,
            8.0,
            if lamp_resp.hovered() {
                Color32::LIGHT_BLUE
            } else {
                Color32::from_rgb(255, 200, 50)
            },
        );
        painter.text(
            screen_pos + Vec2::new(0.0, 12.0),
            egui::Align2::CENTER_TOP,
            id,
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }
}
