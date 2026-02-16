use crate::i18n::LocaleManager;
use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2};
use mapmap_core::module::{MapFlowModule, ModuleManager};

#[derive(Default)]
pub struct ModuleSidebar {
    // Add state here if needed, e.g., for renaming a module
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

            // List of modules
            let modules = manager.list_modules();
            // Optimization: Iterate over references to avoid deep cloning MapFlowModule every frame
            for module in modules {
                let response = self.module_list_item(ui, module);
                response.context_menu(|ui| {
                    if ui.button(locale.t("menu-rename")).clicked() {
                        // TODO: Implement renaming
                        ui.close();
                    }
                    if ui.button(locale.t("menu-duplicate")).clicked() {
                        // TODO: Implement duplication
                        ui.close();
                    }
                    if ui.button(locale.t("menu-delete")).clicked() {
                        action = Some(ModuleSidebarAction::DeleteModule(module.id));
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
                                action = Some(ModuleSidebarAction::SetColor(module.id, color));
                                ui.close();
                            }
                        }
                    });
                });
            }
        });

        action
    }

    fn module_list_item(&self, ui: &mut Ui, module: &MapFlowModule) -> Response {
        let item_size = Vec2::new(ui.available_width(), 24.0);
        let (rect, response) = ui.allocate_exact_size(item_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let color = Color32::from_rgba_premultiplied(
                (module.color[0] * 255.0) as u8,
                (module.color[1] * 255.0) as u8,
                (module.color[2] * 255.0) as u8,
                (module.color[3] * 255.0) as u8,
            );

            let icon_rect = Rect::from_min_size(rect.min, Vec2::new(rect.height(), rect.height()));
            ui.painter().rect_filled(icon_rect.expand(-4.0), 4.0, color);

            let label_rect =
                Rect::from_min_max(Pos2::new(icon_rect.max.x + 5.0, rect.min.y), rect.max);
            ui.painter().text(
                label_rect.left_center(),
                egui::Align2::LEFT_CENTER,
                &module.name,
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


