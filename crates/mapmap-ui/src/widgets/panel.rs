//! Styled UI Panel
//!
//! Provides a consistent frame and background for UI panels.

use egui::{Stroke, Style, Ui};

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
            fill: crate::theme::colors::DARK_GREY,
            corner_radius: egui::CornerRadius::ZERO,
            inner_margin: egui::Margin::same(8),
            outer_margin: egui::Margin::same(0),
            stroke: Stroke::new(1.0, crate::theme::colors::STROKE_GREY),
            ..Default::default()
        };

        frame
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_panel_header(ui, &self.title, |_| {});
                    add_contents(ui)
                })
                .inner
            })
            .inner
    }
}

/// Create a standard "Cyber Dark" panel frame
pub fn cyber_panel_frame(_style: &Style) -> egui::Frame {
    egui::Frame {
        fill: crate::theme::colors::DARK_GREY,
        corner_radius: egui::CornerRadius::ZERO, // Sharp corners
        inner_margin: egui::Margin::same(0), // Removed inner margin to let header touch edges
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
    egui::Frame::default()
        .fill(crate::theme::colors::LIGHTER_GREY)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .corner_radius(egui::CornerRadius::ZERO)
        .stroke(egui::Stroke {
            width: 1.0,
            color: crate::theme::colors::STROKE_GREY,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.strong(title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    add_contents(ui)
                })
                .inner
            })
            .inner
        })
        .inner
}
