use crate::components::Bevy3DModel;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

pub fn model_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Bevy3DModel, Option<&SceneRoot>), Changed<Bevy3DModel>>,
) {
    for (entity, model, maybe_scene) in query.iter() {
        if model.path.is_empty() {
            commands.entity(entity).remove::<SceneRoot>();
            continue;
        }

        // Load the scene (Bevy's asset server handles caching)
        let scene_handle =
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(model.path.clone()));

        // Create transform
        let transform =
            Transform::from_xyz(model.position[0], model.position[1], model.position[2])
                .with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    model.rotation[0].to_radians(),
                    model.rotation[1].to_radians(),
                    model.rotation[2].to_radians(),
                ))
                .with_scale(Vec3::new(model.scale[0], model.scale[1], model.scale[2]));

        // Update entity
        let mut entity_cmd = commands.entity(entity);
        entity_cmd.insert(transform);

        // Only update SceneRoot if it changed to avoid respawning the scene hierarchy
        if let Some(current_root) = maybe_scene {
            if current_root.0 != scene_handle {
                entity_cmd.insert(SceneRoot(scene_handle));
            }
        } else {
            entity_cmd.insert(SceneRoot(scene_handle));
        }
    }
}
