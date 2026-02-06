//! Phase 6: Cyber Panel Widgets
//!
//! Provides standardized "Cyber Dark" panel containers and headers
//! to enforce the Resolume/MadMapper aesthetic.

use crate::theme::colors;
use crate::widgets::icons::{AppIcon, IconManager};
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
        corner_radius: egui::CornerRadius::same(0),
        shadow: egui::Shadow::NONE,
        fill: colors::DARK_GREY,
        stroke: Stroke::new(1.0, colors::STROKE_GREY),
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
    painter.rect_filled(rect, egui::CornerRadius::same(0), colors::LIGHTER_GREY);

    // 2. Accent Stripe (Left)
    let stripe_width = 3.0;
    let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(stripe_width, rect.height()));
    painter.rect_filled(
        stripe_rect,
        egui::CornerRadius::same(0),
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
