use egui::{Pos2, Rect, Vec2};
use vorce_core::module::{ModulePart, ModulePartType};

pub fn auto_layout_parts(parts: &mut [ModulePart]) {
    // Sort parts by type category for left-to-right flow
    let type_order = |pt: &ModulePartType| -> usize {
        match pt {
            ModulePartType::Trigger(_) => 0,
            ModulePartType::Source(_) => 1,
            ModulePartType::Mask(_) => 2,
            ModulePartType::Modulizer(_) => 3,
            ModulePartType::Mesh(_) => 4,
            ModulePartType::Layer(_) => 5,
            ModulePartType::Output(_) => 6,
            ModulePartType::Hue(_) => 7,
        }
    };

    // Group parts by type
    let mut columns: [Vec<usize>; 8] = Default::default();
    for (i, part) in parts.iter().enumerate() {
        let col = type_order(&part.part_type);
        columns[col].push(i);
    }

    // Layout parameters
    let node_width = 200.0;
    let node_height = 120.0;
    let h_spacing = 100.0;
    let v_spacing = 60.0;
    let start_x = 50.0;
    let start_y = 50.0;

    // Position each column
    let mut x = start_x;
    for col in &columns {
        if col.is_empty() {
            continue;
        }

        let mut y = start_y;
        for &part_idx in col {
            parts[part_idx].position = (x, y);
            y += node_height + v_spacing;
        }

        x += node_width + h_spacing;
    }
}

pub fn find_free_position(parts: &[ModulePart], preferred: (f32, f32)) -> (f32, f32) {
    let node_width = 200.0;
    let node_height = 130.0;
    let grid_step = 30.0;

    let mut pos = preferred;
    let mut attempts = 0;

    loop {
        let new_rect =
            Rect::from_min_size(Pos2::new(pos.0, pos.1), Vec2::new(node_width, node_height));

        let has_collision = parts.iter().any(|part| {
            let part_height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
            let part_rect = Rect::from_min_size(
                Pos2::new(part.position.0, part.position.1),
                Vec2::new(node_width, part_height),
            );
            new_rect.intersects(part_rect)
        });

        if !has_collision {
            return pos;
        }

        attempts += 1;
        if attempts > 100 {
            return (preferred.0, preferred.1 + (parts.len() as f32) * 150.0);
        }

        pos.1 += grid_step;
        if pos.1 > preferred.1 + 500.0 {
            pos.1 = preferred.1;
            pos.0 += node_width + 20.0;
        }
    }
}
