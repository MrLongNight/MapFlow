//! Audio Meter Widget
//!
//! Provides two styles of audio level visualization:
//! - Retro: Analog VU meter with needle and arc scale
//! - Digital: Segmented LED bar

use crate::config::AudioMeterStyle;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2, Widget};

/// A widget that displays audio levels.
pub struct AudioMeter {
    style: AudioMeterStyle,
    level_db: f32,
    size: Vec2,
}

impl AudioMeter {
    /// Create a new audio meter
    pub fn new(style: AudioMeterStyle, level_db: f32) -> Self {
        Self {
            style,
            level_db,
            size: Vec2::new(180.0, 40.0), // Default size, can be overridden by layout
        }
    }

    /// Set preferred size
    pub fn desired_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
}

impl Widget for AudioMeter {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());

        if ui.is_rect_visible(rect) {
            match self.style {
                AudioMeterStyle::Retro => draw_retro_meter(ui, rect, self.level_db),
                AudioMeterStyle::Digital => draw_digital_meter(ui, rect, self.level_db),
            }
        }

        response
    }
}

/// Draws an analog retro VU meter
fn draw_retro_meter(ui: &mut egui::Ui, rect: Rect, db: f32) {
    let painter = ui.painter();

    // Background - Dark Retro
    painter.rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 30));
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(80, 80, 80)));

    // Calculate geometry to fit within the box while maximizing arc radius
    // We place the pivot below the widget to create a flatter, wider arc (classic VU look)
    // Constraint: center.y - radius >= rect.min.y (top of widget)

    let offset_down = rect.height() * 0.8;
    let center = rect.center_bottom() + Vec2::new(0.0, offset_down);
    // Radius fits from pivot to 4px below top edge
    let radius = (center.y - rect.min.y) - 4.0;

    // Draw Scale Arcs
    // Visible angle range: -35 deg to +35 deg (narrower because arc is flatter/larger)
    // Left: -20dB, Center: 0dB, Right: +3dB

    // Helper to map angle to position
    let angle_to_pos = |angle_deg: f32, r: f32| -> Pos2 {
        let rad = (angle_deg - 90.0).to_radians();
        center + Vec2::new(rad.cos() * r, rad.sin() * r)
    };

    let start_angle = -35.0;
    let end_angle = 35.0;
    let zero_angle = 20.0; // 0dB position

    // Draw red zone background (0dB to +3dB)
    // Simplified as a thick arc or polygon since egui doesn't have native arc filling yet easily
    // We approximate with line segments
    let red_zone_points: Vec<Pos2> = (0..=10)
        .map(|i| {
            let t = i as f32 / 10.0;
            let angle = zero_angle + t * (end_angle - zero_angle);
            angle_to_pos(angle, radius * 0.7)
        })
        .collect();

    if red_zone_points.len() >= 2 {
        painter.add(egui::Shape::line(
            red_zone_points,
            Stroke::new(6.0, Color32::from_rgba_premultiplied(255, 100, 100, 100)),
        ));
    }

    // Ticks
    let ticks = [
        (-20.0, start_angle, "-20"),
        (-10.0, -20.0, "-10"),
        (-5.0, 0.0, "-5"),
        (0.0, zero_angle, "0"),
        (3.0, end_angle, "+3"),
    ];

    for (_val, angle, label) in ticks.iter() {
        let p1 = angle_to_pos(*angle, radius * 0.6);
        let p2 = angle_to_pos(*angle, radius * 0.7);
        painter.line_segment([p1, p2], Stroke::new(1.5, Color32::from_gray(200)));

        // Draw labels slightly below ticks
        if rect.height() > 30.0 {
            let text_pos = angle_to_pos(*angle, radius * 0.5);
            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(10.0),
                Color32::from_gray(200),
            );
        }
    }

    // Needle Logic
    // Clamp db between -40 and +6
    // Map -20..+3 to angles
    // Simple interpolation
    let clamped_db = db.clamp(-40.0, 6.0);

    // Mapping:
    // -20dB -> start_angle
    // 0dB -> zero_angle
    // +3dB -> end_angle
    // We need a non-linear mapping ideally, but linear sections work for visual
    let needle_angle = if clamped_db < -20.0 {
        start_angle - 5.0 // Resting position
    } else if clamped_db < 0.0 {
        // Linear map -20 to 0 -> start to zero
        start_angle + (clamped_db - -20.0) / 20.0 * (zero_angle - start_angle)
    } else {
        // Linear map 0 to 3 -> zero to end
        zero_angle + (clamped_db - 0.0) / 3.0 * (end_angle - zero_angle)
    };

    let needle_len = radius * 0.9;
    let tip = angle_to_pos(needle_angle, needle_len);

    // Clip needle drawing to the rect (prevent pivot from showing if it's outside)
    // We simulate clipping by ensuring we don't draw outside.
    // Since we calculated radius to fit, the tip is safe.
    // The start of the needle (center) might be outside.
    // We intersect the needle line with the bottom edge if needed,
    // but drawing a line from outside is handled fine by the GPU usually (clipped by window),
    // but egui might not clip to widget rect automatically.
    // To be safe and clean, let's start the needle from the bottom edge of the rect.

    let needle_dir = Vec2::new(
        (needle_angle - 90.0).to_radians().cos(),
        (needle_angle - 90.0).to_radians().sin(),
    );

    // Find intersection with bottom edge y = rect.max.y
    // center.y + t * dir.y = rect.max.y  => t = (rect.max.y - center.y) / dir.y
    // Since center is below rect.max.y, and dir.y is negative (pointing up), t should be positive.
    let t_bottom = (rect.max.y - center.y) / needle_dir.y;
    let start_pos = center + needle_dir * t_bottom.max(0.0);

    // Draw Needle Shadow
    painter.line_segment(
        [start_pos + Vec2::new(2.0, 2.0), tip + Vec2::new(2.0, 2.0)],
        Stroke::new(2.0, Color32::from_black_alpha(50)),
    );

    // Draw Needle
    painter.line_segment(
        [start_pos, tip],
        Stroke::new(1.5, Color32::from_rgb(200, 20, 20)),
    );

    // Pivot cap (only if visible, which it isn't with this geometry, so we skip it)

    // Glass reflection effect (top half lighter)
    painter.rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(rect.width(), rect.height() * 0.4)),
        4.0,
        Color32::from_white_alpha(30),
    );
}

/// Draws a horizontal segmented digital LED meter
fn draw_digital_meter(ui: &mut egui::Ui, rect: Rect, db: f32) {
    let painter = ui.painter();

    // Background
    painter.rect_filled(rect, 2.0, Color32::BLACK);

    let segment_count = 20;
    let padding = 2.0;
    let total_width = rect.width() - (padding * 2.0);
    let segment_width = (total_width - (segment_count as f32 - 1.0)) / segment_count as f32;
    let segment_height = rect.height() - (padding * 2.0);

    // dB range from -60 to +3
    // Map index to dB
    let min_db = -60.0;
    let max_db = 3.0;

    for i in 0..segment_count {
        let t = i as f32 / (segment_count as f32 - 1.0);
        let threshold_db = min_db + t * (max_db - min_db);

        let active = db >= threshold_db;

        // Color logic
        let base_color = if threshold_db >= 0.0 {
            Color32::from_rgb(255, 0, 0) // Red
        } else if threshold_db >= -12.0 {
            Color32::from_rgb(255, 200, 0) // Yellow
        } else {
            Color32::from_rgb(0, 255, 0) // Green
        };

        let color = if active {
            base_color
        } else {
            // Dimmed/Off state
            Color32::from_rgba_premultiplied(
                base_color.r() / 5,
                base_color.g() / 5,
                base_color.b() / 5,
                255,
            )
        };

        let x = rect.min.x + padding + (i as f32 * (segment_width + 1.0));
        let y = rect.min.y + padding;

        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x, y), Vec2::new(segment_width, segment_height)),
            1.0,
            color,
        );
    }
}
