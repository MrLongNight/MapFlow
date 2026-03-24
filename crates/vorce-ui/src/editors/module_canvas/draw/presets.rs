use super::super::state::ModuleCanvas;
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use vorce_core::module::MapFlowModule;

pub fn draw_presets_popup(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    canvas_rect: Rect,
    module: &mut MapFlowModule,
) {
    let popup_width = 280.0;
    let popup_height = 220.0;
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
        Color32::from_rgba_unmultiplied(30, 35, 45, 245),
    );
    painter.rect_stroke(
        popup_rect,
        0.0,
        Stroke::new(2.0, Color32::from_rgb(100, 180, 80)),
        egui::StrokeKind::Middle,
    );

    let inner_rect = popup_rect.shrink(12.0);
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
        ui.vertical(|ui| {
            ui.heading("📋 Presets / Templates");
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    let presets = canvas.presets.clone();
                    if presets.is_empty() {
                        crate::widgets::custom::render_info_label(ui, "No presets found.");
                    }
                    for preset in &presets {
                        ui.horizontal(|ui| {
                            if ui.button(&preset.name).clicked() {
                                module.parts.clear();
                                module.connections.clear();
                                module.next_part_id = 1;
                                let mut part_ids = Vec::new();
                                for (part_type, position, size) in &preset.parts {
                                    let id =
                                        module.add_part_with_type(part_type.clone(), *position);
                                    if let Some(part) =
                                        module.parts.iter_mut().find(|part| part.id == id)
                                    {
                                        part.size = *size;
                                    }
                                    part_ids.push(id);
                                }
                                for (from_idx, from_socket, to_idx, to_socket) in
                                    &preset.connections
                                {
                                    if *from_idx < part_ids.len() && *to_idx < part_ids.len() {
                                        let _ = module.connect_parts(
                                            part_ids[*from_idx],
                                            from_socket.clone(),
                                            part_ids[*to_idx],
                                            to_socket.clone(),
                                        );
                                    }
                                }
                                let _ = module.repair_graph();
                                canvas.show_presets = false;
                            }
                            ui.label(format!("({} nodes)", preset.parts.len()));
                        });
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut canvas.new_preset_name);
                if ui.button("💾 Save Current").clicked() && !canvas.new_preset_name.is_empty() {
                    let mut parts = Vec::new();
                    let mut id_map = std::collections::HashMap::new();

                    for (i, part) in module.parts.iter().enumerate() {
                        parts.push((part.part_type.clone(), part.position, part.size));
                        id_map.insert(part.id, i);
                    }

                    let mut connections = Vec::new();
                    for conn in &module.connections {
                        if let (Some(&from_idx), Some(&to_idx)) =
                            (id_map.get(&conn.from_part), id_map.get(&conn.to_part))
                        {
                            connections.push((
                                from_idx,
                                conn.from_socket.clone(),
                                to_idx,
                                conn.to_socket.clone(),
                            ));
                        }
                    }

                    canvas
                        .presets
                        .push(crate::editors::module_canvas::types::ModulePreset {
                            name: canvas.new_preset_name.clone(),
                            parts,
                            connections,
                        });
                    canvas.new_preset_name.clear();
                }
            });

            ui.add_space(8.0);
            if ui.button("Close").clicked() {
                canvas.show_presets = false;
            }
        });
    });
}
