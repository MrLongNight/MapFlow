use egui::{Color32, Context, RichText, Window};

/// Renders the About dialog.
pub fn show(ctx: &Context, show_about: &mut bool) {
    let mut is_open = *show_about;
    let mut close_clicked = false;

    Window::new(
        RichText::new("ℹ ABOUT VORCE")
            .strong()
            .color(Color32::from_rgb(0, 255, 255)),
    )
    .open(&mut is_open)
    .resizable(false)
    .collapsible(false)
    .default_width(350.0)
    .show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading(RichText::new("Vorce VJ").size(24.0).strong());

            let version = env!("CARGO_PKG_VERSION");
            ui.label(RichText::new(format!("Version {}", version)).color(Color32::GRAY));

            ui.add_space(10.0);
            ui.label("Professional Projection Mapping & VJ Software");
            ui.label("Built with Rust, wgpu and egui.");

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label("© 2026 MrLongNight & Vorce Team");
            ui.hyperlink_to(
                "GitHub Repository",
                "https://github.com/MrLongNight/MapFlow",
            );

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
