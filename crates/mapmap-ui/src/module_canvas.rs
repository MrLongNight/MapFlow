use crate::i18n::LocaleManager;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use mapmap_core::module::{MapFlowModule, ModuleManager, ModulePart, ModulePartId};

#[allow(dead_code)]
pub struct ModuleCanvas {
    /// The ID of the currently active/edited module
    active_module_id: Option<u64>,
    /// Canvas pan offset
    pan_offset: Vec2,
    /// Canvas zoom level
    zoom: f32,
    /// Part being dragged
    dragging_part: Option<(ModulePartId, Vec2)>,
    /// Connection being created
    creating_connection: Option<(ModulePartId, usize, Pos2)>,
}

impl Default for ModuleCanvas {
    fn default() -> Self {
        Self {
            active_module_id: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            dragging_part: None,
            creating_connection: None,
        }
    }
}

impl ModuleCanvas {
    /// Set the active module ID
    pub fn set_active_module(&mut self, module_id: Option<u64>) {
        self.active_module_id = module_id;
    }

    /// Get the active module ID
    pub fn active_module_id(&self) -> Option<u64> {
        self.active_module_id
    }

    pub fn show(&mut self, ui: &mut Ui, manager: &mut ModuleManager, locale: &LocaleManager) {
        // === CANVAS TOOLBAR ===
        ui.horizontal(|ui| {
            ui.add_space(4.0);

            // Create New Module
            if ui
                .button("➕ New Module")
                .on_hover_text("Create a new module")
                .clicked()
            {
                let new_module_id = manager.create_module("New Module".to_string());
                self.active_module_id = Some(new_module_id);
            }

            ui.separator();

            // Part creation tools (only enabled when module is active)
            let has_module = self.active_module_id.is_some();

            ui.add_enabled_ui(has_module, |ui| {
                if ui
                    .button("⚡ Trigger")
                    .on_hover_text("Add a Trigger node")
                    .clicked()
                {
                    if let Some(id) = self.active_module_id {
                        if let Some(module) = manager.get_module_mut(id) {
                            module.add_part(mapmap_core::module::PartType::Trigger, (100.0, 100.0));
                        }
                    }
                }

                if ui
                    .button("〰️ Modulator")
                    .on_hover_text("Add a Modulator node")
                    .clicked()
                {
                    if let Some(id) = self.active_module_id {
                        if let Some(module) = manager.get_module_mut(id) {
                            module
                                .add_part(mapmap_core::module::PartType::Modulator, (200.0, 100.0));
                        }
                    }
                }

                if ui
                    .button("📑 Layer")
                    .on_hover_text("Add a Layer node")
                    .clicked()
                {
                    if let Some(id) = self.active_module_id {
                        if let Some(module) = manager.get_module_mut(id) {
                            module.add_part(mapmap_core::module::PartType::Layer, (300.0, 100.0));
                        }
                    }
                }

                if ui
                    .button("📺 Output")
                    .on_hover_text("Add an Output node")
                    .clicked()
                {
                    if let Some(id) = self.active_module_id {
                        if let Some(module) = manager.get_module_mut(id) {
                            module.add_part(mapmap_core::module::PartType::Output, (400.0, 100.0));
                        }
                    }
                }
            });

            ui.separator();

            // Module selector dropdown
            ui.label("Module:");
            let module_names: Vec<_> = manager
                .list_modules()
                .iter()
                .map(|m| (m.id, m.name.clone()))
                .collect();
            let current_name = self
                .active_module_id
                .and_then(|id| manager.get_module_mut(id))
                .map(|m| m.name.clone())
                .unwrap_or_else(|| "None".to_string());

            egui::ComboBox::from_id_source("module_selector")
                .selected_text(current_name)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.active_module_id, None, "None");
                    for (id, name) in module_names {
                        ui.selectable_value(&mut self.active_module_id, Some(id), name);
                    }
                });

            ui.add_space(4.0);
        });

        ui.separator();

        // Find the active module
        let active_module = self
            .active_module_id
            .and_then(|id| manager.get_module_mut(id));

        if let Some(module) = active_module {
            self.render_canvas(ui, module, locale);
        } else {
            // Show a message if no module is selected
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("🔧 Module Canvas");
                    ui.add_space(10.0);
                    ui.label("Click '➕ New Module' to create a module.");
                    ui.label("Or select an existing module from the dropdown above.");
                });
            });
        }
    }

    fn render_canvas(&mut self, ui: &mut Ui, module: &mut MapFlowModule, locale: &LocaleManager) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        // Handle canvas interactions (pan, zoom)
        if response.dragged() {
            self.pan_offset += response.drag_delta();
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                self.zoom *= 1.0 + scroll * 0.001;
                self.zoom = self.zoom.clamp(0.2, 3.0);
            }
        }

        let to_screen = |pos: Pos2| -> Pos2 {
            response.rect.min + (pos.to_vec2() + self.pan_offset) * self.zoom
        };

        // Draw grid
        self.draw_grid(&painter, response.rect);

        // TODO: Draw connections

        // Draw parts (nodes)
        for part in &module.parts {
            let part_screen_pos = to_screen(Pos2::new(part.position.0, part.position.1));
            let part_size = Vec2::new(180.0, 120.0); // Fixed size for now
            let part_screen_rect = Rect::from_min_size(part_screen_pos, part_size * self.zoom);

            self.draw_part(ui, &painter, part, part_screen_rect, locale);
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_size = 20.0 * self.zoom;
        let color = Color32::from_rgb(40, 40, 40);
        let mut x = rect.left() - self.pan_offset.x % grid_size;
        while x < rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, color),
            );
            x += grid_size;
        }
        let mut y = rect.top() - self.pan_offset.y % grid_size;
        while y < rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, color),
            );
            y += grid_size;
        }
    }

    fn draw_part(
        &self,
        ui: &Ui,
        painter: &egui::Painter,
        part: &ModulePart,
        rect: Rect,
        _locale: &LocaleManager,
    ) -> Response {
        let response = ui.interact(rect, egui::Id::new(part.id), Sense::click_and_drag());

        let bg_color = Color32::from_rgb(50, 50, 60);
        painter.rect_filled(rect, 4.0, bg_color);
        painter.rect_stroke(rect, 4.0, Stroke::new(2.0, Color32::from_rgb(80, 80, 80)));

        // Title bar
        let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 24.0 * self.zoom));
        painter.rect_filled(title_rect, 4.0, Color32::from_rgb(30, 30, 30));

        // TODO: Get proper name for part type
        let part_name = "Module Part";
        painter.text(
            title_rect.center(),
            egui::Align2::CENTER_CENTER,
            part_name,
            egui::FontId::proportional(14.0 * self.zoom),
            Color32::WHITE,
        );

        // TODO: Draw sockets

        response
    }
}
