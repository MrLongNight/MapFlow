use egui::{Response, Ui, Color32, Pos2};

pub fn render_header(ui: &mut Ui, title: &str) {
<<<<<<< HEAD
    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.strong(title);
        ui.separator();
    });
=======
    let desired_size = Vec2::new(ui.available_width(), 24.0);
    // Allocate space for the header
    let (rect, _response) = ui.allocate_at_least(desired_size, Sense::hover());

    let painter = ui.painter();
    // Header background
    painter.rect_filled(rect, egui::Rounding::same(0.0), colors::LIGHTER_GREY);

    let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height()));
    painter.rect_filled(stripe_rect, egui::Rounding::same(0.0), colors::CYAN_ACCENT);

    let text_pos = Pos2::new(rect.min.x + 8.0, rect.center().y);
    painter.text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(14.0),
        ui.visuals().text_color(),
    );
>>>>>>> mary-ux-connections-14566841787494652284
}

pub fn colored_progress_bar(ui: &mut Ui, value: f32) -> Response {
    ui.add(egui::ProgressBar::new(value).show_percentage())
}

pub fn styled_slider(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    _default_value: f32,
) -> Response {
    ui.add(egui::Slider::new(value, range))
}

<<<<<<< HEAD
pub fn styled_slider_log(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    _default_value: f32,
) -> Response {
    ui.add(egui::Slider::new(value, range).logarithmic(true))
=======
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

    response.on_hover_text("Double-click to reset")
>>>>>>> mary-ux-connections-14566841787494652284
}

pub fn styled_drag_value(
    ui: &mut Ui,
    value: &mut f32,
    _speed: f32,
    range: std::ops::RangeInclusive<f32>,
    _default_value: f32,
    _prefix: &str,
    _suffix: &str,
) -> Response {
<<<<<<< HEAD
    ui.add(egui::DragValue::new(value).range(range))
=======
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
            egui::Rounding::same(0.0),
            Stroke::new(1.0, colors::CYAN_ACCENT),
        );
    }

    response.on_hover_text("Double-click to reset")
>>>>>>> mary-ux-connections-14566841787494652284
}

pub fn styled_button(ui: &mut Ui, text: &str, _active: bool) -> Response {
    ui.button(text)
}

<<<<<<< HEAD
pub fn styled_checkbox(ui: &mut Ui, value: &mut bool, text: &str) -> Response {
    ui.checkbox(value, text)
=======
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

    ui.painter()
        .rect(rect, egui::Rounding::same(0.0), bg_fill, stroke);

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
>>>>>>> mary-ux-connections-14566841787494652284
}

pub fn delete_button(ui: &mut Ui) -> bool {
    ui.button("🗑").clicked()
}

pub fn solo_button(ui: &mut Ui, active: bool) -> Response {
    let mut btn = egui::Button::new("S");
    if active {
        btn = btn.fill(Color32::from_rgb(255, 200, 0));
    }
    ui.add(btn)
}

pub fn bypass_button(ui: &mut Ui, active: bool) -> Response {
    let mut btn = egui::Button::new("B");
    if active {
        btn = btn.fill(Color32::from_rgb(255, 80, 80));
    }
    ui.add(btn)
}

pub fn move_up_button(ui: &mut Ui) -> Response {
    ui.button("⏶")
}

pub fn move_down_button(ui: &mut Ui) -> Response {
    ui.button("⏷")
}

pub fn duplicate_button(ui: &mut Ui) -> Response {
    ui.button("⧉")
}

<<<<<<< HEAD
pub fn hold_to_action_button(ui: &mut Ui, text: &str, _color: Color32) -> bool {
    ui.button(text).clicked()
}

pub fn check_hold_state(_ui: &mut Ui, _id: egui::Id, _is_holding: bool) -> (bool, f32) {
    (false, 0.0)
=======
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
    painter.rect(
        rect,
        egui::Rounding::same(4.0),
        visuals.bg_fill,
        visuals.bg_stroke,
    );

    // Draw focus ring if focused
    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
            egui::Rounding::same(6.0),
            Stroke::new(1.0, ui.style().visuals.selection.stroke.color),
        );
    }

    // 2. Progress Fill
    if progress > 0.0 {
        let mut fill_rect = rect;
        fill_rect.max.x = rect.min.x + rect.width() * progress;
        painter.rect_filled(
            fill_rect,
            egui::Rounding::same(4.0),
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
    painter.rect(
        rect,
        egui::Rounding::same(0.0),
        visuals.bg_fill,
        visuals.bg_stroke,
    );

    // Draw focus ring if focused
    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
            egui::Rounding::same(0.0),
            Stroke::new(1.0, ui.style().visuals.selection.stroke.color),
        );
    }

    // 2. Progress Fill
    if progress > 0.0 {
        let mut fill_rect = rect;
        fill_rect.max.y = rect.max.y;
        fill_rect.min.y = rect.max.y - rect.height() * progress;
        painter.rect_filled(
            fill_rect,
            egui::Rounding::same(0.0),
            color.linear_multiply(0.4),
        );
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

    triggered
>>>>>>> mary-ux-connections-14566841787494652284
}

pub fn draw_safety_radial_fill(_painter: &egui::Painter, _center: Pos2, _radius: f32, _progress: f32, _color: Color32) {}

pub fn collapsing_header_with_reset<R>(ui: &mut Ui, title: &str, _default_open: bool, add_contents: impl FnOnce(&mut Ui) -> R) -> bool {
    ui.collapsing(title, add_contents);
    false
}
