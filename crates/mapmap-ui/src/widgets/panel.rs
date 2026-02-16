//! Styled UI Panel
//!
//! Provides a consistent frame and background for UI panels.

use egui::{Color32, Rect, Stroke, Ui};

pub struct StyledPanel {
    title: String,
}

impl StyledPanel {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
        }
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        let frame = egui::Frame {
            fill: Color32::from_rgb(35, 35, 40),
            corner_radius: egui::CornerRadius::same(4),
            inner_margin: egui::Margin::same(8),
            outer_margin: egui::Margin::same(0),
            stroke: Stroke::new(1.0, Color32::from_gray(60)),
            ..Default::default()
        };

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.strong(&self.title);
                });
                ui.separator();
                add_contents(ui)
            })
            .inner
        }).inner
    }

    pub fn show_with_stripe<R>(self, ui: &mut Ui, stripe_color: Color32, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        let frame = egui::Frame {
            fill: Color32::from_rgb(30, 30, 35),
            corner_radius: egui::CornerRadius::same(4),
            inner_margin: egui::Margin::same(8),
            outer_margin: egui::Margin::same(0),
            stroke: Stroke::new(1.0, Color32::from_gray(50)),
            ..Default::default()
        };

        frame.show(ui, |ui| {
            // Draw colored stripe
            let rect = ui.max_rect();
            let _stripe_rect = Rect::from_min_max(rect.min, egui::pos2(rect.min.x + 4.0, rect.max.y));
            ui.painter().rect_filled(_stripe_rect, egui::CornerRadius::ZERO, stripe_color);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.strong(&self.title);
                });
                ui.separator();
                add_contents(ui)
            })
            .inner
        }).inner
    }
}
