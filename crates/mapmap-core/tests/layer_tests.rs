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
    // Default anchor is 0.5 (Center)
    let t = Transform::with_scale(Vec2::new(2.0, 2.0));
    let content_size = Vec2::new(100.0, 100.0);
    let matrix = t.to_matrix(content_size);

    // Center point should stay at center (relative to itself)
    // If the coordinate system is centered at 0,0:
    let center = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let transformed_center = matrix * center;
    assert_eq!(transformed_center, center);

    // A point at (50, 0) (Right edge) should move to (100, 0)
    let right_edge = Vec4::new(50.0, 0.0, 0.0, 1.0);
    let transformed_edge = matrix * right_edge;
    assert_eq!(transformed_edge, Vec4::new(100.0, 0.0, 0.0, 1.0));
}

#[test]
fn test_transform_scale_around_top_left() {
    // Anchor 0,0 (Top Left)
    let mut t = Transform::with_scale(Vec2::new(2.0, 2.0));
    t.anchor = Vec2::ZERO;

    let content_size = Vec2::new(100.0, 100.0);
    let matrix = t.to_matrix(content_size);

    // Logic assumes object is centered at (0,0) in local space
    // Top-Left corner of a 100x100 object is at (-50, -50)
    let top_left = Vec4::new(-50.0, -50.0, 0.0, 1.0);

    // With anchor at Top-Left, the Top-Left corner should NOT move
    let transformed_tl = matrix * top_left;
    assert!((transformed_tl.x - top_left.x).abs() < 1e-5);
    assert!((transformed_tl.y - top_left.y).abs() < 1e-5);

    // The Center (0,0) should move away from Top-Left
    // Distance was (50, 50), now should be (100, 100) from Top-Left (-50, -50)
    // So Center should be at (50, 50)
    let center = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let transformed_center = matrix * center;
    assert!((transformed_center.x - 50.0).abs() < 1e-5);
    assert!((transformed_center.y - 50.0).abs() < 1e-5);
}

#[test]
fn test_transform_rotation_around_center() {
    // Rotate 90 degrees around center
    let t = Transform::with_rotation_z(std::f32::consts::PI / 2.0);
    let matrix = t.to_matrix(Vec2::new(100.0, 100.0));

    // Point (50, 0) -> (0, 50)
    let point = Vec4::new(50.0, 0.0, 0.0, 1.0);
    let transformed = matrix * point;

    assert!(transformed.abs_diff_eq(Vec4::new(0.0, 50.0, 0.0, 1.0), 1e-5));
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
