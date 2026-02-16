use egui::{Response, Ui, Color32};

pub fn render_header(_ui: &mut Ui, _title: &str) {}

pub fn colored_progress_bar(ui: &mut Ui, value: f32) -> Response {
    ui.label(format!("{:.0}%", value * 100.0))
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
