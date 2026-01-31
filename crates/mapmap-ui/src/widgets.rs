//! Phase 6: Custom Styled Widgets
//!
//! This module provides custom `egui` widgets to match the professional VJ software aesthetic.

use crate::theme::colors;
use egui::{lerp, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

pub fn render_header(ui: &mut Ui, title: &str) {
    let desired_size = Vec2::new(ui.available_width(), 24.0);
    let (rect, _response) = ui.allocate_at_least(desired_size, Sense::hover());

    let painter = ui.painter();
    let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height()));
    painter.rect_filled(
        stripe_rect,
        egui::CornerRadius::same(0),
        colors::CYAN_ACCENT,
    );

    let text_pos = Pos2::new(rect.min.x + 8.0, rect.center().y);
    painter.text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(14.0),
        ui.visuals().text_color(),
    );
}

pub fn colored_progress_bar(ui: &mut Ui, value: f32) -> Response {
    let color = if value < 0.5 {
        colors::CYAN_ACCENT // Cyan (Normal)
    } else if value < 0.8 {
        colors::WARN_COLOR // Orange (Warning)
    } else {
        colors::ERROR_COLOR // Red (Limit)
    };

    let bar = egui::ProgressBar::new(value)
        .show_percentage()
        .text(format!("{:.0}%", value * 100.0))
        .fill(color);

    ui.add(bar)
}

pub fn styled_slider(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) -> Response {
    let desired_size = ui.spacing().slider_width * Vec2::new(1.0, 0.5);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());
    let visuals = ui.style().interact(&response);

    if response.dragged() {
        let min = *range.start();
        let max = *range.end();
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            let new_value = egui::remap_clamp(mouse_pos.x, rect.left()..=rect.right(), min..=max);
            *value = new_value;
        }
    }

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(0),
        ui.visuals().widgets.inactive.bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );

    let fill_rect = Rect::from_min_max(
        rect.min,
        Pos2::new(
            lerp(
                (rect.left())..=(rect.right()),
                (*value - *range.start()) / (*range.end() - *range.start()),
            ),
            rect.max.y,
        ),
    );

    ui.painter().rect(
        fill_rect,
        egui::CornerRadius::same(0),
        colors::CYAN_ACCENT,
        Stroke::new(0.0, Color32::TRANSPARENT),
        egui::StrokeKind::Inside,
    );

    response
}

pub fn styled_knob(ui: &mut Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>) -> Response {
    let desired_size = Vec2::new(48.0, 48.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());
    let visuals = ui.style().interact(&response);

    if response.dragged() {
        let center = rect.center();
        let mouse_pos = response.interact_pointer_pos().unwrap();
        let angle = (mouse_pos - center).angle();
        let new_value = egui::remap_clamp(
            angle,
            -std::f32::consts::PI..=std::f32::consts::PI,
            *range.start()..=*range.end(),
        );
        *value = new_value;
    }

    let painter = ui.painter();
    painter.circle(
        rect.center(),
        rect.width() / 2.0,
        visuals.bg_fill,
        visuals.bg_stroke,
    );

    let angle = egui::remap_clamp(
        *value,
        *range.start()..=*range.end(),
        -std::f32::consts::PI..=std::f32::consts::PI,
    );
    let points: Vec<Pos2> = (0..=100)
        .map(|i| {
            let t = i as f32 / 100.0;
            let angle = lerp(-std::f32::consts::PI..=angle, t);
            rect.center() + Vec2::new(angle.cos(), angle.sin()) * rect.width() / 2.0
        })
        .collect();

    painter.add(egui::epaint::Shape::line(
        points,
        Stroke::new(2.0, colors::CYAN_ACCENT),
    ));

    response
}

/// Generic Icon Button Helper
pub fn icon_button(
    ui: &mut Ui,
    text: &str,
    hover_color: Color32,
    active_color: Color32,
    is_active: bool,
) -> Response {
    let desired_size = Vec2::new(24.0, 24.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    let visuals = ui.style().interact(&response);

    // Background fill logic
    let bg_fill = if is_active {
        active_color
    } else if response.hovered() && hover_color != Color32::TRANSPARENT {
        hover_color
    } else {
        visuals.bg_fill
    };

    // Stroke logic
    let stroke = if is_active {
        Stroke::new(1.0, active_color)
    } else {
        visuals.bg_stroke
    };

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(0),
        bg_fill,
        stroke,
        egui::StrokeKind::Inside,
    );

    let text_pos = rect.center();

    // Text color logic: Black if active or hovered with color
    let is_colored = is_active || (response.hovered() && hover_color != Color32::TRANSPARENT);
    let text_color = if is_colored {
        Color32::BLACK
    } else {
        ui.visuals().text_color()
    };

    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(14.0),
        text_color,
    );

    response
}

pub fn bypass_button(ui: &mut Ui, active: bool) -> Response {
    icon_button(ui, "B", Color32::TRANSPARENT, colors::WARN_COLOR, active)
        .on_hover_text("Bypass Layer")
}

pub fn solo_button(ui: &mut Ui, active: bool) -> Response {
    icon_button(ui, "S", Color32::TRANSPARENT, colors::MINT_ACCENT, active)
        .on_hover_text("Solo Layer")
}

pub fn param_button(ui: &mut Ui) -> Response {
    icon_button(ui, "P", colors::CYAN_ACCENT, colors::CYAN_ACCENT, false)
}

pub fn duplicate_button(ui: &mut Ui) -> Response {
    icon_button(ui, "D", colors::CYAN_ACCENT, colors::CYAN_ACCENT, false)
        .on_hover_text("Duplicate Layer")
}

pub fn delete_button(ui: &mut Ui) -> Response {
    icon_button(ui, "X", colors::ERROR_COLOR, colors::ERROR_COLOR, false)
        .on_hover_text("Remove Layer")
}

pub fn move_up_button(ui: &mut Ui) -> Response {
    icon_button(ui, "⬆", colors::CYAN_ACCENT, colors::CYAN_ACCENT, false)
        .on_hover_text("Move Layer Up")
}

pub fn move_down_button(ui: &mut Ui) -> Response {
    icon_button(ui, "⬇", colors::CYAN_ACCENT, colors::CYAN_ACCENT, false)
        .on_hover_text("Move Layer Down")
}

pub fn collapsing_header_with_reset(
    ui: &mut Ui,
    title: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut Ui),
) -> bool {
    let id = ui.make_persistent_id(title);
    let mut reset_clicked = false;
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, default_open)
        .show_header(ui, |ui| {
            ui.label(title);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(egui::Button::new("↺ Reset").small())
                    .on_hover_text(format!("Reset {} defaults", title))
                    .clicked()
                {
                    reset_clicked = true;
                }
            });
        })
        .body(|ui| {
            add_contents(ui);
        });
    reset_clicked
}

/// A standard panel container with Cyber Dark styling
pub fn panel_container(
    ui: &mut Ui,
    title: &str,
    closable: bool,
    add_contents: impl FnOnce(&mut Ui),
) -> bool {
    let mut close_clicked = false;

    egui::Frame::new()
        .fill(colors::DARK_GREY)
        .stroke(egui::Stroke::new(1.0, colors::STROKE_GREY))
        .inner_margin(0.0)
        .show(ui, |ui| {
            // Header
            let desired_size = Vec2::new(ui.available_width(), 24.0);
            let (rect, _response) = ui.allocate_at_least(desired_size, Sense::hover());

            let painter = ui.painter();

            // Header Background
            painter.rect_filled(
                rect,
                egui::CornerRadius::same(0),
                colors::DARKER_GREY, // Slightly darker for header
            );

            // Accent Stripe
            let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height()));
            painter.rect_filled(
                stripe_rect,
                egui::CornerRadius::same(0),
                colors::CYAN_ACCENT,
            );

            // Title
            let text_pos = Pos2::new(rect.min.x + 8.0, rect.center().y);
            painter.text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::proportional(14.0),
                ui.visuals().text_color(),
            );

            // Close Button
            if closable {
                let button_size = 24.0;
                let button_rect = Rect::from_min_size(
                    Pos2::new(rect.max.x - button_size, rect.min.y),
                    Vec2::new(button_size, button_size),
                );

                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(button_rect)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                if child_ui.add(egui::Button::new("✕").frame(false)).clicked() {
                    close_clicked = true;
                }
            }

            // Content
            ui.add_space(4.0);
            egui::Frame::new().inner_margin(4.0).show(ui, add_contents);
        });

    close_clicked
}
