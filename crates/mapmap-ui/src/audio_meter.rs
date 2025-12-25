//! Audio Meter Widget
//!
//! Provides two styles of audio level visualization:
//! - Retro: Analog VU meter with needle and arc scale (Stereo)
//! - Digital: Segmented LED bar (Stereo)

use crate::config::AudioMeterStyle;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2, Widget};

/// A widget that displays stereo audio levels.
pub struct AudioMeter {
    style: AudioMeterStyle,
    left_db: f32,
    right_db: f32,
    size: Vec2,
}

impl AudioMeter {
    /// Create a new stereo audio meter
    pub fn new(style: AudioMeterStyle, left_db: f32, right_db: f32) -> Self {
        let default_size = match style {
            AudioMeterStyle::Retro => Vec2::new(300.0, 80.0),    // Wider for side-by-side
            AudioMeterStyle::Digital => Vec2::new(360.0, 60.0),  // Wide horizontal, slightly taller
        };
        Self {
            style,
            left_db,
            right_db,
            size: default_size,
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
                AudioMeterStyle::Retro => {
                    draw_retro_stereo(ui, rect, self.left_db, self.right_db)
                }
                AudioMeterStyle::Digital => {
                    draw_digital_stereo(ui, rect, self.left_db, self.right_db)
                }
            }
        }

        response
    }
}

/// Helper to draw a Phillips-head screw
fn draw_screw(painter: &egui::Painter, center: Pos2, radius: f32) {
    // Screw head (silver/grey gradient simulated with solid circles)
    painter.circle_filled(center, radius, Color32::from_gray(180));
    painter.circle_stroke(
        center,
        radius,
        Stroke::new(1.0, Color32::from_gray(100)),
    );

    // Cross slot
    let slot_len = radius * 0.6;
    let slot_width = radius * 0.25;
    let color = Color32::from_gray(80);

    // Rotate the cross slightly for realism
    let angle = 45.0f32.to_radians();
    let rot = |p: Pos2| -> Pos2 {
        let v = p - center;
        let rx = v.x * angle.cos() - v.y * angle.sin();
        let ry = v.x * angle.sin() + v.y * angle.cos();
        center + Vec2::new(rx, ry)
    };

    // We can't easily rotate rects with painter.rect, so we draw lines or use a transform if available.
    // Simpler approach: Draw lines with thickness
    let p1 = center + Vec2::new(-slot_len, 0.0);
    let p2 = center + Vec2::new(slot_len, 0.0);
    let p3 = center + Vec2::new(0.0, -slot_len);
    let p4 = center + Vec2::new(0.0, slot_len);

    painter.line_segment([rot(p1), rot(p2)], Stroke::new(slot_width, color));
    painter.line_segment([rot(p3), rot(p4)], Stroke::new(slot_width, color));
}

/// Helper to draw the rack mount frame with screws
fn draw_rack_frame(painter: &egui::Painter, rect: Rect) {
    // Dark metal background
    painter.rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 35));
    // Bezel stroke
    painter.rect_stroke(rect, 4.0, Stroke::new(2.0, Color32::from_rgb(20, 20, 25)));

    // Screws in corners
    let screw_inset = 8.0;
    let screw_radius = 4.0;
    let corners = [
        rect.min + Vec2::new(screw_inset, screw_inset),
        rect.min + Vec2::new(rect.width() - screw_inset, screw_inset),
        rect.max - Vec2::new(screw_inset, screw_inset),
        rect.max - Vec2::new(rect.width() - screw_inset, screw_inset),
    ];

    for center in corners {
        draw_screw(painter, center, screw_radius);
    }
}

/// Draws a stereo retro VU meter layout
fn draw_retro_stereo(ui: &mut egui::Ui, rect: Rect, left_db: f32, right_db: f32) {
    let painter = ui.painter();

    // Draw Rack Frame
    draw_rack_frame(painter, rect);

    // Inner area for meters (inset from frame)
    let inner_rect = rect.shrink(12.0);

    // Split for Left/Right
    let split_width = inner_rect.width() / 2.0;
    let left_rect = Rect::from_min_size(inner_rect.min, Vec2::new(split_width - 4.0, inner_rect.height()));
    let right_rect = Rect::from_min_size(
        inner_rect.min + Vec2::new(split_width + 4.0, 0.0),
        Vec2::new(split_width - 4.0, inner_rect.height()),
    );

    draw_single_retro_meter(ui, left_rect, left_db);
    draw_single_retro_meter(ui, right_rect, right_db);
}

/// Draws a single retro VU meter (reused for L and R)
fn draw_single_retro_meter(ui: &mut egui::Ui, rect: Rect, db: f32) {
    let painter = ui.painter();

    // 1. Background (Off-white/Cream "Waves" style)
    painter.rect_filled(rect, 2.0, Color32::from_rgb(245, 242, 235));
    painter.rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::from_rgb(80, 80, 80)));

    // 2. Geometry Calculation
    // Pivot is below the bottom center to create a wide arc
    // Ideal proportion: Pivot depth approx 80% of width
    let pivot_depth = rect.width() * 0.7;
    let pivot = rect.center_bottom() + Vec2::new(0.0, pivot_depth * 0.15); // Slightly below bottom
    let radius_outer = pivot_depth;

    // Arc Angles (Classic VU: -45 to +45 deg usually, or narrower)
    let angle_min = -35.0f32;
    let angle_max = 35.0f32;
    let angle_zero = 15.0f32; // 0dB position (asymmetric usually, but linear for now)

    // Helper: Polar to Cartesian
    let to_pos = |deg: f32, r: f32| -> Pos2 {
        let rad = (deg - 90.0).to_radians();
        pivot + Vec2::new(rad.cos() * r, rad.sin() * r)
    };

    // 3. Draw Scale and Ticks
    // Range: -20 to +3 dB
    let ticks = [
        (-20.0, angle_min, "-20"),
        (-10.0, angle_min + (angle_zero - angle_min) * 0.5, "-10"),
        (-5.0, angle_min + (angle_zero - angle_min) * 0.75, "-5"),
        (0.0, angle_zero, "0"),
        (3.0, angle_max, "+3"),
    ];

    // Main Arc Line
    // We approximate arc with segments
    let segments = 30;
    let points: Vec<Pos2> = (0..=segments).map(|i| {
        let t = i as f32 / segments as f32;
        let deg = angle_min + t * (angle_max - angle_min);
        to_pos(deg, radius_outer * 0.85) // Scale line radius
    }).collect();
    painter.add(egui::Shape::line(points, Stroke::new(1.5, Color32::BLACK)));

    // Red Zone (0 to +3)
    let red_pts: Vec<Pos2> = (0..=10).map(|i| {
        let t = i as f32 / 10.0;
        let deg = angle_zero + t * (angle_max - angle_zero);
        to_pos(deg, radius_outer * 0.85)
    }).collect();
    if red_pts.len() >= 2 {
        painter.add(egui::Shape::line(
            red_pts,
            Stroke::new(3.0, Color32::from_rgb(200, 50, 50)),
        ));
    }

    // Ticks & Labels
    for (_val, deg, label) in ticks.iter() {
        let p1 = to_pos(*deg, radius_outer * 0.85);
        let p2 = to_pos(*deg, radius_outer * 0.75);
        painter.line_segment([p1, p2], Stroke::new(1.5, Color32::BLACK));

        // Text
        let text_pos = to_pos(*deg, radius_outer * 0.65);
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.0),
            Color32::BLACK,
        );
    }

    // 4. Needle
    // Map db to angle
    let clamped_db = db.clamp(-25.0, 5.0);
    // Linear interpolation for simplicity for now
    // -20 -> min, 0 -> zero, +3 -> max
    let needle_deg = if clamped_db < 0.0 {
        // Range -20..0
        let t = (clamped_db - -20.0) / 20.0;
        angle_min + t * (angle_zero - angle_min)
    } else {
        // Range 0..3
        let t = clamped_db / 3.0;
        angle_zero + t * (angle_max - angle_zero)
    };
    // Ensure we don't go below min (resting)
    let final_deg = needle_deg.max(angle_min - 2.0);

    let needle_tip = to_pos(final_deg, radius_outer * 0.9);
    // Start needle slightly above pivot for realism
    let needle_start = to_pos(final_deg, radius_outer * 0.1);

    // Needle Shadow (offset)
    painter.line_segment(
        [needle_start + Vec2::new(2.0, 2.0), needle_tip + Vec2::new(2.0, 2.0)],
        Stroke::new(2.0, Color32::from_black_alpha(40)),
    );

    // Needle Body
    painter.line_segment(
        [needle_start, needle_tip],
        Stroke::new(2.0, Color32::from_rgb(200, 20, 20)), // Red needle
    );

    // 5. Glass Reflection (Top gradient)
    // Draw a white semi-transparent rect over the top half
    let glass_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), rect.height() * 0.5));
    // We use a mesh for gradient alpha if possible, or just a flat transparent white
    let mut mesh = egui::Mesh::default();
    mesh.add_colored_rect(
        glass_rect,
        Color32::from_white_alpha(30) // Subtle shine
    );
    painter.add(mesh);

    // Add "VU" text
    painter.text(
        rect.center() + Vec2::new(0.0, rect.height() * 0.15),
        egui::Align2::CENTER_CENTER,
        "VU",
        egui::FontId::monospace(14.0),
        Color32::from_gray(100),
    );
}

/// Draws a stereo digital LED meter layout
fn draw_digital_stereo(ui: &mut egui::Ui, rect: Rect, left_db: f32, right_db: f32) {
    let painter = ui.painter();

    // Draw Rack Frame
    draw_rack_frame(painter, rect);

    // Inner area
    let inner_rect = rect.shrink(12.0);

    // Layout: Two bars stacked vertically with scale in between or on bottom
    // We'll put Left on top, Right on bottom, scale in the middle
    let bar_height = (inner_rect.height() - 15.0) / 2.0;

    let left_bar_rect = Rect::from_min_size(inner_rect.min, Vec2::new(inner_rect.width(), bar_height));
    let scale_rect = Rect::from_min_size(
        inner_rect.min + Vec2::new(0.0, bar_height),
        Vec2::new(inner_rect.width(), 15.0)
    );
    let right_bar_rect = Rect::from_min_size(
        inner_rect.min + Vec2::new(0.0, bar_height + 15.0),
        Vec2::new(inner_rect.width(), bar_height)
    );

    draw_led_bar(ui, left_bar_rect, left_db);
    draw_db_scale(ui, scale_rect);
    draw_led_bar(ui, right_bar_rect, right_db);
}

fn draw_led_bar(ui: &mut egui::Ui, rect: Rect, db: f32) {
    let painter = ui.painter();

    // Black background for the bar track
    painter.rect_filled(rect, 1.0, Color32::BLACK);

    let segment_count = 30;
    let padding = 1.0;
    // Calculate individual segment width
    let total_spacing = (segment_count - 1) as f32 * padding;
    let segment_width = (rect.width() - total_spacing) / segment_count as f32;

    // Range: -60dB to +3dB
    let min_db = -60.0;
    let max_db = 3.0;

    for i in 0..segment_count {
        let t = i as f32 / (segment_count as f32 - 1.0);
        let threshold_db = min_db + t * (max_db - min_db);

        // Determine color based on threshold (Gradient)
        // -60..-10: Green
        // -10..0: Yellow
        // 0..+3: Red
        let base_color = if threshold_db >= 0.0 {
            Color32::from_rgb(255, 30, 30) // Red
        } else if threshold_db >= -10.0 {
            Color32::from_rgb(255, 200, 0) // Yellow
        } else {
            Color32::from_rgb(0, 255, 0) // Green
        };

        let active = db >= threshold_db;

        let color = if active {
            base_color
        } else {
            // "Off" LED state (dimmed version of base color)
            Color32::from_rgba_premultiplied(
                base_color.r() / 8,
                base_color.g() / 8,
                base_color.b() / 8,
                255
            )
        };

        let x = rect.min.x + (i as f32 * (segment_width + padding));
        let seg_rect = Rect::from_min_size(
            Pos2::new(x, rect.min.y + 2.0),
            Vec2::new(segment_width, rect.height() - 4.0)
        );

        painter.rect_filled(seg_rect, 0.0, color);
    }
}

fn draw_db_scale(ui: &mut egui::Ui, rect: Rect) {
    let painter = ui.painter();
    let font_id = egui::FontId::proportional(10.0);
    let color = Color32::from_gray(150);

    let labels = [
        (-60.0, "-60"),
        (-40.0, "-40"),
        (-20.0, "-20"),
        (-10.0, "-10"),
        (-5.0, "-5"),
        (0.0, "0"),
        (3.0, "+3")
    ];

    let min_db = -60.0;
    let max_db = 3.0;

    for (val, text) in labels {
        // Map dB to position t (0.0 - 1.0)
        let t = (val - min_db) / (max_db - min_db);
        let x = rect.min.x + t * rect.width();

        // Draw tick
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.min.y + 3.0)],
            Stroke::new(1.0, color)
        );

        // Draw text centered on tick
        painter.text(
            Pos2::new(x, rect.min.y + 8.0),
            egui::Align2::CENTER_CENTER,
            text,
            font_id.clone(),
            color
        );
    }
}
