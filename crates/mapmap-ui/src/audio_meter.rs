//! Audio Meter Widget
//!
//! Provides two styles of audio level visualization:
//! - Retro: Analog VU meter with needle and arc scale
//! - Digital: Segmented LED bar

use crate::config::AudioMeterStyle;
use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, Vec2, Widget};

/// A widget that displays audio levels.
pub struct AudioMeter {
    style: AudioMeterStyle,
    left_db: f32,
    right_db: f32,
    size: Vec2,
}

impl AudioMeter {
    /// Create a new audio meter
    pub fn new(style: AudioMeterStyle, left_db: f32, right_db: f32) -> Self {
        Self {
            style,
            left_db,
            right_db,
            size: Vec2::new(300.0, 40.0), // Default size, can be overridden by layout
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
                    draw_retro_stereo(ui, rect, self.left_db, self.right_db);
                }
                AudioMeterStyle::Digital => {
                    draw_digital_stereo(ui, rect, self.left_db, self.right_db);
                }
            }
        }

        response
    }
}

/// Helper to draw a Phillips screw head
fn draw_screw(painter: &egui::Painter, center: Pos2, size: f32) {
    let color = Color32::from_gray(50);
    let highlight = Color32::from_gray(120);

    // Screw head
    painter.circle_filled(center, size, color);
    // Bevel/Highlight
    painter.circle_stroke(center, size, Stroke::new(1.0, highlight));

    // Phillips cross
    let cross_len = size * 0.6;
    let stroke = Stroke::new(1.5, Color32::from_gray(20));
    painter.line_segment(
        [
            center + Vec2::new(-cross_len, -cross_len),
            center + Vec2::new(cross_len, cross_len),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + Vec2::new(-cross_len, cross_len),
            center + Vec2::new(cross_len, -cross_len),
        ],
        stroke,
    );
}

/// Helper to draw the rack mount frame
fn draw_rack_frame(painter: &egui::Painter, rect: Rect) {
    // Dark metallic background
    painter.rect_filled(rect, 4.0, Color32::from_rgb(20, 20, 25));
    // Subtle bevel
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_white_alpha(30)));

    // Screws in corners
    let screw_size = 3.0;
    let padding = 6.0;

    draw_screw(painter, rect.min + Vec2::new(padding, padding), screw_size);
    draw_screw(
        painter,
        rect.min + Vec2::new(rect.width() - padding, padding),
        screw_size,
    );
    draw_screw(
        painter,
        rect.min + Vec2::new(padding, rect.height() - padding),
        screw_size,
    );
    draw_screw(painter, rect.max - Vec2::new(padding, padding), screw_size);
}

/// Draws a single retro VU meter (analog style)
fn draw_single_retro_meter(painter: &egui::Painter, rect: Rect, db: f32, _label: &str) {
    // Background - Cream/Vintage White
    painter.rect_filled(rect, 2.0, Color32::from_rgb(230, 230, 220));
    // Inner shadow/bezel
    painter.rect_stroke(rect, 2.0, Stroke::new(2.0, Color32::from_black_alpha(40)));

    // Scale Logic
    // Pivot below the widget
    let offset_down = rect.height() * 0.5;
    let center = rect.center_bottom() + Vec2::new(0.0, offset_down);
    let radius = (center.y - rect.min.y) - 2.0;

    let angle_to_pos = |angle_deg: f32, r: f32| -> Pos2 {
        let rad = (angle_deg - 90.0).to_radians();
        center + Vec2::new(rad.cos() * r, rad.sin() * r)
    };

    let start_angle = -45.0;
    let end_angle = 45.0;
    let zero_angle = 15.0;

    // Red Zone Arc
    let red_zone_points: Vec<Pos2> = (0..=10)
        .map(|i| {
            let t = i as f32 / 10.0;
            let angle = zero_angle + t * (end_angle - zero_angle);
            angle_to_pos(angle, radius * 0.85)
        })
        .collect();

    if red_zone_points.len() >= 2 {
        painter.add(Shape::line(
            red_zone_points,
            Stroke::new(4.0, Color32::from_rgb(200, 50, 50)),
        ));
    }

    // Ticks
    let ticks = [
        (-20.0, start_angle, "-20"),
        (-10.0, -25.0, "-10"),
        (-5.0, -5.0, "-5"),
        (0.0, zero_angle, "0"),
        (3.0, end_angle, "+3"),
    ];

    for (_val, angle, _label_text) in ticks.iter() {
        let p1 = angle_to_pos(*angle, radius * 0.75);
        let p2 = angle_to_pos(*angle, radius * 0.85);
        painter.line_segment([p1, p2], Stroke::new(1.0, Color32::BLACK));
    }

    // Needle
    let clamped_db = db.clamp(-40.0, 6.0);
    let needle_angle = if clamped_db < -20.0 {
        start_angle - 5.0
    } else if clamped_db < 0.0 {
        start_angle + (clamped_db - -20.0) / 20.0 * (zero_angle - start_angle)
    } else {
        zero_angle + (clamped_db - 0.0) / 3.0 * (end_angle - zero_angle)
    };

    let tip = angle_to_pos(needle_angle, radius * 0.9);

    // Needle Shadow
    let shadow_offset = Vec2::new(2.0, 2.0);
    painter.line_segment(
        [center + shadow_offset, tip + shadow_offset],
        Stroke::new(1.5, Color32::from_black_alpha(50)),
    );

    // Needle
    painter.line_segment(
        [center, tip],
        Stroke::new(1.5, Color32::from_rgb(200, 20, 20)),
    );

    // Glass Reflection
    let glass_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), rect.height() * 0.5));
    painter.rect_filled(
        glass_rect,
        2.0,
        Color32::from_white_alpha(40), // Stronger shine
    );
}

/// Draws Stereo Retro Meters
fn draw_retro_stereo(ui: &mut egui::Ui, rect: Rect, left: f32, right: f32) {
    let painter = ui.painter();
    draw_rack_frame(painter, rect);

    let content_rect = rect.shrink(4.0);
    let meter_width = (content_rect.width() - 4.0) / 2.0;

    let left_rect = Rect::from_min_size(
        content_rect.min,
        Vec2::new(meter_width, content_rect.height()),
    );
    let right_rect = Rect::from_min_size(
        content_rect.min + Vec2::new(meter_width + 4.0, 0.0),
        Vec2::new(meter_width, content_rect.height()),
    );

    draw_single_retro_meter(painter, left_rect, left, "L");
    draw_single_retro_meter(painter, right_rect, right, "R");
}

/// Draws a horizontal LED bar
fn draw_led_bar(painter: &egui::Painter, rect: Rect, db: f32) {
    // Background
    painter.rect_filled(rect, 1.0, Color32::from_rgb(10, 10, 10));

    let segment_count = 30;
    let padding = 1.0;
    let total_width = rect.width() - (padding * 2.0);
    let segment_width = (total_width - (segment_count as f32 - 1.0)) / segment_count as f32;
    let segment_height = rect.height() - (padding * 2.0);

    let min_db = -60.0;
    let max_db = 6.0;

    for i in 0..segment_count {
        let t = i as f32 / (segment_count as f32 - 1.0);
        let threshold_db = min_db + t * (max_db - min_db);
        let active = db >= threshold_db;

        let base_color = if threshold_db >= 0.0 {
            Color32::from_rgb(255, 50, 50) // Red
        } else if threshold_db >= -12.0 {
            Color32::from_rgb(255, 200, 0) // Yellow
        } else {
            Color32::from_rgb(50, 255, 50) // Green
        };

        let color = if active {
            base_color
        } else {
            Color32::from_rgba_premultiplied(
                base_color.r() / 8,
                base_color.g() / 8,
                base_color.b() / 8,
                255,
            )
        };

        let x = rect.min.x + padding + (i as f32 * (segment_width + 1.0));
        let y = rect.min.y + padding;

        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x, y), Vec2::new(segment_width, segment_height)),
            0.0,
            color,
        );
    }
}

/// Draws Stereo Digital Meters
fn draw_digital_stereo(ui: &mut egui::Ui, rect: Rect, left: f32, right: f32) {
    let painter = ui.painter();
    draw_rack_frame(painter, rect);

    let content_rect = rect.shrink(6.0);
    // Layout: Top Bar (L), Middle Scale, Bottom Bar (R)
    let bar_height = (content_rect.height() - 10.0) / 2.0;

    let left_rect = Rect::from_min_size(
        content_rect.min,
        Vec2::new(content_rect.width(), bar_height),
    );
    let scale_y = left_rect.max.y + 2.0;
    let right_rect = Rect::from_min_size(
        Pos2::new(content_rect.min.x, scale_y + 8.0),
        Vec2::new(content_rect.width(), bar_height),
    );

    draw_led_bar(painter, left_rect, left);

    // Draw central dB scale
    let scale_rect = Rect::from_min_size(
        Pos2::new(content_rect.min.x, scale_y),
        Vec2::new(content_rect.width(), 6.0),
    );

    // Simple ticks for the scale
    let ticks = [-60, -40, -20, -10, -5, 0, 3, 6];
    let min_db = -60.0;
    let max_db = 6.0;

    for &val in &ticks {
        let t = (val as f32 - min_db) / (max_db - min_db);
        if (0.0..=1.0).contains(&t) {
            let x = scale_rect.min.x + t * scale_rect.width();
            painter.line_segment(
                [
                    Pos2::new(x, scale_rect.min.y),
                    Pos2::new(x, scale_rect.max.y),
                ],
                Stroke::new(1.0, Color32::from_gray(100)),
            );
        }
    }

    draw_led_bar(painter, right_rect, right);
}
