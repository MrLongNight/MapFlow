use crate::i18n::LocaleManager;
use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2};
use mapmap_core::module::ModuleManager;

#[derive(Default)]
pub struct ModuleSidebar {
    renaming_id: Option<mapmap_core::module::ModuleId>,
    rename_buffer: String,
    should_focus_rename: bool,
}

impl ModuleSidebar {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        manager: &mut ModuleManager,
        locale: &LocaleManager,
    ) -> Option<ModuleSidebarAction> {
        let mut action = None;

        ui.vertical(|ui| {
            ui.heading(locale.t("panel-modules"));
            ui.separator();

            // Button to add a new module
            if ui.button(locale.t("btn-add-module")).clicked() {
                action = Some(ModuleSidebarAction::AddModule);
            }
            ui.separator();

            // Collect data to decouple from manager borrow
            let modules: Vec<(mapmap_core::module::ModuleId, String, [f32; 4])> = manager
                .list_modules()
                .iter()
                .map(|m| (m.id, m.name.clone(), m.color))
                .collect();

            // List of modules
            for (id, name, color_arr) in modules {
                if self.renaming_id == Some(id) {
                    // Inline renaming
                    ui.horizontal(|ui| {
                        let response = ui.text_edit_singleline(&mut self.rename_buffer);
                        if self.should_focus_rename {
                            response.request_focus();
                            self.should_focus_rename = false;
                        }

                        if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if !self.rename_buffer.trim().is_empty() {
                                manager.rename_module(id, self.rename_buffer.clone());
                            }
                            self.renaming_id = None;
                        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.renaming_id = None;
                        }
                    });
                } else {
                    let response = self.module_list_item(ui, &name, color_arr);
                    response.context_menu(|ui| {
                        if ui.button(locale.t("menu-rename")).clicked() {
                            self.renaming_id = Some(id);
                            self.rename_buffer = name.clone();
                            self.should_focus_rename = true;
                            ui.close();
                        }
                        if ui.button(locale.t("menu-duplicate")).clicked() {
                            manager.duplicate_module(id);
                            ui.close();
                        }
                        if ui.button(locale.t("menu-delete")).clicked() {
                            action = Some(ModuleSidebarAction::DeleteModule(id));
                            ui.close();
                        }
                        ui.separator();
                        // Color picker
                        ui.label("Change Color");
                        let color_palette: Vec<[f32; 4]> = vec![
                            [1.0, 0.2, 0.2, 1.0],
                            [1.0, 0.5, 0.2, 1.0],
                            [1.0, 1.0, 0.2, 1.0],
                            [0.5, 1.0, 0.2, 1.0],
                            [0.2, 1.0, 0.2, 1.0],
                            [0.2, 1.0, 0.5, 1.0],
                            [0.2, 1.0, 1.0, 1.0],
                            [0.2, 0.5, 1.0, 1.0],
                            [0.2, 0.2, 1.0, 1.0],
                            [0.5, 0.2, 1.0, 1.0],
                            [1.0, 0.2, 1.0, 1.0],
                            [1.0, 0.2, 0.5, 1.0],
                            [0.5, 0.5, 0.5, 1.0],
                            [1.0, 0.5, 0.8, 1.0],
                            [0.5, 1.0, 0.8, 1.0],
                            [0.8, 0.5, 1.0, 1.0],
                        ];
                        ui.horizontal_wrapped(|ui| {
                            for color in color_palette {
                                let color32 = Color32::from_rgba_premultiplied(
                                    (color[0] * 255.0) as u8,
                                    (color[1] * 255.0) as u8,
                                    (color[2] * 255.0) as u8,
                                    (color[3] * 255.0) as u8,
                                );
                                if color_button(ui, color32, Vec2::splat(16.0)).clicked() {
                                    action = Some(ModuleSidebarAction::SetColor(id, color));
                                    ui.close();
                                }
                            }
                        });
                    });
                }
            }
        });

        action
    }

    fn module_list_item(&self, ui: &mut Ui, name: &str, color_arr: [f32; 4]) -> Response {
        let item_size = Vec2::new(ui.available_width(), 24.0);
        let (rect, response) = ui.allocate_exact_size(item_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let color = Color32::from_rgba_premultiplied(
                (color_arr[0] * 255.0) as u8,
                (color_arr[1] * 255.0) as u8,
                (color_arr[2] * 255.0) as u8,
                (color_arr[3] * 255.0) as u8,
            );

            let icon_rect = Rect::from_min_size(rect.min, Vec2::new(rect.height(), rect.height()));
            ui.painter().rect_filled(icon_rect.expand(-4.0), 4.0, color);

            let label_rect =
                Rect::from_min_max(Pos2::new(icon_rect.max.x + 5.0, rect.min.y), rect.max);
            ui.painter().text(
                label_rect.left_center(),
                egui::Align2::LEFT_CENTER,
                name,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
        }

        response
    }
}

fn color_button(ui: &mut Ui, color: Color32, size: Vec2) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    ui.painter().rect_filled(rect, 4.0, color);
    response
}

pub enum ModuleSidebarAction {
    AddModule,
    DeleteModule(u64),
    SetColor(u64, [f32; 4]),
    // Other actions like Rename, Duplicate etc.
}
