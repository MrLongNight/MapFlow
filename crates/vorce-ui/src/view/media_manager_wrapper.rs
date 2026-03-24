pub struct MediaManagerUIWrapper {
    pub visible: bool,
}

impl MediaManagerUIWrapper {
    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        library: &mut vorce_core::media_library::MediaLibrary,
    ) {
        if !self.visible {
            return;
        }
        let mut open = self.visible;
        egui::Window::new("Media Manager")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Media Manager Placeholder");
                if ui.button("Refresh").clicked() {
                    library.refresh();
                }
            });
        self.visible = open;
    }
}
