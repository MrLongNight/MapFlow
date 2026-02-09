use crate::components::{AudioReactive, AudioReactiveTarget};
use crate::resources::AudioInputResource;
use bevy::prelude::*;

pub fn audio_reaction_system(
    audio: Res<AudioInputResource>,
    mut query: Query<(
        &AudioReactive,
        &mut Transform,
        Option<&MeshMaterial3d<StandardMaterial>>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (reaction, mut transform, mat_handle_wrapper) in query.iter_mut() {
        let energy = audio.get_energy(&reaction.source);
        let value = reaction.base + (energy * reaction.intensity);

        match reaction.target {
            AudioReactiveTarget::Scale => {
                transform.scale = Vec3::splat(value);
            }
            AudioReactiveTarget::ScaleX => {
                transform.scale.x = value;
            }
            AudioReactiveTarget::ScaleY => {
                transform.scale.y = value;
            }
            AudioReactiveTarget::ScaleZ => {
                transform.scale.z = value;
            }
            AudioReactiveTarget::RotateX => {
                transform.rotation = Quat::from_rotation_x(value);
            }
            AudioReactiveTarget::RotateY => {
                transform.rotation = Quat::from_rotation_y(value);
            }
            AudioReactiveTarget::RotateZ => {
                transform.rotation = Quat::from_rotation_z(value);
            }
            AudioReactiveTarget::PositionY => {
                transform.translation.y = value;
            }
            AudioReactiveTarget::EmissiveIntensity => {
                if let Some(wrapper) = mat_handle_wrapper {
                    if let Some(mat) = materials.get_mut(&wrapper.0) {
                        // Assuming emissive is white, scale intensity.
                        // Simple MVP: Set emissive to white * value
                        mat.emissive = LinearRgba::gray(value);
                    }
                }
            }
        }
    }
}

pub fn setup_3d_scene(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut render_output: ResMut<crate::resources::BevyRenderOutput>,
) {
    // Create render target texture
    let size = bevy::render::render_resource::Extent3d {
        width: 1280,
        height: 720,
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_fill(
        size,
        bevy::render::render_resource::TextureDimension::D2,
        &[0, 0, 0, 255],
        bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );

    image.texture_descriptor.usage = bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT
        | bevy::render::render_resource::TextureUsages::COPY_SRC
        | bevy::render::render_resource::TextureUsages::TEXTURE_BINDING;

    let image_handle = images.add(image);

    render_output.image_handle = image_handle.clone();
    render_output.width = 1280;
    render_output.height = 720;

    // Spawn Shared Engine Camera
    commands
        .spawn((
            Camera3d::default(),
            Camera {
                target: bevy::render::camera::RenderTarget::Image(image_handle.into()),
                ..default()
            },
            Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .insert(crate::components::SharedEngineCamera);

    // Spawn Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

pub fn hex_grid_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<
        (Entity, &crate::components::BevyHexGrid),
        Changed<crate::components::BevyHexGrid>,
    >,
) {
    for (entity, hex_config) in query.iter() {
        // Clear existing children (tiles)
        // commands.entity(entity).despawn_descendants(); // TODO: Fix in Bevy 0.16

        let layout = hexx::HexLayout {
            scale: hexx::Vec2::splat(hex_config.radius),
            orientation: if hex_config.pointy_top {
                hexx::HexOrientation::Pointy
            } else {
                hexx::HexOrientation::Flat
            },
            ..default()
        };

        let mesh = meshes.add(Cuboid::from_size(Vec3::new(
            hex_config.radius * 1.5,
            0.2,
            hex_config.radius * 1.5,
        )));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.8),
            ..default()
        });

        commands.entity(entity).with_children(|parent| {
            for hex in hexx::shapes::hexagon(hexx::Hex::ZERO, hex_config.rings) {
                let pos = layout.hex_to_world_pos(hex);
                parent.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(pos.x, 0.0, pos.y),
                ));
            }
        });
    }
}

pub fn particle_system(
    _commands: Commands,
    query: Query<
        (Entity, &crate::components::BevyParticles),
        Changed<crate::components::BevyParticles>,
    >,
) {
    for (_entity, _p_config) in query.iter() {
        // Update particles logic (Simplified for now)
        // In a real implementation, we would update the bevy_enoki components here.
    }
}

use bevy::render::render_asset::RenderAssets;
use bevy::render::texture::GpuImage;

pub fn frame_readback_system(
    // RenderAssets<GpuImage> maps Handle<Image> -> GpuImage
    _gpu_images: Res<RenderAssets<GpuImage>>,
    _render_output: Res<crate::resources::BevyRenderOutput>,
    _render_device: Res<bevy::render::renderer::RenderDevice>,
    _render_queue: Res<bevy::render::renderer::RenderQueue>,
) {
    // TODO: Re-enable frame readback when Bevy 0.16 / wgpu types are stable and aligned.
    // Currently ImageCopyTexture and friends are missing/renamed in the dependency graph.
}
