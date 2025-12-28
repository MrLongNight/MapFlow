//! Stereo Audio Meter Widget
//!
//! Enhanced audio level visualization with:
//! - Stereo channels (Left/Right)
//! - Realistic mounting frame with screws
//! - Digital: Labeled scale, wider display
//! - Analog: Glass cover effect

use crate::config::AudioMeterStyle;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2, Widget};

/// A stereo audio meter with realistic mounting frame
pub struct StereoAudioMeter {
    style: AudioMeterStyle,
    level_db_left: f32,
    level_db_right: f32,
    size: Vec2,
}

impl StereoAudioMeter {
    /// Create a new stereo audio meter
    pub fn new(style: AudioMeterStyle, level_db_left: f32, level_db_right: f32) -> Self {
        Self {
            style,
            level_db_left,
            level_db_right,
            size: Vec2::new(120.0, 300.0), // Vertical layout
        }
    }

    /// Set preferred size
    pub fn desired_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
}

impl Widget for StereoAudioMeter {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Draw mounting frame
            draw_mounting_frame(painter, rect);

            // Inner area (inside frame)
            let frame_width = 8.0;
            let inner_rect = Rect::from_min_max(
                rect.min + Vec2::splat(frame_width + 4.0),
                rect.max - Vec2::splat(frame_width + 4.0),
            );

            match self.style {
                AudioMeterStyle::Retro => {
                    draw_stereo_retro_meter(
                        ui,
                        inner_rect,
                        self.level_db_left,
                        self.level_db_right,
                    );
                }
                AudioMeterStyle::Digital => {
                    draw_stereo_digital_meter(
                        ui,
                        inner_rect,
                        self.level_db_left,
                        self.level_db_right,
                    );
                }
            }
        }

        response
    }
}

/// Draws the mounting frame with 4 phillips screws
fn draw_mounting_frame(painter: &egui::Painter, rect: Rect) {
    let frame_color = Color32::from_rgb(45, 45, 50);
    let frame_highlight = Color32::from_rgb(65, 65, 70);
    let frame_shadow = Color32::from_rgb(25, 25, 30);

    // Main frame
    painter.rect_filled(rect, 6.0, frame_color);

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
    let screw_offset = 12.0;
    let screw_positions = [
        Pos2::new(rect.min.x + screw_offset, rect.min.y + screw_offset),
        Pos2::new(rect.max.x - screw_offset, rect.min.y + screw_offset),
        Pos2::new(rect.min.x + screw_offset, rect.max.y - screw_offset),
        Pos2::new(rect.max.x - screw_offset, rect.max.y - screw_offset),
    ];

    for pos in screw_positions {
        draw_phillips_screw(painter, pos, 5.0);
    }
}

/// Draws a realistic phillips head screw
fn draw_phillips_screw(painter: &egui::Painter, center: Pos2, radius: f32) {
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
        Stroke::new(1.5, cross_color),
    );
    // Vertical line
    painter.line_segment(
        [
            Pos2::new(center.x, center.y - cross_len),
            Pos2::new(center.x, center.y + cross_len),
        ],
        Stroke::new(1.5, cross_color),
    );
}

/// Draws stereo analog VU meters with glass effect
fn draw_stereo_retro_meter(ui: &mut egui::Ui, rect: Rect, db_left: f32, db_right: f32) {
    let painter = ui.painter();

    // Dark background behind glass
    painter.rect_filled(rect, 4.0, Color32::from_rgb(20, 20, 22));

    // Split into left and right meters
    let meter_width = (rect.width() - 10.0) / 2.0;
    let left_rect = Rect::from_min_size(rect.min, Vec2::new(meter_width, rect.height()));
    let right_rect = Rect::from_min_size(
        Pos2::new(rect.min.x + meter_width + 10.0, rect.min.y),
        Vec2::new(meter_width, rect.height()),
    );

    // Draw each meter
    draw_single_retro_meter(painter, left_rect, db_left, "L");
    draw_single_retro_meter(painter, right_rect, db_right, "R");

    // Glass overlay effect (covers entire area)
    let glass_rect = rect.shrink(2.0);

    // Glass reflection gradient (top lighter)
    painter.rect_filled(
        Rect::from_min_size(
            glass_rect.min,
            Vec2::new(glass_rect.width(), glass_rect.height() * 0.3),
        ),
        4.0,
        Color32::from_white_alpha(25),
    );

    // Glass edge highlight
    painter.rect_stroke(
        glass_rect,
        4.0,
        Stroke::new(1.0, Color32::from_white_alpha(40)),
    );

    // Subtle inner shadow for depth
    painter.rect_stroke(
        glass_rect.shrink(1.0),
        3.0,
        Stroke::new(1.0, Color32::from_black_alpha(60)),
    );
}

/// Draws a single VU meter (for stereo pair)
fn draw_single_retro_meter(painter: &egui::Painter, rect: Rect, db: f32, label: &str) {
    // Meter face background
    painter.rect_filled(rect, 2.0, Color32::from_rgb(240, 235, 220)); // Cream/vintage color

    let center = Pos2::new(rect.center().x, rect.max.y + rect.height() * 0.3);
    let radius = rect.height() * 0.8;

    // Scale arc
    let start_angle = -50.0_f32;
    let end_angle = 50.0_f32;
    let zero_angle = 30.0_f32;

    let angle_to_pos = |angle_deg: f32, r: f32| -> Pos2 {
        let rad = (angle_deg - 90.0).to_radians();
        center + Vec2::new(rad.cos() * r, rad.sin() * r)
    };

    // Red zone (0 to +3 dB)
    let red_points: Vec<Pos2> = (0..=8)
        .map(|i| {
            let t = i as f32 / 8.0;
            let angle = zero_angle + t * (end_angle - zero_angle);
            angle_to_pos(angle, radius * 0.7)
        })
        .collect();

    if red_points.len() >= 2 {
        painter.add(egui::Shape::line(
            red_points,
            Stroke::new(4.0, Color32::from_rgb(200, 60, 60)),
        ));
    }

    // Scale ticks
    let ticks = [
        (-20.0, start_angle),
        (-10.0, -15.0),
        (-5.0, 10.0),
        (0.0, zero_angle),
        (3.0, end_angle),
    ];
    for (_val, angle) in ticks {
        let p1 = angle_to_pos(angle, radius * 0.55);
        let p2 = angle_to_pos(angle, radius * 0.7);
        painter.line_segment([p1, p2], Stroke::new(1.0, Color32::from_gray(40)));
    }

    // Needle
    let clamped_db = db.clamp(-40.0, 6.0);
    let needle_angle = if clamped_db < -20.0 {
        start_angle - 5.0
    } else if clamped_db < 0.0 {
        start_angle + (clamped_db + 20.0) / 20.0 * (zero_angle - start_angle)
    } else {
        zero_angle + clamped_db / 3.0 * (end_angle - zero_angle)
    };

    let needle_tip = angle_to_pos(needle_angle, radius * 0.75);
    let needle_base = Pos2::new(rect.center().x, rect.max.y - 5.0);

    // Needle shadow
    painter.line_segment(
        [
            needle_base + Vec2::new(1.0, 1.0),
            needle_tip + Vec2::new(1.0, 1.0),
        ],
        Stroke::new(2.0, Color32::from_black_alpha(40)),
    );
    // Needle
    painter.line_segment(
        [needle_base, needle_tip],
        Stroke::new(1.5, Color32::from_rgb(180, 30, 30)),
    );

    // Channel label
    painter.text(
        Pos2::new(rect.center().x, rect.min.y + 12.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(14.0),
        Color32::from_gray(60),
    );
}

/// Draws stereo digital LED meter with labeled scale
fn draw_stereo_digital_meter(ui: &mut egui::Ui, rect: Rect, db_left: f32, db_right: f32) {
    let painter = ui.painter();

    // Dark background
    painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 15, 18));

    // Layout: Scale | Left Bar | Right Bar | Scale
    let scale_width = 25.0;
    let bar_gap = 4.0;
    let bar_width = (rect.width() - scale_width * 2.0 - bar_gap * 3.0) / 2.0;

    let scale_left_rect = Rect::from_min_size(rect.min, Vec2::new(scale_width, rect.height()));
    let left_bar_rect = Rect::from_min_size(
        Pos2::new(rect.min.x + scale_width + bar_gap, rect.min.y),
        Vec2::new(bar_width, rect.height()),
    );
    let right_bar_rect = Rect::from_min_size(
        Pos2::new(left_bar_rect.max.x + bar_gap, rect.min.y),
        Vec2::new(bar_width, rect.height()),
    );
    let scale_right_rect = Rect::from_min_size(
        Pos2::new(right_bar_rect.max.x + bar_gap, rect.min.y),
        Vec2::new(scale_width, rect.height()),
    );

    // Draw scale on left side
    draw_db_scale(painter, scale_left_rect, true);

    // Draw bars
    draw_vertical_led_bar(painter, left_bar_rect, db_left);
    draw_vertical_led_bar(painter, right_bar_rect, db_right);

    // Draw scale on right side
    draw_db_scale(painter, scale_right_rect, false);

    // Channel labels
    painter.text(
        Pos2::new(left_bar_rect.center().x, rect.max.y - 8.0),
        egui::Align2::CENTER_CENTER,
        "L",
        egui::FontId::proportional(10.0),
        Color32::from_gray(150),
    );
    painter.text(
        Pos2::new(right_bar_rect.center().x, rect.max.y - 8.0),
        egui::Align2::CENTER_CENTER,
        "R",
        egui::FontId::proportional(10.0),
        Color32::from_gray(150),
    );
}

/// Draws dB scale labels
fn draw_db_scale(painter: &egui::Painter, rect: Rect, align_right: bool) {
    let labels = [
        ("+3", 0.0),
        ("0", 0.05),
        ("-3", 0.1),
        ("-6", 0.15),
        ("-10", 0.22),
        ("-20", 0.37),
        ("-30", 0.52),
        ("-40", 0.67),
        ("-50", 0.82),
        ("-∞", 0.97),
    ];

    let align = if align_right {
        egui::Align2::RIGHT_CENTER
    } else {
        egui::Align2::LEFT_CENTER
    };

    let x = if align_right {
        rect.max.x - 2.0
    } else {
        rect.min.x + 2.0
    };

    for (label, t) in labels {
        let y = rect.min.y + t * rect.height();
        painter.text(
            Pos2::new(x, y),
            align,
            label,
            egui::FontId::proportional(9.0),
            Color32::from_gray(140),
        );
    }
}

/// Draws a vertical LED bar meter
fn draw_vertical_led_bar(painter: &egui::Painter, rect: Rect, db: f32) {
    let segment_count = 30;
    let padding = 2.0;
    let segment_gap = 1.0;
    let segment_height =
        (rect.height() - padding * 2.0 - (segment_count as f32 - 1.0) * segment_gap)
            / segment_count as f32;
    let segment_width = rect.width() - padding * 2.0;

    let min_db = -60.0;
    let max_db = 3.0;

    for i in 0..segment_count {
        // Segments go from top (hot) to bottom (cold)
        let t = i as f32 / (segment_count as f32 - 1.0);
        let threshold_db = max_db - t * (max_db - min_db);

        let active = db >= threshold_db;

        // Color gradient: Red (top) -> Yellow -> Green (bottom)
        let base_color = if threshold_db >= 0.0 {
            Color32::from_rgb(255, 0, 0) // Red
        } else if threshold_db >= -6.0 {
            Color32::from_rgb(255, 180, 0) // Orange-Yellow
        } else if threshold_db >= -12.0 {
            Color32::from_rgb(255, 255, 0) // Yellow
        } else {
            Color32::from_rgb(0, 255, 0) // Green
        };

        let color = if active {
            base_color
        } else {
            Color32::from_rgba_premultiplied(
                base_color.r() / 6,
                base_color.g() / 6,
                base_color.b() / 6,
                255,
            )
        };

        let x = rect.min.x + padding;
        let y = rect.min.y + padding + i as f32 * (segment_height + segment_gap);

        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x, y), Vec2::new(segment_width, segment_height)),
            1.0,
            color,
        );
    }
}
