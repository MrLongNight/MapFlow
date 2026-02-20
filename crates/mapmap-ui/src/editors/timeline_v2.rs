//! Phase 6: Enhanced Timeline Editor with Keyframe Animation
//!
//! Multi-track timeline with keyframe animation, using mapmap_core::animation types.

use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::animation::AnimValue;
use mapmap_core::effect_animation::EffectParameterAnimator;
use serde::{Deserialize, Serialize};

/// Timeline editor view state (data is in AnimationClip)
#[derive(Serialize, Deserialize)]
pub struct TimelineV2 {
    /// Playhead position (in seconds) - purely for visualization if not synced
    pub playhead: f32,
    /// Zoom level (pixels per second)
    pub zoom: f32,
    /// Pan offset
    pub pan_offset: f32,
    /// Snap settings
    pub snap_enabled: bool,
    pub snap_interval: f32,
    /// Selected keyframes (track_name, key_time_us)
    pub selected_keyframes: Vec<(String, u64)>,
    /// Show curve editor
    pub show_curve_editor: bool,
}

impl Default for TimelineV2 {
    fn default() -> Self {
        Self {
            playhead: 0.0,
            zoom: 100.0,
            pan_offset: 0.0,
            snap_enabled: true,
            snap_interval: 0.1, // 100ms default snap
            selected_keyframes: Vec::new(),
            show_curve_editor: false,
        }
    }
}

impl TimelineV2 {
    /// Snap time to grid
    fn snap_time(&self, time: f32) -> f32 {
        if self.snap_enabled && self.snap_interval > 0.0 {
            (time / self.snap_interval).round() * self.snap_interval
        } else {
            time
        }
    }

    /// Render the timeline UI interacting with the EffectParameterAnimator
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        animator: &mut EffectParameterAnimator,
    ) -> Option<TimelineAction> {
        let mut action = None;

        // Sync local playhead with animator
        self.playhead = animator.get_current_time() as f32;

        let duration = animator.duration() as f32;

        // Toolbar
        ui.horizontal(|ui| {
            if animator.is_playing() {
                if ui.button("⏸ Pause").clicked() {
                    action = Some(TimelineAction::Pause);
                }
            } else if ui.button("⏵ Play").clicked() {
                action = Some(TimelineAction::Play);
            }

            if ui.button("⏹ Stop").clicked() {
                action = Some(TimelineAction::Stop);
            }

            ui.separator();

            ui.label(format!("Time: {:.2}s", self.playhead));

            ui.separator();

            // Snap settings
            ui.checkbox(&mut self.snap_enabled, "Snap");
            if self.snap_enabled {
                ui.add(
                    egui::DragValue::new(&mut self.snap_interval)
                        .prefix("Snap: ")
                        .suffix("s")
                        .speed(0.01)
                        .range(0.01..=10.0),
                );
            }

            ui.separator();

            // Zoom controls
            ui.label(format!("Zoom: {:.0}px/s", self.zoom));
            if ui.button("➕").clicked() {
                self.zoom *= 1.2;
            }
            if ui.button("➖").clicked() {
                self.zoom /= 1.2;
            }

            // Loop toggle
            let mut looping = animator.clip().looping;
            if ui.checkbox(&mut looping, "Loop").changed() {
                animator.set_looping(looping);
            }
        });

        ui.separator();

        // Timeline area
        egui::ScrollArea::both().show(ui, |ui| {
            let clip = animator.clip(); // Get immutable ref first to calculate size
            let track_count = clip.tracks.len();

            let available_height = 50.0 + (track_count as f32 * 60.0);
            let available_width = (duration * self.zoom).max(ui.available_width());

            let (response, painter) = ui.allocate_painter(
                Vec2::new(available_width, available_height),
                Sense::click_and_drag(),
            );

            let rect = response.rect;

            // Draw time ruler
            let ruler_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 30.0));
            painter.rect_filled(ruler_rect, 0.0, Color32::from_rgb(40, 40, 40));

            // Draw time ticks
            let tick_interval = if self.zoom > 100.0 { 0.1 } else { 1.0 };
            let mut time = 0.0;
            while time <= duration {
                let x = rect.min.x + time * self.zoom;
                let h = if (time % 1.0).abs() < 0.001 {
                    15.0
                } else {
                    8.0
                };

                if x >= rect.min.x && x <= rect.max.x {
                    painter.line_segment(
                        [
                            Pos2::new(x, ruler_rect.max.y - h),
                            Pos2::new(x, ruler_rect.max.y),
                        ],
                        Stroke::new(1.0, Color32::from_rgb(150, 150, 150)),
                    );

                    if (time % 1.0).abs() < 0.001 {
                        painter.text(
                            Pos2::new(x + 2.0, ruler_rect.min.y + 2.0),
                            egui::Align2::LEFT_TOP,
                            format!("{:.0}s", time),
                            egui::FontId::proportional(12.0),
                            Color32::WHITE,
                        );
                    }
                }
                time += tick_interval;
            }

            // Draw playhead
            let playhead_x = rect.min.x + self.playhead * self.zoom;
            painter.line_segment(
                [
                    Pos2::new(playhead_x, ruler_rect.min.y),
                    Pos2::new(playhead_x, rect.max.y),
                ],
                Stroke::new(2.0, Color32::from_rgb(255, 50, 50)),
            );

            // Handle ruler scrubbing
            if response.hovered() || response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if pos.y <= ruler_rect.max.y && response.is_pointer_button_down_on() {
                        let time = (pos.x - rect.min.x) / self.zoom;
                        let snapped = if ui.input(|i| i.modifiers.shift) {
                            time // Bypass snap
                        } else {
                            self.snap_time(time)
                        };

                        action = Some(TimelineAction::Seek(snapped));
                    }
                }
            }

            // Access mutable clip again via animator if needed for modifications (actions)
            // But here we just iterate for drawing
            let tracks = &animator.clip().tracks; // Use bindings? No, show tracks.

            let track_start_y = ruler_rect.max.y;

            for (i, track) in tracks.iter().enumerate() {
                let track_y = track_start_y + (i as f32 * 60.0);
                let track_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x, track_y),
                    Vec2::new(rect.width(), 60.0),
                );

                // Alternating background
                let bg_color = if i % 2 == 0 {
                    Color32::from_rgb(30, 30, 30)
                } else {
                    Color32::from_rgb(35, 35, 35)
                };
                painter.rect_filled(track_rect, 0.0, bg_color);

                // Track name
                painter.text(
                    Pos2::new(track_rect.min.x + 5.0, track_rect.min.y + 10.0),
                    egui::Align2::LEFT_TOP,
                    &track.name,
                    egui::FontId::proportional(14.0),
                    Color32::from_rgb(200, 200, 200),
                );

                // Draw keyframes and curves
                // Flatten keyframes for iteration
                let keyframes = track.keyframes_ordered();

                // Draw curve (only for Float/numeric types mostly)
                if keyframes.len() >= 2 {
                    // Simple polyline for now
                    let mut points = Vec::new();
                    for kf in &keyframes {
                        let t = kf.time as f32;
                        let val = match &kf.value {
                            AnimValue::Float(v) => *v,
                            AnimValue::Vec3(v) => v[0], // Only visualize X/R component for now
                            AnimValue::Vec4(v) => v[0],
                            AnimValue::Color(v) => v[0], // Red / 0 component
                            _ => 0.0,
                        };

                        // Normalize value: map 0..1 to height? Or auto-scale.
                        // Assuming 0..1 range for effects usually.
                        let normalized = val.clamp(0.0, 1.0);
                        let x = rect.min.x + t * self.zoom;
                        let y = track_rect.max.y - 10.0 - (normalized * 40.0);
                        points.push(Pos2::new(x, y));
                    }

                    if !points.is_empty() {
                        painter.add(egui::Shape::line(
                            points,
                            Stroke::new(2.0, Color32::from_rgb(100, 200, 255)),
                        ));
                    }
                }

                // Draw Keyframe Diamonds
                for kf in &keyframes {
                    let kf_time = kf.time as f32;
                    let val = match &kf.value {
                        AnimValue::Float(v) => *v,
                        _ => 0.0, // Default for non-floats in view
                    };
                    let normalized = val.clamp(0.0, 1.0);

                    let x = rect.min.x + kf_time * self.zoom;
                    let y = track_rect.max.y - 10.0 - (normalized * 40.0);

                    let diamond_size = 6.0;
                    let diamond = vec![
                        Pos2::new(x, y - diamond_size),
                        Pos2::new(x + diamond_size, y),
                        Pos2::new(x, y + diamond_size),
                        Pos2::new(x - diamond_size, y),
                    ];

                    // Interaction logic could go here (click to select)
                    painter.add(egui::Shape::convex_polygon(
                        diamond,
                        Color32::YELLOW,
                        Stroke::new(1.0, Color32::WHITE),
                    ));
                }
            }
        });

        action
    }
}

/// Actions triggered by timeline
pub enum TimelineAction {
    Play,
    Pause,
    Stop,
    Seek(f32),
}
