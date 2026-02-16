//! Styled UI Panel
//!
//! Provides a consistent frame and background for UI panels.

use crate::theme::colors;
use crate::widgets::icons::{AppIcon, IconManager};
use egui::{Color32, Pos2, Rect, Sense, Stroke, Style, Ui, Vec2};

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
            rounding: egui::Rounding::same(4.0),
            inner_margin: egui::Margin::same(8.0),
            outer_margin: egui::Margin::same(0.0),
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

pub fn cyber_panel_frame(_style: &Style) -> egui::Frame {
    egui::Frame {
        fill: Color32::from_rgb(20, 20, 25),
        rounding: egui::Rounding::same(2.0),
        inner_margin: egui::Margin::same(4.0),
        stroke: Stroke::new(1.0, Color32::from_gray(40)),
        ..Default::default()
    }
}

/// Renders a standardized panel header with title and right-aligned actions.
///
/// This widget consumes the full width available.
pub fn render_panel_header(
    ui: &mut Ui,
    title: &str,
    icon: Option<AppIcon>,
    icon_manager: Option<&IconManager>,
    add_actions: impl FnOnce(&mut Ui),
) {
    let height = 28.0;
    let desired_size = Vec2::new(ui.available_width(), height);
    let (rect, _response) = ui.allocate_at_least(desired_size, Sense::hover());

    let painter = ui.painter();

    // 1. Background
    painter.rect_filled(rect, egui::Rounding::same(0.0), colors::LIGHTER_GREY);

    // 2. Accent Stripe (Left)
    let stripe_width = 3.0;
    let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(stripe_width, rect.height()));
    painter.rect_filled(
        stripe_rect,
        egui::Rounding::same(0.0),
        colors::CYAN_ACCENT,
    );

    let mut current_x = rect.min.x + stripe_width + 8.0;

    // 3. Optional Icon
    if let (Some(icon), Some(im)) = (icon, icon_manager) {
        let icon_size = 16.0;
        let icon_rect = Rect::from_center_size(
            Pos2::new(current_x + icon_size / 2.0, rect.center().y),
            Vec2::splat(icon_size),
        );

        if let Some(texture) = im.get(icon) {
            painter.image(
                texture.id(),
                icon_rect,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            current_x += icon_size + 8.0;
        }
    }

    // 4. Title Text
    let text_pos = Pos2::new(current_x, rect.center().y);
    painter.text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        title.to_uppercase(),
        egui::FontId::proportional(14.0), // Standard header size
        Color32::WHITE,
    );

    // 5. Right-aligned Actions Area
    let mut actions_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );

    // Add padding from right edge
    actions_ui.add_space(4.0);

    add_actions(&mut actions_ui);
}
