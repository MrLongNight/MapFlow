//! Styled UI Panel
//!
//! Provides a consistent frame and background for UI panels.

use egui::{Color32, Stroke, Ui, Style};

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
}

/// Create a standard "Cyber Dark" panel frame
pub fn cyber_panel_frame(_style: &Style) -> egui::Frame {
    egui::Frame {
        fill: crate::theme::colors::DARK_GREY,
        corner_radius: egui::CornerRadius::ZERO, // Sharp corners
        inner_margin: egui::Margin::same(4),
        stroke: Stroke::new(1.0, crate::theme::colors::STROKE_GREY),
        ..Default::default()
    }
}

/// Render a standard panel header with title and optional right-side content
pub fn render_panel_header<R>(
    ui: &mut Ui,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        ui.strong(title);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            add_contents(ui)
        })
        .inner
    })
    .inner
}
