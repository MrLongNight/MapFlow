use super::state::ModuleCanvas;
use egui::{Color32, Pos2, Sense, Stroke, Ui, Vec2};

impl ModuleCanvas {
    /// Render the 2D Spatial Editor for Hue lamps
    pub fn render_hue_spatial_editor(
        &self,
        ui: &mut Ui,
        lamp_positions: &mut std::collections::HashMap<String, (f32, f32)>,
    ) {
        let editor_size = Vec2::new(300.0, 300.0);
        let (response, painter) = ui.allocate_painter(editor_size, Sense::click_and_drag());
        let rect = response.rect;

        // Draw background (Room representation)
        painter.rect_filled(rect, 4.0, Color32::from_gray(30));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::GRAY),
            egui::StrokeKind::Middle,
        );

        // Draw grid
        let grid_steps = 5;
        for i in 1..grid_steps {
            let t = i as f32 / grid_steps as f32;
            let x = rect.min.x + t * rect.width();
            let y = rect.min.y + t * rect.height();

            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(1.0, Color32::from_white_alpha(20)),
            );
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0, Color32::from_white_alpha(20)),
            );
        }

        // Labels
        painter.text(
            rect.center_top() + Vec2::new(0.0, 10.0),
            egui::Align2::CENTER_TOP,
            "Front (TV/Screen)",
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );

        // If empty, add dummy lamps for visualization/testing
        if lamp_positions.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No Lamps Mapped",
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
            // Typically we would populate this from the Entertainment Area config
            if ui.button("Add Test Lamps").clicked() {
                lamp_positions.insert("1".to_string(), (0.2, 0.2)); // Front Left
                lamp_positions.insert("2".to_string(), (0.8, 0.2)); // Front Right
                lamp_positions.insert("3".to_string(), (0.2, 0.8)); // Rear Left
                lamp_positions.insert("4".to_string(), (0.8, 0.8)); // Rear Right
            }
            return;
        }

        let to_screen = |x: f32, y: f32| -> Pos2 {
            Pos2::new(
                rect.min.x + x.clamp(0.0, 1.0) * rect.width(),
                rect.min.y + y.clamp(0.0, 1.0) * rect.height(),
            )
        };

        // Handle lamp dragging
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let _is_dragging = ui.input(|i| i.pointer.primary_down());

        let mut dragged_lamp = None;

        // If dragging, find closest lamp
        if response.dragged() {
            if let Some(pos) = pointer_pos {
                // Find closest lamp within radius
                let mut min_dist = f32::MAX;
                let mut closest_id = None;

                for (id, (lx, ly)) in lamp_positions.iter() {
                    let lamp_pos = to_screen(*lx, *ly);
                    let dist = lamp_pos.distance(pos);
                    if dist < 20.0 && dist < min_dist {
                        min_dist = dist;
                        closest_id = Some(id.clone());
                    }
                }

                if let Some(id) = closest_id {
                    dragged_lamp = Some(id);
                }
            }
        }

        if let Some(id) = dragged_lamp {
            if let Some(pos) = pointer_pos {
                // Update position
                let nx = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                let ny = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                lamp_positions.insert(id, (nx, ny));
            }
        }

        // Draw Lamps
        for (id, (lx, ly)) in lamp_positions.iter() {
            let pos = to_screen(*lx, *ly);

            // Draw lamp body
            painter.circle_filled(pos, 8.0, Color32::from_rgb(255, 200, 100));
            painter.circle_stroke(pos, 8.0, Stroke::new(2.0, Color32::WHITE));

            // Draw Label
            painter.text(
                pos + Vec2::new(0.0, 12.0),
                egui::Align2::CENTER_TOP,
                id,
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );
        }
    }

    /// Render Hue bridge discovery UI
    #[rustfmt::skip]
    pub fn render_hue_bridge_discovery(&mut self, ui: &mut egui::Ui, current_ip: &mut String) {
        if ui.button("🔍 Discover Bridges").clicked() {
            let (tx, rx) = std::sync::mpsc::channel();
            self.hue_discovery_rx = Some(rx);
            // Spawn async task
            #[cfg(feature = "tokio")]
            {
                self.hue_status_message = Some("Searching...".to_string());
                let task = async move {
                    let result = mapmap_control::hue::api::discovery::discover_bridges().await
                        .map_err(|e| e.to_string());
                    let _ = tx.send(result);
                };
                tokio::spawn(task);
            }
            #[cfg(not(feature = "tokio"))]
            {
                let _ = tx;
                self.hue_status_message = Some("Async runtime not available".to_string());
            }
        }

        if !self.hue_bridges.is_empty() {
            ui.separator();
            ui.label("Select Bridge:");
            for bridge in &self.hue_bridges {
                if ui
                    .button(format!("{} ({})", bridge.id, bridge.ip))
                    .clicked()
                {
                    *current_ip = bridge.ip.clone();
                }
            }
        }
    }
}
