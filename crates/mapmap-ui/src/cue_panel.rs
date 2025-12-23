//! Cue System UI Panel
use egui;

#[derive(Default)]
pub struct CuePanel {
    pub visible: bool, // Allow visibility control
}

impl CuePanel {
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }
        egui::Window::new("Cue System")
            .open(&mut self.visible) // Allow closing
            .default_size([300.0, 400.0])
            .show(ctx, |ui| {
                ui.label("Cues will be listed here.");
            });
    }
}
