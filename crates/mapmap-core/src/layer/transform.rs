use crate::layer::types::ResizeMode;
use glam::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Transform properties for layers (Phase 1, Month 4)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    /// Position offset in pixels (X, Y)
    pub position: Vec2,
    /// Scale factor (Width, Height) - 1.0 = 100%
    pub scale: Vec2,
    /// Rotation in radians (X, Y, Z) - Euler angles
    pub rotation: Vec3,
    /// Anchor point for transform origin (0-1 normalized, 0.5 = center)
    pub anchor: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: Vec3::ZERO,
            anchor: Vec2::splat(0.5), // Center by default
        }
    }
}

impl Transform {
    /// Create a new identity transform
    pub fn identity() -> Self {
        Self::default()
    }

    /// Create transform with position
    pub fn with_position(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create transform with scale
    pub fn with_scale(scale: Vec2) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Create transform with uniform scale
    pub fn with_uniform_scale(scale: f32) -> Self {
        Self {
            scale: Vec2::splat(scale),
            ..Default::default()
        }
    }

    /// Create transform with rotation (in radians)
    pub fn with_rotation(rotation: Vec3) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    /// Set Z rotation (most common for 2D)
    pub fn with_rotation_z(angle: f32) -> Self {
        Self {
            rotation: Vec3::new(0.0, 0.0, angle),
            ..Default::default()
        }
    }

    /// Calculate 4x4 transformation matrix
    /// Order: Translate → Rotate → Scale (TRS)
    pub fn to_matrix(&self, content_size: Vec2) -> Mat4 {
        // Calculate pivot point (origin for rotation/scale) based on anchor
        // For 0..1 meshes, (0,0) is Top-Left.
        // Pivot needs to be absolute offset from Top-Left.
        let pivot = content_size * self.anchor;

        // Build transformation matrix
        // 1. Translate pivot to origin
        let translate_pivot_to_origin = Mat4::from_translation(Vec3::new(-pivot.x, -pivot.y, 0.0));

        // 2. Scale
        let scale = Mat4::from_scale(Vec3::new(self.scale.x, self.scale.y, 1.0));

        // 3. Rotate (Euler XYZ order)
        let rotation = Mat4::from_euler(
            glam::EulerRot::XYZ,
            self.rotation.x,
            self.rotation.y,
            self.rotation.z,
        );

        // 4. Translate back to pivot + apply final position
        let translate_final = Mat4::from_translation(Vec3::new(
            pivot.x + self.position.x,
            pivot.y + self.position.y,
            0.0,
        ));

        // Combine: Final Translation → Rotation → Scale → Pivot Translation
        translate_final * rotation * scale * translate_pivot_to_origin
    }

    /// Apply resize mode to this transform
    pub fn apply_resize_mode(&mut self, mode: ResizeMode, source_size: Vec2, target_size: Vec2) {
        let (scale, position) = mode.calculate_transform(source_size, target_size);
        self.scale = scale;
        self.position = position;
    }
}
