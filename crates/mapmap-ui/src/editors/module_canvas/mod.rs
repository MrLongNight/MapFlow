use crate::i18n::LocaleManager;
use crate::UIAction;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use egui_node_editor::*;
use mapmap_core::{
    audio_reactive::AudioTriggerData,
    module::{
        EffectType as ModuleEffectType, MapFlowModule, ModuleId, ModuleManager, ModulePartId,
        TriggerType,
    },
};
use std::borrow::Cow;

pub mod diagnostics;
pub mod draw;
pub mod inspector;
pub mod mesh;
pub mod state;
pub mod types;
pub mod utils;

pub use state::ModuleCanvas;
use types::*;

// Implement NodeTemplateTrait for MyNodeTemplate
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

impl ModuleCanvas {
    pub fn ensure_icons_loaded(&mut self, ctx: &egui::Context) {
        utils::ensure_icons_loaded(&mut self.plug_icons, ctx);
    }

    pub fn sync_mesh_editor_to_current_selection(
        &mut self,
        part: &mapmap_core::module::ModulePart,
    ) {
        mesh::sync_mesh_editor_to_current_selection(self, part);
    }

    pub fn apply_mesh_editor_to_selection(&mut self, part: &mut mapmap_core::module::ModulePart) {
        mesh::apply_mesh_editor_to_selection(self, part);
    }

    pub fn render_mesh_editor_ui(
        &mut self,
        ui: &mut Ui,
        mesh: &mut mapmap_core::module::MeshType,
        part_id: ModulePartId,
        id_salt: u64,
    ) {
        mesh::render_mesh_editor_ui(self, ui, mesh, part_id, id_salt);
    }

    pub fn take_playback_commands(&mut self) -> Vec<(ModulePartId, MediaPlaybackCommand)> {
        std::mem::take(&mut self.pending_playback_commands)
    }

    pub fn get_selected_part_id(&self) -> Option<ModulePartId> {
        self.selected_parts.last().copied()
    }

    pub fn set_default_effect_params(
        effect_type: ModuleEffectType,
        params: &mut std::collections::HashMap<String, f32>,
    ) {
        inspector::set_default_effect_params(effect_type, params);
    }

    pub fn render_inspector_for_part(
        &mut self,
        ui: &mut Ui,
        part: &mut mapmap_core::module::ModulePart,
        actions: &mut Vec<UIAction>,
        module_id: mapmap_core::module::ModuleId,
        shared_media_ids: &[String],
    ) {
        inspector::render_inspector_for_part(self, ui, part, actions, module_id, shared_media_ids);
    }

    pub fn set_active_module(&mut self, module_id: Option<u64>) {
        self.active_module_id = module_id;
        // Also clear selection when switching modules
        self.selected_parts.clear();
        self.dragging_part = None;
        self.creating_connection = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn active_module_id(&self) -> Option<u64> {
        self.active_module_id
    }

    pub fn set_audio_data(&mut self, data: AudioTriggerData) {
        self.audio_trigger_data = data;
    }

    pub fn get_audio_trigger_data(&self) -> Option<&AudioTriggerData> {
        Some(&self.audio_trigger_data)
    }

    pub fn get_rms_volume(&self) -> f32 {
        self.audio_trigger_data.rms_volume
    }

    pub fn is_beat_detected(&self) -> bool {
        self.audio_trigger_data.beat_detected
    }

    #[cfg(feature = "midi")]
    pub fn process_midi_message(&mut self, message: mapmap_control::midi::MidiMessage) {
        // Check if we're in learn mode for any part
        if let Some(part_id) = self.midi_learn_part_id {
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
                    self.learned_midi = Some((part_id, channel, note, true));
                    self.midi_learn_part_id = None;
                }
                _ => {}
            }
        }
    }

    #[cfg(not(feature = "midi"))]
    pub fn process_midi_message(&mut self, _message: ()) {}

    fn safe_delete_selection(&mut self, module: &mut MapFlowModule) {
        if self.selected_parts.is_empty() {
            return;
        }

        let mut actions = Vec::new();
        let parts_to_delete: Vec<ModulePartId> = self.selected_parts.clone();
        let mut connections_to_delete = Vec::new();

        for conn in module.connections.iter() {
            if parts_to_delete.contains(&conn.from_part) || parts_to_delete.contains(&conn.to_part)
            {
                connections_to_delete.push(conn.clone());
            }
        }

        for conn in connections_to_delete {
            actions.push(CanvasAction::DeleteConnection { connection: conn });
        }

        for part_id in &parts_to_delete {
            if let Some(part) = module.parts.iter().find(|p| p.id == *part_id) {
                actions.push(CanvasAction::DeletePart {
                    part_data: part.clone(),
                });
            }
        }

        let batch_action = CanvasAction::Batch(actions);

        module.connections.retain(|c| {
            !parts_to_delete.contains(&c.from_part) && !parts_to_delete.contains(&c.to_part)
        });

        module.parts.retain(|p| !parts_to_delete.contains(&p.id));

        self.undo_stack.push(batch_action);
        self.redo_stack.clear();
        self.selected_parts.clear();
    }

    fn apply_undo_action(module: &mut MapFlowModule, action: &CanvasAction) {
        match action {
            CanvasAction::AddPart { part_id, .. } => {
                module.parts.retain(|p| p.id != *part_id);
            }
            CanvasAction::DeletePart { part_data } => {
                module.parts.push(part_data.clone());
            }
            CanvasAction::MovePart {
                part_id, old_pos, ..
            } => {
                if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                    part.position = *old_pos;
                }
            }
            CanvasAction::AddConnection { connection } => {
                module.connections.retain(|c| {
                    !(c.from_part == connection.from_part
                        && c.to_part == connection.to_part
                        && c.from_socket == connection.from_socket
                        && c.to_socket == connection.to_socket)
                });
            }
            CanvasAction::DeleteConnection { connection } => {
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
                module.parts.push(part_data.clone());
            }
            CanvasAction::DeletePart { part_data } => {
                module.parts.retain(|p| p.id != part_data.id);
            }
            CanvasAction::MovePart {
                part_id, new_pos, ..
            } => {
                if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                    part.position = *new_pos;
                }
            }
            CanvasAction::AddConnection { connection } => {
                module.connections.push(connection.clone());
            }
            CanvasAction::DeleteConnection { connection } => {
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

    pub fn show(
        &mut self,
        ui: &mut Ui,
        manager: &mut ModuleManager,
        locale: &LocaleManager,
        actions: &mut Vec<crate::UIAction>,
    ) {
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

        egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(8, 6))
            .fill(ui.visuals().panel_fill)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    draw::render_add_node_menu_content(ui, manager, None, self.active_module_id);

                    ui.separator();

                    if ui.button("\u{1F4BE} Save Presets").clicked() {
                        self.show_presets = true;
                    }

                    ui.separator();

                    if ui
                        .button("\u{1F50D}")
                        .on_hover_text("Search Nodes (Ctrl+F)")
                        .clicked()
                    {
                        self.show_search = !self.show_search;
                    }

                    if ui
                        .button("\u{2714} Check")
                        .on_hover_text("Check Module")
                        .clicked()
                    {
                        if let Some(module_id) = self.active_module_id {
                            if let Some(module) = manager.get_module(module_id) {
                                self.diagnostic_issues =
                                    mapmap_core::diagnostics::check_module_integrity(module);
                                self.show_diagnostics = true;
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Center").clicked() {
                                self.pan_offset = Vec2::ZERO;
                                self.zoom = 1.0;
                            }
                            if ui.button("\u{2795}").on_hover_text("Zoom In").clicked() {
                                self.zoom = (self.zoom + 0.1).clamp(0.2, 3.0);
                            }
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
            self.render_canvas(ui, manager, module_id, locale, actions);
        } else {
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
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let canvas_rect = response.rect;

        let drag_started_on_empty = response.drag_started() && self.dragging_part.is_none();

        let middle_button = ui.input(|i| i.pointer.middle_down());
        if response.dragged()
            && self.dragging_part.is_none()
            && self.creating_connection.is_none()
            && (middle_button || self.panning_canvas)
        {
            self.pan_offset += response.drag_delta();
        }

        if drag_started_on_empty && !middle_button {
            // Panning will be set later if no part is clicked
        }

        let ctrl_held = ui.input(|i| i.modifiers.ctrl);

        if response.secondary_clicked()
            && self.dragging_part.is_none()
            && self.creating_connection.is_none()
        {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                self.context_menu_pos = Some(pointer_pos);
                self.context_menu_part = None;
                self.context_menu_connection = None;
            }
        }

        // Keyboard Actions
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::A)) {
            self.selected_parts = module.parts.iter().map(|p| p.id).collect();
        }

        if !ui.memory(|m| m.focused().is_some())
            && (ui.input(|i| i.key_pressed(egui::Key::Delete))
                || ui.input(|i| i.key_pressed(egui::Key::Backspace)))
            && !self.selected_parts.is_empty()
        {
            self.safe_delete_selection(module);
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.show_search {
                self.show_search = false;
            } else {
                self.selected_parts.clear();
            }
        }

        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::F)) {
            self.show_search = !self.show_search;
            if self.show_search {
                self.search_filter.clear();
            }
        }

        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::Z)) && !self.undo_stack.is_empty() {
            if let Some(action) = self.undo_stack.pop() {
                Self::apply_undo_action(module, &action);
                self.redo_stack.push(action);
            }
        }

        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::Y)) && !self.redo_stack.is_empty() {
            if let Some(action) = self.redo_stack.pop() {
                Self::apply_redo_action(module, &action);
                self.undo_stack.push(action);
            }
        }

        // Draw grid
        draw::draw_grid(self, &painter, canvas_rect);

        let zoom = self.zoom;
        let pan_offset = self.pan_offset;
        let canvas_min = canvas_rect.min.to_vec2();

        let to_screen = move |pos: Pos2| -> Pos2 { pos * zoom + pan_offset + canvas_min };
        let from_screen = move |pos: Pos2| -> Pos2 { (pos - pan_offset - canvas_min) / zoom };

        let remove_conn_idx = draw::draw_connections(self, ui, &painter, module, &to_screen);
        if let Some(idx) = remove_conn_idx {
            if idx < module.connections.len() {
                module.connections.remove(idx);
            }
        }

        let mut all_sockets = Vec::new();
        let mut clicked_on_part = false;
        let mut delete_part_id = None;
        let mut resize_ops = Vec::new();
        let mut drag_delta = Vec2::ZERO;

        for part in &mut module.parts {
            let node_width = 200.0;
            let title_height = 28.0;
            let socket_offset_y = 10.0;
            let socket_spacing = 22.0;
            let socket_start_y = part.position.1 + title_height + socket_offset_y;

            for (i, socket) in part.inputs.iter().enumerate() {
                let y = socket_start_y + i as f32 * socket_spacing;
                let pos = Pos2::new(part.position.0, y);
                all_sockets.push(SocketInfo {
                    part_id: part.id,
                    socket_idx: i,
                    is_output: false,
                    socket_type: socket.socket_type,
                    position: to_screen(pos),
                });
            }

            for (i, socket) in part.outputs.iter().enumerate() {
                let y = socket_start_y + i as f32 * socket_spacing;
                let pos = Pos2::new(part.position.0 + node_width, y);
                all_sockets.push(SocketInfo {
                    part_id: part.id,
                    socket_idx: i,
                    is_output: true,
                    socket_type: socket.socket_type,
                    position: to_screen(pos),
                });
            }

            let part_pos = to_screen(Pos2::new(part.position.0, part.position.1));
            let (w, h) = part.size.unwrap_or_else(|| {
                let h = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                (200.0, h)
            });
            let part_rect = Rect::from_min_size(part_pos, Vec2::new(w, h) * self.zoom);

            if self.selected_parts.contains(&part.id) {
                let highlight_rect = part_rect.expand(4.0 * self.zoom);
                painter.rect_stroke(
                    highlight_rect,
                    0.0,
                    Stroke::new(2.0 * self.zoom, Color32::from_rgb(0, 229, 255)),
                    egui::StrokeKind::Middle,
                );

                let handle_size = 12.0 * self.zoom;
                let handle_rect = Rect::from_min_size(
                    Pos2::new(part_rect.max.x - handle_size, part_rect.max.y - handle_size),
                    Vec2::splat(handle_size),
                );
                painter.rect_filled(handle_rect, 0.0, Color32::from_rgb(0, 229, 255));
                painter.line_segment(
                    [
                        handle_rect.min + Vec2::new(3.0, handle_size - 3.0),
                        handle_rect.min + Vec2::new(handle_size - 3.0, 3.0),
                    ],
                    Stroke::new(1.5, Color32::from_gray(40)),
                );

                let resize_response = ui.interact(
                    handle_rect,
                    egui::Id::new((part.id, "resize")),
                    Sense::drag(),
                );

                if resize_response.drag_started() {
                    self.resizing_part = Some((part.id, (w, h)));
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

            draw::draw_part_with_delete(self, ui, &painter, part, part_rect, actions, module.id);

            let part_id = part.id;
            let interact_rect = part_rect.shrink(4.0);
            let part_response = ui.interact(
                interact_rect,
                egui::Id::new(part_id),
                Sense::click_and_drag(),
            );

            if part_response.hovered() {
                for socket_info in &all_sockets {
                    if socket_info.part_id == part_id {
                        let dist = socket_info
                            .position
                            .distance(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()));
                        if dist < 10.0 * self.zoom && part_response.drag_started() {
                            self.creating_connection = Some((
                                part_id,
                                socket_info.socket_idx,
                                socket_info.is_output,
                                socket_info.socket_type,
                                socket_info.position,
                            ));
                            clicked_on_part = true;
                        }
                    }
                }
            }

            if part_response.clicked() {
                clicked_on_part = true;
                if ui.input(|i| i.modifiers.shift) {
                    if self.selected_parts.contains(&part_id) {
                        self.selected_parts.retain(|&id| id != part_id);
                    } else {
                        self.selected_parts.push(part_id);
                    }
                } else if !self.selected_parts.contains(&part_id) {
                    self.selected_parts.clear();
                    self.selected_parts.push(part_id);
                }
            }

            if part_response.drag_started() {
                clicked_on_part = true;
                if self.creating_connection.is_none() {
                    if !self.selected_parts.contains(&part_id) {
                        if !ui.input(|i| i.modifiers.shift) {
                            self.selected_parts.clear();
                        }
                        self.selected_parts.push(part_id);
                    }
                    self.dragging_part = Some((part_id, Vec2::ZERO));
                }
            }

            if let Some((dragged_id, _accumulator)) = self.dragging_part {
                if dragged_id == part_id && self.creating_connection.is_none() {
                    let delta = part_response.drag_delta() / self.zoom;
                    drag_delta = delta;
                }
            }

            if part_response.drag_stopped() {
                self.dragging_part = None;
                if let Some((from_part, from_idx, is_output, _, _)) =
                    self.creating_connection.take()
                {
                    if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        for target in &all_sockets {
                            if target.position.distance(pointer_pos) < 15.0 * self.zoom
                                && target.part_id != from_part
                                && target.is_output != is_output
                            {
                                let (out_part, out_idx, in_part, in_idx) = if is_output {
                                    (from_part, from_idx, target.part_id, target.socket_idx)
                                } else {
                                    (target.part_id, target.socket_idx, from_part, from_idx)
                                };

                                let exists = module.connections.iter().any(|c| {
                                    c.from_part == out_part
                                        && c.from_socket == out_idx
                                        && c.to_part == in_part
                                        && c.to_socket == in_idx
                                });

                                if !exists {
                                    module.connections.push(
                                        mapmap_core::module::ModuleConnection {
                                            from_part: out_part,
                                            from_socket: out_idx,
                                            to_part: in_part,
                                            to_socket: in_idx,
                                        },
                                    );
                                    ui.ctx().request_repaint();
                                }
                            }
                        }
                    }
                }
            }

            let delete_rect = draw::get_delete_button_rect(self, part_rect);
            let delete_id = egui::Id::new((part_id, "delete"));
            let delete_response = ui.interact(delete_rect, delete_id, Sense::click());
            if delete_response.hovered() {
                clicked_on_part = true;
            }
            let (triggered, _) = crate::widgets::check_hold_state(
                ui,
                delete_id,
                delete_response.is_pointer_button_down_on(),
            );
            if triggered {
                delete_part_id = Some(part_id);
            }
        }

        // Apply drag delta
        if drag_delta != Vec2::ZERO {
            for pid in &self.selected_parts {
                if let Some(p) = module.parts.iter_mut().find(|p| p.id == *pid) {
                    p.position.0 += drag_delta.x;
                    p.position.1 += drag_delta.y;
                }
            }
        }

        for (part_id, delta) in resize_ops {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                let current_size = part.size.unwrap_or_else(|| {
                    let h = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                    (200.0, h)
                });
                let new_w = (current_size.0 + delta.x).max(100.0);
                let new_h = (current_size.1 + delta.y).max(50.0);
                part.size = Some((new_w, new_h));
            }
        }

        if drag_started_on_empty && !clicked_on_part && !middle_button {
            self.panning_canvas = true;
        }

        if let Some(pid) = delete_part_id {
            module
                .connections
                .retain(|c| c.from_part != pid && c.to_part != pid);
            module.parts.retain(|p| p.id != pid);
        }

        if let Some((from_part_id, _, from_is_output, ref from_type, start_pos)) =
            self.creating_connection
        {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                let mut color = utils::get_socket_color(from_type);
                for socket in &all_sockets {
                    if socket.position.distance(pointer_pos) < 15.0 * self.zoom {
                        if socket.part_id != from_part_id
                            && socket.is_output != from_is_output
                            && socket.socket_type == *from_type
                        {
                            color = Color32::GREEN;
                        } else {
                            color = Color32::RED;
                        }
                        break;
                    }
                }

                painter.line_segment([start_pos, pointer_pos], Stroke::new(3.0, color));
                painter.circle_filled(pointer_pos, 5.0, color);
            }
        }

        draw::draw_mini_map(self, &painter, canvas_rect, module);

        if self.show_search {
            draw::draw_search_popup(self, ui, canvas_rect, module);
        }

        if self.show_presets {
            draw::draw_presets_popup(self, ui, canvas_rect, module);
        }

        diagnostics::render_diagnostics_popup(self, ui);

        if self.context_menu_part.is_none() && self.context_menu_connection.is_none() {
            if let Some(pos) = self.context_menu_pos {
                let menu_rect = Rect::from_min_size(pos, Vec2::new(180.0, 250.0));

                if ui.input(|i| i.pointer.any_click())
                    && !menu_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
                {
                    self.context_menu_pos = None;
                } else {
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

                    let inner = menu_rect.shrink(8.0);
                    ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                        ui.vertical(|ui| {
                            ui.heading("\u{2795} Add Node");
                            ui.separator();
                            let canvas_pos = from_screen(pos);
                            draw::render_add_node_menu_content(
                                ui,
                                manager,
                                Some((canvas_pos.x, canvas_pos.y)),
                                self.active_module_id,
                            );
                        });
                    });
                }
            }
        }
    }
}
