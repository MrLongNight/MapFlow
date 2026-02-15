//! Phase 6: Custom Styled Widgets
//!
//! This module provides custom `egui` widgets to match the professional VJ software aesthetic.

use crate::theme::colors;
use egui::{lerp, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

pub fn render_header(ui: &mut Ui, title: &str) {
    let desired_size = Vec2::new(ui.available_width(), 24.0);
    // Allocate space for the header
    let (rect, _response) = ui.allocate_at_least(desired_size, Sense::hover());

    let painter = ui.painter();
    // Header background
Response

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
    default_value: f32,
) -> Response {
    let desired_size = ui.spacing().slider_width * Vec2::new(1.0, 0.5);
    let (rect, mut response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());
    let visuals = ui.style().interact(&response);

Response
            response.mark_changed();
        }
    }

Response
    // Double-click to reset
    if response.double_clicked() {
        *value = default_value;
        response.mark_changed();
    } else if response.dragged() {
        let min = *range.start();
        let max = *range.end();
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            let new_value = egui::remap_clamp(mouse_pos.x, rect.left()..=rect.right(), min..=max);
            *value = new_value;
            response.mark_changed();
        }
    }

    ui.painter().rect(
        rect,
        egui::Rounding::same(0.0),
        colors::DARKER_GREY, // Track background
        visuals.bg_stroke,
    );

    let t = (*value - *range.start()) / (*range.end() - *range.start());
    let fill_rect = Rect::from_min_max(
        rect.min,
        Pos2::new(
            lerp((rect.left())..=(rect.right()), t.clamp(0.0, 1.0)),
            rect.max.y,
        ),
    );

    // Accent color logic
    let is_changed = (*value - default_value).abs() > 0.001;
    let fill_color = if is_changed {
        colors::CYAN_ACCENT
    } else {
        colors::CYAN_ACCENT.linear_multiply(0.7)
    };

    ui.painter().rect(
        fill_rect,
        egui::Rounding::same(0.0),
        fill_color,
        Stroke::new(0.0, Color32::TRANSPARENT),
    );

    // Value Text
    let text = format!("{:.2}", value);
    let text_color = if response.hovered() || response.dragged() {
        Color32::WHITE
    } else if is_changed {
        colors::CYAN_ACCENT
    } else {
        Color32::from_gray(180)
    };

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(12.0),
        text_color,
    );

    // Draw focus ring
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            egui::Rounding::same(0.0),
            Stroke::new(1.0, ui.style().visuals.selection.stroke.color),
        );
    }

    // Accessibility info
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(egui::WidgetType::Slider, true, "Custom Slider");
        info.value = Some(*value as f64);
        info
    });

    response.on_hover_text("Double-click to reset")
}

pub fn styled_slider_log(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    default_value: f32,
) -> Response {
    let desired_size = ui.spacing().slider_width * Vec2::new(1.0, 0.5);
    let (rect, mut response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());
    let visuals = ui.style().interact(&response);

    // Keyboard support (multiplicative step)
    if response.has_focus() {
        let step_factor = if ui.input(|i| i.modifiers.shift) {
            1.2 // Large step
        } else {
            1.05 // Small step
        };

        if ui.input(|i| i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowUp)) {
            *value = (*value * step_factor).clamp(*range.start(), *range.end());
            response.mark_changed();
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown))
        {
            *value = (*value / step_factor).clamp(*range.start(), *range.end());
            response.mark_changed();
        }
    }

    // Accessibility metadata
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(egui::WidgetType::Slider, true, "");
        info.value = Some(*value as f64);
        info
    });

    // Double-click to reset
    if response.double_clicked() {
        *value = default_value;
        response.mark_changed();
    } else if response.dragged() {
        let min = *range.start();
        let max = *range.end();
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            let t = egui::remap_clamp(mouse_pos.x, rect.left()..=rect.right(), 0.0..=1.0);
            if min > 0.0 && max > 0.0 {
                *value = min * (max / min).powf(t);
            } else {
                *value = egui::remap_clamp(t, 0.0..=1.0, min..=max);
            }
            response.mark_changed();
        }
    }

    ui.painter().rect(
        rect,
        0.0,
        colors::DARKER_GREY, // Track background
        visuals.bg_stroke,
    );

    let min = *range.start();
    let max = *range.end();
    let t = if min > 0.0 && max > 0.0 && *value > 0.0 {
        ((value.max(min) / min).ln()) / ((max / min).ln())
    } else {
        (*value - min) / (max - min)
    };

    let fill_rect = Rect::from_min_max(
        rect.min,
        Pos2::new(
            lerp((rect.left())..=(rect.right()), t.clamp(0.0, 1.0)),
            rect.max.y,
        ),
    );

    // Accent color logic
    let is_changed = (*value - default_value).abs() > 0.001;
    let fill_color = if is_changed {
        colors::CYAN_ACCENT
    } else {
        colors::CYAN_ACCENT.linear_multiply(0.7)
    };

    ui.painter().rect(
        fill_rect,
        0.0,
        fill_color,
        Stroke::new(0.0, Color32::TRANSPARENT),
    );

    // Value Text
    let text = format!("{:.2}", value);
    let text_color = if response.hovered() || response.dragged() {
        Color32::WHITE
    } else if is_changed {
        colors::CYAN_ACCENT
    } else {
        Color32::from_gray(180)
    };

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(12.0),
        text_color,
    );

    response.on_hover_text("Double-click to reset")
}

pub fn styled_drag_value(
    ui: &mut Ui,
    value: &mut f32,
    speed: f32,
    range: std::ops::RangeInclusive<f32>,
    default_value: f32,
    prefix: &str,
    suffix: &str,
) -> Response {
    let is_changed = (*value - default_value).abs() > 0.001;

    // Use scope to customize spacing or style if needed
    let response = ui.add(
        egui::DragValue::new(value)
            .speed(speed)
            .range(range)
            .prefix(prefix)
            .suffix(suffix),
    );

    if response.double_clicked() {
        *value = default_value;
    }

    // Visual feedback for changed value
    if is_changed {
        ui.painter().rect_stroke(
            response.rect.expand(1.0),
Response
            Stroke::new(1.0, colors::CYAN_ACCENT),
        );
    }

    response.on_hover_text("Double-click to reset")
}

pub fn styled_knob(ui: &mut Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>) -> Response {
    let desired_size = Vec2::new(48.0, 48.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());
    let visuals = ui.style().interact(&response);

    // Keyboard interaction
    if response.has_focus() {
        let range_span = range.end() - range.start();
        let step = range_span / 100.0;
        let large_step = range_span / 10.0;

        let mut delta = 0.0;
        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown))
        {
            delta -= if ui.input(|i| i.modifiers.shift) {
                large_step
            } else {
                step
            };
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowUp)) {
            delta += if ui.input(|i| i.modifiers.shift) {
                large_step
            } else {
                step
            };
        }

        if delta != 0.0 {
            *value = (*value + delta).clamp(*range.start(), *range.end());
            response.mark_changed();
        }
    }

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

    // Draw focus ring
    if response.has_focus() {
        ui.painter().circle_stroke(
            rect.center(),
            rect.width() / 2.0 + 2.0,
            Stroke::new(1.0, ui.style().visuals.selection.stroke.color),
        );
    }

    // Accessibility info
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(egui::WidgetType::Slider, true, "Knob");
        info.value = Some(*value as f64);
        info
    });

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
    } else if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        visuals.bg_fill
    };

    // Stroke logic
    let stroke = if is_active {
        Stroke::new(1.0, active_color)
    } else {
        visuals.bg_stroke
    };

Response

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

pub fn delete_button(ui: &mut Ui) -> bool {
    hold_to_action_icon(ui, "X", colors::ERROR_COLOR)
}

pub fn move_up_button(ui: &mut Ui) -> Response {
    icon_button(ui, "⬆", colors::CYAN_ACCENT, colors::CYAN_ACCENT, false)
        .on_hover_text("Move Layer Up")
}

pub fn move_down_button(ui: &mut Ui) -> Response {
    icon_button(ui, "⬇", colors::CYAN_ACCENT, colors::CYAN_ACCENT, false)
        .on_hover_text("Move Layer Down")
}

/// Helper function to manage hold-to-action state
pub fn check_hold_state(ui: &mut Ui, id: egui::Id, is_interacting: bool) -> (bool, f32) {
    let hold_duration = 0.6; // seconds
    let start_time_id = id.with("start_time");
    let progress_id = id.with("progress");
    let triggered_id = id.with("triggered");

    let mut start_time: Option<f64> = ui.data_mut(|d| d.get_temp(start_time_id));
    let mut already_triggered: bool = ui.data_mut(|d| d.get_temp(triggered_id)).unwrap_or(false);
    let mut triggered = false;
    let mut progress = 0.0;

    if is_interacting {
        let now = ui.input(|i| i.time);
        if start_time.is_none() {
            start_time = Some(now);
            ui.data_mut(|d| d.insert_temp(start_time_id, start_time));
        }

        let elapsed = now - start_time.unwrap();
        progress = (elapsed as f32 / hold_duration).clamp(0.0, 1.0);

        // Store progress for visualization
        ui.data_mut(|d| d.insert_temp(progress_id, progress));

        if progress >= 1.0 {
            if !already_triggered {
                triggered = true;
                already_triggered = true;
                ui.data_mut(|d| d.insert_temp(triggered_id, already_triggered));
            }
        } else {
            ui.ctx().request_repaint(); // Animate
        }
    } else {
        // Reset everything on release
        if start_time.is_some() || already_triggered {
            ui.data_mut(|d| d.remove_temp::<Option<f64>>(start_time_id));
            ui.data_mut(|d| d.remove_temp::<f32>(progress_id));
            ui.data_mut(|d| d.remove_temp::<bool>(triggered_id));
        }
    }

    (triggered, progress)
}

/// A safety button that requires holding down for 0.6s to trigger (Mouse or Keyboard)
pub fn hold_to_action_button(ui: &mut Ui, text: &str, color: Color32) -> bool {
    // Small button size
    let text_galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(12.0),
        ui.visuals().text_color(),
    );
    let size = Vec2::new(text_galley.size().x + 20.0, 20.0);

    // Use Sense::click() for accessibility (focus/tab navigation)
    let (rect, response) = ui.allocate_at_least(size, Sense::click());

    // Use response.id for unique state storage to prevent collisions
    let state_id = response.id.with("hold_state");

    // Check inputs:
    // 1. Mouse/Touch: is_pointer_button_down_on()
    // 2. Keyboard: has_focus() && key_down(Space/Enter)
    let is_interacting = response.is_pointer_button_down_on()
        || (response.has_focus()
            && (ui.input(|i| i.key_down(egui::Key::Space) || i.key_down(egui::Key::Enter))));

    let (triggered, progress) = check_hold_state(ui, state_id, is_interacting);

    // --- Visuals ---
    let visuals = ui.style().interact(&response);
    let painter = ui.painter();

    // 1. Background
Response

    // Draw focus ring if focused
    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
Response
            Stroke::new(1.0, ui.style().visuals.selection.stroke.color),
        );
    }

    // 2. Progress Fill
    if progress > 0.0 {
        let mut fill_rect = rect;
        fill_rect.max.x = rect.min.x + rect.width() * progress;
        painter.rect_filled(
            fill_rect,
Response
            color.linear_multiply(0.4), // Transparent version of action color
        );
    }

    // 3. Text
    let text_color = if triggered {
        color
    } else {
        visuals.text_color()
    };
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(12.0),
        text_color,
    );

    // Tooltip
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.on_hover_text("Hold to confirm (Mouse or Space/Enter)");

    // Accessibility info
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text));

    triggered
}

/// A safety icon button that requires holding down for 0.6s to trigger
pub fn hold_to_action_icon(ui: &mut Ui, icon_text: &str, color: Color32) -> bool {
    let desired_size = Vec2::new(24.0, 24.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    // State
    let state_id = response.id.with("hold_icon");
    let is_interacting = response.is_pointer_button_down_on()
        || (response.has_focus()
            && (ui.input(|i| i.key_down(egui::Key::Space) || i.key_down(egui::Key::Enter))));

    let (triggered, progress) = check_hold_state(ui, state_id, is_interacting);

    // --- Visuals ---
    let visuals = ui.style().interact(&response);
    let painter = ui.painter();

    // 1. Background
Response

    // Draw focus ring if focused
    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
Response
            Stroke::new(1.0, ui.style().visuals.selection.stroke.color),
        );
    }

    // 2. Progress Fill
    if progress > 0.0 {
        let mut fill_rect = rect;
        fill_rect.max.y = rect.max.y;
        fill_rect.min.y = rect.max.y - rect.height() * progress;
Response
    }

    // 3. Icon
    let text_color = if triggered {
        color
    } else {
        visuals.text_color()
    };
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon_text,
        egui::FontId::proportional(14.0),
        text_color,
    );

    // Tooltip
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.on_hover_text("Hold to confirm");

    // Accessibility info
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, icon_text));

    triggered
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
                if hold_to_action_button(ui, "↺ Reset", colors::WARN_COLOR) {
                    reset_clicked = true;
                }
            });
        })
        .body(|ui| {
            add_contents(ui);
        });
    reset_clicked
}


