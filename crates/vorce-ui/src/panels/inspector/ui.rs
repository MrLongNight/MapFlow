use crate::theme::colors;
use egui::{Color32, Ui};

pub fn inspector_section(
    ui: &mut Ui,
    title: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut Ui),
) {
    egui::CollapsingHeader::new(title)
        .default_open(default_open)
        .show(ui, |ui| {
            egui::Frame::NONE
                .fill(colors::LIGHTER_GREY)
                .inner_margin(8.0)
                .corner_radius(egui::CornerRadius::ZERO)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    add_contents(ui);
                });
        });
}

pub fn inspector_row(ui: &mut Ui, label: &str, add_contents: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            add_contents(ui);
        });
    });
}

pub fn inspector_value(ui: &mut Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(Color32::WHITE).size(12.0));
}
