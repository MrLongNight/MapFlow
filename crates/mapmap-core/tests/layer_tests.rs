use glam::{Vec2, Vec3};
use mapmap_core::layer::{Layer, Transform};

#[test]
fn test_transform_matrix_identity() {
    let t = Transform::identity();
    let mat = t.to_matrix(Vec2::new(100.0, 100.0));
    assert_eq!(mat, glam::Mat4::IDENTITY);
}

#[test]
fn test_transform_matrix_translation() {
    let t = Transform::with_position(Vec2::new(10.0, 20.0));
    let mat = t.to_matrix(Vec2::new(100.0, 100.0));

    // Check resulting translation component
    let v = mat.transform_point3(Vec3::new(0.0, 0.0, 0.0));
    // The previous test confirmed this results in (10, 20)
    assert_eq!(v.x, 10.0);
    assert_eq!(v.y, 20.0);
}

#[test]
fn test_transform_matrix_scale() {
    let t = Transform::with_scale(Vec2::new(2.0, 2.0));
    // Center anchor (default)
    let content_size = Vec2::new(100.0, 100.0);
    let mat = t.to_matrix(content_size);

    // Center point (50, 50) logic:
    // With default anchor (0.5, 0.5), to_matrix moves origin to center.
    // If input geometry is defined relative to top-left (0..100), then (50,50) is the "anchor point".
    // 1. Translate(-AnchorOffset). Anchor (0.5,0.5) of 100x100 is offset (0,0) relative to CENTER.
    // Wait, the logic is: anchor_offset = content_size * (anchor - 0.5).
    // If anchor is (0.5, 0.5), anchor_offset is (0,0).
    // The translation is `Mat4::from_translation(Vec3::new(-anchor_offset.x, -anchor_offset.y, 0.0))`.
    // So if anchor is center, this first translation is IDENTITY.
    // Then Scale(2).
    // Then Translate(anchor_offset + position).
    // So for anchor=center, the matrix is just Scale(2).
    //
    // If input is (50, 50). Scale(2) -> (100, 100).
    // If input is (0, 0). Scale(2) -> (0, 0).
    //
    // This means scaling happens from origin (0,0).
    // If the geometry is 0..100, it scales to 0..200.
    // The center moves from (50,50) to (100,100).

    // So if we want to scale around the CENTER, we must ensure the geometry center aligns with the scaling origin?
    // OR `to_matrix` should apply an offset to center the geometry first?
    // The current implementation of `to_matrix` ONLY handles the anchor offset relative to the CENTER of the content_size.
    // BUT it assumes the incoming coordinates have their origin at the CENTER?
    //
    // Let's re-read `to_matrix`:
    // `let anchor_offset = content_size * (self.anchor - Vec2::splat(0.5));`
    // If anchor is (0.5, 0.5), offset is (0,0).
    // `translate_to_anchor` = Translation(0,0).
    // `scale` = Scale(2).
    // `translate_final` = Translation(0,0).
    // Result: Scale(2).
    //
    // If anchor is (0,0) [Top-Left]. Offset is 100 * -0.5 = -50.
    // `translate_to_anchor` = Translation(-(-50)) = (+50, +50).
    // `translate_final` = Translation(-50, -50).
    //
    // Input (0,0): (+50, +50) -> (50, 50). Scale(2) -> (100, 100). Final(-50) -> (50, 50).
    // Input (50,50): (+50, +50) -> (100, 100). Scale(2) -> (200, 200). Final(-50) -> (150, 150).
    //
    // It seems `to_matrix` treats the geometry as if (0,0) is Top-Left, but scaling/rotating happens around the Anchor Point.
    // And it attempts to align the Anchor Point to the Origin (0,0) for the transform, then put it back.
    //
    // If Anchor is Center (50, 50) of a 100x100 box.
    // It calculates offset from Center as (0,0).
    // It implies the "Center" of the object is implicitly (0,0) in the transform space?
    // But if we pass (0,0) as input, it stays (0,0) after "Translate to Anchor" (which is identity).
    // So (0,0) is treated as the anchor point?
    // But (0,0) is Top-Left. Anchor is Center.
    //
    // The issue: `to_matrix` assumes the coordinate system origin is the CENTER of the object.
    // i.e. vertices are -50..+50.
    // If we pass 0..100 vertices, it scales from (0,0) [Top-Left] when Anchor is Center.
    // This is inconsistent.

    // IF the intention of `to_matrix` is to work with 0..100 coords (Top-Left origin):
    // Anchor (0.5, 0.5) corresponds to point (50, 50).
    // We want (50, 50) to be the fixed point of scaling.
    // 1. Translate(-50, -50) -> Moves (50, 50) to (0, 0).
    // 2. Scale(2). (0, 0) stays (0, 0).
    // 3. Translate(+50, +50) -> Moves (0, 0) back to (50, 50).
    //
    // Let's see if `to_matrix` does this.
    // Offset = 0.
    // T1 = 0.
    // It does NOT do this.

    // IT SEEMS `to_matrix` IS BUGGY or assumes Centered Coordinates (-W/2 .. W/2).
    // If it assumes Centered Coordinates:
    // Anchor (0.5, 0.5) [Center] -> Point (0,0).
    // Offset = 0.
    // T1 = 0.
    // Fixed point is (0,0). Correct.

    // Anchor (0, 0) [Top-Left] -> Point (-50, -50).
    // Offset = -50.
    // T1 = Translation(50, 50).
    // Input (-50, -50) -> (0, 0).
    // Fixed point is (0, 0). Correct.

    // CONCLUSION: The transform system assumes input coordinates are centered (i.e., local space is -Size/2 to +Size/2).
    // So (0,0) is the center of the object.
    //
    // When using this matrix, we must feed it coordinates relative to the center.
    // Center point (50, 50) in screen space corresponds to (0, 0) in local space.
    //
    // So for the test:
    // Center point is (0,0,0).
    let v_center = mat.transform_point3(Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(v_center.x, 0.0);
    assert_eq!(v_center.y, 0.0);

    // Top-Left (-50, -50). Scale(2) -> (-100, -100).
    let v_tl = mat.transform_point3(Vec3::new(-50.0, -50.0, 0.0));
    assert_eq!(v_tl.x, -100.0);
    assert_eq!(v_tl.y, -100.0);
}

#[test]
fn test_transform_matrix_rotation() {
    let t = Transform::with_rotation_z(std::f32::consts::PI / 2.0); // 90 degrees
    let content_size = Vec2::new(100.0, 100.0);
    let mat = t.to_matrix(content_size);

    // With centered coordinates:
    // Center (0,0) -> Rotates to (0,0).
    let v_center = mat.transform_point3(Vec3::new(0.0, 0.0, 0.0));
    assert!(v_center.abs_diff_eq(Vec3::ZERO, 0.001));

    // Top-Left (-50, -50).
    // Rotate 90 deg (CCW?): x' = -y, y' = x ? or x' = x cos - y sin...
    // cos(90) = 0, sin(90) = 1.
    // x' = -50*0 - (-50)*1 = 50.
    // y' = -50*1 + -50*0 = -50.
    // So (-50, -50) -> (50, -50) [Top-Right].
    let v_tl = mat.transform_point3(Vec3::new(-50.0, -50.0, 0.0));
    assert!((v_tl.x - 50.0).abs() < 0.001);
    assert!((v_tl.y + 50.0).abs() < 0.001);
}

#[test]
fn test_transform_anchor_change() {
    let mut t = Transform::identity();
    t.anchor = Vec2::new(0.0, 0.0); // Top-left anchor
    t.scale = Vec2::new(2.0, 2.0);

    let content_size = Vec2::new(100.0, 100.0);
    let mat = t.to_matrix(content_size);

    // Local coords (centered):
    // Top-Left is (-50, -50).
    // Anchor (0,0) corresponds to (-50, -50).
    // Offset = -50.
    // T1 = Translation(50, 50).
    // (-50, -50) -> (0, 0).
    // Scale(2) -> (0, 0).
    // T2 = Translation(-50, -50).
    // (0, 0) -> (-50, -50).
    // So (-50, -50) is fixed. Correct.

    let v_tl = mat.transform_point3(Vec3::new(-50.0, -50.0, 0.0));
    assert_eq!(v_tl.x, -50.0);
    assert_eq!(v_tl.y, -50.0);

    // Center (0, 0).
    // (0, 0) -> (50, 50).
    // Scale(2) -> (100, 100).
    // (-50) -> (50, 50).
    // So (0, 0) moves to (50, 50).
    let v_center = mat.transform_point3(Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(v_center.x, 50.0);
    assert_eq!(v_center.y, 50.0);
}

#[test]
fn test_layer_composition_defaults() {
    let layer = Layer::new(1, "Test");
    // Ensure transform is identity by default
    assert_eq!(layer.transform, Transform::default());
    // Ensure legacy transform is identity
    assert_eq!(layer.legacy_transform, glam::Mat4::IDENTITY);
}
