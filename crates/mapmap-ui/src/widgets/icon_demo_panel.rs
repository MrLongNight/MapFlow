//! Icon Demo Panel
//!
//! Shows all available Ultimate Colors icons for preview.

use crate::core::responsive::ResponsiveLayout;
use crate::i18n::LocaleManager;
use crate::icons::{AppIcon, IconManager};

/// Panel to display all available icons
pub struct IconDemoPanel {
    /// Whether the panel is visible
    pub visible: bool,
    /// Icon display size
    pub icon_size: f32,
}

impl Default for IconDemoPanel {
    fn default() -> Self {
        Self {
            visible: false,
            icon_size: 48.0,
        }
    }
}

impl IconDemoPanel {
    /// Create a new icon demo panel
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the icon demo panel
    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        icons: Option<&IconManager>,
        _locale: &LocaleManager,
    ) {
        if !self.visible {
            return;
        }

        let layout = ResponsiveLayout::new(ctx);
        let window_size = layout.window_size(800.0, 600.0);

        egui::Window::new("🎨 Icon Gallery - Ultimate Colors")
            .default_size(window_size)
            .resizable(true)
            .scroll([false, true])
            .open(&mut self.visible)
            .scroll([false, true])
            .show(ctx, |ui| {
                ui.heading("Ultimate Colors - Free Icons");
                ui.separator();

                // Icon size slider
                ui.horizontal(|ui| {
                    ui.label("Icon Size:");
                    ui.add(egui::Slider::new(&mut self.icon_size, 24.0..=128.0).suffix("px"));
                });

                ui.separator();

                if let Some(icon_manager) = icons {
                    // Display icons in a grid
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let available_width = ui.available_width();
                        // Adjust icon size for compact layouts if not overridden by slider
                        let display_icon_size = if layout.is_compact() {
                            self.icon_size * 0.75
                        } else {
                            self.icon_size
                        };

                        let cols = ((available_width - 20.0) / (display_icon_size + 80.0))
                            .floor()
                            .max(1.0) as usize;

                        egui::Grid::new("icon_grid")
                            .spacing([16.0, 16.0])
                            .show(ui, |ui| {
                                for (i, icon) in AppIcon::all().iter().enumerate() {
                                    ui.vertical(|ui| {
                                        ui.set_width(display_icon_size + 60.0);

                                        // Icon background
                                        egui::Frame::default()
                                            .fill(egui::Color32::from_rgb(30, 35, 45))
                                            .rounding(8.0)
                                            .inner_margin(12.0)
                                            .show(ui, |ui| {
                                                ui.centered_and_justified(|ui| {
                                                    if let Some(img) =
                                                        icon_manager.image(*icon, display_icon_size)
                                                    {
                                                        ui.add(img);
                                                    } else {
                                                        ui.label("❌");
                                                    }
                                                });
                                            });

                                        // Icon name
                                        ui.label(
                                            egui::RichText::new(format!("{:?}", icon))
                                                .size(10.0)
                                                .color(egui::Color32::GRAY),
                                        );
                                    });

                                    if (i + 1) % cols == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    });
                } else {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        "⚠ Icons not loaded. Make sure assets/icons folder exists.",
                    );

                    ui.separator();
                    ui.label("Expected icon files:");
                    for icon in AppIcon::all() {
                        ui.label(format!("  • {}", icon.file_name()));
                    }
                }
            });
    }
}
