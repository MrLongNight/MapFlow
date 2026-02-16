use egui::{Response, Ui, Color32, Rect, Stroke, Sense, Pos2};

pub fn render_header(ui: &mut Ui, title: &str) {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.strong(title);
        ui.separator();
    });
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
    _speed: f32,
    range: std::ops::RangeInclusive<f32>,
    _default_value: f32,
    _prefix: &str,
    _suffix: &str,
) -> Response {
    ui.add(egui::DragValue::new(value).range(range))
}

pub fn styled_button(ui: &mut Ui, text: &str, _active: bool) -> Response {
    ui.button(text)
}

pub fn styled_checkbox(ui: &mut Ui, value: &mut bool, text: &str) -> Response {
    ui.checkbox(value, text)
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

pub fn hold_to_action_button(ui: &mut Ui, text: &str, _color: Color32) -> bool {
    ui.button(text).clicked()
}

pub fn check_hold_state(_ui: &mut Ui, _id: egui::Id, _is_holding: bool) -> (bool, f32) {
    (false, 0.0)
}

pub fn draw_safety_radial_fill(_painter: &egui::Painter, _center: Pos2, _radius: f32, _progress: f32, _color: Color32) {}

pub fn collapsing_header_with_reset<R>(ui: &mut Ui, title: &str, _default_open: bool, add_contents: impl FnOnce(&mut Ui) -> R) -> bool {
    ui.collapsing(title, add_contents);
    false
}
