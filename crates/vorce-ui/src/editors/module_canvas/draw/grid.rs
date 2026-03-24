use super::super::state::ModuleCanvas;
use egui::{Color32, Pos2, Rect, Stroke};

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
