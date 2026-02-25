use super::state::ModuleCanvas;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use mapmap_core::module::MapFlowModule;

impl ModuleCanvas {
    pub fn draw_mini_map(
        &self,
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

            let (_, title_color, _, _) = Self::get_part_style(&part.part_type);
            painter.rect_filled(part_rect, 1.0, title_color);
        }

        // Draw viewport rectangle
        let viewport_min = to_map(Pos2::new(
            -self.pan_offset.x / self.zoom,
            -self.pan_offset.y / self.zoom,
        ));
        let viewport_max = to_map(Pos2::new(
            (-self.pan_offset.x + canvas_rect.width()) / self.zoom,
            (-self.pan_offset.y + canvas_rect.height()) / self.zoom,
        ));
        let viewport_rect = Rect::from_min_max(viewport_min, viewport_max).intersect(map_rect);
        painter.rect_stroke(
            viewport_rect,
            0.0,
            Stroke::new(1.5, Color32::WHITE),
            egui::StrokeKind::Middle,
        );
    }

    pub fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
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
}
