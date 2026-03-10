use super::geometry;
use super::state::ModuleCanvas;
use super::utils;
use crate::theme::colors;
use crate::UIAction;
use egui::epaint::CubicBezierShape;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::module::{
    BevyCameraMode, BlendModeType, EffectType, HueNodeType, LayerType, MapFlowModule, MaskShape,
    MaskType, ModuleId, ModuleManager, ModulePart, ModulePartType, ModulizerType, OutputType,
    SourceType, TriggerType,
};

pub fn draw_grid(canvas: &ModuleCanvas, painter: &egui::Painter, rect: Rect) {
    let grid_size = 20.0 * canvas.zoom;
    let color = Color32::from_rgb(40, 40, 40);
    let mut x = rect.left() - canvas.pan_offset.x % grid_size;
    while x < rect.right() {
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            Stroke::new(1.0, color),
        );
        x += grid_size;
    }
    let mut y = rect.top() - canvas.pan_offset.y % grid_size;
    while y < rect.bottom() {
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            Stroke::new(1.0, color),
        );
        y += grid_size;
    }
}

pub fn draw_mini_map(
    canvas: &ModuleCanvas,
    painter: &egui::Painter,
    canvas_rect: Rect,
    module: &MapFlowModule,
) {
    if module.parts.is_empty() {
        return;
    }

    // Mini-map size and position
    let map_size = Vec2::new(150.0, 100.0);
    let map_margin = 10.0;
    let map_rect = Rect::from_min_size(
        Pos2::new(
            canvas_rect.max.x - map_size.x - map_margin,
            canvas_rect.max.y - map_size.y - map_margin,
        ),
        map_size,
    );

    // Background
    painter.rect_filled(
        map_rect,
        0.0,
        Color32::from_rgba_unmultiplied(30, 30, 40, 200),
    );
    painter.rect_stroke(
        map_rect,
        0.0,
        Stroke::new(1.0, Color32::from_gray(80)),
        egui::StrokeKind::Middle,
    );

    // Calculate bounds of all parts
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for part in &module.parts {
        let height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
        min_x = min_x.min(part.position.0);
        min_y = min_y.min(part.position.1);
        max_x = max_x.max(part.position.0 + 200.0);
        max_y = max_y.max(part.position.1 + height);
    }

    // Add padding
    let padding = 50.0;
    min_x -= padding;
    min_y -= padding;
    max_x += padding;
    max_y += padding;

    let world_width = (max_x - min_x).max(1.0);
    let world_height = (max_y - min_y).max(1.0);

    // Scale to fit in mini-map
    let scale_x = (map_size.x - 8.0) / world_width;
    let scale_y = (map_size.y - 8.0) / world_height;
    let scale = scale_x.min(scale_y);

    let to_map = |pos: Pos2| -> Pos2 {
        Pos2::new(
            map_rect.min.x + 4.0 + (pos.x - min_x) * scale,
            map_rect.min.y + 4.0 + (pos.y - min_y) * scale,
        )
    };

    // Draw parts as small rectangles
    for part in &module.parts {
        let height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
        let part_min = to_map(Pos2::new(part.position.0, part.position.1));
        let part_max = to_map(Pos2::new(part.position.0 + 200.0, part.position.1 + height));
        let part_rect = Rect::from_min_max(part_min, part_max);

        let (_, title_color, _, _) = utils::get_part_style(&part.part_type);
        painter.rect_filled(part_rect, 1.0, title_color);
    }

    // Draw viewport rectangle
    let viewport_min = to_map(Pos2::new(
        -canvas.pan_offset.x / canvas.zoom,
        -canvas.pan_offset.y / canvas.zoom,
    ));
    let viewport_max = to_map(Pos2::new(
        (-canvas.pan_offset.x + canvas_rect.width()) / canvas.zoom,
        (-canvas.pan_offset.y + canvas_rect.height()) / canvas.zoom,
    ));
    let viewport_rect = Rect::from_min_max(viewport_min, viewport_max).intersect(map_rect);
    painter.rect_stroke(
        viewport_rect,
        0.0,
        Stroke::new(1.5, Color32::WHITE),
        egui::StrokeKind::Middle,
    );
}

pub fn draw_connections<F>(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    painter: &egui::Painter,
    module: &MapFlowModule,
    to_screen: &F,
) -> Option<usize>
where
    F: Fn(Pos2) -> Pos2,
{
    let node_width = 200.0;
    let title_height = 28.0;
    let socket_offset_y = 10.0;
    let socket_spacing = 22.0;
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let secondary_clicked = ui.input(|i| i.pointer.secondary_clicked());
    let alt_held = ui.input(|i| i.modifiers.alt);

    let mut remove_idx = None;

    for (conn_idx, conn) in module.connections.iter().enumerate() {
        let from_part = module.parts.iter().find(|p| p.id == conn.from_part);
        let to_part = module.parts.iter().find(|p| p.id == conn.to_part);

        if let (Some(from), Some(to)) = (from_part, to_part) {
            let socket_type = if let Some(socket) = from.outputs.get(conn.from_socket) {
                &socket.socket_type
            } else if let Some(socket) = to.inputs.get(conn.to_socket) {
                &socket.socket_type
            } else {
                &mapmap_core::module::ModuleSocketType::Media
            };
            let cable_color = utils::get_socket_color(socket_type);

            let from_local_y = title_height
                + socket_offset_y
                + conn.from_socket as f32 * socket_spacing
                + socket_spacing / 2.0;
            let from_socket_world =
                Pos2::new(from.position.0 + node_width, from.position.1 + from_local_y);

            let to_local_y = title_height
                + socket_offset_y
                + conn.to_socket as f32 * socket_spacing
                + socket_spacing / 2.0;
            let to_socket_world = Pos2::new(to.position.0, to.position.1 + to_local_y);

            let start_pos = to_screen(from_socket_world);
            let end_pos = to_screen(to_socket_world);

            let plug_size = 20.0 * canvas.zoom;

            let icon_name = match socket_type {
                mapmap_core::module::ModuleSocketType::Trigger => "audio-jack1.1.svg",
                mapmap_core::module::ModuleSocketType::Media => "plug.svg",
                mapmap_core::module::ModuleSocketType::Effect => "usb-cable.svg",
                mapmap_core::module::ModuleSocketType::Layer => "power-plug.svg",
                mapmap_core::module::ModuleSocketType::Output => "audio-jack_2.svg",
                mapmap_core::module::ModuleSocketType::Link => "audio-jack_1.2.svg",
            };

            let is_new_jack = icon_name == "audio-jack1.1.svg" || icon_name == "audio-jack_2.svg";
            let is_trigger = matches!(socket_type, mapmap_core::module::ModuleSocketType::Trigger);

            let cable_start = start_pos;
            let cable_end = end_pos;

            let (ctrl1, ctrl2) =
                geometry::calculate_control_points(cable_start, cable_end, canvas.zoom);

            let mut is_hovered = false;
            if let Some(pos) = pointer_pos {
                let steps = 20;
                let threshold = 5.0 * canvas.zoom.max(1.0);

                if geometry::is_point_near_cubic_bezier(
                    pos,
                    cable_start,
                    ctrl1,
                    ctrl2,
                    cable_end,
                    threshold,
                    steps,
                ) {
                    is_hovered = true;
                }
            }

            let mut progress = 0.0;
            if is_hovered {
                if secondary_clicked {
                    canvas.context_menu_connection = Some(conn_idx);
                    canvas.context_menu_pos = pointer_pos;
                    canvas.context_menu_part = None;
                }

                let is_interacting = alt_held && ui.input(|i| i.pointer.primary_down());
                let conn_id = ui.id().with(("delete_conn", conn_idx));
                let (triggered, p) = crate::widgets::check_hold_state(ui, conn_id, is_interacting);
                progress = p;

                if triggered {
                    remove_idx = Some(conn_idx);
                }
            }

            let (stroke_width, stroke_color, glow_width) = if is_hovered {
                if alt_held {
                    if progress > 0.0 {
                        let pulse = (ui.input(|i| i.time) * 20.0).sin().abs() as f32;
                        let color = Color32::RED.linear_multiply(0.5 + 0.5 * pulse);
                        (
                            (4.0 + progress * 4.0) * canvas.zoom,
                            color,
                            (10.0 + progress * 20.0) * canvas.zoom,
                        )
                    } else {
                        (4.0 * canvas.zoom, Color32::RED, 10.0 * canvas.zoom)
                    }
                } else {
                    (3.0 * canvas.zoom, Color32::WHITE, 8.0 * canvas.zoom)
                }
            } else {
                (2.0 * canvas.zoom, cable_color, 6.0 * canvas.zoom)
            };

            let glow_stroke = Stroke::new(glow_width, cable_color.linear_multiply(0.3));
            painter.add(CubicBezierShape::from_points_stroke(
                [cable_start, ctrl1, ctrl2, cable_end],
                false,
                Color32::TRANSPARENT,
                glow_stroke,
            ));

            let cable_stroke = Stroke::new(stroke_width, stroke_color);
            painter.add(CubicBezierShape::from_points_stroke(
                [cable_start, ctrl1, ctrl2, cable_end],
                false,
                Color32::TRANSPARENT,
                cable_stroke,
            ));

            if canvas.zoom > 0.6 {
                let time = ui.input(|i| i.time);
                let flow_t = (time * 1.5).fract() as f32;
                let flow_pos = geometry::calculate_cubic_bezier_point(
                    flow_t,
                    cable_start,
                    ctrl1,
                    ctrl2,
                    cable_end,
                );

                painter.circle_filled(
                    flow_pos,
                    3.0 * canvas.zoom,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 150),
                );
            }
            if let Some(texture) = canvas.plug_icons.get(icon_name) {
                use std::f32::consts::PI;

                let draw_rotated =
                    |pos: Pos2,
                     angle: f32,
                     size: f32,
                     uv: Rect,
                     painter: &egui::Painter,
                     texture_id: egui::TextureId| {
                        let mut mesh = egui::Mesh::with_texture(texture_id);
                        let rotation = egui::emath::Rot2::from_angle(angle);
                        let half_size = size / 2.0;

                        let corners = [
                            Pos2::new(-half_size, -half_size),
                            Pos2::new(half_size, -half_size),
                            Pos2::new(half_size, half_size),
                            Pos2::new(-half_size, half_size),
                        ];

                        let uvs = [
                            Pos2::new(uv.min.x, uv.min.y),
                            Pos2::new(uv.max.x, uv.min.y),
                            Pos2::new(uv.max.x, uv.max.y),
                            Pos2::new(uv.min.x, uv.max.y),
                        ];

                        for i in 0..4 {
                            mesh.vertices.push(egui::epaint::Vertex {
                                pos: pos + rotation * corners[i].to_vec2(),
                                uv: uvs[i],
                                color: Color32::WHITE,
                            });
                        }
                        mesh.add_triangle(0, 1, 2);
                        mesh.add_triangle(0, 2, 3);
                        painter.add(mesh);
                    };

                let (source_angle, target_angle) = if is_new_jack {
                    (PI, 0.0)
                } else if is_trigger {
                    (PI + PI / 4.0, 0.0 + PI / 4.0)
                } else {
                    (PI, 0.0)
                };

                draw_rotated(
                    start_pos,
                    source_angle,
                    plug_size,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    painter,
                    texture.id(),
                );

                draw_rotated(
                    end_pos,
                    target_angle,
                    plug_size,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    painter,
                    texture.id(),
                );
            } else {
                painter.circle_filled(start_pos, 6.0 * canvas.zoom, cable_color);
                painter.circle_filled(end_pos, 6.0 * canvas.zoom, cable_color);
            }

            if progress > 0.0 {
                if let Some(pos) = pointer_pos {
                    let overlay_painter = ui.ctx().layer_painter(egui::LayerId::new(
                        egui::Order::Tooltip,
                        ui.id().with("overlay"),
                    ));

                    use std::f32::consts::TAU;
                    let radius = 15.0 * canvas.zoom;
                    let stroke = Stroke::new(3.0 * canvas.zoom, Color32::RED);

                    overlay_painter.circle_stroke(
                        pos,
                        radius,
                        Stroke::new(2.0, Color32::RED.linear_multiply(0.2)),
                    );

                    let start_angle = -TAU / 4.0;
                    let end_angle = start_angle + progress * TAU;
                    let n_points = 32;
                    let points: Vec<Pos2> = (0..=n_points)
                        .map(|i| {
                            let t = i as f32 / n_points as f32;
                            let angle = egui::lerp(start_angle..=end_angle, t);
                            pos + Vec2::new(angle.cos(), angle.sin()) * radius
                        })
                        .collect();

                    overlay_painter.add(egui::Shape::line(points, stroke));

                    overlay_painter.text(
                        pos + Vec2::new(0.0, radius + 5.0),
                        egui::Align2::CENTER_TOP,
                        "HOLD TO DELETE",
                        egui::FontId::proportional(10.0 * canvas.zoom),
                        Color32::RED,
                    );
                }
            }
        }
    }

    remove_idx
}

#[allow(clippy::too_many_arguments)]
pub fn draw_part_with_delete(
    canvas: &ModuleCanvas,
    ui: &Ui,
    painter: &egui::Painter,
    part: &ModulePart,
    rect: Rect,
    actions: &mut Vec<UIAction>,
    module_id: ModuleId,
    meter_style: crate::config::AudioMeterStyle,
) {
    let (_bg_color, title_color, icon, name) = utils::get_part_style(&part.part_type);
    let category = utils::get_part_category(&part.part_type);

    let is_audio_trigger = matches!(
        part.part_type,
        ModulePartType::Trigger(TriggerType::AudioFFT { .. })
    );
    let mut audio_trigger_value = 0.0;
    let mut threshold = 0.0;
    let mut is_audio_active = false;

    if let ModulePartType::Trigger(TriggerType::AudioFFT {
        band, threshold: t, ..
    }) = &part.part_type
    {
        threshold = *t;
        let index = *band as usize;
        if let Some(val) = canvas.audio_trigger_data.band_energies.get(index) {
            audio_trigger_value = *val;
            is_audio_active = audio_trigger_value > threshold;
        }
    }

    let generic_trigger_value = canvas
        .last_trigger_values
        .get(&part.id)
        .copied()
        .unwrap_or(0.0);
    let is_generic_active = generic_trigger_value > 0.1;

    let trigger_value = if is_generic_active {
        generic_trigger_value
    } else {
        audio_trigger_value
    };
    let is_active = is_audio_active || is_generic_active;

    if is_active {
        let glow_intensity = (trigger_value * 2.0).min(1.0);
        let base_color =
            Color32::from_rgba_unmultiplied(255, (160.0 * glow_intensity) as u8, 0, 255);

        for i in 1..=4 {
            let expansion = i as f32 * 1.5 * canvas.zoom;
            let alpha = (100.0 / (i as f32)).min(255.0) as u8;
            let color = base_color
                .linear_multiply(glow_intensity)
                .gamma_multiply(alpha as f32 / 255.0);

            painter.rect_stroke(
                rect.expand(expansion),
                0.0,
                Stroke::new(1.0 * canvas.zoom, color),
                egui::StrokeKind::Middle,
            );
        }

        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(
                2.0 * canvas.zoom,
                Color32::WHITE.gamma_multiply(180.0 * glow_intensity / 255.0),
            ),
            egui::StrokeKind::Middle,
        );
    }

    let is_midi_learn = canvas.midi_learn_part_id == Some(part.id);
    if is_midi_learn {
        let time = ui.input(|i| i.time);
        let pulse = (time * 8.0).sin().abs() as f32;
        let learn_color = Color32::from_rgb(0, 200, 255).linear_multiply(pulse);

        painter.rect_stroke(
            rect.expand(4.0 * canvas.zoom),
            0.0,
            Stroke::new(2.0 * canvas.zoom, learn_color),
            egui::StrokeKind::Middle,
        );

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "WAITING FOR MIDI...",
            egui::FontId::proportional(12.0 * canvas.zoom),
            Color32::WHITE.gamma_multiply(200.0 * pulse / 255.0),
        );
    }

    let neutral_bg = colors::DARK_GREY;
    painter.rect_filled(rect, 0.0, neutral_bg);

    if let mapmap_core::module::ModulePartType::Source(
        mapmap_core::module::SourceType::MediaFile { .. },
    ) = &part.part_type
    {
        if ui.rect_contains_pointer(rect) {
            if let Some(dropped_path) = ui
                .ctx()
                .data(|d| d.get_temp::<std::path::PathBuf>(egui::Id::new("media_path")))
            {
                painter.rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    egui::StrokeKind::Middle,
                );

                if ui.input(|i| i.pointer.any_released()) {
                    actions.push(UIAction::SetMediaFile(
                        module_id,
                        part.id,
                        dropped_path.to_string_lossy().to_string(),
                    ));
                }
            }
        }
    }

    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.5 * canvas.zoom, title_color.linear_multiply(0.8)),
        egui::StrokeKind::Middle,
    );

    let title_height = 28.0 * canvas.zoom;
    let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), title_height));

    painter.rect_filled(title_rect, 0.0, colors::LIGHTER_GREY);

    let stripe_height = 3.0 * canvas.zoom;
    let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), stripe_height));
    painter.rect_filled(stripe_rect, 0.0, title_color);

    painter.line_segment(
        [
            Pos2::new(rect.min.x, rect.min.y + title_height),
            Pos2::new(rect.max.x, rect.min.y + title_height),
        ],
        Stroke::new(1.0, colors::STROKE_GREY),
    );

    let preview_rect = Rect::from_min_max(
        Pos2::new(
            rect.min.x + 2.0 * canvas.zoom,
            rect.min.y + title_height + 2.0 * canvas.zoom,
        ),
        Pos2::new(
            rect.max.x - 2.0 * canvas.zoom,
            rect.max.y - 2.0 * canvas.zoom,
        ),
    );

    if let Some(&texture_id) = canvas.node_previews.get(&(module_id, part.id)) {
        painter.image(
            texture_id,
            preview_rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.rect_filled(preview_rect, 0.0, Color32::from_gray(15));
    }

    let mut cursor_x = rect.min.x + 8.0 * canvas.zoom;
    let center_y = title_rect.center().y;

    let icon_galley = ui.painter().layout_no_wrap(
        icon.to_string(),
        egui::FontId::proportional(16.0 * canvas.zoom),
        Color32::WHITE,
    );
    painter.galley(
        Pos2::new(cursor_x, center_y - icon_galley.size().y / 2.0),
        icon_galley.clone(),
        Color32::WHITE,
    );
    cursor_x += icon_galley.size().x + 6.0 * canvas.zoom;

    let category_text = category.to_uppercase();
    let category_color = Color32::from_white_alpha(160);
    let category_galley = ui.painter().layout_no_wrap(
        category_text,
        egui::FontId::proportional(10.0 * canvas.zoom),
        category_color,
    );
    painter.galley(
        Pos2::new(cursor_x, center_y - category_galley.size().y / 2.0),
        category_galley.clone(),
        category_color,
    );
    cursor_x += category_galley.size().x + 6.0 * canvas.zoom;

    let name_galley = ui.painter().layout_no_wrap(
        name.to_string(),
        egui::FontId::proportional(14.0 * canvas.zoom),
        Color32::WHITE,
    );
    painter.galley(
        Pos2::new(cursor_x, center_y - name_galley.size().y / 2.0),
        name_galley,
        Color32::WHITE,
    );

    let delete_button_rect = get_delete_button_rect(canvas, rect);

    let delete_id = egui::Id::new((part.id, "delete"));
    let progress = ui
        .ctx()
        .data(|d| d.get_temp::<f32>(delete_id.with("progress")))
        .unwrap_or(0.0);

    crate::widgets::custom::draw_safety_radial_fill(
        painter,
        delete_button_rect.center(),
        10.0 * canvas.zoom,
        progress,
        Color32::from_rgb(255, 50, 50),
    );

    painter.text(
        delete_button_rect.center(),
        egui::Align2::CENTER_CENTER,
        "x",
        egui::FontId::proportional(16.0 * canvas.zoom),
        Color32::from_rgba_unmultiplied(255, 100, 100, 200),
    );

    let property_text = utils::get_part_property_text(&part.part_type);
    let has_property_text = !property_text.is_empty();

    if has_property_text {
        let property_y = rect.max.y - 10.0 * canvas.zoom;
        painter.text(
            Pos2::new(rect.center().x, property_y),
            egui::Align2::CENTER_CENTER,
            property_text,
            egui::FontId::proportional(10.0 * canvas.zoom),
            Color32::from_gray(180),
        );
    }

    if let mapmap_core::module::ModulePartType::Source(
        mapmap_core::module::SourceType::MediaFile { .. },
    ) = &part.part_type
    {
        if let Some(info) = canvas.player_info.get(&part.id) {
            let duration = info.duration.max(0.001);
            let progress = (info.current_time / duration).clamp(0.0, 1.0) as f32;
            let is_playing = info.is_playing;

            let offset_from_bottom = if has_property_text { 28.0 } else { 12.0 };
            let bar_height = 4.0 * canvas.zoom;
            let bar_y = rect.max.y - (offset_from_bottom * canvas.zoom) - bar_height;
            let bar_width = rect.width() - 20.0 * canvas.zoom;
            let bar_x = rect.min.x + 10.0 * canvas.zoom;

            let bar_bg =
                Rect::from_min_size(Pos2::new(bar_x, bar_y), Vec2::new(bar_width, bar_height));
            painter.rect_filled(bar_bg, 2.0 * canvas.zoom, Color32::from_gray(30));

            let progress_width = (progress * bar_width).max(2.0 * canvas.zoom);
            let progress_rect = Rect::from_min_size(
                Pos2::new(bar_x, bar_y),
                Vec2::new(progress_width, bar_height),
            );

            let color = if is_playing {
                Color32::from_rgb(100, 255, 100)
            } else {
                Color32::from_rgb(255, 200, 50)
            };

            painter.rect_filled(progress_rect, 2.0 * canvas.zoom, color);

            let interact_rect = bar_bg.expand(6.0 * canvas.zoom);
            let bar_response = ui.interact(
                interact_rect,
                ui.id().with(("seek", part.id)),
                Sense::click_and_drag(),
            );

            if bar_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            if bar_response.clicked() || bar_response.dragged() {
                if let Some(pos) = bar_response.interact_pointer_pos() {
                    let seek_norm = ((pos.x - bar_x) / bar_width).clamp(0.0, 1.0);
                    let seek_s = seek_norm as f64 * duration;
                    actions.push(UIAction::MediaCommand(
                        part.id,
                        super::types::MediaPlaybackCommand::Seek(seek_s),
                    ));
                }
            }
        }
    }

    if is_audio_trigger {
        let offset_from_bottom = if has_property_text { 28.0 } else { 12.0 };

        let meter_height = match meter_style {
            crate::config::AudioMeterStyle::Retro => 12.0 * canvas.zoom,
            crate::config::AudioMeterStyle::Digital => 4.0 * canvas.zoom,
        };

        let meter_y = rect.max.y - (offset_from_bottom * canvas.zoom) - meter_height;
        let meter_width = rect.width() - 20.0 * canvas.zoom;
        let meter_x = rect.min.x + 10.0 * canvas.zoom;

        match meter_style {
            crate::config::AudioMeterStyle::Retro => {
                let meter_bg = Rect::from_min_size(
                    Pos2::new(meter_x, meter_y),
                    Vec2::new(meter_width, meter_height),
                );
                painter.rect_filled(meter_bg, 2.0, Color32::from_rgb(230, 225, 210));

                let arc_rect = meter_bg.shrink(2.0 * canvas.zoom);
                let clamped_val = trigger_value.clamp(0.0, 1.0);
                let pivot = Pos2::new(meter_bg.center().x, meter_bg.max.y + meter_height * 0.5);
                let radius = meter_height * 1.5;

                let start_angle = -40.0_f32.to_radians();
                let end_angle = 40.0_f32.to_radians();
                let needle_angle = start_angle + (end_angle - start_angle) * clamped_val;

                let needle_tip =
                    pivot + Vec2::new(needle_angle.sin() * radius, -needle_angle.cos() * radius);

                let bounded_tip = Pos2::new(
                    needle_tip.x.clamp(arc_rect.min.x, arc_rect.max.x),
                    needle_tip.y.max(arc_rect.min.y),
                );

                painter.line_segment(
                    [
                        Pos2::new(meter_x + meter_width * 0.8, meter_y + meter_height * 0.5),
                        Pos2::new(meter_x + meter_width * 0.95, meter_y + meter_height * 0.5),
                    ],
                    Stroke::new(1.0 * canvas.zoom, Color32::from_rgb(200, 50, 50)),
                );

                let visible_base = Pos2::new(pivot.x, meter_bg.max.y);
                painter.line_segment(
                    [visible_base, bounded_tip],
                    Stroke::new(1.5 * canvas.zoom, Color32::from_rgb(180, 40, 40)),
                );

                painter.rect_stroke(
                    meter_bg,
                    2.0,
                    Stroke::new(1.0, Color32::from_white_alpha(40)),
                    egui::StrokeKind::Inside,
                );
            }
            crate::config::AudioMeterStyle::Digital => {
                let meter_bg = Rect::from_min_size(
                    Pos2::new(meter_x, meter_y),
                    Vec2::new(meter_width, meter_height),
                );
                painter.rect_filled(meter_bg, 2.0, Color32::from_gray(20));

                let num_segments = 20;
                let segment_spacing = 1.0 * canvas.zoom;
                let segment_width = (meter_width - (num_segments as f32 - 1.0) * segment_spacing)
                    / num_segments as f32;

                for i in 0..num_segments {
                    let t = i as f32 / num_segments as f32;
                    if t > trigger_value {
                        break;
                    }

                    let seg_x = meter_x + i as f32 * (segment_width + segment_spacing);
                    let seg_rect = Rect::from_min_size(
                        Pos2::new(seg_x, meter_y),
                        Vec2::new(segment_width, meter_height),
                    );

                    let seg_color = if t < 0.6 {
                        Color32::from_rgb(0, 255, 100)
                    } else if t < 0.85 {
                        Color32::from_rgb(255, 180, 0)
                    } else {
                        Color32::from_rgb(255, 50, 50)
                    };

                    painter.rect_filled(seg_rect, 1.0, seg_color);
                }

                let threshold_x = meter_x + threshold * meter_width;
                painter.line_segment(
                    [
                        Pos2::new(threshold_x, meter_y - 2.0),
                        Pos2::new(threshold_x, meter_y + meter_height + 2.0),
                    ],
                    Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 50, 50, 200)),
                );
            }
        }
    }

    let socket_start_y = rect.min.y + title_height + 10.0 * canvas.zoom;
    for (i, socket) in part.inputs.iter().enumerate() {
        let socket_y = socket_start_y + i as f32 * 22.0 * canvas.zoom;
        let socket_pos = Pos2::new(rect.min.x, socket_y);
        let socket_radius = 7.0 * canvas.zoom;

        let socket_color = utils::get_socket_color(&socket.socket_type);

        let is_hovered = if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            socket_pos.distance(pointer_pos) < socket_radius * 1.5
        } else {
            false
        };

        let ring_stroke = if is_hovered {
            let pulse = (ui.input(|i| i.time) * 10.0).sin() as f32 * 0.2 + 0.8;
            Stroke::new(3.0 * canvas.zoom, Color32::WHITE.linear_multiply(pulse))
        } else {
            Stroke::new(2.0 * canvas.zoom, socket_color)
        };
        painter.circle_stroke(socket_pos, socket_radius, ring_stroke);
        painter.circle_filled(
            socket_pos,
            socket_radius - 2.0 * canvas.zoom,
            Color32::from_gray(20),
        );
        painter.circle_filled(
            socket_pos,
            2.0 * canvas.zoom,
            if is_hovered {
                socket_color
            } else {
                Color32::from_gray(100)
            },
        );

        let type_name = socket.socket_type.name();
        let display_name = if socket
            .name
            .to_lowercase()
            .contains(&type_name.to_lowercase())
        {
            socket.name.clone()
        } else {
            format!("{} ({})", socket.name, type_name)
        };

        painter.text(
            Pos2::new(rect.min.x + 14.0 * canvas.zoom, socket_y),
            egui::Align2::LEFT_CENTER,
            &display_name,
            egui::FontId::proportional(11.0 * canvas.zoom),
            Color32::from_gray(230),
        );
    }

    for (i, socket) in part.outputs.iter().enumerate() {
        let socket_y = socket_start_y + i as f32 * 22.0 * canvas.zoom;
        let socket_pos = Pos2::new(rect.max.x, socket_y);
        let socket_radius = 7.0 * canvas.zoom;

        let socket_color = utils::get_socket_color(&socket.socket_type);

        let is_hovered = if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            socket_pos.distance(pointer_pos) < socket_radius * 1.5
        } else {
            false
        };

        let ring_stroke = if is_hovered {
            let pulse = (ui.input(|i| i.time) * 10.0).sin() as f32 * 0.2 + 0.8;
            Stroke::new(3.0 * canvas.zoom, Color32::WHITE.linear_multiply(pulse))
        } else {
            Stroke::new(2.0 * canvas.zoom, socket_color)
        };
        painter.circle_stroke(socket_pos, socket_radius, ring_stroke);
        painter.circle_filled(
            socket_pos,
            socket_radius - 2.0 * canvas.zoom,
            Color32::from_gray(20),
        );
        painter.circle_filled(
            socket_pos,
            2.0 * canvas.zoom,
            if is_hovered {
                socket_color
            } else {
                Color32::from_gray(100)
            },
        );

        let type_name = socket.socket_type.name();
        let display_name = if socket
            .name
            .to_lowercase()
            .contains(&type_name.to_lowercase())
        {
            socket.name.clone()
        } else {
            format!("{} ({})", socket.name, type_name)
        };

        painter.text(
            Pos2::new(rect.max.x - 14.0 * canvas.zoom, socket_y),
            egui::Align2::RIGHT_CENTER,
            &display_name,
            egui::FontId::proportional(11.0 * canvas.zoom),
            Color32::from_gray(230),
        );
    }
}

pub fn get_delete_button_rect(canvas: &ModuleCanvas, part_rect: Rect) -> Rect {
    let title_height = 28.0 * canvas.zoom;
    Rect::from_center_size(
        Pos2::new(
            part_rect.max.x - 10.0 * canvas.zoom,
            part_rect.min.y + title_height * 0.5,
        ),
        Vec2::splat(20.0 * canvas.zoom),
    )
}

pub fn draw_search_popup(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    canvas_rect: Rect,
    module: &mut MapFlowModule,
) {
    let popup_width = 300.0;
    let popup_height = 200.0;
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
        Color32::from_rgba_unmultiplied(30, 30, 40, 240),
    );
    painter.rect_stroke(
        popup_rect,
        0.0,
        Stroke::new(2.0, Color32::from_rgb(80, 120, 200)),
        egui::StrokeKind::Middle,
    );

    let inner_rect = popup_rect.shrink(10.0);
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut canvas.search_filter);
            });
            ui.add_space(8.0);

            let filter_lower = canvas.search_filter.to_lowercase();
            let matching_parts: Vec<_> = module
                .parts
                .iter()
                .filter(|p| {
                    if filter_lower.is_empty() {
                        return true;
                    }
                    let name = utils::get_part_property_text(&p.part_type).to_lowercase();
                    let (_, _, _, type_name) = utils::get_part_style(&p.part_type);
                    name.contains(&filter_lower) || type_name.to_lowercase().contains(&filter_lower)
                })
                .take(6)
                .collect();

            egui::ScrollArea::vertical()
                .max_height(120.0)
                .show(ui, |ui| {
                    for part in matching_parts {
                        let (_, _, icon, type_name) = utils::get_part_style(&part.part_type);
                        let label = format!(
                            "{} {} - {}",
                            icon,
                            type_name,
                            utils::get_part_property_text(&part.part_type)
                        );
                        if ui
                            .selectable_label(canvas.selected_parts.contains(&part.id), &label)
                            .clicked()
                        {
                            canvas.selected_parts.clear();
                            canvas.selected_parts.push(part.id);
                            canvas.pan_offset =
                                Vec2::new(-part.position.0 + 200.0, -part.position.1 + 150.0);
                            canvas.show_search = false;
                        }
                    }
                });
        });
    });
}

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
                        ui.label("No presets found.");
                    }
                    for preset in &presets {
                        ui.horizontal(|ui| {
                            if ui.button(&preset.name).clicked() {
                                module.parts.clear();
                                module.connections.clear();
                                let mut part_ids = Vec::new();
                                let mut next_id = 1;
                                for (part_type, position, size) in &preset.parts {
                                    let id = next_id;
                                    next_id += 1;
                                    let (inputs, outputs) =
                                        utils::get_sockets_for_part_type(part_type);
                                    module.parts.push(mapmap_core::module::ModulePart {
                                        id,
                                        part_type: part_type.clone(),
                                        position: *position,
                                        size: *size,
                                        inputs,
                                        outputs,
                                        link_data: mapmap_core::module::NodeLinkData::default(),
                                        trigger_targets: std::collections::HashMap::new(),
                                    });
                                    part_ids.push(id);
                                }
                                for (from_idx, from_socket, to_idx, to_socket) in
                                    &preset.connections
                                {
                                    if *from_idx < part_ids.len() && *to_idx < part_ids.len() {
                                        module.connections.push(
                                            mapmap_core::module::ModuleConnection {
                                                from_part: part_ids[*from_idx],
                                                from_socket: *from_socket,
                                                to_part: part_ids[*to_idx],
                                                to_socket: *to_socket,
                                            },
                                        );
                                    }
                                }
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
                            connections.push((from_idx, conn.from_socket, to_idx, conn.to_socket));
                        }
                    }

                    canvas.presets.push(super::types::ModulePreset {
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

pub fn render_add_node_menu_content(
    ui: &mut Ui,
    manager: &mut ModuleManager,
    pos_override: Option<(f32, f32)>,
    active_module_id: Option<u64>,
) {
    let mut module = if let Some(id) = active_module_id {
        manager.get_module_mut(id)
    } else {
        None
    };

    if let Some(module) = &mut module {
        let mut add_node = |part_type: ModulePartType| {
            let preferred_pos = pos_override.unwrap_or((200.0, 200.0));
            let pos = utils::find_free_position(&module.parts, preferred_pos);
            module.add_part_with_type(part_type, pos);
        };

        ui.menu_button("\u{26A1} Triggers", |ui| {
            if ui.button("🥁 Beat").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Beat));
                ui.close();
            }
            if ui.button("\u{1F50A} Audio FFT").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::AudioFFT {
                    band: mapmap_core::module::AudioBand::Bass,
                    threshold: 0.5,
                    output_config: mapmap_core::module::AudioTriggerOutputConfig::default(),
                }));
                ui.close();
            }
            if ui.button("\u{1F3B2} Random").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Random {
                    min_interval_ms: 500,
                    max_interval_ms: 2000,
                    probability: 0.5,
                }));
                ui.close();
            }
            if ui.button("⏱️ Fixed Timer").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Fixed {
                    interval_ms: 1000,
                    offset_ms: 0,
                }));
                ui.close();
            }
            if ui.button("\u{1F3B9} MIDI").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Midi {
                    channel: 1,
                    note: 60,
                    device: String::new(),
                }));
                ui.close();
            }
            if ui.button("\u{1F4E1} OSC").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Osc {
                    address: "/trigger".to_string(),
                }));
                ui.close();
            }
            if ui.button("⌨️ Shortcut").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Shortcut {
                    key_code: "Space".to_string(),
                    modifiers: 0,
                }));
                ui.close();
            }
        });

        ui.menu_button("\u{1F4F9} Sources", |ui| {
            if ui.button("📁 Media File").clicked() {
                add_node(ModulePartType::Source(SourceType::new_media_file(
                    String::new(),
                )));
                ui.close();
            }
            if ui.button("\u{1F3A8} Shader").clicked() {
                add_node(ModulePartType::Source(SourceType::Shader {
                    name: "Default".to_string(),
                    params: Vec::new(),
                }));
                ui.close();
            }
            #[cfg(feature = "ndi")]
            if ui.button("\u{1F4E1} NDI Input").clicked() {
                add_node(ModulePartType::Source(SourceType::NdiInput {
                    source_name: None,
                }));
                ui.close();
            }
            #[cfg(target_os = "windows")]
            if ui.button("\u{1F6B0} Spout Input").clicked() {
                add_node(ModulePartType::Source(SourceType::SpoutInput {
                    sender_name: String::new(),
                }));
                ui.close();
            }
            ui.separator();
            ui.label("Bevy 3D:");
            if ui.button("📝 3D Text").clicked() {
                add_node(ModulePartType::Source(SourceType::Bevy3DText {
                    text: "Hello 3D".to_string(),
                    font_size: 20.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                    position: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    alignment: "Center".to_string(),
                }));
                ui.close();
            }
            if ui.button("\u{1F9CA} 3D Shape").clicked() {
                add_node(ModulePartType::Source(SourceType::Bevy3DShape {
                    shape_type: mapmap_core::module::BevyShapeType::Cube,
                    position: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                    color: [1.0, 0.5, 0.0, 1.0],
                    unlit: false,
                    outline_width: 0.0,
                    outline_color: [1.0, 1.0, 1.0, 1.0],
                }));
                ui.close();
            }
            if ui.button("\u{1F3A5} Camera").clicked() {
                add_node(ModulePartType::Source(SourceType::BevyCamera {
                    mode: BevyCameraMode::Orbit {
                        radius: 10.0,
                        speed: 10.0,
                        target: [0.0, 0.0, 0.0],
                        height: 5.0,
                    },
                    fov: 60.0,
                    active: true,
                }));
                ui.close();
            }
        });

        ui.menu_button("\u{1F3AD} Masks", |ui| {
            if ui.button("\u{2B55} Shape").clicked() {
                add_node(ModulePartType::Mask(MaskType::Shape(MaskShape::Circle)));
                ui.close();
            }
            if ui.button("\u{1F308} Gradient").clicked() {
                add_node(ModulePartType::Mask(MaskType::Gradient {
                    angle: 0.0,
                    softness: 0.5,
                }));
                ui.close();
            }
        });

        ui.menu_button("🎛️ Modulators", |ui| {
            if ui.button("🎚️ Blend Mode").clicked() {
                add_node(ModulePartType::Modulizer(ModulizerType::BlendMode(
                    BlendModeType::Normal,
                )));
                ui.close();
            }
            ui.separator();
            for effect in [
                EffectType::LoadLUT,
                EffectType::Blur,
                EffectType::Pixelate,
                EffectType::Glitch,
                EffectType::Kaleidoscope,
                EffectType::EdgeDetect,
                EffectType::Colorize,
                EffectType::HueShift,
            ] {
                if ui.button(effect.name()).clicked() {
                    add_node(ModulePartType::Modulizer(ModulizerType::Effect {
                        effect_type: effect,
                        params: std::collections::HashMap::new(),
                    }));
                    ui.close();
                }
            }
        });

        ui.menu_button("\u{1F4D1} Layers", |ui| {
            if ui.button("\u{1F4D1} Single Layer").clicked() {
                add_node(ModulePartType::Layer(LayerType::Single {
                    id: 0,
                    name: "New Layer".to_string(),
                    opacity: 1.0,
                    blend_mode: None,
                    mesh: mapmap_core::module::MeshType::default(),
                    mapping_mode: false,
                }));
                ui.close();
            }
            if ui.button("📁 Layer Group").clicked() {
                add_node(ModulePartType::Layer(LayerType::Group {
                    name: "New Group".to_string(),
                    opacity: 1.0,
                    blend_mode: None,
                    mesh: mapmap_core::module::MeshType::default(),
                    mapping_mode: false,
                }));
                ui.close();
            }
            if ui.button("\u{1F4D1} All Layers").clicked() {
                add_node(ModulePartType::Layer(LayerType::All {
                    opacity: 1.0,
                    blend_mode: None,
                }));
                ui.close();
            }
        });

        ui.menu_button("\u{1F4A1} Philips Hue", |ui| {
            if ui.button("\u{1F4A1} Single Lamp").clicked() {
                add_node(ModulePartType::Hue(HueNodeType::SingleLamp {
                    id: String::new(),
                    name: "New Lamp".to_string(),
                    brightness: 1.0,
                    color: [1.0, 1.0, 1.0],
                    effect: None,
                    effect_active: false,
                }));
                ui.close();
            }
        });

        ui.separator();
        if ui.button("\u{1F5BC} Output").clicked() {
            add_node(ModulePartType::Output(OutputType::Projector {
                id: 1,
                name: "Projector 1".to_string(),
                hide_cursor: false,
                target_screen: 0,
                show_in_preview_panel: true,
                extra_preview_window: false,
                output_width: 0,
                output_height: 0,
                output_fps: 60.0,
                ndi_enabled: false,
                ndi_stream_name: String::new(),
            }));
            ui.close();
        }
    }
}

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
