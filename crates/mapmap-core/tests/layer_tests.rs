use glam::{Vec2, Vec4};
use mapmap_core::{
    layer::{ResizeMode, Transform},
    Layer,
};

#[test]
fn test_transform_identity() {
    let t = Transform::identity();
    let point = Vec4::new(10.0, 10.0, 0.0, 1.0);

    let matrix = t.to_matrix(Vec2::new(100.0, 100.0));
    let transformed = matrix * point;

    assert_eq!(transformed, point);
}

#[test]
fn test_transform_position() {
    let t = Transform::with_position(Vec2::new(10.0, 20.0));
    let point = Vec4::new(0.0, 0.0, 0.0, 1.0);

    let matrix = t.to_matrix(Vec2::new(100.0, 100.0));
    let transformed = matrix * point;

    assert_eq!(transformed, Vec4::new(10.0, 20.0, 0.0, 1.0));
}

#[test]
fn test_transform_scale_around_center() {
    // Coordinate System: Top-Left is (0,0). Bottom-Right is (100, 100).
    // Default anchor is 0.5 (Center) -> Pivot at (50, 50).
    let t = Transform::with_scale(Vec2::new(2.0, 2.0));
    let content_size = Vec2::new(100.0, 100.0);
    let matrix = t.to_matrix(content_size);

    // 1. Center Point (50, 50)
    // Relative to Pivot (50, 50) -> (0, 0)
    // Scale 2.0 -> (0, 0)
    // Back to Pivot -> (50, 50)
    let center = Vec4::new(50.0, 50.0, 0.0, 1.0);
    let transformed_center = matrix * center;
    assert_eq!(transformed_center, center);

    // 2. Right Edge Point (100, 50)
    // Relative -> (50, 0)
    // Scale 2.0 -> (100, 0)
    // Back -> (150, 50)
    // So it moves outward by 50 units.
    let right_edge = Vec4::new(100.0, 50.0, 0.0, 1.0);
    let transformed_edge = matrix * right_edge;
    assert_eq!(transformed_edge, Vec4::new(150.0, 50.0, 0.0, 1.0));
}

#[test]
fn test_transform_scale_around_top_left() {
    // Anchor 0,0 (Top Left) -> Pivot at (0, 0)
    let mut t = Transform::with_scale(Vec2::new(2.0, 2.0));
    t.anchor = Vec2::ZERO;

    let content_size = Vec2::new(100.0, 100.0);
    let matrix = t.to_matrix(content_size);

    // 1. Top-Left Point (0, 0)
    // Relative -> (0, 0)
    // Scale -> (0, 0)
    // Back -> (0, 0)
    // Should stay fixed.
    let top_left = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let transformed_tl = matrix * top_left;
    assert!((transformed_tl.x - top_left.x).abs() < 1e-5);
    assert!((transformed_tl.y - top_left.y).abs() < 1e-5);

    // 2. Center Point (50, 50)
    // Relative -> (50, 50)
    // Scale 2.0 -> (100, 100)
    // Back -> (100, 100)
    // Moves away from TL.
    let center = Vec4::new(50.0, 50.0, 0.0, 1.0);
    let transformed_center = matrix * center;
    assert!((transformed_center.x - 100.0).abs() < 1e-5);
    assert!((transformed_center.y - 100.0).abs() < 1e-5);
}

#[test]
fn test_transform_rotation_around_center() {
    // Coordinate System: Top-Left (0,0). Center (50,50).
    // Anchor 0.5 -> Pivot (50, 50).
    // Rotate 90 degrees around Z (CCW).
    let t = Transform::with_rotation_z(std::f32::consts::FRAC_PI_2);
    let matrix = t.to_matrix(Vec2::new(100.0, 100.0));

    // Point: Right Edge (100, 50)
    // Relative to Pivot (50, 50) -> (50, 0)
    // Rotate 90 deg CCW: (x, y) -> (-y, x) => (0, 50)
    // Back to Pivot -> (50, 100) [Bottom Edge]
    let point = Vec4::new(100.0, 50.0, 0.0, 1.0);
    let transformed = matrix * point;

    assert!(transformed.abs_diff_eq(Vec4::new(50.0, 100.0, 0.0, 1.0), 1e-5));
}

#[test]
fn test_transform_apply_resize_mode() {
    let mut t = Transform::identity();
    let source = Vec2::new(100.0, 50.0); // 2:1
    let target = Vec2::new(100.0, 100.0); // 1:1

    // Fill: scale should be max(100/100, 100/50) = 2.0
    t.apply_resize_mode(ResizeMode::Fill, source, target);
    assert_eq!(t.scale, Vec2::splat(2.0));

    // Fit: scale should be min(100/100, 100/50) = 1.0
    t.apply_resize_mode(ResizeMode::Fit, source, target);
    assert_eq!(t.scale, Vec2::splat(1.0));
}

#[test]
fn test_layer_integration_transform() {
    let mut layer = Layer::new(1, "Test Layer");
    layer.transform = Transform::with_position(Vec2::new(100.0, 200.0));

    let matrix = layer.get_transform_matrix(Vec2::new(1920.0, 1080.0));

    // Check translation part of the matrix
    // Col 3 (w_axis) should hold the translation
    assert_eq!(matrix.w_axis.x, 100.0);
    assert_eq!(matrix.w_axis.y, 200.0);
}

#[test]
fn test_layer_manager_crud() {
    let mut manager = mapmap_core::LayerManager::new();
    assert!(manager.is_empty());
    assert_eq!(manager.len(), 0);

    // Create
    let id1 = manager.create_layer("Layer 1");
    assert_eq!(manager.len(), 1);

    let _id2 = manager.create_layer("Layer 2");
    assert_eq!(manager.len(), 2);

    // Read
    let layer1 = manager.get_layer(id1).unwrap();
    assert_eq!(layer1.name, "Layer 1");

    // Update (Rename)
    assert!(manager.rename_layer(id1, "Layer 1 Renamed"));
    assert_eq!(manager.get_layer(id1).unwrap().name, "Layer 1 Renamed");

    // Delete
    let removed = manager.remove_layer(id1).unwrap();
    assert_eq!(removed.id, id1);
    assert_eq!(manager.len(), 1);
    assert!(manager.get_layer(id1).is_none());

    // Clear
    manager.clear();
    assert!(manager.is_empty());
}

#[test]
fn test_layer_manager_group_and_reparent() {
    let mut manager = mapmap_core::LayerManager::new();

    let group_id = manager.create_group("Group 1");
    assert!(manager.get_layer(group_id).unwrap().is_group);

    let child_id = manager.create_layer("Child 1");
    let grandchild_id = manager.create_layer("Grandchild 1");

    // Reparent
    manager.reparent_layer(child_id, Some(group_id));
    manager.reparent_layer(grandchild_id, Some(child_id));

    assert_eq!(
        manager.get_layer(child_id).unwrap().parent_id,
        Some(group_id)
    );
    assert_eq!(
        manager.get_layer(grandchild_id).unwrap().parent_id,
        Some(child_id)
    );

    // Descendant check
    assert!(manager.is_descendant(grandchild_id, group_id));
    assert!(manager.is_descendant(child_id, group_id));
    assert!(!manager.is_descendant(group_id, grandchild_id));

    // Prevent cycles
    manager.reparent_layer(group_id, Some(grandchild_id));
    // Should still be None
    assert_eq!(manager.get_layer(group_id).unwrap().parent_id, None);

    // Remove group should orphan children
    manager.remove_layer(group_id);
    assert_eq!(manager.get_layer(child_id).unwrap().parent_id, None);
}

#[test]
fn test_layer_manager_z_order() {
    let mut manager = mapmap_core::LayerManager::new();
    let id1 = manager.create_layer("1");
    let id2 = manager.create_layer("2");
    let id3 = manager.create_layer("3");

    // Initial order: 1, 2, 3

    // Move up
    assert!(manager.move_layer_up(id2));
    // Order: 1, 3, 2
    let layers = manager.layers();
    assert_eq!(layers[1].id, id3);
    assert_eq!(layers[2].id, id2);

    // Move down
    assert!(manager.move_layer_down(id2));
    // Order: 1, 2, 3
    let layers = manager.layers();
    assert_eq!(layers[1].id, id2);

    // Move to
    assert!(manager.move_layer_to(id1, 2));
    // Order: 2, 3, 1
    let layers = manager.layers();
    assert_eq!(layers[2].id, id1);

    // Swap
    assert!(manager.swap_layers(id1, id2));
    // Order: 1, 3, 2
    let layers = manager.layers();
    assert_eq!(layers[0].id, id1);
    assert_eq!(layers[2].id, id2);
}

#[test]
fn test_layer_manager_effective_opacity() {
    let mut manager = mapmap_core::LayerManager::new();
    manager.composition.master_opacity = 0.5;

    let group_id = manager.create_group("Group");
    manager.get_layer_mut(group_id).unwrap().opacity = 0.8;

    let child_id = manager.create_layer("Child");
    manager.get_layer_mut(child_id).unwrap().opacity = 0.5;
    manager.reparent_layer(child_id, Some(group_id));

    let child = manager.get_layer(child_id).unwrap();
    // 0.5 (child) * 0.8 (group) * 0.5 (master) = 0.2
    assert!((manager.get_effective_opacity(child) - 0.2).abs() < f32::EPSILON);
}

#[test]
fn test_layer_manager_visible_layers() {
    let mut manager = mapmap_core::LayerManager::new();
    let id1 = manager.create_layer("1");
    let id2 = manager.create_layer("2");

    // Set paints so they should render
    manager.get_layer_mut(id1).unwrap().paint_id = Some(1);
    manager.get_layer_mut(id2).unwrap().paint_id = Some(2);

    assert_eq!(manager.visible_layers().count(), 2);

    // Bypass id1
    manager.get_layer_mut(id1).unwrap().toggle_bypass();
    assert_eq!(manager.visible_layers().count(), 1);
    assert_eq!(manager.visible_layers().next().unwrap().id, id2);

    // Unbypass and Solo id1
    manager.get_layer_mut(id1).unwrap().toggle_bypass();
    manager.get_layer_mut(id1).unwrap().toggle_solo();

    // Only id1 should be visible
    let visible: Vec<_> = manager.visible_layers().collect();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, id1);
}

#[test]
fn test_layer_manager_duplicate() {
    let mut manager = mapmap_core::LayerManager::new();
    let id1 = manager.create_layer("Original");
    manager.get_layer_mut(id1).unwrap().opacity = 0.42;

    let new_id = manager.duplicate_layer(id1).unwrap();
    let new_layer = manager.get_layer(new_id).unwrap();

    assert_eq!(new_layer.name, "Original (copy)");
    assert_eq!(new_layer.opacity, 0.42);
    assert_eq!(manager.len(), 2);
}

#[test]
fn test_layer_manager_eject_all() {
    let mut manager = mapmap_core::LayerManager::new();
    let id1 = manager.create_layer("1");
    let id2 = manager.create_layer("2");

    manager.get_layer_mut(id1).unwrap().paint_id = Some(1);
    manager.get_layer_mut(id2).unwrap().paint_id = Some(2);

    manager.eject_all();

    assert!(manager.get_layer(id1).unwrap().paint_id.is_none());
    assert!(manager.get_layer(id2).unwrap().paint_id.is_none());
}
