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

pub fn bypass_button(ui: &mut Ui, active: bool) -> Response {
    let text = "B";
    let desired_size = Vec2::new(24.0, 24.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    let visuals = ui.style().interact(&response);
    let bg_fill = if active {
        colors::WARN_COLOR
    } else {
        visuals.bg_fill
    };

    // Border: if active, use warning color, else normal
    let stroke = if active {
        Stroke::new(1.0, colors::WARN_COLOR)
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
    // Text color: Black if active (contrast on Orange), else text color
    let text_color = if active {
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

    response.on_hover_text("Bypass Layer")
}

pub fn solo_button(ui: &mut Ui, active: bool) -> Response {
    let text = "S";
    let desired_size = Vec2::new(24.0, 24.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    let visuals = ui.style().interact(&response);
    let bg_fill = if active {
        colors::MINT_ACCENT
    } else {
        visuals.bg_fill
    };

    let stroke = if active {
        Stroke::new(1.0, colors::MINT_ACCENT)
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
    // Text color: Black if active (contrast on Mint), else text color
    let text_color = if active {
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

    response.on_hover_text("Solo Layer")
}

pub fn param_button(ui: &mut Ui) -> Response {
    let text = "P";
    let desired_size = Vec2::new(24.0, 24.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    let visuals = ui.style().interact(&response);
    let bg_fill = if response.hovered() {
        colors::CYAN_ACCENT
    } else {
        visuals.bg_fill
    };

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(0),
        bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );

    let text_pos = rect.center();
    let text_color = if response.hovered() {
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

pub fn duplicate_button(ui: &mut Ui) -> Response {
    let text = "D";
    let desired_size = Vec2::new(24.0, 24.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    let visuals = ui.style().interact(&response);
    let bg_fill = if response.hovered() {
        colors::CYAN_ACCENT
    } else {
        visuals.bg_fill
    };

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(0),
        bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );

    let text_pos = rect.center();
    let text_color = if response.hovered() {
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

    response.on_hover_text("Duplicate Layer")
}

pub fn delete_button(ui: &mut Ui) -> Response {
    let text = "X";
    let desired_size = Vec2::new(24.0, 24.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    let visuals = ui.style().interact(&response);
    let bg_fill = if response.hovered() {
        colors::ERROR_COLOR
    } else {
        visuals.bg_fill
    };

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(0),
        bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );

    let text_pos = rect.center();
    let text_color = if response.hovered() {
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

    response.on_hover_text("Remove Layer")
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
