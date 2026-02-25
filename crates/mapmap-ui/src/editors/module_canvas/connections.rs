use super::state::ModuleCanvas;
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use egui::epaint::CubicBezierShape;
use mapmap_core::module::{MapFlowModule, ModulePartType, ModuleSocket, ModuleSocketType};

impl ModuleCanvas {
    pub fn draw_connections<F>(
        &mut self,
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
                    &ModuleSocketType::Media // Fallback
                };
                let cable_color = Self::get_socket_color(socket_type);

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
                let plug_size = 20.0 * self.zoom;

                let icon_name = match socket_type {
                    ModuleSocketType::Trigger => "audio-jack.svg",
                    ModuleSocketType::Media => "plug.svg",
                    ModuleSocketType::Effect => "usb-cable.svg",
                    ModuleSocketType::Layer => "power-plug.svg",
                    ModuleSocketType::Output => "power-plug.svg",
                    ModuleSocketType::Link => "power-plug.svg",
                };

                // Draw Cable (Bezier)
                let cable_start = start_pos;
                let cable_end = end_pos;

                let control_offset = (cable_end.x - cable_start.x).abs() * 0.4;
                let control_offset = control_offset.max(40.0 * self.zoom);

                let ctrl1 = Pos2::new(cable_start.x + control_offset, cable_start.y);
                let ctrl2 = Pos2::new(cable_end.x - control_offset, cable_end.y);

                // Hit Detection (Approximate Bezier with segments)
                let mut is_hovered = false;
                if let Some(pos) = pointer_pos {
                    let steps = 20;
                    let threshold = 5.0 * self.zoom.max(1.0); // Adjust hit area with zoom

                    // OPTIMIZATION: Broad-phase AABB Check
                    let min_x =
                        cable_start.x.min(cable_end.x).min(ctrl1.x).min(ctrl2.x) - threshold;
                    let max_x =
                        cable_start.x.max(cable_end.x).max(ctrl1.x).max(ctrl2.x) + threshold;
                    let min_y =
                        cable_start.y.min(cable_end.y).min(ctrl1.y).min(ctrl2.y) - threshold;
                    let max_y =
                        cable_start.y.max(cable_end.y).max(ctrl1.y).max(ctrl2.y) + threshold;

                    let in_aabb =
                        pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y;

                    if in_aabb {
                        // Iterative Bezier calculation (De Casteljau's algorithm logic unrolled/simplified)
                        let mut prev_p = cable_start;
                        for i in 1..=steps {
                            let t = i as f32 / steps as f32;
                            let l1 = cable_start.lerp(ctrl1, t);
                            let l2 = ctrl1.lerp(ctrl2, t);
                            let l3 = ctrl2.lerp(cable_end, t);
                            let q1 = l1.lerp(l2, t);
                            let q2 = l2.lerp(l3, t);
                            let p = q1.lerp(q2, t);

                            // Distance to segment
                            let segment = p - prev_p;
                            let len_sq = segment.length_sq();
                            if len_sq > 0.0 {
                                let t_proj = ((pos - prev_p).dot(segment) / len_sq).clamp(0.0, 1.0);
                                let closest = prev_p + segment * t_proj;
                                if pos.distance(closest) < threshold {
                                    is_hovered = true;
                                    break;
                                }
                            }
                            prev_p = p;
                        }
                    }
                }

                // Handle Interaction
                let mut progress = 0.0;
                if is_hovered {
                    if secondary_clicked {
                        self.context_menu_connection = Some(conn_idx);
                        self.context_menu_pos = pointer_pos;
                        self.context_menu_part = None;
                    }

                    // Hold to delete (Alt + Click + Hold)
                    let is_interacting = alt_held && ui.input(|i| i.pointer.primary_down());
                    let conn_id = ui.id().with(("delete_conn", conn_idx));
                    let (triggered, p) =
                        crate::widgets::check_hold_state(ui, conn_id, is_interacting);
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
                                (4.0 + progress * 4.0) * self.zoom,
                                color,
                                (10.0 + progress * 20.0) * self.zoom,
                            )
                        } else {
                            (4.0 * self.zoom, Color32::RED, 10.0 * self.zoom)
                        }
                    } else {
                        // Normal Hover
                        (3.0 * self.zoom, Color32::WHITE, 8.0 * self.zoom)
                    }
                } else {
                    (2.0 * self.zoom, cable_color, 6.0 * self.zoom)
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
                if self.zoom > 0.6 {
                    let time = ui.input(|i| i.time);
                    let flow_t = (time * 1.5).fract() as f32;
                    let l1 = cable_start.lerp(ctrl1, flow_t);
                    let l2 = ctrl1.lerp(ctrl2, flow_t);
                    let l3 = ctrl2.lerp(cable_end, flow_t);
                    let q1 = l1.lerp(l2, flow_t);
                    let q2 = l2.lerp(l3, flow_t);
                    let flow_pos = q1.lerp(q2, flow_t);

                    painter.circle_filled(
                        flow_pos,
                        3.0 * self.zoom,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 150),
                    );
                }
                // Draw Plugs on top of cable
                if let Some(texture) = self.plug_icons.get(icon_name) {
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
                    painter.circle_filled(start_pos, 6.0 * self.zoom, cable_color);
                    painter.circle_filled(end_pos, 6.0 * self.zoom, cable_color);
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
                        let radius = 15.0 * self.zoom;
                        let stroke = Stroke::new(3.0 * self.zoom, Color32::RED);

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
                            egui::FontId::proportional(10.0 * self.zoom),
                            Color32::RED,
                        );
                    }
                }
            }
        }

        remove_idx
    }

    pub fn get_socket_color(socket_type: &ModuleSocketType) -> Color32 {
        match socket_type {
            ModuleSocketType::Trigger => Color32::from_rgb(180, 100, 220),
            ModuleSocketType::Media => Color32::from_rgb(100, 180, 220),
            ModuleSocketType::Effect => Color32::from_rgb(220, 180, 100),
            ModuleSocketType::Layer => Color32::from_rgb(100, 220, 140),
            ModuleSocketType::Output => Color32::from_rgb(220, 100, 100),
            ModuleSocketType::Link => Color32::from_rgb(200, 200, 200),
        }
    }

    pub fn get_sockets_for_part_type(
        part_type: &ModulePartType,
    ) -> (Vec<ModuleSocket>, Vec<ModuleSocket>) {
        match part_type {
            ModulePartType::Trigger(_) => (
                vec![],
                vec![ModuleSocket {
                    name: "Trigger Out".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
            ),
            ModulePartType::Source(_) => (
                vec![ModuleSocket {
                    name: "Trigger In".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Mask(_) => (
                vec![
                    ModuleSocket {
                        name: "Media In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Mask In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                ],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Modulizer(_) => (
                vec![
                    ModuleSocket {
                        name: "Media In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Trigger In".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Mesh(_) => (vec![], vec![]),
            ModulePartType::Layer(_) => (
                vec![ModuleSocket {
                    name: "Media In".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
                vec![ModuleSocket {
                    name: "Layer Out".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
            ),
            ModulePartType::Output(_) => (
                vec![ModuleSocket {
                    name: "Layer In".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
                vec![],
            ),
            ModulePartType::Hue(_) => (
                vec![
                    ModuleSocket {
                        name: "Brightness".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                    ModuleSocket {
                        name: "Color (RGB)".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Strobe".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![],
            ),
        }
    }
}
