use crate::theme::colors;
use crate::widgets::icons::{AppIcon, IconManager};
use egui::{
    lerp, Color32, CornerRadius, Key, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, WidgetInfo,
    WidgetType,
};

pub fn render_header(ui: &mut Ui, title: &str) {
    let desired_size = Vec2::new(ui.available_width(), 24.0);
    // Allocate space for the header
    let (rect, _response) = ui.allocate_at_least(desired_size, Sense::hover());

    let painter = ui.painter();
    // Header background
    painter.rect_filled(rect, CornerRadius::ZERO, colors::LIGHTER_GREY);

    let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height()));
    painter.rect_filled(stripe_rect, CornerRadius::ZERO, colors::CYAN_ACCENT);

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
    ui.add(egui::ProgressBar::new(value).show_percentage())
}

pub fn styled_slider(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    default_value: f32,
) -> Response {
    let desired_size = Vec2::new(ui.available_width(), 20.0);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    let visuals = ui.style().interact(&response);

    // Double-click to reset
    if response.double_clicked() {
        *value = default_value;
    } else if response.dragged() {
        let min = *range.start();
        let max = *range.end();
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            let new_value = egui::remap_clamp(mouse_pos.x, rect.left()..=rect.right(), min..=max);
            *value = new_value;
        }
    } else if response.has_focus() {
        // Keyboard support
        let step = (*range.end() - *range.start()) / 100.0;
        let small_step = step * 0.1;
        let large_step = step * 10.0;

        let mut new_value = *value;

        if ui.input(|i| i.key_pressed(Key::ArrowLeft)) {
            let s = if ui.input(|i| i.modifiers.shift) {
                large_step
            } else if ui.input(|i| i.modifiers.ctrl) {
                small_step
            } else {
                step
            };
            new_value -= s;
        }
        if ui.input(|i| i.key_pressed(Key::ArrowRight)) {
            let s = if ui.input(|i| i.modifiers.shift) {
                large_step
            } else if ui.input(|i| i.modifiers.ctrl) {
                small_step
            } else {
                step
            };
            new_value += s;
        }

        *value = new_value.clamp(*range.start(), *range.end());
    }

    ui.painter().rect(
        rect,
        CornerRadius::ZERO,
        colors::DARKER_GREY, // Track background
        visuals.bg_stroke,
        egui::StrokeKind::Middle,
    );

    // Draw focus ring if focused
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            CornerRadius::ZERO,
            Stroke::new(1.0, ui.style().visuals.selection.stroke.color),
            egui::StrokeKind::Middle,
        );
    }

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
        CornerRadius::ZERO,
        fill_color,
        Stroke::new(0.0, Color32::TRANSPARENT),
        egui::StrokeKind::Middle,
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

    // Accessibility
    response.widget_info(|| {
        let mut info = WidgetInfo::labeled(WidgetType::Slider, ui.is_enabled(), "Slider");
        info.value = Some(*value as f64);
        info
    });

    response.on_hover_text("Double-click to reset, Drag to adjust, Arrows to fine tune")
}

pub fn styled_slider_log(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    _default_value: f32,
) -> Response {
    ui.add(egui::Slider::new(value, range).logarithmic(true))
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
            CornerRadius::ZERO,
            Stroke::new(1.0, colors::CYAN_ACCENT),
            egui::StrokeKind::Middle,
        );
    }

    response.on_hover_text("Double-click to reset")
}

pub fn styled_button(ui: &mut Ui, text: &str, _active: bool) -> Response {
    ui.button(text)
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

    // Accessibility info
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), text));

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

    ui.painter().rect(
        rect,
        CornerRadius::ZERO,
        bg_fill,
        stroke,
        egui::StrokeKind::Middle,
    );

    // Draw focus ring if focused
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            CornerRadius::ZERO,
            Stroke::new(1.0, ui.style().visuals.selection.stroke.color),
            egui::StrokeKind::Middle,
        );
    }

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
    ui.button("🗑").clicked()
}

pub fn lock_button(ui: &mut Ui, active: bool) -> Response {
    // Using a simple "L" as fallback for Lock icon
    let mut btn = egui::Button::new("🔒");
    if active {
        // Reddish fill for Locked state
        btn = btn.fill(Color32::from_rgb(200, 50, 50));
    }
    ui.add(btn)
}

pub fn move_up_button(ui: &mut Ui) -> Response {
    ui.button("⏶")
}

pub fn move_down_button(ui: &mut Ui) -> Response {
    ui.button("⏷")
}

/// Helper function to handle hold-to-confirm logic.
///
/// Returns a tuple: `(triggered, progress)`
/// - `triggered`: `true` if the hold action completed successfully.
/// - `progress`: Normalized progress (0.0 to 1.0) of the hold action.
pub fn check_hold_state(ui: &mut Ui, id: egui::Id, is_interacting: bool) -> (bool, f32) {
    let hold_duration = 0.6; // seconds

    // Use specific IDs for state storage to avoid collisions
    let start_time_id = id.with("start_time");
    let progress_id = id.with("progress");

    let mut start_time: Option<f64> = ui.data_mut(|d| d.get_temp(start_time_id));
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

        // Store progress for external visualization if needed
        ui.data_mut(|d| d.insert_temp(progress_id, progress));

        if progress >= 1.0 {
            triggered = true;
            ui.data_mut(|d| d.remove_temp::<Option<f64>>(start_time_id)); // Reset
            ui.data_mut(|d| d.remove_temp::<f32>(progress_id));
        } else {
            ui.ctx().request_repaint(); // Animate
        }
    } else if start_time.is_some() {
        // Reset if released early
        ui.data_mut(|d| d.remove_temp::<Option<f64>>(start_time_id));
        ui.data_mut(|d| d.remove_temp::<f32>(progress_id));
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

    // Accessibility info
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), text));

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
    painter.rect(
        rect,
        CornerRadius::same(4),
        visuals.bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Middle,
    );

    // Draw focus ring if focused
    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
            CornerRadius::same(6),
            Stroke::new(1.0, ui.style().visuals.selection.stroke.color),
            egui::StrokeKind::Middle,
        );
    }

    // 2. Progress Fill
    if progress > 0.0 {
        let mut fill_rect = rect;
        fill_rect.max.x = rect.min.x + rect.width() * progress;
        painter.rect_filled(
            fill_rect,
            CornerRadius::same(4),
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

    triggered
}

/// A safety icon button that requires holding down for 0.6s to trigger.
/// Visualizes progress with a ring overlay.
pub fn hold_to_action_icon(
    ui: &mut Ui,
    icon_manager: Option<&IconManager>,
    icon: AppIcon,
    size: f32,
    color: Color32,
) -> bool {
    let desired_size = Vec2::splat(size + 8.0); // Add padding for ring
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

    // Accessibility info
    let enabled = ui.is_enabled();
    let label = format!("{:?} Action", icon);
    response.widget_info(move || WidgetInfo::labeled(WidgetType::Button, enabled, label.clone()));

    let state_id = response.id.with("hold_state");

    // Check inputs
    let is_interacting = response.is_pointer_button_down_on()
        || (response.has_focus()
            && (ui.input(|i| i.key_down(egui::Key::Space) || i.key_down(egui::Key::Enter))));

    let (triggered, progress) = check_hold_state(ui, state_id, is_interacting);

    // Visuals
    let painter = ui.painter();
    let center = rect.center();

    // Draw focus ring if focused
    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
            CornerRadius::same(6),
            Stroke::new(1.0, ui.style().visuals.selection.stroke.color),
            egui::StrokeKind::Middle,
        );
    }

    // Draw Icon
    if let Some(mgr) = icon_manager {
        if let Some(texture) = mgr.get(icon) {
            let icon_rect = Rect::from_center_size(center, Vec2::splat(size));
            let tint = if response.hovered() || is_interacting {
                Color32::WHITE
            } else {
                Color32::from_gray(200)
            };
            painter.image(
                texture.id(),
                icon_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                tint,
            );
        } else {
            // Fallback text if texture not found
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "?",
                egui::FontId::proportional(size),
                color,
            );
        }
    } else {
        // Fallback if no manager
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            "!",
            egui::FontId::proportional(size),
            color,
        );
    }

    // Draw Progress Ring
    if progress > 0.0 {
        use std::f32::consts::TAU;
        let radius = size / 2.0 + 2.0;
        let stroke = Stroke::new(2.0, color);

        // Background ring (faint)
        painter.circle_stroke(center, radius, Stroke::new(2.0, color.linear_multiply(0.2)));

        // Better visual: Arc using points
        let start_angle = -TAU / 4.0; // Top
        let end_angle = start_angle + progress * TAU;
        let n_points = 32;
        let points: Vec<Pos2> = (0..=n_points)
            .map(|i| {
                let t = i as f32 / n_points as f32;
                let angle = lerp(start_angle..=end_angle, t);
                center + Vec2::new(angle.cos(), angle.sin()) * radius
            })
            .collect();

        painter.add(egui::Shape::line(points, stroke));
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.on_hover_text("Hold to confirm");

    triggered
}

pub fn draw_safety_radial_fill(
    _painter: &egui::Painter,
    _center: Pos2,
    _radius: f32,
    _progress: f32,
    _color: Color32,
) {
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
