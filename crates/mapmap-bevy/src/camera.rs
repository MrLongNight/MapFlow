use crate::components::{BevyCameraNode, CameraMode, SharedEngineCamera};
use bevy::prelude::*;

/// System to sync the active camera with the BevyCameraNode configuration
pub fn sync_camera_nodes(
    time: Res<Time>,
    camera_nodes: Query<&BevyCameraNode>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<SharedEngineCamera>>,
) {
    // If no camera node exists, we leave the camera alone (or it stays at default).
    // If multiple exist, we pick the last one iterated.
    if let Some(node) = camera_nodes.iter().last() {
        for (mut transform, mut projection) in cameras.iter_mut() {
            // Update FOV
            if let Projection::Perspective(ref mut persp) = *projection {
                // clamp fov to reasonable values to avoid panic
                let fov_deg = node.fov.clamp(1.0, 179.0);
                persp.fov = fov_deg.to_radians();
            }

            // Update Transform
            match node.mode {
                CameraMode::Static => {
                    // Fixed position looking at target
                    if node.position != node.target {
                        *transform = Transform::from_translation(node.position)
                            .looking_at(node.target, Vec3::Y);
                    } else {
                        *transform = Transform::from_translation(node.position);
                    }
                }
                CameraMode::Orbit => {
                    let t = time.elapsed_secs();
                    let angle = t * node.speed;

                    // Orbit around Y axis
                    // distance defines radius on XZ plane (roughly)
                    // position.y is added as height offset
                    let rotation = Quat::from_rotation_y(angle);
                    let offset = rotation * Vec3::new(0.0, 0.0, node.distance);

                    // Height offset from config
                    let height = Vec3::new(0.0, node.position.y, 0.0);

                    let pos = node.target + offset + height;

                    *transform = Transform::from_translation(pos).looking_at(node.target, Vec3::Y);
                }
                CameraMode::Fly => {
                    let t = time.elapsed_secs();
                    // Fly from 'position' in direction of 'target'
                    // If target == position, use Z-forward
                    let dir = if node.target != node.position {
                        (node.target - node.position).normalize()
                    } else {
                        Vec3::NEG_Z
                    };

                    let pos = node.position + dir * node.speed * t;

                    *transform = Transform::from_translation(pos).looking_to(dir, Vec3::Y);
                }
            }
        }
    }
}
