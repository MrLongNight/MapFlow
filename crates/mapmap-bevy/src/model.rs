use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use crate::components::Bevy3DModel;

pub fn model_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Bevy3DModel), Changed<Bevy3DModel>>,
) {
    for (entity, model) in query.iter() {
        if model.path.is_empty() {
            continue;
        }

        // Load the scene (Bevy's asset server handles caching)
        let scene_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset(model.path.clone()));

        // Create transform
        let transform = Transform::from_xyz(
            model.position[0],
            model.position[1],
            model.position[2],
        )
        .with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            model.rotation[0].to_radians(),
            model.rotation[1].to_radians(),
            model.rotation[2].to_radians(),
        ))
        .with_scale(Vec3::new(
            model.scale[0],
            model.scale[1],
            model.scale[2],
        ));

        // Update entity
        // We insert SceneRoot and Transform.
        // If the entity already has them, they will be updated.
        commands.entity(entity)
            .insert(SceneRoot(scene_handle))
            .insert(transform);
    }
}
