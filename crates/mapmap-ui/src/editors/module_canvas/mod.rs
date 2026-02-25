use crate::i18n::LocaleManager;
use crate::theme::colors;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::{
    module::{
        MapFlowModule, ModuleId, ModuleManager, ModulePartId, ModulePartType, TriggerType,
    },
};

pub mod types;
pub mod state;
pub mod drawing;
pub mod nodes;
pub mod connections;
pub mod inspector;
pub mod popups;
pub mod mesh;
pub mod hue;

pub use self::types::*;
pub use self::state::ModuleCanvas;

use egui_node_editor::*;
use std::borrow::Cow;

impl NodeTemplateTrait for MyNodeTemplate {
    type NodeData = MyNodeData;
    type DataType = MyDataType;
    type ValueType = MyValueType;
    type UserState = MyUserState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        Cow::Borrowed(&self.label)
    }

    fn node_graph_label(&self, _user_state: &mut Self::UserState) -> String {
        self.label.clone()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        MyNodeData {
            title: self.label.clone(),
            part_type: mapmap_core::module::ModulePartType::Trigger(TriggerType::Beat), // Mock
            original_part_id: 0,
        }
    }

    fn build_node(
        &self,
        _graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        _node_id: NodeId,
    ) {
        // Mock
    }
}

#[cfg(feature = "ndi")]
use mapmap_io::ndi::NdiSource;
#[cfg(feature = "ndi")]
use std::sync::mpsc;

impl ModuleCanvas {
    /// Process incoming MIDI message for MIDI Learn
    #[cfg(feature = "midi")]
    pub fn process_midi_message(&mut self, message: mapmap_control::midi::MidiMessage) {
        // Check if we're in learn mode for any part
        if let Some(part_id) = self.midi_learn_part_id {
            // We received a MIDI message while in learn mode
            // Store the learned values in a pending result
            // The actual module update will happen in the show() method
            // For now, we log it and clear learn mode
            match message {
                mapmap_control::midi::MidiMessage::ControlChange {
                    channel,
                    controller,
                    ..
                } => {
                    tracing::info!(
                        "MIDI Learn: Part {:?} assigned to CC {} on channel {}",
                        part_id,
                        controller,
                        channel
                    );
                    // Store learned values - will be applied in UI
                    self.learned_midi = Some((part_id, channel, controller, false));
                    self.midi_learn_part_id = None;
                }
                mapmap_control::midi::MidiMessage::NoteOn { channel, note, .. } => {
                    tracing::info!(
                        "MIDI Learn: Part {:?} assigned to Note {} on channel {}",
                        part_id,
                        note,
                        channel
                    );
                    // Store learned values - will be applied in UI
                    self.learned_midi = Some((part_id, channel, note, true));
                    self.midi_learn_part_id = None;
                }
                _ => {
                    // Ignore other message types during learn
                }
            }
        }
    }

    /// Process incoming MIDI message (no-op without midi feature)
    #[cfg(not(feature = "midi"))]
    pub fn process_midi_message(&mut self, _message: ()) {}

    /// Renders the menu to add new nodes to the canvas
    fn render_add_node_menu(&mut self, ui: &mut egui::Ui, manager: &mut ModuleManager) {
        ui.menu_button("\u{2795} Add Node", |ui| {
            self.render_add_node_menu_content(ui, manager, None);
        });
    }

    fn apply_undo_action(module: &mut MapFlowModule, action: &CanvasAction) {
        match action {
            CanvasAction::AddPart { part_id, .. } => {
                // Undo add = delete
                module.parts.retain(|p| p.id != *part_id);
            }
            CanvasAction::DeletePart { part_data } => {
                // Undo delete = restore
                module.parts.push(part_data.clone());
            }
            CanvasAction::MovePart {
                part_id, old_pos, ..
            } => {
                // Undo move = restore old position
                if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                    part.position = *old_pos;
                }
            }
            CanvasAction::AddConnection { connection } => {
                // Undo add connection = delete
                module.connections.retain(|c| {
                    !(c.from_part == connection.from_part
                        && c.to_part == connection.to_part
                        && c.from_socket == connection.from_socket
                        && c.to_socket == connection.to_socket)
                });
            }
            CanvasAction::DeleteConnection { connection } => {
                // Undo delete connection = restore
                module.connections.push(connection.clone());
            }
            CanvasAction::Batch(actions) => {
                for action in actions.iter().rev() {
                    Self::apply_undo_action(module, action);
                }
            }
        }
    }

    fn apply_redo_action(module: &mut MapFlowModule, action: &CanvasAction) {
        match action {
            CanvasAction::AddPart { part_data, .. } => {
                // Redo add = add again
                module.parts.push(part_data.clone());
            }
            CanvasAction::DeletePart { part_data } => {
                // Redo delete = delete again
                module.parts.retain(|p| p.id != part_data.id);
            }
            CanvasAction::MovePart {
                part_id, new_pos, ..
            } => {
                // Redo move = apply new position
                if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                    part.position = *new_pos;
                }
            }
            CanvasAction::AddConnection { connection } => {
                // Redo add connection = add again
                module.connections.push(connection.clone());
            }
            CanvasAction::DeleteConnection { connection } => {
                // Redo delete connection = delete again
                module.connections.retain(|c| {
                    !(c.from_part == connection.from_part
                        && c.to_part == connection.to_part
                        && c.from_socket == connection.from_socket
                        && c.to_socket == connection.to_socket)
                });
            }
            CanvasAction::Batch(actions) => {
                for action in actions.iter() {
                    Self::apply_redo_action(module, action);
                }
            }
        }
    }

    fn safe_delete_selection(&mut self, module: &mut MapFlowModule) {
        if self.selected_parts.is_empty() {
            return;
        }

        let mut actions = Vec::new();

        // 1. Identify all parts to delete
        let parts_to_delete: Vec<ModulePartId> = self.selected_parts.clone();

        // 2. Identify all connections to delete (connected to any selected part)
        // We need to capture the connection data for undo
        let mut connections_to_delete = Vec::new();

        for conn in module.connections.iter() {
            if parts_to_delete.contains(&conn.from_part) || parts_to_delete.contains(&conn.to_part)
            {
                connections_to_delete.push(conn.clone());
            }
        }

        // Add DeleteConnection actions
        for conn in connections_to_delete {
            actions.push(CanvasAction::DeleteConnection { connection: conn });
        }

        // 3. Capture part data for undo
        for part_id in &parts_to_delete {
            if let Some(part) = module.parts.iter().find(|p| p.id == *part_id) {
                actions.push(CanvasAction::DeletePart {
                    part_data: part.clone(),
                });
            }
        }

        // 4. Create Batch Action
        let batch_action = CanvasAction::Batch(actions);

        // 5. Execute Deletions (Modify Module)
        // Remove connections first
        module.connections.retain(|c| {
            !parts_to_delete.contains(&c.from_part) && !parts_to_delete.contains(&c.to_part)
        });

        // Remove parts
        module.parts.retain(|p| !parts_to_delete.contains(&p.id));

        // 6. Update Stacks
        self.undo_stack.push(batch_action);
        self.redo_stack.clear();
        self.selected_parts.clear();
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        manager: &mut ModuleManager,
        locale: &LocaleManager,
        actions: &mut Vec<crate::UIAction>,
    ) {
        // === KEYBOARD SHORTCUTS ===
        if !self.selected_parts.is_empty()
            && !ui.memory(|m| m.focused().is_some())
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Space))
        {
            if let Some(module_id) = self.active_module_id {
                if let Some(module) = manager.get_module_mut(module_id) {
                    for part_id in &self.selected_parts {
                        if let Some(part) = module.parts.iter().find(|p| p.id == *part_id) {
                            if let mapmap_core::module::ModulePartType::Source(
                                mapmap_core::module::SourceType::MediaFile { .. },
                            ) = &part.part_type
                            {
                                // Toggle playback
                                let is_playing = self
                                    .player_info
                                    .get(part_id)
                                    .map(|info| info.is_playing)
                                    .unwrap_or(false);

                                let command = if is_playing {
                                    MediaPlaybackCommand::Pause
                                } else {
                                    MediaPlaybackCommand::Play
                                };
                                self.pending_playback_commands.push((*part_id, command));
                            }
                        }
                    }
                }
            }
        }

        // === APPLY LEARNED MIDI VALUES ===
        if let Some((part_id, channel, cc_or_note, is_note)) = self.learned_midi.take() {
            if let Some(module_id) = self.active_module_id {
                if let Some(module) = manager.get_module_mut(module_id) {
                    if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                        if let mapmap_core::module::ModulePartType::Trigger(TriggerType::Midi {
                            channel: ref mut ch,
                            note: ref mut n,
                            ..
                        }) = part.part_type
                        {
                            *ch = channel;
                            *n = cc_or_note;
                            tracing::info!(
                                "Applied MIDI Learn: Channel={}, {}={}",
                                channel,
                                if is_note { "Note" } else { "CC" },
                                cc_or_note
                            );
                        }
                    }
                }
            }
        }

        // === CANVAS TOOLBAR ===
        egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(8, 6))
            .fill(ui.visuals().panel_fill)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // --- ROW 1: Module Context & Adding Nodes ---
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;

                        // LEFT: Module Selector & Info
                        ui.push_id("module_context", |ui| {
                            let mut module_names: Vec<(u64, String)> = manager
                                .list_modules()
                                .iter()
                                .map(|m| (m.id, m.name.clone()))
                                .collect();
                            module_names
                                .sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));

                            let current_name = self
                                .active_module_id
                                .and_then(|id| manager.get_module(id))
                                .map(|m| m.name.clone())
                                .unwrap_or_else(|| "â€” Select Module â€”".to_string());

                            egui::ComboBox::from_id_salt("module_selector")
                                .selected_text(current_name)
                                .width(160.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.active_module_id,
                                        None,
                                        "â€” None â€”",
                                    );
                                    ui.separator();
                                    for (id, name) in &module_names {
                                        ui.selectable_value(
                                            &mut self.active_module_id,
                                            Some(*id),
                                            name,
                                        );
                                    }
                                });

                            if ui
                                .button("\u{2795} New")
                                .on_hover_text("Create a new module")
                                .clicked()
                            {
                                let new_id = manager
                                    .create_module(manager.get_next_available_name("New Module"));
                                self.active_module_id = Some(new_id);
                            }

                            if let Some(module_id) = self.active_module_id {
                                if let Some(module) = manager.get_module_mut(module_id) {
                                    ui.separator();
                                    ui.add(
                                        egui::TextEdit::singleline(&mut module.name)
                                            .desired_width(120.0)
                                            .hint_text("Name"),
                                    );

                                    let mut color_f32 = module.color;
                                    if ui
                                        .color_edit_button_rgba_unmultiplied(&mut color_f32)
                                        .clicked()
                                    {
                                        module.color = color_f32;
                                    }

                                    if ui
                                        .button("\u{1F5D1}")
                                        .on_hover_text("Delete Module")
                                        .clicked()
                                    {
                                        manager.delete_module(module_id);
                                        self.active_module_id = None;
                                    }
                                }
                            }
                        });

                        ui.separator();

                        // CENTER/RIGHT (Top Row): Add Node Menu
                        let has_module = self.active_module_id.is_some();
                        ui.add_enabled_ui(has_module, |ui| {
                            self.render_add_node_menu(ui, manager);
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("Canvas Editor")
                                    .strong()
                                    .color(ui.visuals().strong_text_color()),
                            );
                        });
                    });

                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);

                    // --- ROW 2: View Controls & Utilities ---
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        // Utility Buttons
                        if self.active_module_id.is_some() {
                            if ui.button("📋 Presets").clicked() {
                                self.show_presets = !self.show_presets;
                            }
                            if ui.button("âŠž Auto Layout").clicked() {
                                if let Some(id) = self.active_module_id {
                                    if let Some(m) = manager.get_module_mut(id) {
                                        Self::auto_layout_parts(&mut m.parts);
                                    }
                                }
                            }
                            if ui.button("🔍 Search").clicked() {
                                self.show_search = !self.show_search;
                            }

                            let check_label = if self.diagnostic_issues.is_empty() {
                                "âœ“"
                            } else {
                                "\u{26A0}"
                            };
                            if ui
                                .button(check_label)
                                .on_hover_text("Check Integrity")
                                .clicked()
                            {
                                if let Some(id) = self.active_module_id {
                                    if let Some(m) = manager.get_module(id) {
                                        self.diagnostic_issues =
                                            mapmap_core::diagnostics::check_module_integrity(m);
                                        self.show_diagnostics = true;
                                    }
                                }
                            }
                        }

                        // Right Aligned View Controls
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("âŠ¡").on_hover_text("Reset View").clicked() {
                                self.zoom = 1.0;
                                self.pan_offset = Vec2::ZERO;
                            }
                            ui.label(format!("{:.0}%", self.zoom * 100.0));
                            if ui.button("+").on_hover_text("Zoom In").clicked() {
                                self.zoom = (self.zoom + 0.1).clamp(0.2, 3.0);
                            }
                            ui.add(
                                egui::Slider::new(&mut self.zoom, 0.2..=3.0)
                                    .show_value(false)
                                    .trailing_fill(true),
                            );
                            if ui.button("âˆ’").on_hover_text("Zoom Out").clicked() {
                                self.zoom = (self.zoom - 0.1).clamp(0.2, 3.0);
                            }
                            ui.label("Zoom:");
                        });
                    });
                });
            });

        ui.add_space(1.0);
        ui.separator();

        if let Some(module_id) = self.active_module_id {
            // Render the canvas taking up the full available space
            self.render_canvas(ui, manager, module_id, locale, actions);
            // Properties popup removed - moved to docked inspector
        } else {
            // Show a message if no module is selected
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("🔧 Module Canvas");
                    ui.add_space(10.0);
                    ui.label("Click '\u{2795} New Module' to create a module.");
                    ui.label("Or select an existing module from the dropdown above.");
                });
            });
        }
    }

    fn render_canvas(
        &mut self,
        ui: &mut Ui,
        manager: &mut ModuleManager,
        module_id: ModuleId,
        _locale: &LocaleManager,
        actions: &mut Vec<crate::UIAction>,
    ) {
        let module = if let Some(m) = manager.get_module_mut(module_id) {
            m
        } else {
            return;
        };
        self.ensure_icons_loaded(ui.ctx());

        let canvas_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(canvas_rect, Sense::click_and_drag());

        // Handle panning (Middle click or Alt+Drag)
        if response.dragged_by(egui::PointerButton::Middle)
            || (response.dragged_by(egui::PointerButton::Primary)
                && ui.input(|i| i.modifiers.alt))
        {
            let pan_delta = response.drag_delta();
            self.pan_offset += pan_delta;
            self.panning_canvas = true;
        } else if response.drag_stopped() {
            self.panning_canvas = false;
        }

        // Handle Zoom (Ctrl + Scroll)
        if ui.rect_contains_pointer(canvas_rect) {
            let scroll_delta = ui.input(|i| i.raw_scroll_delta);
            if scroll_delta.y != 0.0 && ui.input(|i| i.modifiers.ctrl) {
                let zoom_factor = if scroll_delta.y > 0.0 { 1.1 } else { 0.9 };
                let old_zoom = self.zoom;
                self.zoom = (self.zoom * zoom_factor).clamp(0.2, 3.0);

                // Zoom towards mouse pointer
                if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let pointer_in_canvas = pointer_pos - canvas_rect.min;
                    let zoom_ratio = self.zoom / old_zoom;
                    self.pan_offset =
                        pointer_in_canvas - (pointer_in_canvas - self.pan_offset) * zoom_ratio;
                }
            }
        }

        // Coordinate conversion helpers
        let pan_offset = self.pan_offset;
        let zoom = self.zoom;
        let to_screen = move |pos: Pos2| -> Pos2 {
            canvas_rect.min + pan_offset + (pos.to_vec2()) * zoom
        };

        let from_screen = move |pos: Pos2| -> Pos2 {
            let v = (pos - canvas_rect.min - pan_offset) / zoom;
            Pos2::new(v.x, v.y)
        };

        // Draw background grid
        let painter = ui.painter_at(canvas_rect);
        self.draw_grid(&painter, canvas_rect);

        // Handle Box Selection
        if self.panning_canvas {
            // Don't box select while panning
        } else if response.drag_started_by(egui::PointerButton::Primary)
            && !ui.input(|i| i.modifiers.alt)
        {
            // Only start box select if not clicking on any node/socket (handled by hit testing later)
            // But here we don't know yet. The logic is usually: if interaction didn't hit anything else.
            // In Egui immediate mode, we check logic order.
            // Here we just set start pos, and clear it if we hit a node later?
            // Actually, we process interactions after drawing for z-order, but here we can check overlap.
            self.box_select_start = response.interact_pointer_pos();
        }

        if let Some(start_pos) = self.box_select_start {
            if ui.input(|i| i.pointer.primary_down()) {
                if let Some(current_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let selection_rect = Rect::from_two_pos(start_pos, current_pos);
                    painter.rect_filled(
                        selection_rect,
                        0.0,
                        Color32::from_rgba_unmultiplied(100, 200, 255, 30),
                    );
                    painter.rect_stroke(
                        selection_rect,
                        0.0,
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 200, 255, 100)),
                        egui::StrokeKind::Middle,
                    );

                    // Select parts inside rect
                    if !ui.input(|i| i.modifiers.shift) {
                        self.selected_parts.clear();
                    }
                    let selection_in_canvas = Rect::from_two_pos(
                        from_screen(selection_rect.min),
                        from_screen(selection_rect.max),
                    );

                    for part in &module.parts {
                        let part_rect = Rect::from_min_size(
                            Pos2::new(part.position.0, part.position.1),
                            Vec2::new(200.0, 100.0), // Approx size
                        );
                        if selection_in_canvas.intersects(part_rect) {
                            if !self.selected_parts.contains(&part.id) {
                                self.selected_parts.push(part.id);
                            }
                        }
                    }
                }
            } else {
                self.box_select_start = None;
            }
        }

        // Draw connections (and handle their interaction)
        let connection_removed_idx =
            self.draw_connections(ui, &painter, module, &to_screen);

        if let Some(idx) = connection_removed_idx {
            if idx < module.connections.len() {
                module.connections.remove(idx);
            }
        }

        // Handle Connection Creation
        if let Some((_part_id, _socket_idx, _is_output, ref _socket_type, _start_pos)) =
            self.creating_connection
        {
            if ui.input(|i| i.pointer.any_released()) {
                // Connection release logic is handled in socket interaction or here if dropped on empty space
                // If we are here, we might have dropped on empty space
                // Check if we dropped on a socket
                // This logic is complex in immediate mode without retained state of socket positions.
                // We'll handle "drop on socket" inside the socket drawing loop/logic if possible.
                // If dropped here (on canvas background), cancel.
                self.creating_connection = None;
            }
        }

        // Gather all socket rects for hit testing during connection creation
        // We need to know where sockets are.
        // Option: Calculate them first or do it in the drawing loop.
        // We'll do it in the drawing loop.

        // Draw Nodes
        let mut delete_part_id = None;
        let mut all_sockets = Vec::new(); // To store socket positions for connection snapping
        let mut move_ops = Vec::new(); // Collect move operations

        // We iterate parts to draw and handle interaction
        // Z-order: Selected parts on top?
        // For now, draw in order.

        for part in &module.parts {
            // Calculate part rect
            let pos_screen = to_screen(Pos2::new(part.position.0, part.position.1));
            // Size estimation (should match draw_part)
            let (part_width, part_height) = part.size.unwrap_or_else(|| {
                let h = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                (200.0, h)
            });
            let rect = Rect::from_min_size(pos_screen, Vec2::new(part_width, part_height) * self.zoom);

            // Collect sockets for hit testing
            // Inputs
            let title_height = 28.0 * self.zoom;
            let socket_start_y = rect.min.y + title_height + 10.0 * self.zoom;
            for (i, input) in part.inputs.iter().enumerate() {
                let socket_pos = Pos2::new(
                    rect.min.x,
                    socket_start_y + i as f32 * 22.0 * self.zoom,
                );
                all_sockets.push(SocketInfo {
                    part_id: part.id,
                    socket_idx: i,
                    is_output: false,
                    socket_type: input.socket_type.clone(),
                    position: socket_pos,
                });
            }
            // Outputs
            for (i, output) in part.outputs.iter().enumerate() {
                let socket_pos = Pos2::new(
                    rect.max.x,
                    socket_start_y + i as f32 * 22.0 * self.zoom,
                );
                all_sockets.push(SocketInfo {
                    part_id: part.id,
                    socket_idx: i,
                    is_output: true,
                    socket_type: output.socket_type.clone(),
                    position: socket_pos,
                });
            }

            // Interact with Part Body
            let part_id = &part.id;
            let part_response = ui.interact(
                rect,
                ui.id().with(("part", *part_id)),
                Sense::click_and_drag(),
            );

            if part_response.clicked() {
                if !ui.input(|i| i.modifiers.shift) {
                    self.selected_parts.clear();
                }
                if !self.selected_parts.contains(part_id) {
                    self.selected_parts.push(*part_id);
                }
                // Stop box selection if we clicked a node
                self.box_select_start = None;
            } else if part_response.secondary_clicked() {
                self.context_menu_part = Some(*part_id);
                self.context_menu_pos = ui.input(|i| i.pointer.hover_pos());
                self.context_menu_connection = None;
            }

            if part_response.drag_started() {
                self.dragging_part = Some((*part_id, Vec2::ZERO));
            }

            if part_response.dragged() {
                if let Some((dragged_id, accumulator)) = self.dragging_part {
                    if dragged_id == *part_id {
                        let delta = part_response.drag_delta() / self.zoom;
                        let mut effective_move = delta;
                        let accumulator = accumulator + delta;

                        // Grid Snapping (Alt to disable)
                        let alt_held = ui.input(|i| i.modifiers.alt);
                        if !alt_held {
                            let grid_size = 20.0;
                            // Check if accumulated movement crosses grid threshold
                            if accumulator.x.abs() >= grid_size || accumulator.y.abs() >= grid_size {
                                let snap_x = (accumulator.x / grid_size).round() * grid_size;
                                let snap_y = (accumulator.y / grid_size).round() * grid_size;
                                effective_move = Vec2::new(snap_x, snap_y);
                                let consumed_accum = effective_move;

                                // Apply move to all selected parts
                                let moving_parts = if self.selected_parts.contains(part_id) {
                                    self.selected_parts.clone()
                                } else {
                                    vec![*part_id]
                                };

                                for moving_id in &moving_parts {
                                    move_ops.push((*moving_id, effective_move));
                                }
                                // Consume accumulator only if move succeeded
                                if !alt_held {
                                    self.dragging_part =
                                        Some((dragged_id, accumulator - consumed_accum));
                                }
                            }
                        } else {
                             // Apply move immediately if Alt is held
                             let moving_parts = if self.selected_parts.contains(part_id) {
                                    self.selected_parts.clone()
                                } else {
                                    vec![*part_id]
                                };

                                for moving_id in &moving_parts {
                                    move_ops.push((*moving_id, effective_move));
                                }
                        }
                    }
                }
            }

            if part_response.drag_stopped() {
                self.dragging_part = None;
            }

            // Check for delete button click (x in top-right corner of title bar)
            let delete_button_rect = self.get_delete_button_rect(rect);
            let delete_id = egui::Id::new((*part_id, "delete"));
            let delete_response = ui
                .interact(delete_button_rect, delete_id, Sense::click())
                .on_hover_text("Hold to delete (Mouse or Space/Enter)");

            // Mary StyleUX: Hold-to-Confirm for Node Deletion (Safety)
            if delete_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            let is_holding_delete = delete_response.is_pointer_button_down_on()
                || (delete_response.has_focus()
                    && ui.input(|i| i.key_down(egui::Key::Space) || i.key_down(egui::Key::Enter)));

            let (triggered, _) = crate::widgets::check_hold_state(ui, delete_id, is_holding_delete);

            if triggered {
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

        // Apply move operations
        for (id, delta) in move_ops {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == id) {
                part.position.0 += delta.x;
                part.position.1 += delta.y;
            }
        }

        // Resize operations to apply after the loop
        let mut resize_ops = Vec::new();

        // Draw parts (nodes) with delete buttons and selection highlight
        for part in &module.parts {
            let part_screen_pos = to_screen(Pos2::new(part.position.0, part.position.1));

            // Use custom size or calculate default
            let (part_width, part_height) = part.size.unwrap_or_else(|| {
                let default_height =
                    80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                (200.0, default_height)
            });
            let part_size = Vec2::new(part_width, part_height);
            let part_screen_rect = Rect::from_min_size(part_screen_pos, part_size * self.zoom);

            // Draw selection highlight if selected
            if self.selected_parts.contains(&part.id) {
                let highlight_rect = part_screen_rect.expand(4.0 * self.zoom);
                // "Cyber" selection: Neon Cyan, Sharp Corners
                painter.rect_stroke(
                    highlight_rect,
                    0.0, // Sharp corners
                    Stroke::new(2.0 * self.zoom, Color32::from_rgb(0, 229, 255)),
                    egui::StrokeKind::Middle,
                );

                // Draw resize handle at bottom-right corner
                let handle_size = 12.0 * self.zoom;
                let handle_rect = Rect::from_min_size(
                    Pos2::new(
                        part_screen_rect.max.x - handle_size,
                        part_screen_rect.max.y - handle_size,
                    ),
                    Vec2::splat(handle_size),
                );
                // Cyan resize handle, sharp
                painter.rect_filled(handle_rect, 0.0, Color32::from_rgb(0, 229, 255));
                // Draw diagonal lines for resize indicator
                painter.line_segment(
                    [
                        handle_rect.min + Vec2::new(3.0, handle_size - 3.0),
                        handle_rect.min + Vec2::new(handle_size - 3.0, 3.0),
                    ],
                    Stroke::new(1.5, Color32::from_gray(40)),
                );

                // Handle resize drag interaction
                let resize_response = ui.interact(
                    handle_rect,
                    egui::Id::new((part.id, "resize")),
                    Sense::drag(),
                );

                if resize_response.drag_started() {
                    self.resizing_part = Some((part.id, (part_width, part_height)));
                }

                if resize_response.dragged() {
                    if let Some((id, _original_size)) = self.resizing_part {
                        if id == part.id {
                            let delta = resize_response.drag_delta() / self.zoom;
                            resize_ops.push((part.id, delta));
                        }
                    }
                }

                if resize_response.drag_stopped() {
                    self.resizing_part = None;
                }
            }

            self.draw_part_with_delete(ui, &painter, part, part_screen_rect, actions, module.id);
        }

        // Apply resize operations
        for (part_id, delta) in resize_ops {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                // Initialize size if None
                let current_size = part.size.unwrap_or_else(|| {
                    let h = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                    (200.0, h)
                });
                let new_w = (current_size.0 + delta.x).max(100.0);
                let new_h = (current_size.1 + delta.y).max(50.0);
                part.size = Some((new_w, new_h));
            }
        }

        // Draw connection being created with visual feedback
        if let Some((from_part_id, _from_socket_idx, from_is_output, ref from_type, start_pos)) =
            self.creating_connection
        {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                // Check if hovering over a compatible socket
                let socket_radius = 8.0 * self.zoom;
                let mut is_valid_target = false;
                let mut near_socket = false;

                for socket in &all_sockets {
                    if socket.position.distance(pointer_pos) < socket_radius * 2.0 {
                        near_socket = true;
                        // Valid if: different part, opposite direction, same type
                        if socket.part_id != from_part_id
                            && socket.is_output != from_is_output
                            && socket.socket_type == *from_type
                        {
                            is_valid_target = true;
                        }
                        break;
                    }
                }

                // Color based on validity
                let color = if near_socket {
                    if is_valid_target {
                        Color32::from_rgb(50, 255, 100) // Green = valid
                    } else {
                        Color32::from_rgb(255, 80, 80) // Red = invalid
                    }
                } else {
                    Self::get_socket_color(from_type) // Default socket color
                };

                // Draw the connection line
                painter.line_segment([start_pos, pointer_pos], Stroke::new(3.0, color));

                // Draw a circle at the end point
                painter.circle_filled(pointer_pos, 5.0, color);
            }
        }

        // Draw mini-map in bottom-right corner
        self.draw_mini_map(&painter, canvas_rect, module);

        // Draw search popup if visible
        if self.show_search {
            self.draw_search_popup(ui, canvas_rect, module);
        }

        // Draw presets popup if visible
        if self.show_presets {
            self.draw_presets_popup(ui, canvas_rect, module);
        }

        // Draw diagnostics popup if visible
        self.render_diagnostics_popup(ui);

        // Draw context menu for parts
        if let (Some(part_id), Some(pos)) = (self.context_menu_part, self.context_menu_pos) {
            let menu_width = 150.0;
            let menu_height = 80.0;
            let menu_rect = Rect::from_min_size(pos, Vec2::new(menu_width, menu_height));

            // Draw menu background
            let painter = ui.painter();
            painter.rect_filled(
                menu_rect,
                0.0,
                Color32::from_rgba_unmultiplied(40, 40, 50, 250),
            );
            painter.rect_stroke(
                menu_rect,
                0.0,
                Stroke::new(1.0, Color32::from_rgb(80, 80, 100)),
                egui::StrokeKind::Middle,
            );

            // Menu items
            let inner_rect = menu_rect.shrink(4.0);
            ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                ui.vertical(|ui| {
                    if ui.button("âš™ Open Properties").clicked() {
                        // Select the part to show it in the inspector
                        self.selected_parts.clear();
                        self.selected_parts.push(part_id);
                        self.context_menu_part = None;
                        self.context_menu_pos = None;
                    }
                    if crate::widgets::hold_to_action_button(
                        ui,
                        "\u{1F5D1} Delete",
                        colors::ERROR_COLOR,
                    ) {
                        // Remove connections and part
                        module
                            .connections
                            .retain(|c| c.from_part != part_id && c.to_part != part_id);
                        module.parts.retain(|p| p.id != part_id);
                        self.context_menu_part = None;
                        self.context_menu_pos = None;
                    }
                });
            });

            // Close menu on click outside
            if ui.input(|i| i.pointer.any_click())
                && !menu_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
            {
                self.context_menu_part = None;
                self.context_menu_pos = None;
            }
        }

        // Draw context menu for connections
        if let (Some(conn_idx), Some(pos)) = (self.context_menu_connection, self.context_menu_pos) {
            let menu_width = 150.0;
            let menu_height = 40.0;
            let menu_rect = Rect::from_min_size(pos, Vec2::new(menu_width, menu_height));

            // Draw menu background
            let painter = ui.painter();
            painter.rect_filled(
                menu_rect,
                0.0,
                Color32::from_rgba_unmultiplied(40, 40, 50, 250),
            );
            painter.rect_stroke(
                menu_rect,
                0.0,
                Stroke::new(1.0, Color32::from_rgb(80, 80, 100)),
                egui::StrokeKind::Middle,
            );

            // Menu items
            let inner_rect = menu_rect.shrink(4.0);
            ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                ui.vertical(|ui| {
                    if crate::widgets::hold_to_action_button(
                        ui,
                        "\u{1F5D1} Delete Connection",
                        colors::ERROR_COLOR,
                    ) {
                        if conn_idx < module.connections.len() {
                            module.connections.remove(conn_idx);
                        }
                        self.context_menu_connection = None;
                        self.context_menu_pos = None;
                    }
                });
            });

            // Close menu on click outside
            if ui.input(|i| i.pointer.any_click())
                && !menu_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
            {
                self.context_menu_connection = None;
                self.context_menu_pos = None;
            }
        }

        // Draw context menu for adding nodes (canvas level)
        if self.context_menu_part.is_none() && self.context_menu_connection.is_none() {
            if let Some(pos) = self.context_menu_pos {
                let menu_width = 180.0;
                let menu_height = 250.0; // Estimate or let it be dynamic
                let menu_rect = Rect::from_min_size(pos, Vec2::new(menu_width, menu_height));

                // Draw menu background
                let painter = ui.painter();
                painter.rect_filled(
                    menu_rect,
                    4.0,
                    Color32::from_rgba_unmultiplied(30, 30, 40, 245),
                );
                painter.rect_stroke(
                    menu_rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgb(80, 100, 150)),
                    egui::StrokeKind::Middle,
                );

                // Menu items
                let inner_rect = menu_rect.shrink(8.0);
                ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                    ui.vertical(|ui| {
                        ui.heading("\u{2795} Add Node");
                        ui.separator();

                        // Convert screen position to canvas position for node placement
                        let canvas_pos = from_screen(pos);
                        let pos_tuple = (canvas_pos.x, canvas_pos.y);

                        self.render_add_node_menu_content(ui, manager, Some(pos_tuple));
                    });
                });

                // Close menu on click outside
                if ui.input(|i| i.pointer.any_click())
                    && !menu_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
                {
                    self.context_menu_pos = None;
                }
            }
        }
    }
}
