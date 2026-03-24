use super::super::{state::ModuleCanvas, utils};
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use vorce_core::module::MapFlowModule;

pub fn draw_search_popup(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    canvas_rect: Rect,
    module: &mut MapFlowModule,
) {
    let popup_width = 300.0;
    let popup_height = 200.0;
    let popup_rect = Rect::from_min_size(
        Pos2::new(
            canvas_rect.center().x - popup_width / 2.0,
            canvas_rect.min.y + 50.0,
        ),
        Vec2::new(popup_width, popup_height),
    );

    let painter = ui.painter();
    painter.rect_filled(
        popup_rect,
        0.0,
        Color32::from_rgba_unmultiplied(30, 30, 40, 240),
    );
    painter.rect_stroke(
        popup_rect,
        0.0,
        Stroke::new(2.0, Color32::from_rgb(80, 120, 200)),
        egui::StrokeKind::Middle,
    );

    let inner_rect = popup_rect.shrink(10.0);
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut canvas.search_filter);
            });
            ui.add_space(8.0);

            let filter_lower = canvas.search_filter.to_lowercase();
            let matching_parts: Vec<_> = module
                .parts
                .iter()
                .filter(|p| {
                    if filter_lower.is_empty() {
                        return true;
                    }
                    let name = utils::get_part_property_text(&p.part_type).to_lowercase();
                    let (_, _, _, type_name) = utils::get_part_style(&p.part_type);
                    name.contains(&filter_lower) || type_name.to_lowercase().contains(&filter_lower)
                })
                .take(6)
                .collect();

            egui::ScrollArea::vertical()
                .max_height(120.0)
                .show(ui, |ui| {
                    for part in matching_parts {
                        let (_, _, icon, type_name) = utils::get_part_style(&part.part_type);
                        let label = format!(
                            "{} {} - {}",
                            icon,
                            type_name,
                            utils::get_part_property_text(&part.part_type)
                        );
                        if ui
                            .selectable_label(canvas.selected_parts.contains(&part.id), &label)
                            .clicked()
                        {
                            canvas.selected_parts.clear();
                            canvas.selected_parts.push(part.id);
                            canvas.pan_offset =
                                Vec2::new(-part.position.0 + 200.0, -part.position.1 + 150.0);
                            canvas.show_search = false;
                        }
                    }
                });
        });
    });
}
