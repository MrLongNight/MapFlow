use super::controller;
use super::diagnostics;
use super::draw;
use super::state::ModuleCanvas;
use super::types::*;
use super::utils;
use crate::i18n::LocaleManager;
use crate::UIAction;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::module::{ModuleId, ModuleManager, TriggerType};

pub fn show(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    manager: &mut ModuleManager,
    locale: &LocaleManager,
    actions: &mut Vec<UIAction>,
) {
    if !canvas.selected_parts.is_empty()
        && !ui.memory(|m| m.focused().is_some())
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Space))
    {
        if let Some(module_id) = canvas.active_module_id {
            if let Some(module) = manager.get_module_mut(module_id) {
                for part_id in &canvas.selected_parts {
                    if let Some(part) = module.parts.iter().find(|p| p.id == *part_id) {
                        if let mapmap_core::module::ModulePartType::Source(
                            mapmap_core::module::SourceType::MediaFile { .. },
                        ) = &part.part_type
                        {
                            let is_playing = canvas
                                .player_info
                                .get(part_id)
                                .map(|info| info.is_playing)
                                .unwrap_or(false);

                            let command = if is_playing {
                                MediaPlaybackCommand::Pause
                            } else {
                                MediaPlaybackCommand::Play
                            };
                            canvas.pending_playback_commands.push((*part_id, command));
                        }
                    }
                }
            }
        }
    }

    if let Some((part_id, channel, cc_or_note, is_note)) = canvas.learned_midi.take() {
        if let Some(module_id) = canvas.active_module_id {
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

    if let Some(module_id) = canvas.active_module_id {
        render_canvas(canvas, ui, manager, module_id, locale, actions);
    } else {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("🔧 Module Canvas");
                ui.add_space(10.0);
                ui.label("Click '➕ New Module' to create a module.");
                ui.label("Please select an existing module from the toolbar above.");
            });
        });
    }
}

pub fn render_canvas(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    manager: &mut ModuleManager,
    module_id: ModuleId,
    _locale: &LocaleManager,
    actions: &mut Vec<UIAction>,
) {
    let module = if let Some(m) = manager.get_module_mut(module_id) {
        m
    } else {
        return;
    };
    utils::ensure_icons_loaded(&mut canvas.plug_icons, ui.ctx());
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
    let canvas_rect = response.rect;

    let drag_started_on_empty = response.drag_started() && canvas.dragging_part.is_none();

    let middle_button = ui.input(|i| i.pointer.middle_down());
    if response.dragged()
        && canvas.dragging_part.is_none()
        && canvas.creating_connection.is_none()
        && (middle_button || canvas.panning_canvas)
    {
        canvas.pan_offset += response.drag_delta();
    }

    let ctrl_held = ui.input(|i| i.modifiers.ctrl);

    if response.secondary_clicked()
        && canvas.dragging_part.is_none()
        && canvas.creating_connection.is_none()
    {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            canvas.context_menu_pos = Some(pointer_pos);
            canvas.context_menu_part = None;
            canvas.context_menu_connection = None;
        }
    }

    // Keyboard Actions
    if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::A)) {
        canvas.selected_parts = module.parts.iter().map(|p| p.id).collect();
    }

    if !ui.memory(|m| m.focused().is_some())
        && (ui.input(|i| i.key_pressed(egui::Key::Delete))
            || ui.input(|i| i.key_pressed(egui::Key::Backspace)))
        && !canvas.selected_parts.is_empty()
    {
        controller::safe_delete_selection(canvas, module);
    }

    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        if canvas.show_search {
            canvas.show_search = false;
        } else {
            canvas.selected_parts.clear();
        }
    }

    if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::F)) {
        canvas.show_search = !canvas.show_search;
        if canvas.show_search {
            canvas.search_filter.clear();
        }
    }

    if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::Z)) && !canvas.undo_stack.is_empty() {
        if let Some(action) = canvas.undo_stack.pop() {
            controller::apply_undo_action(module, &action);
            canvas.redo_stack.push(action);
        }
    }

    if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::Y)) && !canvas.redo_stack.is_empty() {
        if let Some(action) = canvas.redo_stack.pop() {
            controller::apply_redo_action(module, &action);
            canvas.undo_stack.push(action);
        }
    }

    // Draw grid
    draw::draw_grid(canvas, &painter, canvas_rect);

    let zoom = canvas.zoom;
    let pan_offset = canvas.pan_offset;
    let canvas_min = canvas_rect.min.to_vec2();

    let to_screen = move |pos: Pos2| -> Pos2 { pos * zoom + pan_offset + canvas_min };
    let from_screen = move |pos: Pos2| -> Pos2 { (pos - pan_offset - canvas_min) / zoom };

    let remove_conn_idx = draw::draw_connections(canvas, ui, &painter, module, &to_screen);
    if let Some(idx) = remove_conn_idx {
        if idx < module.connections.len() {
            module.connections.remove(idx);
        }
    }

    // 1. Collect ALL socket positions first
    let mut all_sockets = Vec::new();
    let node_width = 200.0;
    let title_height = 28.0;
    let socket_offset_y = 10.0;
    let socket_spacing = 22.0;

    for part in &module.parts {
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
    }

    let mut clicked_on_part = false;
    let mut delete_part_id = None;
    let mut resize_ops = Vec::new();
    let mut drag_delta = Vec2::ZERO;

    // 2. Render parts and handle interactions
    for part in &mut module.parts {
        let part_pos = to_screen(Pos2::new(part.position.0, part.position.1));
        let (w, h) = part.size.unwrap_or_else(|| {
            let h = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
            (200.0, h)
        });
        let part_rect = Rect::from_min_size(part_pos, Vec2::new(w, h) * canvas.zoom);

        if canvas.selected_parts.contains(&part.id) {
            let highlight_rect = part_rect.expand(4.0 * canvas.zoom);
            painter.rect_stroke(
                highlight_rect,
                0.0,
                Stroke::new(2.0 * canvas.zoom, Color32::from_rgb(0, 229, 255)),
                egui::StrokeKind::Middle,
            );

            let handle_size = 12.0 * canvas.zoom;
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
                canvas.resizing_part = Some((part.id, (w, h)));
            }

            if resize_response.dragged() {
                if let Some((id, _original_size)) = canvas.resizing_part {
                    if id == part.id {
                        let delta = resize_response.drag_delta() / canvas.zoom;
                        resize_ops.push((part.id, delta));
                    }
                }
            }

            if resize_response.drag_stopped() {
                canvas.resizing_part = None;
            }
        }

        draw::draw_part_with_delete(canvas, ui, &painter, part, part_rect, actions, module.id);

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
                    if dist < 12.0 * canvas.zoom && part_response.drag_started() {
                        canvas.creating_connection = Some((
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
                if canvas.selected_parts.contains(&part_id) {
                    canvas.selected_parts.retain(|&id| id != part_id);
                } else {
                    canvas.selected_parts.push(part_id);
                }
            } else if !canvas.selected_parts.contains(&part_id) {
                canvas.selected_parts.clear();
                canvas.selected_parts.push(part_id);
            }
        }

        if part_response.drag_started() {
            clicked_on_part = true;
            if canvas.creating_connection.is_none() {
                if !canvas.selected_parts.contains(&part_id) {
                    if !ui.input(|i| i.modifiers.shift) {
                        canvas.selected_parts.clear();
                    }
                    canvas.selected_parts.push(part_id);
                }
                canvas.dragging_part = Some((part_id, Vec2::ZERO));
            }
        }

        if let Some((dragged_id, _accumulator)) = canvas.dragging_part {
            if dragged_id == part_id && canvas.creating_connection.is_none() {
                let delta = part_response.drag_delta() / canvas.zoom;
                drag_delta = delta;
            }
        }

        if part_response.drag_stopped() {
            canvas.dragging_part = None;
        }

        let delete_rect = draw::get_delete_button_rect(canvas, part_rect);
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

    // 3. Global Connection Release
    if ui.input(|i| i.pointer.any_released()) {
        if let Some((from_part, from_idx, is_output, _, _)) = canvas.creating_connection.take() {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                for target in &all_sockets {
                    if target.position.distance(pointer_pos) < 30.0 * canvas.zoom
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
                            module
                                .connections
                                .push(mapmap_core::module::ModuleConnection {
                                    from_part: out_part,
                                    from_socket: out_idx,
                                    to_part: in_part,
                                    to_socket: in_idx,
                                });
                            ui.ctx().request_repaint();
                        }
                    }
                }
            }
        }
    }

    // 4. Apply drag delta to all selected parts
    if drag_delta != Vec2::ZERO {
        for pid in &canvas.selected_parts {
            if let Some(p) = module.parts.iter_mut().find(|p| p.id == *pid) {
                p.position.0 += drag_delta.x;
                p.position.1 += drag_delta.y;
            }
        }
    }

    // 5. Apply resize operations
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
        canvas.panning_canvas = true;
    }

    if let Some(pid) = delete_part_id {
        module
            .connections
            .retain(|c| c.from_part != pid && c.to_part != pid);
        module.parts.retain(|p| p.id != pid);
    }

    if let Some((from_part_id, _, from_is_output, ref from_type, start_pos)) =
        canvas.creating_connection
    {
        if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let mut color = utils::get_socket_color(from_type);
            for socket in &all_sockets {
                if socket.position.distance(pointer_pos) < 15.0 * canvas.zoom {
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

    draw::draw_mini_map(canvas, &painter, canvas_rect, module);

    if canvas.show_search {
        draw::draw_search_popup(canvas, ui, canvas_rect, module);
    }

    if canvas.show_presets {
        draw::draw_presets_popup(canvas, ui, canvas_rect, module);
    }

    diagnostics::render_diagnostics_popup(canvas, ui);

    if !ui.memory(|m| m.focused().is_some()) && ui.input(|i| i.key_pressed(egui::Key::Tab)) {
        canvas.show_quick_create = true;
        canvas.quick_create_pos = ui
            .input(|i| i.pointer.hover_pos())
            .unwrap_or(canvas_rect.center());
        canvas.quick_create_filter.clear();
        canvas.quick_create_selected_index = 0;
    }

    draw::draw_quick_create_popup(canvas, ui, canvas_rect, manager, canvas.active_module_id);

    if canvas.context_menu_part.is_none() && canvas.context_menu_connection.is_none() {
        if let Some(pos) = canvas.context_menu_pos {
            let menu_rect = Rect::from_min_size(pos, Vec2::new(180.0, 250.0));

            if ui.input(|i| i.pointer.any_click())
                && !menu_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
            {
                canvas.context_menu_pos = None;
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
                            canvas.active_module_id,
                        );
                    });
                });
            }
        }
    }
}
