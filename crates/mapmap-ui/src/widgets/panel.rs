//! Phase 6: Cyber Panel Widgets
//!
//! Provides standardized "Cyber Dark" panel containers and headers
//! to enforce the Resolume/MadMapper aesthetic.

use crate::theme::colors;
use egui::{Color32, Frame, Pos2, Rect, Sense, Stroke, Ui, Vec2};

/// Returns a Frame styled for "Cyber Dark" panels.
///
/// Use this with `egui::SidePanel::frame()` or `egui::Window::frame()`.
///
/// # Styles
/// - Fill: `DARK_GREY` (Standard Panel BG)
/// - Stroke: `STROKE_GREY` (1px Border)
/// - Rounding: 0.0 (Sharp corners)
pub fn cyber_panel_frame(_style: &egui::Style) -> Frame {
    Frame {
        inner_margin: egui::Margin::ZERO, // Header handles spacing
        outer_margin: egui::Margin::ZERO,
        rounding: egui::Rounding::default(),
        shadow: egui::Shadow::default(),
        fill: colors::DARK_GREY,
        stroke: Stroke::new(1.0, colors::STROKE_GREY),
    }
}

/// Renders a standardized panel header with title and right-aligned actions.
///
/// This widget consumes the full width available.
///
/// # Layout
/// - **Background**: `LIGHTER_GREY` (Distinct header bar)
/// - **Accent**: `CYAN_ACCENT` (Left stripe)
/// - **Title**: Uppercase, Bold, White
/// - **Actions**: Right-aligned, user-provided closure
///
/// # Example
/// ```rust
/// render_panel_header(ui, "BROWSER", |ui| {
///     if ui.button("X").clicked() { close_panel(); }
/// });
/// ```
pub fn render_panel_header(ui: &mut Ui, title: &str, add_actions: impl FnOnce(&mut Ui)) {
    let height = 28.0;
    let desired_size = Vec2::new(ui.available_width(), height);
    let (rect, _response) = ui.allocate_at_least(desired_size, Sense::hover());

    let painter = ui.painter();

    // 1. Background
    painter.rect_filled(rect, 0.0, colors::LIGHTER_GREY);

    // 2. Accent Stripe (Left)
    let stripe_width = 3.0;
    let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(stripe_width, rect.height()));
    painter.rect_filled(
        stripe_rect,
        0.0,
        colors::CYAN_ACCENT,
    );

    // 3. Title Text
    let text_pos = Pos2::new(rect.min.x + stripe_width + 8.0, rect.center().y);
    painter.text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        title.to_uppercase(),
        egui::FontId::proportional(14.0), // Standard header size
        Color32::WHITE,
    );

    // 4. Right-aligned Actions Area
    // We allocate a child UI to allow standard button placement
    let mut actions_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );

    // Add padding from right edge
    actions_ui.add_space(4.0);

    add_actions(&mut actions_ui);
}
