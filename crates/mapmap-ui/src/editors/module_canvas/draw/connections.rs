#![allow(clippy::too_many_arguments)]
use super::super::{geometry, state::ModuleCanvas, utils};
use egui::epaint::CubicBezierShape;
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use mapmap_core::module::MapFlowModule;

pub fn draw_connections<F>(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    painter: &egui::Painter,
    module: &MapFlowModule,
    to_screen: &F,
    node_animations_enabled: bool,
    animation_profile: crate::config::AnimationProfile,
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

            if node_animations_enabled
                && animation_profile != crate::config::AnimationProfile::Off
                && canvas.zoom > 0.6
            {
                let time = ui.input(|i| i.time);
                let flow_speed = match animation_profile {
                    crate::config::AnimationProfile::Subtle => 1.2,
                    crate::config::AnimationProfile::Cinematic => 2.2,
                    crate::config::AnimationProfile::Off => 0.0,
                };
                let flow_t = (time * flow_speed).fract() as f32;
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
