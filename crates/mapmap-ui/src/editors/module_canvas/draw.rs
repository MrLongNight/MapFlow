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
    let _primary_clicked = ui.input(|i| i.pointer.primary_clicked());

    let mut remove_idx = None;

    for (conn_idx, conn) in module.connections.iter().enumerate() {
        // Find source and target parts
        let from_part = module.parts.iter().find(|p| p.id == conn.from_part);
        let to_part = module.parts.iter().find(|p| p.id == conn.to_part);

        if let (Some(from), Some(to)) = (from_part, to_part) {
            // Determine cable color based on socket type
            let socket_type = if let Some(socket) = from.outputs.get(conn.from_socket) {
                &socket.socket_type
            } else if let Some(socket) = to.inputs.get(conn.to_socket) {
                &socket.socket_type
            } else {
                &mapmap_core::module::ModuleSocketType::Media // Fallback
            };
            let cable_color = utils::get_socket_color(socket_type);

            // Calculate WORLD positions
            // Output: Right side + center of socket height
            let from_local_y = title_height
                + socket_offset_y
                + conn.from_socket as f32 * socket_spacing
                + socket_spacing / 2.0;
            let from_socket_world =
                Pos2::new(from.position.0 + node_width, from.position.1 + from_local_y);

            // Input: Left side + center of socket height
            let to_local_y = title_height
                + socket_offset_y
                + conn.to_socket as f32 * socket_spacing
                + socket_spacing / 2.0;
            let to_socket_world = Pos2::new(to.position.0, to.position.1 + to_local_y);

            // Convert to SCREEN positions
            let start_pos = to_screen(from_socket_world);
            let end_pos = to_screen(to_socket_world);

            // Draw Plugs - plugs should point INTO the nodes
            let plug_size = 20.0 * canvas.zoom;

            let icon_name = match socket_type {
                mapmap_core::module::ModuleSocketType::Trigger => "audio-jack.svg",
                mapmap_core::module::ModuleSocketType::Media => "plug.svg",
                mapmap_core::module::ModuleSocketType::Effect => "usb-cable.svg",
                mapmap_core::module::ModuleSocketType::Layer => "power-plug.svg",
                mapmap_core::module::ModuleSocketType::Output => "power-plug.svg",
                mapmap_core::module::ModuleSocketType::Link => "power-plug.svg",
            };

            // Draw Cable (Bezier)
            let cable_start = start_pos;
            let cable_end = end_pos;

            let (ctrl1, ctrl2) =
                geometry::calculate_control_points(cable_start, cable_end, canvas.zoom);

            // Hit Detection (Approximate Bezier with segments)
            let mut is_hovered = false;
            if let Some(pos) = pointer_pos {
                let steps = 20;
                let threshold = 5.0 * canvas.zoom.max(1.0); // Adjust hit area with zoom

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

            // Handle Interaction
            let mut progress = 0.0;
            if is_hovered {
                if secondary_clicked {
                    canvas.context_menu_connection = Some(conn_idx);
                    canvas.context_menu_pos = pointer_pos;
                    canvas.context_menu_part = None;
                }

                // Hold to delete (Alt + Click + Hold)
                let is_interacting = alt_held && ui.input(|i| i.pointer.primary_down());
                let conn_id = ui.id().with(("delete_conn", conn_idx));
                let (triggered, p) = crate::widgets::check_hold_state(ui, conn_id, is_interacting);
                progress = p;

                if triggered {
                    remove_idx = Some(conn_idx);
                }
            }

            // Visual Style
            let (stroke_width, stroke_color, glow_width) = if is_hovered {
                if alt_held {
                    // Destructive Mode
                    if progress > 0.0 {
                        // Animate while holding
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
                    // Normal Hover
                    (3.0 * canvas.zoom, Color32::WHITE, 8.0 * canvas.zoom)
                }
            } else {
                (2.0 * canvas.zoom, cable_color, 6.0 * canvas.zoom)
            };

            // Glow (Behind)
            let glow_stroke = Stroke::new(glow_width, cable_color.linear_multiply(0.3));
            painter.add(CubicBezierShape::from_points_stroke(
                [cable_start, ctrl1, ctrl2, cable_end],
                false,
                Color32::TRANSPARENT,
                glow_stroke,
            ));

            // Core Cable (Front)
            let cable_stroke = Stroke::new(stroke_width, stroke_color);
            painter.add(CubicBezierShape::from_points_stroke(
                [cable_start, ctrl1, ctrl2, cable_end],
                false,
                Color32::TRANSPARENT,
                cable_stroke,
            ));

            // Add flow animation
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
            // Draw Plugs on top of cable
            if let Some(texture) = canvas.plug_icons.get(icon_name) {
                // Source Plug at OUTPUT socket - pointing LEFT (into node)
                let start_rect = Rect::from_center_size(start_pos, Vec2::splat(plug_size));
                // Flip horizontally so plug points left (into node)
                painter.image(
                    texture.id(),
                    start_rect,
                    Rect::from_min_max(Pos2::new(1.0, 0.0), Pos2::new(0.0, 1.0)),
                    Color32::WHITE,
                );

                // Target Plug at INPUT socket - pointing RIGHT (into node)
                let end_rect = Rect::from_center_size(end_pos, Vec2::splat(plug_size));
                // Normal orientation (pointing right into node)
                painter.image(
                    texture.id(),
                    end_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                // Fallback circles
                painter.circle_filled(start_pos, 6.0 * canvas.zoom, cable_color);
                painter.circle_filled(end_pos, 6.0 * canvas.zoom, cable_color);
            }

            // Draw Hold Progress Overlay
            if progress > 0.0 {
                if let Some(pos) = pointer_pos {
                    // Draw arc using overlay painter
                    let overlay_painter = ui.ctx().layer_painter(egui::LayerId::new(
                        egui::Order::Tooltip,
                        ui.id().with("overlay"),
                    ));

                    use std::f32::consts::TAU;
                    let radius = 15.0 * canvas.zoom;
                    let stroke = Stroke::new(3.0 * canvas.zoom, Color32::RED);

                    // Background ring
                    overlay_painter.circle_stroke(
                        pos,
                        radius,
                        Stroke::new(2.0, Color32::RED.linear_multiply(0.2)),
                    );

                    // Progress arc
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

                    // Text hint
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

pub fn draw_part_with_delete(
    canvas: &ModuleCanvas,
    ui: &Ui,
    painter: &egui::Painter,
    part: &ModulePart,
    rect: Rect,
    actions: &mut Vec<UIAction>,
    module_id: ModuleId,
) {
    // Get part color and name based on type
    let (_bg_color, title_color, icon, name) = utils::get_part_style(&part.part_type);
    let category = utils::get_part_category(&part.part_type);

    // Helper: get audio trigger state
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
        // Cast AudioBand enum to usize for indexing
        let index = *band as usize;
        if let Some(val) = canvas.audio_trigger_data.band_energies.get(index) {
            audio_trigger_value = *val;
            is_audio_active = audio_trigger_value > threshold;
        }
    }

    // Check generic trigger value from evaluator
    let generic_trigger_value = canvas
        .last_trigger_values
        .get(&part.id)
        .copied()
        .unwrap_or(0.0);
    let is_generic_active = generic_trigger_value > 0.1;

    // Combine
    let trigger_value = if is_generic_active {
        generic_trigger_value
    } else {
        audio_trigger_value
    };
    let is_active = is_audio_active || is_generic_active;

    // Draw glow effect if active
    if is_active {
        let glow_intensity = (trigger_value * 2.0).min(1.0);
        let base_color =
            Color32::from_rgba_unmultiplied(255, (160.0 * glow_intensity) as u8, 0, 255);

        // Cyber-Glow: Multi-layered sharp strokes
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

        // Inner "Light" border
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

    // MIDI Learn Highlight
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

    // Draw background (Dark Neutral for high contrast)
    let neutral_bg = colors::DARK_GREY;
    painter.rect_filled(rect, 0.0, neutral_bg);

    // Handle drag and drop for Media Files
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

    // Node border
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.5 * canvas.zoom, title_color.linear_multiply(0.8)),
        egui::StrokeKind::Middle,
    );

    // Title bar
    let title_height = 28.0 * canvas.zoom;
    let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), title_height));

    // Title bar background
    painter.rect_filled(title_rect, 0.0, colors::LIGHTER_GREY);

    // Title bar Top Accent Stripe
    let stripe_height = 3.0 * canvas.zoom;
    let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), stripe_height));
    painter.rect_filled(stripe_rect, 0.0, title_color);

    // Title separator line
    painter.line_segment(
        [
            Pos2::new(rect.min.x, rect.min.y + title_height),
            Pos2::new(rect.max.x, rect.min.y + title_height),
        ],
        Stroke::new(1.0, colors::STROKE_GREY),
    );

    // Enhanced Title Rendering
    let mut cursor_x = rect.min.x + 8.0 * canvas.zoom;
    let center_y = title_rect.center().y;

    // 1. Icon
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

    // 2. Category
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

    // 3. Name
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

    // Delete button
    let delete_button_rect = get_delete_button_rect(canvas, rect);

    // Retrieve hold progress for visualization (Mary StyleUX)
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

    // Draw property display based on part type
    let property_text = utils::get_part_property_text(&part.part_type);
    let has_property_text = !property_text.is_empty();

    if has_property_text {
        // Position at the bottom of the node to avoid overlapping sockets
        let property_y = rect.max.y - 10.0 * canvas.zoom;
        painter.text(
            Pos2::new(rect.center().x, property_y),
            egui::Align2::CENTER_CENTER,
            property_text,
            egui::FontId::proportional(10.0 * canvas.zoom),
            Color32::from_gray(180),
        );
    }

    // Draw Media Playback Progress Bar
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

            // Background
            let bar_bg =
                Rect::from_min_size(Pos2::new(bar_x, bar_y), Vec2::new(bar_width, bar_height));
            painter.rect_filled(bar_bg, 2.0 * canvas.zoom, Color32::from_gray(30));

            // Progress
            let progress_width = (progress * bar_width).max(2.0 * canvas.zoom);
            let progress_rect = Rect::from_min_size(
                Pos2::new(bar_x, bar_y),
                Vec2::new(progress_width, bar_height),
            );

            let color = if is_playing {
                Color32::from_rgb(100, 255, 100) // Green
            } else {
                Color32::from_rgb(255, 200, 50) // Yellow/Orange
            };

            painter.rect_filled(progress_rect, 2.0 * canvas.zoom, color);

            // Interaction (Seek)
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

    // Draw audio trigger VU meter and live value display
    if is_audio_trigger {
        let offset_from_bottom = if has_property_text { 28.0 } else { 12.0 };
        let meter_height = 4.0 * canvas.zoom; // Thinner meter
        let meter_y = rect.max.y - (offset_from_bottom * canvas.zoom) - meter_height;
        let meter_width = rect.width() - 20.0 * canvas.zoom;
        let meter_x = rect.min.x + 10.0 * canvas.zoom;

        // Background bar
        let meter_bg = Rect::from_min_size(
            Pos2::new(meter_x, meter_y),
            Vec2::new(meter_width, meter_height),
        );
        painter.rect_filled(meter_bg, 2.0, Color32::from_gray(20));

        // Value bar with Hardware-Segments
        let num_segments = 20;
        let segment_spacing = 1.0 * canvas.zoom;
        let segment_width =
            (meter_width - (num_segments as f32 - 1.0) * segment_spacing) / num_segments as f32;

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
                Color32::from_rgb(0, 255, 100) // Green
            } else if t < 0.85 {
                Color32::from_rgb(255, 180, 0) // Orange
            } else {
                Color32::from_rgb(255, 50, 50) // Red
            };

            painter.rect_filled(seg_rect, 1.0, seg_color);
        }

        // Threshold line
        let threshold_x = meter_x + threshold * meter_width;
        painter.line_segment(
            [
                Pos2::new(threshold_x, meter_y - 2.0),
                Pos2::new(threshold_x, meter_y + meter_height + 2.0),
            ],
            Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 50, 50, 200)),
        );
    }

    // Draw input sockets (left side)
    let socket_start_y = rect.min.y + title_height + 10.0 * canvas.zoom;
    for (i, socket) in part.inputs.iter().enumerate() {
        let socket_y = socket_start_y + i as f32 * 22.0 * canvas.zoom;
        let socket_pos = Pos2::new(rect.min.x, socket_y);
        let socket_radius = 7.0 * canvas.zoom;

        // Socket "Port" style
        let socket_color = utils::get_socket_color(&socket.socket_type);

        // Check hover
        let is_hovered = if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            socket_pos.distance(pointer_pos) < socket_radius * 1.5
        } else {
            false
        };

        // Outer ring (Socket Color)
        let ring_stroke = if is_hovered {
            let pulse = (ui.input(|i| i.time) * 10.0).sin() as f32 * 0.2 + 0.8;
            Stroke::new(3.0 * canvas.zoom, Color32::WHITE.linear_multiply(pulse))
        } else {
            Stroke::new(2.0 * canvas.zoom, socket_color)
        };
        painter.circle_stroke(socket_pos, socket_radius, ring_stroke);
        // Inner hole (Dark)
        painter.circle_filled(
            socket_pos,
            socket_radius - 2.0 * canvas.zoom,
            Color32::from_gray(20),
        );
        // Inner dot (Connector contact)
        painter.circle_filled(
            socket_pos,
            2.0 * canvas.zoom,
            if is_hovered {
                socket_color
            } else {
                Color32::from_gray(100)
            },
        );

        // Socket label
        painter.text(
            Pos2::new(rect.min.x + 14.0 * canvas.zoom, socket_y),
            egui::Align2::LEFT_CENTER,
            &socket.name,
            egui::FontId::proportional(11.0 * canvas.zoom),
            Color32::from_gray(230),
        );
    }

    // Draw output sockets (right side)
    for (i, socket) in part.outputs.iter().enumerate() {
        let socket_y = socket_start_y + i as f32 * 22.0 * canvas.zoom;
        let socket_pos = Pos2::new(rect.max.x, socket_y);
        let socket_radius = 7.0 * canvas.zoom;

        // Socket "Port" style
        let socket_color = utils::get_socket_color(&socket.socket_type);

        // Check hover
        let is_hovered = if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            socket_pos.distance(pointer_pos) < socket_radius * 1.5
        } else {
            false
        };

        // Outer ring (Socket Color)
        let ring_stroke = if is_hovered {
            let pulse = (ui.input(|i| i.time) * 10.0).sin() as f32 * 0.2 + 0.8;
            Stroke::new(3.0 * canvas.zoom, Color32::WHITE.linear_multiply(pulse))
        } else {
            Stroke::new(2.0 * canvas.zoom, socket_color)
        };
        painter.circle_stroke(socket_pos, socket_radius, ring_stroke);
        // Inner hole (Dark)
        painter.circle_filled(
            socket_pos,
            socket_radius - 2.0 * canvas.zoom,
            Color32::from_gray(20),
        );
        // Inner dot (Connector contact)
        painter.circle_filled(
            socket_pos,
            2.0 * canvas.zoom,
            if is_hovered {
                socket_color
            } else {
                Color32::from_gray(100)
            },
        );

        // Socket label
        painter.text(
            Pos2::new(rect.max.x - 14.0 * canvas.zoom, socket_y),
            egui::Align2::RIGHT_CENTER,
            &socket.name,
            egui::FontId::proportional(11.0 * canvas.zoom),
            Color32::from_gray(230),
        );

        // Draw live value meter for output sockets
        // This requires get_socket_live_value which is not extracted yet
        // I will assume it's in utils or just not call it for now if it's complex
        // It's in mod.rs around 5857. It uses module evaluator implicitly or something?
        // Ah, it uses `self.last_trigger_values` but maps it to sockets.
        // It's specific to the canvas. I should implement it here or in utils.
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
    // Search popup in top-center
    let popup_width = 300.0;
    let popup_height = 200.0;
    let popup_rect = Rect::from_min_size(
        Pos2::new(
            canvas_rect.center().x - popup_width / 2.0,
            canvas_rect.min.y + 50.0,
        ),
        Vec2::new(popup_width, popup_height),
    );

    // Draw popup background
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

    // Popup content
    let inner_rect = popup_rect.shrink(10.0);
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut canvas.search_filter);
            });
            ui.add_space(8.0);

            // Filter and show matching nodes
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
                            // Center view on selected node
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
    // Presets popup in top-center
    let popup_width = 280.0;
    let popup_height = 220.0;
    let popup_rect = Rect::from_min_size(
        Pos2::new(
            canvas_rect.center().x - popup_width / 2.0,
            canvas_rect.min.y + 50.0,
        ),
        Vec2::new(popup_width, popup_height),
    );

    // Draw popup background
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

    // Popup content
    let inner_rect = popup_rect.shrink(12.0);
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
        ui.vertical(|ui| {
            ui.heading("📋 Presets / Templates");
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    let presets = canvas.presets.clone();
                    for preset in &presets {
                        ui.horizontal(|ui| {
                            if ui.button(&preset.name).clicked() {
                                // Clear current and load preset
                                module.parts.clear();
                                module.connections.clear();

                                // Add parts from preset
                                let mut part_ids = Vec::new();
                                let mut next_id =
                                    module.parts.iter().map(|p| p.id).max().unwrap_or(0) + 1;
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

                                // Add connections
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
        // Simplified helpers to add nodes directly
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
            if ui.button("⏱️ï¸  Fixed Timer").clicked() {
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
            if ui.button("âŒ¨ï¸  Shortcut").clicked() {
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
            // Effects
            for effect in [
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
