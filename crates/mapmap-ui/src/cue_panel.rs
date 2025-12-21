//! Cue System UI Panel

use imgui::{Condition, Ui};

#[derive(Default)]
pub struct CuePanel {}

impl CuePanel {
    pub fn render(&mut self, ui: &Ui) {
        ui.window("Cue System")
            .size([300.0, 400.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Cues will be listed here.");
            });
    }
}
