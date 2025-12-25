use crate::i18n::LocaleManager;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::module::{
    MapFlowModule, ModuleManager, ModulePart, ModulePartId, ModuleSocketType,
};

/// Information about a socket position for hit detection
#[derive(Clone)]
struct SocketInfo {
    part_id: ModulePartId,
    socket_idx: usize,
    is_output: bool,
    socket_type: ModuleSocketType,
    position: Pos2,
}

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
    /// Connection being created: (from_part, from_socket_idx, is_output, socket_type, start_pos)
    creating_connection: Option<(ModulePartId, usize, bool, ModuleSocketType, Pos2)>,
    /// Part ID pending deletion (set when X button clicked)
    pending_delete: Option<ModulePartId>,
}

impl Default for ModuleCanvas {
    fn default() -> Self {
        Self {
            active_module_id: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            dragging_part: None,
            creating_connection: None,
            pending_delete: None,
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
                // === SIGNAL FLOW ORDER: Trigger → Source → Mask → Modulator → Layer → Output ===

                if ui
                    .button("⚡ Trigger")
                    .on_hover_text("Add a Trigger node (Audio/MIDI/OSC/Keyboard)")
                    .clicked()
                {
                    if let Some(id) = self.active_module_id {
                        if let Some(module) = manager.get_module_mut(id) {
                            module.add_part(mapmap_core::module::PartType::Trigger, (100.0, 100.0));
                        }
                    }
                }

                if ui
                    .button("🎬 Source")
                    .on_hover_text("Add a Source node (Media/Shader/Live Input)")
                    .clicked()
                {
                    if let Some(id) = self.active_module_id {
                        if let Some(module) = manager.get_module_mut(id) {
                            module.add_part(mapmap_core::module::PartType::Source, (200.0, 100.0));
                        }
                    }
                }

                if ui
                    .button("🎭 Mask")
                    .on_hover_text("Add a Mask node (File/Shape/Gradient)")
                    .clicked()
                {
                    if let Some(id) = self.active_module_id {
                        if let Some(module) = manager.get_module_mut(id) {
                            module.add_part(mapmap_core::module::PartType::Mask, (300.0, 100.0));
                        }
                    }
                }

                if ui
                    .button("〰️ Modulator")
                    .on_hover_text("Add a Modulator/Effect node")
                    .clicked()
                {
                    if let Some(id) = self.active_module_id {
                        if let Some(module) = manager.get_module_mut(id) {
                            module
                                .add_part(mapmap_core::module::PartType::Modulator, (400.0, 100.0));
                        }
                    }
                }

                if ui
                    .button("📑 Layer")
                    .on_hover_text("Add a Layer node (Mapping/Mesh)")
                    .clicked()
                {
                    if let Some(id) = self.active_module_id {
                        if let Some(module) = manager.get_module_mut(id) {
                            module.add_part(mapmap_core::module::PartType::Layer, (500.0, 100.0));
                        }
                    }
                }

                if ui
                    .button("📺 Output")
                    .on_hover_text("Add an Output node (Projector/Preview)")
                    .clicked()
                {
                    if let Some(id) = self.active_module_id {
                        if let Some(module) = manager.get_module_mut(id) {
                            module.add_part(mapmap_core::module::PartType::Output, (600.0, 100.0));
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

            ui.separator();

            // === ZOOM CONTROLS ===
            ui.label("Zoom:");

            // Zoom out button
            if ui.button("−").on_hover_text("Zoom out").clicked() {
                self.zoom = (self.zoom - 0.1).clamp(0.2, 3.0);
            }

            // Zoom slider
            ui.add(
                egui::Slider::new(&mut self.zoom, 0.2..=3.0)
                    .show_value(false)
                    .clamp_to_range(true),
            );

            // Zoom in button
            if ui.button("+").on_hover_text("Zoom in").clicked() {
                self.zoom = (self.zoom + 0.1).clamp(0.2, 3.0);
            }

            // Zoom percentage display
            ui.label(format!("{:.0}%", self.zoom * 100.0));

            // Fit to view button
            if ui
                .button("⊡")
                .on_hover_text("Fit to view / Reset zoom")
                .clicked()
            {
                self.zoom = 1.0;
                self.pan_offset = Vec2::ZERO;
            }

            // === MODULE MANAGEMENT (only when module selected) ===
            if let Some(module_id) = self.active_module_id {
                ui.separator();

                // Get module for editing
                if let Some(module) = manager.get_module_mut(module_id) {
                    // Module name editor
                    ui.label("Name:");
                    let mut name = module.name.clone();
                    let name_response = ui.add(
                        egui::TextEdit::singleline(&mut name)
                            .desired_width(100.0)
                            .hint_text("Module name"),
                    );
                    if name_response.changed() {
                        module.name = name;
                    }

                    // Color picker button (shows current color)
                    let color = Color32::from_rgba_unmultiplied(
                        (module.color[0] * 255.0) as u8,
                        (module.color[1] * 255.0) as u8,
                        (module.color[2] * 255.0) as u8,
                        (module.color[3] * 255.0) as u8,
                    );
                    let color_btn = ui
                        .add(
                            egui::Button::new("🎨")
                                .fill(color)
                                .min_size(Vec2::splat(20.0)),
                        )
                        .on_hover_text("Module timeline color");

                    if color_btn.clicked() {
                        // Cycle through preset colors
                        let presets = [
                            [0.8, 0.3, 0.3, 1.0], // Red
                            [0.3, 0.8, 0.3, 1.0], // Green
                            [0.3, 0.3, 0.8, 1.0], // Blue
                            [0.8, 0.8, 0.3, 1.0], // Yellow
                            [0.8, 0.3, 0.8, 1.0], // Magenta
                            [0.3, 0.8, 0.8, 1.0], // Cyan
                            [0.8, 0.5, 0.2, 1.0], // Orange
                        ];
                        let current_idx =
                            presets.iter().position(|c| *c == module.color).unwrap_or(0);
                        module.color = presets[(current_idx + 1) % presets.len()];
                    }
                }

                // Delete module button
                if ui.button("🗑").on_hover_text("Delete this module").clicked() {
                    manager.delete_module(module_id);
                    self.active_module_id = None;
                }
            }

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

    fn render_canvas(&mut self, ui: &mut Ui, module: &mut MapFlowModule, _locale: &LocaleManager) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let canvas_rect = response.rect;

        // Handle canvas pan (only when not dragging a part)
        if response.dragged() && self.dragging_part.is_none() {
            self.pan_offset += response.drag_delta();
        }

        // Handle zoom
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                self.zoom *= 1.0 + scroll * 0.001;
                self.zoom = self.zoom.clamp(0.2, 3.0);
            }
        }

        let to_screen =
            |pos: Pos2| -> Pos2 { canvas_rect.min + (pos.to_vec2() + self.pan_offset) * self.zoom };

        let _from_screen = |screen_pos: Pos2| -> Pos2 {
            let v = (screen_pos - canvas_rect.min) / self.zoom - self.pan_offset;
            Pos2::new(v.x, v.y)
        };

        // Draw grid
        self.draw_grid(&painter, canvas_rect);

        // Draw connections first (behind nodes)
        self.draw_connections(&painter, module, &to_screen);

        // Collect socket positions for hit detection
        let mut all_sockets: Vec<SocketInfo> = Vec::new();

        // Collect part info and socket positions
        let part_rects: Vec<_> = module
            .parts
            .iter()
            .map(|part| {
                let part_screen_pos = to_screen(Pos2::new(part.position.0, part.position.1));
                let part_height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                let part_size = Vec2::new(180.0, part_height);
                let rect = Rect::from_min_size(part_screen_pos, part_size * self.zoom);

                // Calculate socket positions
                let title_height = 28.0 * self.zoom;
                let socket_start_y = rect.min.y + title_height + 10.0 * self.zoom;

                // Input sockets (left side)
                for (i, socket) in part.inputs.iter().enumerate() {
                    let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
                    all_sockets.push(SocketInfo {
                        part_id: part.id,
                        socket_idx: i,
                        is_output: false,
                        socket_type: socket.socket_type.clone(),
                        position: Pos2::new(rect.min.x, socket_y),
                    });
                }

                // Output sockets (right side)
                for (i, socket) in part.outputs.iter().enumerate() {
                    let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
                    all_sockets.push(SocketInfo {
                        part_id: part.id,
                        socket_idx: i,
                        is_output: true,
                        socket_type: socket.socket_type.clone(),
                        position: Pos2::new(rect.max.x, socket_y),
                    });
                }

                (part.id, rect)
            })
            .collect();

        // Handle socket clicks for creating connections
        let socket_radius = 8.0 * self.zoom;
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let clicked = ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary));
        let released = ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary));

        if let Some(pos) = pointer_pos {
            // Check if clicking on a socket
            if clicked {
                for socket in &all_sockets {
                    if socket.position.distance(pos) < socket_radius {
                        // Start creating a connection
                        self.creating_connection = Some((
                            socket.part_id,
                            socket.socket_idx,
                            socket.is_output,
                            socket.socket_type.clone(),
                            socket.position,
                        ));
                        break;
                    }
                }
            }

            // Check if releasing on a compatible socket
            if released && self.creating_connection.is_some() {
                if let Some((from_part, from_socket, from_is_output, ref from_type, _)) =
                    self.creating_connection.clone()
                {
                    for socket in &all_sockets {
                        if socket.position.distance(pos) < socket_radius {
                            // Validate connection: must be different parts, opposite directions, same type
                            if socket.part_id != from_part
                                && socket.is_output != from_is_output
                                && socket.socket_type == *from_type
                            {
                                // Create connection (from output to input)
                                if from_is_output {
                                    module.add_connection(
                                        from_part,
                                        from_socket,
                                        socket.part_id,
                                        socket.socket_idx,
                                    );
                                } else {
                                    module.add_connection(
                                        socket.part_id,
                                        socket.socket_idx,
                                        from_part,
                                        from_socket,
                                    );
                                }
                            }
                            break;
                        }
                    }
                }
                self.creating_connection = None;
            }
        }

        // Clear connection if mouse released without hitting a socket
        if released && self.creating_connection.is_some() {
            self.creating_connection = None;
        }

        // Handle part dragging and delete buttons
        let mut delete_part_id: Option<ModulePartId> = None;

        for (part_id, rect) in &part_rects {
            let part_response =
                ui.interact(*rect, egui::Id::new(*part_id), Sense::click_and_drag());

            if part_response.drag_started() && self.creating_connection.is_none() {
                self.dragging_part = Some((*part_id, Vec2::ZERO));
            }

            if part_response.dragged() {
                if let Some((dragged_id, _)) = self.dragging_part {
                    if dragged_id == *part_id {
                        let delta = part_response.drag_delta() / self.zoom;
                        if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                            part.position.0 += delta.x;
                            part.position.1 += delta.y;
                        }
                    }
                }
            }

            if part_response.drag_stopped() {
                self.dragging_part = None;
            }

            // Check for delete button click (× in top-right corner of title bar)
            let delete_button_rect = Rect::from_min_size(
                Pos2::new(rect.max.x - 20.0 * self.zoom, rect.min.y),
                Vec2::splat(20.0 * self.zoom),
            );
            let delete_response = ui.interact(
                delete_button_rect,
                egui::Id::new((*part_id, "delete")),
                Sense::click(),
            );
            if delete_response.clicked() {
                delete_part_id = Some(*part_id);
            }
        }

        // Process pending deletion
        if let Some(part_id) = delete_part_id {
            // Remove all connections involving this part
            module
                .connections
                .retain(|c| c.from_part != part_id && c.to_part != part_id);
            // Remove the part
            module.parts.retain(|p| p.id != part_id);
        }

        // Draw parts (nodes) with delete buttons
        for part in &module.parts {
            let part_screen_pos = to_screen(Pos2::new(part.position.0, part.position.1));
            let part_height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
            let part_size = Vec2::new(180.0, part_height);
            let part_screen_rect = Rect::from_min_size(part_screen_pos, part_size * self.zoom);

            self.draw_part_with_delete(&painter, part, part_screen_rect);
        }

        // Draw connection being created
        if let Some((_from_part_id, _from_socket_idx, _is_output, ref socket_type, start_pos)) =
            self.creating_connection
        {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                let color = Self::get_socket_color(&socket_type);
                painter.line_segment([start_pos, pointer_pos], Stroke::new(3.0, color));
            }
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

    fn draw_connections<F>(&self, painter: &egui::Painter, module: &MapFlowModule, to_screen: &F)
    where
        F: Fn(Pos2) -> Pos2,
    {
        for conn in &module.connections {
            // Find source and target parts
            let from_part = module.parts.iter().find(|p| p.id == conn.from_part);
            let to_part = module.parts.iter().find(|p| p.id == conn.to_part);

            if let (Some(from), Some(to)) = (from_part, to_part) {
                // Calculate socket positions
                let from_pos = to_screen(Pos2::new(from.position.0, from.position.1));
                let to_pos = to_screen(Pos2::new(to.position.0, to.position.1));

                let _from_height = 80.0 + (from.inputs.len().max(from.outputs.len()) as f32) * 20.0;
                let socket_y_offset = 50.0 + conn.from_socket as f32 * 20.0;
                let from_socket_pos = Pos2::new(
                    from_pos.x + 180.0 * self.zoom, // Right side
                    from_pos.y + socket_y_offset * self.zoom,
                );

                let to_socket_y_offset = 50.0 + conn.to_socket as f32 * 20.0;
                let to_socket_pos = Pos2::new(
                    to_pos.x, // Left side
                    to_pos.y + to_socket_y_offset * self.zoom,
                );

                // Draw bezier curve
                let control_offset = (to_socket_pos.x - from_socket_pos.x).abs() * 0.4;
                let ctrl1 = Pos2::new(from_socket_pos.x + control_offset, from_socket_pos.y);
                let ctrl2 = Pos2::new(to_socket_pos.x - control_offset, to_socket_pos.y);

                // Draw as line segments (approximating bezier)
                let steps = 20;
                for i in 0..steps {
                    let t1 = i as f32 / steps as f32;
                    let t2 = (i + 1) as f32 / steps as f32;
                    let p1 = Self::bezier_point(from_socket_pos, ctrl1, ctrl2, to_socket_pos, t1);
                    let p2 = Self::bezier_point(from_socket_pos, ctrl1, ctrl2, to_socket_pos, t2);
                    painter
                        .line_segment([p1, p2], Stroke::new(2.0, Color32::from_rgb(100, 180, 255)));
                }
            }
        }
    }

    fn bezier_point(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        Pos2::new(
            uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x,
            uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y,
        )
    }

    fn draw_part_with_delete(&self, painter: &egui::Painter, part: &ModulePart, rect: Rect) {
        // Get part color and name based on type
        let (bg_color, title_color, icon, name) = Self::get_part_style(&part.part_type);

        // Draw background
        painter.rect_filled(rect, 6.0 * self.zoom, bg_color);
        painter.rect_stroke(
            rect,
            6.0 * self.zoom,
            Stroke::new(2.0, Color32::from_rgb(80, 80, 90)),
        );

        // Title bar
        let title_height = 28.0 * self.zoom;
        let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), title_height));
        painter.rect_filled(
            title_rect,
            egui::Rounding {
                nw: 6.0 * self.zoom,
                ne: 6.0 * self.zoom,
                sw: 0.0,
                se: 0.0,
            },
            title_color,
        );

        // Title text with icon (offset slightly left to make room for × button)
        let title_text = format!("{} {}", icon, name);
        painter.text(
            Pos2::new(
                title_rect.center().x - 8.0 * self.zoom,
                title_rect.center().y,
            ),
            egui::Align2::CENTER_CENTER,
            title_text,
            egui::FontId::proportional(13.0 * self.zoom),
            Color32::WHITE,
        );

        // Delete button (× in top-right corner)
        let delete_button_pos = Pos2::new(
            rect.max.x - 12.0 * self.zoom,
            rect.min.y + title_height * 0.5,
        );
        painter.text(
            delete_button_pos,
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::proportional(16.0 * self.zoom),
            Color32::from_rgba_unmultiplied(255, 100, 100, 200),
        );

        // Draw input sockets (left side)
        let socket_start_y = rect.min.y + title_height + 10.0 * self.zoom;
        for (i, socket) in part.inputs.iter().enumerate() {
            let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
            let socket_pos = Pos2::new(rect.min.x, socket_y);
            let socket_radius = 6.0 * self.zoom;

            // Socket circle
            let socket_color = Self::get_socket_color(&socket.socket_type);
            painter.circle_filled(socket_pos, socket_radius, socket_color);
            painter.circle_stroke(socket_pos, socket_radius, Stroke::new(1.5, Color32::WHITE));

            // Socket label
            painter.text(
                Pos2::new(rect.min.x + 12.0 * self.zoom, socket_y),
                egui::Align2::LEFT_CENTER,
                &socket.name,
                egui::FontId::proportional(10.0 * self.zoom),
                Color32::from_gray(200),
            );
        }

        // Draw output sockets (right side)
        for (i, socket) in part.outputs.iter().enumerate() {
            let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
            let socket_pos = Pos2::new(rect.max.x, socket_y);
            let socket_radius = 6.0 * self.zoom;

            // Socket circle
            let socket_color = Self::get_socket_color(&socket.socket_type);
            painter.circle_filled(socket_pos, socket_radius, socket_color);
            painter.circle_stroke(socket_pos, socket_radius, Stroke::new(1.5, Color32::WHITE));

            // Socket label
            painter.text(
                Pos2::new(rect.max.x - 12.0 * self.zoom, socket_y),
                egui::Align2::RIGHT_CENTER,
                &socket.name,
                egui::FontId::proportional(10.0 * self.zoom),
                Color32::from_gray(200),
            );
        }
    }

    fn get_part_style(
        part_type: &mapmap_core::module::ModulePartType,
    ) -> (Color32, Color32, &'static str, &'static str) {
        use mapmap_core::module::ModulePartType;
        match part_type {
            ModulePartType::Trigger(_) => (
                Color32::from_rgb(60, 50, 70),   // bg
                Color32::from_rgb(130, 80, 180), // title
                "⚡",
                "Trigger",
            ),
            ModulePartType::Source(_) => (
                Color32::from_rgb(50, 60, 70),
                Color32::from_rgb(80, 140, 180),
                "🎬",
                "Source",
            ),
            ModulePartType::Mask(_) => (
                Color32::from_rgb(60, 55, 70),
                Color32::from_rgb(160, 100, 180),
                "🎭",
                "Mask",
            ),
            ModulePartType::Modulizer(_) => (
                Color32::from_rgb(60, 60, 50),
                Color32::from_rgb(180, 140, 60),
                "〰️",
                "Modulator",
            ),
            ModulePartType::LayerAssignment(_) => (
                Color32::from_rgb(50, 70, 60),
                Color32::from_rgb(80, 180, 120),
                "📑",
                "Layer",
            ),
            ModulePartType::Output(_) => (
                Color32::from_rgb(70, 50, 50),
                Color32::from_rgb(180, 80, 80),
                "📺",
                "Output",
            ),
        }
    }

    fn get_socket_color(socket_type: &mapmap_core::module::ModuleSocketType) -> Color32 {
        use mapmap_core::module::ModuleSocketType;
        match socket_type {
            ModuleSocketType::Trigger => Color32::from_rgb(180, 100, 220),
            ModuleSocketType::Media => Color32::from_rgb(100, 180, 220),
            ModuleSocketType::Effect => Color32::from_rgb(220, 180, 100),
            ModuleSocketType::Layer => Color32::from_rgb(100, 220, 140),
            ModuleSocketType::Output => Color32::from_rgb(220, 100, 100),
        }
    }
}
