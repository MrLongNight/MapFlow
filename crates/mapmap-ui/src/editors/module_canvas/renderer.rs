use super::controller;
use super::diagnostics;
use super::draw;
use super::state::ModuleCanvas;
use super::types::*;
use super::utils;
use super::ModuleCanvasRenderOptions;
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
    options: ModuleCanvasRenderOptions,
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
        let mut applied = false;
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
                        applied = true;
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
        if applied {
            manager.mark_dirty();
        }
    }

    if let Some(module_id) = canvas.active_module_id {
        render_canvas(canvas, ui, manager, module_id, locale, actions, options);
    } else {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("ðŸ”§ Module Canvas");
                ui.add_space(10.0);
                ui.label("Click 'âž• New Module' to create a module.");
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
    options: ModuleCanvasRenderOptions,
) {
    let mut module_changed = false;
    let mut needs_repair = false;
    let mut open_add_menu = None;
    let quick_create_before_parts;

    {
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

        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::A)) {
            canvas.selected_parts = module.parts.iter().map(|p| p.id).collect();
        }

        if !ui.memory(|m| m.focused().is_some())
            && (ui.input(|i| i.key_pressed(egui::Key::Delete))
                || ui.input(|i| i.key_pressed(egui::Key::Backspace)))
            && !canvas.selected_parts.is_empty()
        {
            controller::safe_delete_selection(canvas, module);
            module_changed = true;
            needs_repair = true;
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
                module_changed = true;
                needs_repair = true;
            }
        }

        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::Y)) && !canvas.redo_stack.is_empty() {
            if let Some(action) = canvas.redo_stack.pop() {
                controller::apply_redo_action(module, &action);
                canvas.undo_stack.push(action);
                module_changed = true;
                needs_repair = true;
            }
        }

        draw::draw_grid(canvas, &painter, canvas_rect);

        let zoom = canvas.zoom;
        let pan_offset = canvas.pan_offset;
        let canvas_min = canvas_rect.min.to_vec2();

        let to_screen = move |pos: Pos2| -> Pos2 { pos * zoom + pan_offset + canvas_min };
        let from_screen = move |pos: Pos2| -> Pos2 { (pos - pan_offset - canvas_min) / zoom };

        let animation_profile = {
            use crate::config::AnimationProfile;
            if options.reduce_motion_enabled {
                AnimationProfile::Off
            } else {
                let dt = ui.input(|i| i.stable_dt).max(0.0001);
                let fps = 1.0 / dt;
                match options.animation_profile {
                    AnimationProfile::Cinematic if fps < 40.0 => AnimationProfile::Subtle,
                    AnimationProfile::Subtle | AnimationProfile::Cinematic if fps < 28.0 => {
                        AnimationProfile::Off
                    }
                    profile => profile,
                }
            }
        };

        if let Some(idx) = super::draw::draw_connections(
            canvas,
            ui,
            &painter,
            module,
            &to_screen,
            options.node_animations_enabled,
            animation_profile,
        ) {
            if idx < module.connections.len() {
                module.connections.remove(idx);
                module_changed = true;
                needs_repair = true;
            }
        }

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

            draw::draw_part_with_delete(
                canvas,
                ui,
                &painter,
                part,
                part_rect,
                actions,
                module.id,
                options.meter_style,
                options.node_animations_enabled,
                animation_profile,
            );

            let part_id = part.id;

            for socket_info in &all_sockets {
                if socket_info.part_id != part_id {
                    continue;
                }

                let socket_rect =
                    Rect::from_center_size(socket_info.position, Vec2::splat(24.0 * canvas.zoom));
                let socket_resp = ui.interact(
                    socket_rect,
                    egui::Id::new((part_id, socket_info.is_output, socket_info.socket_idx)),
                    Sense::click_and_drag(),
                );

                if socket_resp.clicked()
                    && socket_info.is_output
                    && (socket_info.socket_type == mapmap_core::module::ModuleSocketType::Event
                        || socket_info.socket_type
                            == mapmap_core::module::ModuleSocketType::Control)
                {
                    actions.push(UIAction::ManualTrigger(module_id, part_id));
                }

                if socket_resp.drag_started() {
                    canvas.creating_connection = Some((
                        part_id,
                        socket_info.socket_idx,
                        socket_info.is_output,
                        socket_info.socket_type,
                        socket_info.position,
                    ));
                    clicked_on_part = true;
                }

                if socket_resp.hovered() {
                    clicked_on_part = true;
                }
            }

            let interact_rect = part_rect.shrink(2.0);
            let part_response = ui.interact(
                interact_rect,
                egui::Id::new(part_id),
                Sense::click_and_drag(),
            );

            if part_response.hovered() {
                clicked_on_part = true;
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
                    drag_delta = part_response.drag_delta() / canvas.zoom;
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

        if ui.input(|i| i.pointer.any_released()) {
            if let Some((from_part, from_idx, is_output, _from_type, _)) =
                canvas.creating_connection.take()
            {
                if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let mut closest_socket = None;
                    let mut min_dist = 25.0 * canvas.zoom;

                    for target in &all_sockets {
                        let dist = target.position.distance(pointer_pos);
                        if dist < min_dist
                            && target.part_id != from_part
                            && target.is_output != is_output
                        {
                            min_dist = dist;
                            closest_socket = Some(target);
                        }
                    }

                    if let Some(target) = closest_socket {
                        let (out_part, out_idx, in_part, in_idx) = if is_output {
                            let source = module.part(from_part).unwrap();
                            let target_p = module.part(target.part_id).unwrap();
                            (
                                from_part,
                                source.outputs[from_idx].id.clone(),
                                target.part_id,
                                target_p.inputs[target.socket_idx].id.clone(),
                            )
                        } else {
                            let source = module.part(target.part_id).unwrap();
                            let target_p = module.part(from_part).unwrap();
                            (
                                target.part_id,
                                source.outputs[target.socket_idx].id.clone(),
                                from_part,
                                target_p.inputs[from_idx].id.clone(),
                            )
                        };

                        if module
                            .connect_parts(out_part, out_idx, in_part, in_idx)
                            .unwrap_or(false)
                        {
                            module_changed = true;
                            needs_repair = true;
                            ui.ctx().request_repaint();
                        }
                    }
                }
            }
        }

        if drag_delta != Vec2::ZERO {
            for pid in &canvas.selected_parts {
                if let Some(part) = module.parts.iter_mut().find(|part| part.id == *pid) {
                    part.position.0 += drag_delta.x;
                    part.position.1 += drag_delta.y;
                    module_changed = true;
                }
            }
        }

        for (part_id, delta) in resize_ops {
            if let Some(part) = module.parts.iter_mut().find(|part| part.id == part_id) {
                let current_size = part.size.unwrap_or_else(|| {
                    let h = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                    (200.0, h)
                });
                let new_w = (current_size.0 + delta.x).max(100.0);
                let new_h = (current_size.1 + delta.y).max(50.0);
                part.size = Some((new_w, new_h));
                module_changed = true;
            }
        }

        if drag_started_on_empty && !clicked_on_part && !middle_button {
            canvas.panning_canvas = true;
        }

        if let Some(pid) = delete_part_id {
            module
                .connections
                .retain(|connection| connection.from_part != pid && connection.to_part != pid);
            module.parts.retain(|part| part.id != pid);
            module_changed = true;
            needs_repair = true;
        }

        if let Some((from_part_id, from_idx, from_is_output, ref from_type, start_pos)) =
            canvas.creating_connection
        {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                let mut color = utils::get_socket_color(from_type);
                for socket in &all_sockets {
                    if socket.position.distance(pointer_pos) < 15.0 * canvas.zoom {
                        let is_valid = if socket.part_id != from_part_id
                            && socket.is_output != from_is_output
                        {
                            let (out_part, out_idx, in_part, in_idx) = if from_is_output {
                                let source = module.part(from_part_id).unwrap();
                                let target_p = module.part(socket.part_id).unwrap();
                                (
                                    from_part_id,
                                    source.outputs[from_idx].id.clone(),
                                    socket.part_id,
                                    target_p.inputs[socket.socket_idx].id.clone(),
                                )
                            } else {
                                let source = module.part(socket.part_id).unwrap();
                                let target_p = module.part(from_part_id).unwrap();
                                (
                                    socket.part_id,
                                    source.outputs[socket.socket_idx].id.clone(),
                                    from_part_id,
                                    target_p.inputs[from_idx].id.clone(),
                                )
                            };
                            module
                                .validate_connection(out_part, out_idx, in_part, in_idx)
                                .is_ok()
                        } else {
                            false
                        };
                        color = if is_valid {
                            Color32::GREEN
                        } else {
                            Color32::RED
                        };
                        break;
                    }
                }

                let preview_stroke =
                    if options.short_circuit_animation_enabled && color == Color32::RED {
                        let pulse = (ui.input(|i| i.time) as f32 * 10.0).sin().abs();
                        Stroke::new(3.0 + pulse * 2.0, color.gamma_multiply(0.8 + pulse * 0.2))
                    } else {
                        Stroke::new(3.0, color)
                    };

                painter.line_segment([start_pos, pointer_pos], preview_stroke);
                painter.circle_filled(pointer_pos, 5.0, color);
            }
        }

        draw::draw_mini_map(canvas, &painter, canvas_rect, module);

        if canvas.show_search {
            draw::draw_search_popup(canvas, ui, canvas_rect, module);
        }

        if canvas.show_presets {
            let before_parts = module.parts.len();
            let before_connections = module.connections.len();
            draw::draw_presets_popup(canvas, ui, canvas_rect, module);
            if module.parts.len() != before_parts || module.connections.len() != before_connections
            {
                module_changed = true;
                needs_repair = true;
            }
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

        quick_create_before_parts = module.parts.len();

        if let Some(conn_idx) = canvas.context_menu_connection {
            if let Some(pos) = canvas.context_menu_pos {
                let menu_rect = Rect::from_min_size(pos, Vec2::new(150.0, 50.0));
                if ui.input(|i| i.pointer.any_click())
                    && !menu_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
                {
                    canvas.context_menu_connection = None;
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
                        Stroke::new(1.0, Color32::from_rgb(200, 80, 80)),
                        egui::StrokeKind::Middle,
                    );

                    let inner = menu_rect.shrink(8.0);
                    ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                        ui.vertical(|ui| {
                            if crate::widgets::custom::hold_to_action_button(
                                ui,
                                "ðŸ—‘ Delete Connection",
                                crate::theme::colors::ERROR_COLOR,
                                "Delete Connection",
                            ) {
                                if conn_idx < module.connections.len() {
                                    module.connections.remove(conn_idx);
                                    module_changed = true;
                                    needs_repair = true;
                                }
                                canvas.context_menu_connection = None;
                                ui.ctx().request_repaint();
                            }
                        });
                    });
                }
            }
        } else if canvas.context_menu_part.is_none() {
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
                    open_add_menu = Some((inner, from_screen(pos)));
                }
            }
        }

        egui::Area::new(egui::Id::new("canvas_zoom_area"))
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
            .show(ui.ctx(), |ui| {
                crate::widgets::panel::cyber_panel_frame(ui.style()).show(
                    ui,
                    |ui: &mut egui::Ui| {
                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            if ui
                                .button(egui::RichText::new("-").strong())
                                .on_hover_text("Zoom Out")
                                .clicked()
                            {
                                canvas.zoom = (canvas.zoom / 1.2).max(0.1);
                            }

                            ui.add(
                                egui::Slider::new(&mut canvas.zoom, 0.1..=2.0)
                                    .show_value(false)
                                    .trailing_fill(true),
                            );

                            if ui
                                .button(egui::RichText::new("+").strong())
                                .on_hover_text("Zoom In")
                                .clicked()
                            {
                                canvas.zoom = (canvas.zoom * 1.2).min(2.0);
                            }
                            ui.label(
                                egui::RichText::new(format!("{:.0}%", canvas.zoom * 100.0))
                                    .size(11.0)
                                    .color(Color32::WHITE),
                            );
                        });
                    },
                );
            });

        if needs_repair {
            let repair_report = module.repair_graph();
            if repair_report.changed() {
                tracing::warn!(
                    "Module canvas repaired module {} after edit: refreshed_parts={}, removed_connections={}, removed_trigger_targets={}, normalized_parts={}",
                    module.id,
                    repair_report.refreshed_parts,
                    repair_report.removed_connections,
                    repair_report.removed_trigger_targets,
                    repair_report.normalized_parts
                );
                module_changed = true;
            }
        }
    }

    draw::draw_quick_create_popup(canvas, ui, ui.max_rect(), manager, canvas.active_module_id);
    if manager
        .get_module(module_id)
        .map(|module| module.parts.len() != quick_create_before_parts)
        .unwrap_or(false)
    {
        module_changed = true;
    }

    if let Some((inner, canvas_pos)) = open_add_menu {
        let before_parts = manager
            .get_module(module_id)
            .map(|module| module.parts.len())
            .unwrap_or(0);
        ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
            ui.vertical(|ui| {
                ui.heading("\u{2795} Add Node");
                ui.separator();
                draw::render_add_node_menu_content(
                    ui,
                    manager,
                    Some((canvas_pos.x, canvas_pos.y)),
                    canvas.active_module_id,
                );
            });
        });
        if manager
            .get_module(module_id)
            .map(|module| module.parts.len() != before_parts)
            .unwrap_or(false)
        {
            module_changed = true;
        }
    }

    if module_changed {
        if let Some(module) = manager.get_module_mut(module_id) {
            let repair_report = module.repair_graph();
            if repair_report.changed() {
                tracing::warn!(
                    "Module canvas finalized repair for module {}: refreshed_parts={}, removed_connections={}, removed_trigger_targets={}, normalized_parts={}",
                    module.id,
                    repair_report.refreshed_parts,
                    repair_report.removed_connections,
                    repair_report.removed_trigger_targets,
                    repair_report.normalized_parts
                );
            }
        }
        manager.mark_dirty();
    }
}
