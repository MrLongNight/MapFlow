use egui::{Context, Window, Color32, RichText};

/// Renders the About dialog.
pub fn show(ctx: &Context, show_about: &mut bool) {
    let mut is_open = *show_about;
    let mut close_clicked = false;

    Window::new(RichText::new("ℹ ABOUT MAPFLOW").strong().color(Color32::from_rgb(0, 255, 255)))
        .open(&mut is_open)
        .resizable(false)
        .collapsible(false)
        .default_width(350.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(RichText::new("MapFlow VJ").size(24.0).strong());
                ui.label(RichText::new("Version 1.0.0-RC1 (Rescue Edition)").color(Color32::GRAY));

                ui.add_space(10.0);
                ui.label("Professional Projection Mapping & VJ Software");
                ui.label("Built with Rust, wgpu and egui.");

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label("© 2026 MrLongNight & MapFlow Team");
                ui.hyperlink_to("GitHub Repository", "https://github.com/MrLongNight/MapFlow");

                ui.add_space(20.0);
                if ui.button("Close").clicked() {
                    close_clicked = true;
                }
                ui.add_space(10.0);
            });
        });

    if close_clicked {
        *show_about = false;
    } else {
        *show_about = is_open;
    }
}
