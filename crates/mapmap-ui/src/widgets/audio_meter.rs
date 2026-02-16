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
    level_db_left: f32,
    level_db_right: f32,
    width: f32,
}

impl AudioMeter {
    /// Create a new audio meter
    pub fn new(style: AudioMeterStyle, level_db_left: f32, level_db_right: f32) -> Self {
        let width = match style {
            AudioMeterStyle::Retro => 300.0,
            AudioMeterStyle::Digital => 360.0,
        };
        Self {
            style,
            level_db_left,
            level_db_right,
            width,
        }
    }

    /// Set preferred width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Widget for AudioMeter {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Expand vertically to fill available space, but clamp to reasonable limits
        // to prevent layout explosions or zero-height issues.
        // We use available_height() but clamp it because sometimes it can be infinite or 0.
        let h = ui.available_height().clamp(40.0, 120.0);

        let desired_size = Vec2::new(self.width, h);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Draw rack frame
            draw_rack_frame(painter, rect);

            // Inner content rect (inset for frame)
            let frame_width = 8.0;
            let content_rect = rect.shrink(frame_width);

            match self.style {
                AudioMeterStyle::Retro => {
                    draw_retro_stereo(ui, content_rect, self.level_db_left, self.level_db_right)
                }
                AudioMeterStyle::Digital => {
                    draw_digital_stereo(ui, content_rect, self.level_db_left, self.level_db_right)
                }
            }
        }

        response
    }
}

/// Draws the mounting frame with 4 phillips screws
fn draw_rack_frame(painter: &egui::Painter, rect: Rect) {
    let frame_color = Color32::from_rgb(45, 45, 50);
    let frame_highlight = Color32::from_rgb(65, 65, 70);
    let frame_shadow = Color32::from_rgb(25, 25, 30);

    // Main frame
    painter.rect_filled(rect, 0.0, frame_color);

    // Beveled edges (highlight top-left, shadow bottom-right)
    painter.line_segment(
        [rect.left_top(), Pos2::new(rect.right(), rect.top())],
        Stroke::new(2.0, frame_highlight),
    );
    painter.line_segment(
        [rect.left_top(), Pos2::new(rect.left(), rect.bottom())],
        Stroke::new(2.0, frame_highlight),
    );
    painter.line_segment(
        [rect.right_bottom(), Pos2::new(rect.right(), rect.top())],
        Stroke::new(2.0, frame_shadow),
    );
    painter.line_segment(
        [rect.right_bottom(), Pos2::new(rect.left(), rect.bottom())],
        Stroke::new(2.0, frame_shadow),
    );

    // Draw 4 screws
    let screw_offset = 10.0;
    // Ensure we don't overlap if rect is too small
    if rect.width() > 30.0 && rect.height() > 30.0 {
        let screw_positions = [
            Pos2::new(rect.min.x + screw_offset, rect.min.y + screw_offset),
            Pos2::new(rect.max.x - screw_offset, rect.min.y + screw_offset),
            Pos2::new(rect.min.x + screw_offset, rect.max.y - screw_offset),
            Pos2::new(rect.max.x - screw_offset, rect.max.y - screw_offset),
        ];

        for pos in screw_positions {
            draw_screw(painter, pos, 4.0);
        }
    }
}

/// Draws a realistic phillips head screw
fn draw_screw(painter: &egui::Painter, center: Pos2, radius: f32) {
    // Screw head
    painter.circle_filled(center, radius, Color32::from_rgb(80, 80, 85));
    painter.circle_stroke(
        center,
        radius,
        Stroke::new(0.5, Color32::from_rgb(40, 40, 45)),
    );

    // Inner recess (darker)
    painter.circle_filled(center, radius * 0.7, Color32::from_rgb(50, 50, 55));

    // Phillips cross (+)
    let cross_len = radius * 0.6;
    let cross_color = Color32::from_rgb(30, 30, 35);

    // Horizontal line
    painter.line_segment(
        [
            Pos2::new(center.x - cross_len, center.y),
            Pos2::new(center.x + cross_len, center.y),
        ],
        Stroke::new(1.0, cross_color),
    );
    // Vertical line
    painter.line_segment(
        [
            Pos2::new(center.x, center.y - cross_len),
            Pos2::new(center.x, center.y + cross_len),
        ],
        Stroke::new(1.0, cross_color),
    );
}

/// Draws stereo analog VU meters with glass effect
fn draw_retro_stereo(ui: &mut egui::Ui, rect: Rect, db_left: f32, db_right: f32) {
    let painter = ui.painter();

    // Dark background behind glass
    painter.rect_filled(rect, 0.0, Color32::from_rgb(230, 225, 210)); // Cream/vintage color

    // Split into left and right meters
    let meter_width = (rect.width() - 4.0) / 2.0;
    let left_rect = Rect::from_min_size(rect.min, Vec2::new(meter_width, rect.height()));
    let right_rect = Rect::from_min_size(
        Pos2::new(rect.min.x + meter_width + 4.0, rect.min.y),
        Vec2::new(meter_width, rect.height()),
    );

    // Draw each meter
    draw_single_retro_meter(painter, left_rect, db_left, "L");
    draw_single_retro_meter(painter, right_rect, db_right, "R");

    // Glass overlay effect (covers entire area)
    let glass_rect = rect.shrink(1.0);

    // Glass reflection gradient (top lighter)
    painter.rect_filled(
        Rect::from_min_size(
            glass_rect.min,
            Vec2::new(glass_rect.width(), glass_rect.height() * 0.4),
        ),
        4.0,
        Color32::from_white_alpha(15),
    );

    // Glass edge highlight
    painter.rect_stroke(
        glass_rect,
        4.0,
        Stroke::new(1.0, Color32::from_white_alpha(30)),
<<<<<<< HEAD
        egui::StrokeKind::Middle,
=======
>>>>>>> mary-ux-connections-14566841787494652284
    );
}

fn draw_single_retro_meter(painter: &egui::Painter, rect: Rect, db: f32, label: &str) {
    // Meter face background
    painter.rect_filled(rect, 0.0, Color32::from_rgb(230, 225, 210)); // Cream/vintage color

    // Calculate geometry
    // We want the pivot to be well below the rect
    let pivot_offset = rect.height() * 0.8;
    let center = rect.center_bottom() + Vec2::new(0.0, pivot_offset);
    let radius = pivot_offset + rect.height() * 0.85;

    // Scale arc
    let start_angle = -35.0_f32;
    let end_angle = 35.0_f32;
    let zero_angle = 15.0_f32;

    let angle_to_pos = |angle_deg: f32, r: f32| -> Pos2 {
        let rad = (angle_deg - 90.0).to_radians();
        center + Vec2::new(rad.cos() * r, rad.sin() * r)
    };

    // Red zone (0 to +3 dB)
    let red_points: Vec<Pos2> = (0..=5)
        .map(|i| {
            let t = i as f32 / 5.0;
            let angle = zero_angle + t * (end_angle - zero_angle);
            angle_to_pos(angle, radius * 0.65)
        })
        .collect();

    if red_points.len() >= 2 {
        painter.add(egui::Shape::line(
            red_points,
            Stroke::new(5.0, Color32::from_rgba_premultiplied(200, 60, 60, 100)),
        ));
    }

    // Scale ticks
    let ticks = [
        (-20.0, start_angle),
        (-10.0, -15.0),
        (-5.0, 0.0),
        (0.0, zero_angle),
        (3.0, end_angle),
    ];
    for (_val, angle) in ticks {
        let p1 = angle_to_pos(angle, radius * 0.55);
        let p2 = angle_to_pos(angle, radius * 0.65);
        painter.line_segment([p1, p2], Stroke::new(1.5, Color32::from_gray(50)));

        // Labels could be added here if space permits
    }

    // Needle
    // If db is very negative (or NEG_INFINITY), show needle at minimum position
    let clamped_db = if db.is_finite() {
        db.clamp(-40.0, 6.0)
    } else {
        -40.0
    };
    // Linear approximation for visualization
    let needle_angle = if clamped_db < -20.0 {
        start_angle - 5.0
    } else if clamped_db < 0.0 {
        start_angle + (clamped_db + 20.0) / 20.0 * (zero_angle - start_angle)
    } else {
        zero_angle + clamped_db / 3.0 * (end_angle - zero_angle)
    };

    let needle_tip = angle_to_pos(needle_angle, radius * 0.7);

    // Needle intersection with bottom edge (approximate for visual cleanliness)
    let _needle_base = rect.center_bottom() - Vec2::new(0.0, 2.0);

    // We compute where the needle enters the visible area
    let dir = (needle_tip - center).normalized();
    // Intersection with rect.max.y
    let t_base = (rect.max.y - 2.0 - center.y) / dir.y;
    let visible_base = center + dir * t_base;

    // Draw needle
    painter.line_segment(
        [visible_base, needle_tip],
        Stroke::new(1.5, Color32::from_rgb(180, 40, 40)),
    );

    // Shadow
    painter.line_segment(
        [
            visible_base + Vec2::new(2.0, 2.0),
            needle_tip + Vec2::new(2.0, 2.0),
        ],
        Stroke::new(2.0, Color32::from_black_alpha(40)),
    );

    // Channel label
    painter.text(
        Pos2::new(rect.center().x, rect.min.y + 10.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(12.0),
        Color32::from_gray(80),
    );
}

/// Draws stereo digital LED meter (Horizontal Bars)
fn draw_digital_stereo(ui: &mut egui::Ui, rect: Rect, db_left: f32, db_right: f32) {
    let painter = ui.painter();

    // Dark background
    painter.rect_filled(rect, 0.0, Color32::from_rgb(10, 10, 12));

    // Layout:
    // Top: L
    // Middle: Scale
    // Bottom: R

    let total_h = rect.height();
    let bar_h = (total_h * 0.35).min(15.0); // Max 15px height for bar
    let scale_h = (total_h - 2.0 * bar_h).max(0.0);

    let l_rect = Rect::from_min_size(
        rect.min + Vec2::new(4.0, 2.0),
        Vec2::new(rect.width() - 8.0, bar_h),
    );
    let scale_rect = Rect::from_min_size(
        Pos2::new(rect.min.x + 4.0, l_rect.max.y),
        Vec2::new(rect.width() - 8.0, scale_h),
    );
    let r_rect = Rect::from_min_size(
        Pos2::new(rect.min.x + 4.0, scale_rect.max.y),
        Vec2::new(rect.width() - 8.0, bar_h),
    );

    // Draw Bars
    draw_horizontal_led_bar(painter, l_rect, db_left);
    draw_horizontal_led_bar(painter, r_rect, db_right);

    // Draw Scale
    draw_horizontal_scale(painter, scale_rect);

    // Labels overlay
    painter.text(
        l_rect.left_center() + Vec2::new(4.0, 0.0),
        egui::Align2::LEFT_CENTER,
        "L",
        egui::FontId::proportional(10.0),
        Color32::WHITE,
    );
    painter.text(
        r_rect.left_center() + Vec2::new(4.0, 0.0),
        egui::Align2::LEFT_CENTER,
        "R",
        egui::FontId::proportional(10.0),
        Color32::WHITE,
    );
}

fn draw_horizontal_led_bar(painter: &egui::Painter, rect: Rect, db: f32) {
    let segment_count = 40;
    let _padding = 1.0;
    let total_w = rect.width();
    let seg_w = (total_w - (segment_count as f32 - 1.0)) / segment_count as f32;

    let min_db = -60.0;
    let max_db = 3.0;

    for i in 0..segment_count {
        let t = i as f32 / (segment_count as f32 - 1.0);
        let threshold_db = min_db + t * (max_db - min_db);

        // If db is very negative (or NEG_INFINITY), show no active segments
        let active = db.is_finite() && db >= threshold_db;

        let color = if threshold_db >= 0.0 {
            Color32::from_rgb(255, 50, 50)
        } else if threshold_db >= -10.0 {
            Color32::from_rgb(255, 200, 0)
        } else {
            Color32::from_rgb(0, 255, 0)
        };

        let final_color = if active {
            color
        } else {
            Color32::from_rgba_premultiplied(color.r() / 6, color.g() / 6, color.b() / 6, 255)
        };

        let x = rect.min.x + i as f32 * (seg_w + 1.0);
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x, rect.min.y), Vec2::new(seg_w, rect.height())),
            0.0,
            final_color,
        );
    }
}

fn draw_horizontal_scale(painter: &egui::Painter, rect: Rect) {
    let min_db = -60.0;
    let max_db = 3.0;

    let db_to_x = |db: f32| -> f32 {
        let t = (db - min_db) / (max_db - min_db);
        rect.min.x + t * rect.width()
    };

    let tick_vals = [-40.0, -20.0, -10.0, -6.0, 0.0, 3.0];
    for val in tick_vals {
        let x = db_to_x(val);
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(1.0, Color32::from_gray(60)),
        );

        if rect.height() > 8.0 && (val == -40.0 || val == -20.0 || val == 0.0) {
            painter.text(
                Pos2::new(x, rect.center().y),
                egui::Align2::CENTER_CENTER,
                format!("{:.0}", val),
                egui::FontId::proportional(9.0),
                Color32::from_gray(150),
            );
        }
    }
}
