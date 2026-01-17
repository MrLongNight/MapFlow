//! Phase 6: Custom Styled Widgets
//!
//! This module provides custom `egui` widgets to match the professional VJ software aesthetic.

use egui::{lerp, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

pub fn render_header(ui: &mut Ui, title: &str) {
    let desired_size = Vec2::new(ui.available_width(), 24.0);
    let (rect, _response) = ui.allocate_at_least(desired_size, Sense::hover());

    let painter = ui.painter();
    // Use widget active color for the stripe (Theme aware)
    let stripe_color = ui.visuals().widgets.active.bg_fill;
    let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height()));
    painter.rect_filled(stripe_rect, 0.0, stripe_color);

    // Optional: Add a subtle background to the header for better separation
    painter.rect_filled(
        rect,
        0.0,
        ui.visuals().widgets.noninteractive.bg_fill.linear_multiply(0.5),
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
        Color32::from_rgb(0, 255, 0) // Green
    } else if value < 0.8 {
        Color32::from_rgb(255, 255, 0) // Yellow
    } else {
        Color32::from_rgb(255, 0, 0) // Red
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
        visuals.rounding,
        ui.visuals().widgets.inactive.bg_fill,
        visuals.bg_stroke,
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
        visuals.rounding,
        ui.visuals().selection.bg_fill, // Theme-aware selection color
        Stroke::new(0.0, Color32::TRANSPARENT),
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
        Stroke::new(2.0, ui.visuals().selection.bg_fill), // Theme-aware knob arc
    ));

    response
}

pub fn bypass_button(ui: &mut Ui, active: bool) -> Response {
    let text = "B";
    let desired_size = Vec2::new(24.0, 24.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    let visuals = ui.style().interact(&response);
    let bg_fill = if active {
        ui.visuals().widgets.active.bg_fill // Theme-aware active color
    } else {
        visuals.bg_fill
    };

    ui.painter()
        .rect(rect, visuals.rounding, bg_fill, visuals.bg_stroke);

    let text_pos = rect.center();
    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(14.0),
        ui.visuals()
            .override_text_color
            .unwrap_or(visuals.fg_stroke.color),
    );

    response
}

pub fn param_button(ui: &mut Ui) -> Response {
    let text = "P";
    let desired_size = Vec2::new(24.0, 24.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    let visuals = ui.style().interact(&response);
    let bg_fill = if response.hovered() {
        ui.visuals().warn_fg_color // Theme-aware warn/active
    } else {
        visuals.bg_fill
    };

    ui.painter()
        .rect(rect, visuals.rounding, bg_fill, visuals.bg_stroke);

    let text_pos = rect.center();
    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(14.0),
        ui.visuals()
            .override_text_color
            .unwrap_or(visuals.fg_stroke.color),
    );

    response
}

pub fn delete_button(ui: &mut Ui) -> Response {
    let text = "X";
    let desired_size = Vec2::new(24.0, 24.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    let visuals = ui.style().interact(&response);
    let bg_fill = if response.hovered() {
        ui.visuals().error_fg_color // Theme-aware error
    } else {
        visuals.bg_fill
    };

    ui.painter()
        .rect(rect, visuals.rounding, bg_fill, visuals.bg_stroke);

    let text_pos = rect.center();
    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(14.0),
        ui.visuals()
            .override_text_color
            .unwrap_or(visuals.fg_stroke.color),
    );

    response
}
