//! Audio Meter Widget
//!
//! Provides accessible audio visualization:
//! - Stereo: Analog VU meter (Retro) or LED Bar (Digital)
//! - Mono: Single horizontal bar (for node graphs)
//! - Spectrum: Frequency band visualization (FFT)

use crate::config::AudioMeterStyle;
use crate::theme::colors;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2, Widget, WidgetInfo, WidgetType};

/// The data mode for the audio meter
#[derive(Clone, Debug)]
pub enum AudioMeterMode {
    /// Stereo levels in dB (-60 to +3)
    Stereo { left_db: f32, right_db: f32 },
    /// Single channel level (0.0 to 1.0)
    Mono { level: f32 },
    /// Frequency spectrum bands (0.0 to 1.0)
    Spectrum { bands: Vec<f32> },
}

/// A widget that displays audio levels.
pub struct AudioMeter {
    mode: AudioMeterMode,
    style: AudioMeterStyle,
    width: Option<f32>,
    height: Option<f32>,
    beat_active: bool,
}

impl AudioMeter {
    /// Create a new stereo audio meter (Retro/Digital based on style)
    pub fn new(style: AudioMeterStyle, level_db_left: f32, level_db_right: f32) -> Self {
        Self {
            mode: AudioMeterMode::Stereo {
                left_db: level_db_left,
                right_db: level_db_right,
            },
            style,
            width: None,
            height: None,
            beat_active: false,
        }
    }

    /// Create a new mono audio meter (0.0 - 1.0)
    pub fn new_mono(level: f32) -> Self {
        Self {
            mode: AudioMeterMode::Mono { level },
            style: AudioMeterStyle::Digital, // Default style for mono
            width: None,
            height: None,
            beat_active: false,
        }
    }

    /// Create a new spectrum visualizer (bands 0.0 - 1.0)
    pub fn new_spectrum(bands: &[f32]) -> Self {
        Self {
            mode: AudioMeterMode::Spectrum {
                bands: bands.to_vec(),
            },
            style: AudioMeterStyle::Digital,
            width: None,
            height: None,
            beat_active: false,
        }
    }

    /// Set preferred width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set preferred height
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set beat activation state (highlights lower bands in spectrum)
    pub fn beat(mut self, active: bool) -> Self {
        self.beat_active = active;
        self
    }
}

impl Widget for AudioMeter {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (desired_size, sense) = match &self.mode {
            AudioMeterMode::Stereo { .. } => {
                let w = self.width.unwrap_or(match self.style {
                    AudioMeterStyle::Retro => 300.0,
                    AudioMeterStyle::Digital => 360.0,
                });
                let h = self.height.unwrap_or_else(|| ui.available_height().clamp(40.0, 120.0));
                (Vec2::new(w, h), Sense::hover())
            }
            AudioMeterMode::Mono { .. } => {
                let w = self.width.unwrap_or(ui.available_width());
                let h = self.height.unwrap_or(4.0);
                (Vec2::new(w, h), Sense::hover())
            }
            AudioMeterMode::Spectrum { .. } => {
                let w = self.width.unwrap_or(ui.available_width());
                let h = self.height.unwrap_or(60.0);
                (Vec2::new(w, h), Sense::hover())
            }
        };

        let (rect, response) = ui.allocate_exact_size(desired_size, sense);

        // Accessibility Info
        response.widget_info(|| {
            let label = match &self.mode {
                AudioMeterMode::Stereo { left_db, right_db } => {
                    format!("Stereo Meter: L {:.1}dB, R {:.1}dB", left_db, right_db)
                }
                AudioMeterMode::Mono { level } => {
                    format!("Audio Level: {:.0}%", level * 100.0)
                }
                AudioMeterMode::Spectrum { bands } => {
                    format!("Audio Spectrum with {} bands", bands.len())
                }
            };
            WidgetInfo::labeled(WidgetType::Other, true, label)
        });

        if ui.is_rect_visible(rect) {
            match &self.mode {
                AudioMeterMode::Stereo { left_db, right_db } => match self.style {
                    AudioMeterStyle::Retro => {
                        draw_rack_frame(ui.painter(), rect);
                        let content_rect = rect.shrink(8.0);
                        draw_retro_stereo(ui, content_rect, *left_db, *right_db);
                    }
                    AudioMeterStyle::Digital => {
                        draw_rack_frame(ui.painter(), rect);
                        let content_rect = rect.shrink(8.0);
                        draw_digital_stereo(ui, content_rect, *left_db, *right_db);
                    }
                },
                AudioMeterMode::Mono { level } => {
                    draw_mono_bar(ui, rect, *level);
                }
                AudioMeterMode::Spectrum { bands } => {
                    draw_spectrum(ui, rect, bands, self.beat_active);
                }
            }
        }

        response
    }
}

// --- Drawing Implementations ---

fn draw_rack_frame(painter: &egui::Painter, rect: Rect) {
    let frame_color = Color32::from_rgb(45, 45, 50);
    let frame_highlight = Color32::from_rgb(65, 65, 70);
    let frame_shadow = Color32::from_rgb(25, 25, 30);

    painter.rect_filled(rect, 0.0, frame_color);

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

    let screw_offset = 10.0;
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

fn draw_screw(painter: &egui::Painter, center: Pos2, radius: f32) {
    painter.circle_filled(center, radius, Color32::from_rgb(80, 80, 85));
    painter.circle_stroke(
        center,
        radius,
        Stroke::new(0.5, Color32::from_rgb(40, 40, 45)),
    );
    painter.circle_filled(center, radius * 0.7, Color32::from_rgb(50, 50, 55));

    let cross_len = radius * 0.6;
    let cross_color = Color32::from_rgb(30, 30, 35);

    painter.line_segment(
        [
            Pos2::new(center.x - cross_len, center.y),
            Pos2::new(center.x + cross_len, center.y),
        ],
        Stroke::new(1.0, cross_color),
    );
    painter.line_segment(
        [
            Pos2::new(center.x, center.y - cross_len),
            Pos2::new(center.x, center.y + cross_len),
        ],
        Stroke::new(1.0, cross_color),
    );
}

fn draw_retro_stereo(ui: &mut egui::Ui, rect: Rect, db_left: f32, db_right: f32) {
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, Color32::from_rgb(230, 225, 210));

    let meter_width = (rect.width() - 4.0) / 2.0;
    let left_rect = Rect::from_min_size(rect.min, Vec2::new(meter_width, rect.height()));
    let right_rect = Rect::from_min_size(
        Pos2::new(rect.min.x + meter_width + 4.0, rect.min.y),
        Vec2::new(meter_width, rect.height()),
    );

    draw_single_retro_meter(painter, left_rect, db_left, "L");
    draw_single_retro_meter(painter, right_rect, db_right, "R");

    let glass_rect = rect.shrink(1.0);
    painter.rect_filled(
        Rect::from_min_size(
            glass_rect.min,
            Vec2::new(glass_rect.width(), glass_rect.height() * 0.4),
        ),
        4.0,
        Color32::from_white_alpha(15),
    );
    painter.rect_stroke(
        glass_rect,
        4.0,
        Stroke::new(1.0, Color32::from_white_alpha(30)),
        egui::StrokeKind::Middle,
    );
}

fn draw_single_retro_meter(painter: &egui::Painter, rect: Rect, db: f32, label: &str) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(230, 225, 210));

    let pivot_offset = rect.height() * 0.8;
    let center = rect.center_bottom() + Vec2::new(0.0, pivot_offset);
    let radius = pivot_offset + rect.height() * 0.85;

    let start_angle = -35.0_f32;
    let end_angle = 35.0_f32;
    let zero_angle = 15.0_f32;

    let angle_to_pos = |angle_deg: f32, r: f32| -> Pos2 {
        let rad = (angle_deg - 90.0).to_radians();
        center + Vec2::new(rad.cos() * r, rad.sin() * r)
    };

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
    }

    let clamped_db = if db.is_finite() {
        db.clamp(-40.0, 6.0)
    } else {
        -40.0
    };

    let needle_angle = if clamped_db < -20.0 {
        start_angle - 5.0
    } else if clamped_db < 0.0 {
        start_angle + (clamped_db + 20.0) / 20.0 * (zero_angle - start_angle)
    } else {
        zero_angle + clamped_db / 3.0 * (end_angle - zero_angle)
    };

    let needle_tip = angle_to_pos(needle_angle, radius * 0.7);
    let dir = (needle_tip - center).normalized();
    let t_base = (rect.max.y - 2.0 - center.y) / dir.y;
    let visible_base = center + dir * t_base;

    painter.line_segment(
        [visible_base, needle_tip],
        Stroke::new(1.5, Color32::from_rgb(180, 40, 40)),
    );
    painter.line_segment(
        [
            visible_base + Vec2::new(2.0, 2.0),
            needle_tip + Vec2::new(2.0, 2.0),
        ],
        Stroke::new(2.0, Color32::from_black_alpha(40)),
    );

    painter.text(
        Pos2::new(rect.center().x, rect.min.y + 10.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(12.0),
        Color32::from_gray(80),
    );
}

fn draw_digital_stereo(ui: &mut egui::Ui, rect: Rect, db_left: f32, db_right: f32) {
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, Color32::from_rgb(10, 10, 12));

    let total_h = rect.height();
    let bar_h = (total_h * 0.35).min(15.0);
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

    draw_horizontal_led_bar(painter, l_rect, db_left);
    draw_horizontal_led_bar(painter, r_rect, db_right);
    draw_horizontal_scale(painter, scale_rect);

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
    let total_w = rect.width();
    let seg_w = (total_w - (segment_count as f32 - 1.0)) / segment_count as f32;

    let min_db = -60.0;
    let max_db = 3.0;

    for i in 0..segment_count {
        let t = i as f32 / (segment_count as f32 - 1.0);
        let threshold_db = min_db + t * (max_db - min_db);
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

// --- New Drawers for Mono/Spectrum ---

fn draw_mono_bar(ui: &mut egui::Ui, rect: Rect, level: f32) {
    let painter = ui.painter();
    painter.rect_filled(rect, 2.0, colors::DARKER_GREY);

    let num_segments = 20;
    let segment_spacing = 1.0;
    let segment_width = (rect.width() - (num_segments as f32 - 1.0) * segment_spacing) / num_segments as f32;

    for i in 0..num_segments {
        let t = i as f32 / num_segments as f32;
        if t > level {
            break;
        }

        let seg_x = rect.min.x + i as f32 * (segment_width + segment_spacing);
        let seg_rect = Rect::from_min_size(
            Pos2::new(seg_x, rect.min.y),
            Vec2::new(segment_width, rect.height()),
        );

        let seg_color = if t < 0.6 {
            colors::MINT_ACCENT // Green
        } else if t < 0.85 {
            colors::WARN_COLOR // Yellow/Orange
        } else {
            colors::ERROR_COLOR // Red
        };

        painter.rect_filled(seg_rect, 1.0, seg_color);
    }
}

fn draw_spectrum(ui: &mut egui::Ui, rect: Rect, bands: &[f32], beat_active: bool) {
    let painter = ui.painter();
    painter.rect_filled(rect, 2.0, colors::DARKER_GREY);
    painter.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0, colors::STROKE_GREY),
        egui::StrokeKind::Middle,
    );

    let num_bands = bands.len();
    if num_bands == 0 {
        return;
    }

    let spacing = 2.0;
    let band_width = ((rect.width() - (num_bands as f32 + 1.0) * spacing) / num_bands as f32).max(1.0);

    for i in 0..num_bands {
        let energy = bands[i];
        let x = rect.min.x + spacing + i as f32 * (band_width + spacing);
        let h = (energy * (rect.height() - 2.0 * spacing)).max(1.0);

        let band_rect = Rect::from_min_max(
            Pos2::new(x, rect.max.y - spacing - h),
            Pos2::new(x + band_width, rect.max.y - spacing),
        );

        let color = if beat_active && i < 2 {
            colors::MINT_ACCENT // Beat hit!
        } else if energy > 0.8 {
            colors::CYAN_ACCENT // High intensity
        } else {
            colors::CYAN_ACCENT.linear_multiply(0.6 + (energy * 0.4))
        };

        painter.rect_filled(band_rect, 1.0, color);
    }
}
