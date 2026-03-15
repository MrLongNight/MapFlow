use super::super::{state::ModuleCanvas, utils};
use egui::{Rect, Ui};
use mapmap_core::module::ModuleManager;

pub fn draw_quick_create_popup(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    _canvas_rect: Rect,
    manager: &mut ModuleManager,
    active_module_id: Option<u64>,
) {
    if !canvas.show_quick_create {
        return;
    }
    let popup_pos = canvas.quick_create_pos;
    let catalog = utils::build_node_catalog();
    let filter_lower = canvas.quick_create_filter.to_lowercase();
    let filtered_items: Vec<&utils::NodeCatalogItem> = catalog
        .iter()
        .filter(|item| {
            if filter_lower.is_empty() {
                true
            } else {
                item.label.to_lowercase().contains(&filter_lower)
                    || item.search_tags.contains(&filter_lower)
            }
        })
        .collect();
    if filtered_items.is_empty() {
        canvas.quick_create_selected_index = 0;
    } else if canvas.quick_create_selected_index >= filtered_items.len() {
        canvas.quick_create_selected_index = filtered_items.len() - 1;
    }
    let mut commit_creation = false;
    let mut close_popup = false;
    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        if !filtered_items.is_empty() {
            canvas.quick_create_selected_index =
                (canvas.quick_create_selected_index + 1) % filtered_items.len();
        }
    } else if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        if !filtered_items.is_empty() {
            if canvas.quick_create_selected_index == 0 {
                canvas.quick_create_selected_index = filtered_items.len() - 1;
            } else {
                canvas.quick_create_selected_index -= 1;
            }
        }
    } else if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        commit_creation = true;
    } else if ui.input(|i| i.key_pressed(egui::Key::Escape))
        || ui.input(|i| i.key_pressed(egui::Key::Tab))
    {
        close_popup = true;
    }
    let area = egui::Area::new("quick_create_popup".into())
        .fixed_pos(popup_pos)
        .order(egui::Order::Foreground)
        .constrain(true);
    area.show(ui.ctx(), |ui| {
        egui::Frame::menu(ui.style()).show(ui, |ui| {
            ui.set_width(250.0);
            let response = ui.add(
                egui::TextEdit::singleline(&mut canvas.quick_create_filter)
                    .hint_text("Type to create...")
                    .lock_focus(true),
            );
            if canvas.show_quick_create && response.changed() {
                response.request_focus();
            }
            if canvas.show_quick_create && !response.has_focus() {
                response.request_focus();
            }
            ui.separator();
            if filtered_items.is_empty() {
                ui.label(
                    egui::RichText::new("No matching nodes found.")
                        .weak()
                        .italics(),
                );
            } else {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for (i, item) in filtered_items.iter().enumerate() {
                            let is_selected = i == canvas.quick_create_selected_index;
                            let (_, _, icon, _) = utils::get_part_style(&item.part_type);
                            let label_text = format!("{} {}", icon, item.label);
                            let response = ui.selectable_label(is_selected, label_text);
                            if response.clicked() {
                                canvas.quick_create_selected_index = i;
                                commit_creation = true;
                            }
                            if is_selected {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                        }
                    });
            }
        });
    });
    if commit_creation {
        if let Some(item) = filtered_items.get(canvas.quick_create_selected_index) {
            if let Some(module_id) = active_module_id {
                if let Some(module) = manager.get_module_mut(module_id) {
                    let canvas_min = _canvas_rect.min.to_vec2();
                    let pos_screen = canvas.quick_create_pos;
                    let pan = canvas.pan_offset;
                    let zoom = canvas.zoom;
                    let x = (pos_screen.x - pan.x - canvas_min.x) / zoom;
                    let y = (pos_screen.y - pan.y - canvas_min.y) / zoom;
                    let final_pos = utils::find_free_position(&module.parts, (x, y));
                    module.add_part_with_type(item.part_type.clone(), final_pos);
                }
            }
        }
        close_popup = true;
    }
    if close_popup {
        canvas.show_quick_create = false;
        canvas.quick_create_filter.clear();
    }
}
