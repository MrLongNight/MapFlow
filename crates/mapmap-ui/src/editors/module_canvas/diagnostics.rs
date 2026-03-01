use super::state::ModuleCanvas;
use egui::{Color32, Pos2, Stroke, Ui, Vec2};

pub fn render_diagnostics_popup(canvas: &mut ModuleCanvas, ui: &mut Ui) {
    if !canvas.show_diagnostics {
        return;
    }

    let popup_size = Vec2::new(350.0, 250.0);
    let available = ui.available_rect_before_wrap();
    let popup_pos = Pos2::new(
        (available.min.x + available.max.x - popup_size.x) / 2.0,
        (available.min.y + available.max.y - popup_size.y) / 2.0,
    );
    let popup_rect = egui::Rect::from_min_size(popup_pos, popup_size);

    // Background
    let painter = ui.painter();
    painter.rect_filled(
        popup_rect,
        0.0,
        Color32::from_rgba_unmultiplied(30, 35, 45, 245),
    );
    painter.rect_stroke(
        popup_rect,
        0.0,
        Stroke::new(2.0, Color32::from_rgb(180, 100, 80)),
        egui::StrokeKind::Middle,
    );

    let inner_rect = popup_rect.shrink(12.0);
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
        ui.vertical(|ui| {
            ui.heading(if canvas.diagnostic_issues.is_empty() {
                "âœ“ Module Check: OK"
            } else {
                "\u{26A0} Module Check: Issues Found"
            });
            ui.add_space(8.0);

            if canvas.diagnostic_issues.is_empty() {
                ui.label("No issues found. Your module looks good!");
            } else {
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for issue in &canvas.diagnostic_issues {
                            let (icon, color) = match issue.severity {
                                mapmap_core::diagnostics::IssueSeverity::Error => {
                                    ("❌", Color32::RED)
                                }
                                mapmap_core::diagnostics::IssueSeverity::Warning => {
                                    ("\u{26A0}", Color32::YELLOW)
                                }
                                mapmap_core::diagnostics::IssueSeverity::Info => {
                                    ("\u{2139}", Color32::LIGHT_BLUE)
                                }
                            };
                            ui.horizontal(|ui| {
                                ui.colored_label(color, icon);
                                ui.label(&issue.message);
                            });
                        }
                    });
            }

            ui.add_space(8.0);
            if ui.button("Close").clicked() {
                canvas.show_diagnostics = false;
            }
        });
    });
}
