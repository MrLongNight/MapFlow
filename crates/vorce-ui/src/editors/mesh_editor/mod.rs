//! Phase 6: Advanced Warp Mesh Editor
//!
//! Advanced mesh editing with Bezier control points, subdivision surfaces,
//! symmetry mode, snap to grid/guides, and copy/paste functionality.

mod interaction;
mod state;
mod types;
pub mod ui;

pub use state::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editors::mesh_editor::interaction::MeshEditorInteraction;

    #[test]
    fn test_hit_detection_and_dragging() {
        let mut editor = MeshEditor::new();
        // Create a single vertex at (100, 100)
        editor.vertices.push(Vertex {
            position: egui::Pos2::new(100.0, 100.0),
            control_in: Some(egui::Vec2::new(-20.0, 0.0)),
            control_out: Some(egui::Vec2::new(20.0, 0.0)),
            selected: false,
        });
        editor.mode = EditMode::Bezier;

        // 1. Test Hit Detection (Control Point)
        // Click near control_out (100 + 20 = 120, 100)
        let input_click = InteractionInput {
            pointer_pos: egui::Pos2::new(122.0, 100.0), // Within 6.0 radius
            clicked: false,
            dragged: false,
            drag_delta: egui::Vec2::ZERO,
            drag_started: true,
            drag_stopped: false,
        };
        editor.handle_interaction(input_click);

        // Should be dragging ControlOut of vertex 0
        match editor.dragging_element {
            Some(DragElement::ControlOut(0, _)) => {}
            _ => panic!(
                "Should be dragging ControlOut, got {:?}",
                editor.dragging_element
            ),
        }

        // 2. Test Dragging
        // Drag to (150, 100).
        // Original offset was (120, 100) - (122, 100) = (-2, 0).
        // Target abs pos = (150, 100) + (-2, 0) = (148, 100).
        // Snap OFF.
        // New ctrl = (148, 100) - (100, 100) = (48, 0).
        let input_drag = InteractionInput {
            pointer_pos: egui::Pos2::new(150.0, 100.0),
            clicked: false,
            dragged: true,
            drag_delta: egui::Vec2::new(28.0, 0.0),
            drag_started: false,
            drag_stopped: false,
        };
        editor.handle_interaction(input_drag);

        let v = &editor.vertices[0];
        let ctrl_out = v.control_out.unwrap();
        assert!(
            (ctrl_out.x - 48.0).abs() < 0.001,
            "Expected 48.0, got {}",
            ctrl_out.x
        );

        // 3. Test Drag Stop
        let input_stop = InteractionInput {
            pointer_pos: egui::Pos2::new(150.0, 100.0),
            clicked: false,
            dragged: false,
            drag_delta: egui::Vec2::ZERO,
            drag_started: false,
            drag_stopped: true,
        };
        editor.handle_interaction(input_stop);
        assert!(editor.dragging_element.is_none());
    }

    #[test]
    fn test_grid_snapping() {
        let mut editor = MeshEditor::new();
        editor.vertices.push(Vertex {
            position: egui::Pos2::new(100.0, 100.0),
            control_in: None,
            control_out: None,
            selected: false,
        });
        editor.mode = EditMode::Bezier;
        editor.snap_to_grid = true;
        editor.grid_size = 20.0;

        // Start drag on vertex (100, 100)
        let input_start = InteractionInput {
            pointer_pos: egui::Pos2::new(101.0, 101.0), // Slight offset
            clicked: false,
            dragged: false,
            drag_delta: egui::Vec2::ZERO,
            drag_started: true,
            drag_stopped: false,
        };
        editor.handle_interaction(input_start);

        // Drag to (115, 115)
        // Offset = (100, 100) - (101, 101) = (-1, -1)
        // Target = (115, 115) + (-1, -1) = (114, 114)
        // Snap (114, 114) to grid 20 -> (120, 120) or (100, 120)?
        // 114/20 = 5.7 -> 6 * 20 = 120.
        let input_drag = InteractionInput {
            pointer_pos: egui::Pos2::new(115.0, 115.0),
            clicked: false,
            dragged: true,
            drag_delta: egui::Vec2::ZERO,
            drag_started: false,
            drag_stopped: false,
        };
        editor.handle_interaction(input_drag);

        let v = &editor.vertices[0];
        assert_eq!(v.position, egui::Pos2::new(120.0, 120.0));
    }

    #[test]
    fn test_control_point_creation() {
        let mut editor = MeshEditor::new();
        editor.vertices.push(Vertex {
            position: egui::Pos2::new(100.0, 100.0),
            control_in: None,
            control_out: None,
            selected: false,
        });
        editor.mode = EditMode::Bezier;

        // Click on vertex
        let input_click = InteractionInput {
            pointer_pos: egui::Pos2::new(102.0, 102.0),
            clicked: true,
            dragged: false,
            drag_delta: egui::Vec2::ZERO,
            drag_started: false,
            drag_stopped: false,
        };
        editor.handle_interaction(input_click);

        let v = &editor.vertices[0];
        assert!(v.control_in.is_some());
        assert!(v.control_out.is_some());
        assert_eq!(v.control_in.unwrap(), egui::Vec2::new(-30.0, 0.0));
    }
}
