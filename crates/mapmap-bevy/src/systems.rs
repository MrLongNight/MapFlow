use crate::components::{AudioReactive, AudioReactiveTarget};
use crate::resources::AudioInputResource;
use bevy::prelude::*;
use rand::Rng;

pub fn audio_reaction_system(
    audio: Res<AudioInputResource>,
    mut query: Query<(
        &AudioReactive,
        &mut Transform,
        Option<&MeshMaterial3d<StandardMaterial>>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (reaction, mut transform, mat_handle) in query.iter_mut() {
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
                if let Some(MeshMaterial3d(handle)) = mat_handle {
                    if let Some(mat) = materials.get_mut(handle) {
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
        commands.entity(entity).despawn_related::<Children>();

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
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(
        Entity,
        &crate::components::BevyParticles,
        Option<&mut crate::components::ParticleEmitter>,
        Option<&Mesh3d>,
    )>,
) {
    let delta_time = time.delta_secs();

    for (entity, config, mut emitter_opt, mesh_opt) in query.iter_mut() {
        // Initialize emitter if missing
        if emitter_opt.is_none() {
            commands
                .entity(entity)
                .insert(crate::components::ParticleEmitter::default());
            continue; // Wait for next frame
        }
        let emitter = emitter_opt.as_mut().unwrap();

        // Initialize mesh if missing
        if mesh_opt.is_none() {
            let mut mesh = Mesh::new(
                bevy::render::mesh::PrimitiveTopology::TriangleList,
                bevy::render::render_asset::RenderAssetUsages::default(),
            );
            // Initial empty buffers to avoid validation errors
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 0]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 0.0]; 0]);

            let mesh_handle = meshes.add(mesh);

            let material_handle = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                alpha_mode: AlphaMode::Add,
                unlit: true,
                ..default()
            });

            commands
                .entity(entity)
                .insert((Mesh3d(mesh_handle), MeshMaterial3d(material_handle)));
            continue;
        }

        // Spawn new particles
        emitter.spawn_accumulator += config.rate * delta_time;
        if emitter.spawn_accumulator > 1.0 {
            let mut rng = rand::rng();
            while emitter.spawn_accumulator > 1.0 {
                emitter.spawn_accumulator -= 1.0;

                let velocity = Vec3::new(
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                )
                .normalize_or_zero()
                    * config.speed;

                emitter.particles.push(crate::components::Particle {
                    position: Vec3::ZERO, // Relative to entity transform
                    velocity,
                    lifetime: config.lifetime,
                    age: 0.0,
                    color_start: LinearRgba::new(
                        config.color_start[0],
                        config.color_start[1],
                        config.color_start[2],
                        config.color_start[3],
                    ),
                    color_end: LinearRgba::new(
                        config.color_end[0],
                        config.color_end[1],
                        config.color_end[2],
                        config.color_end[3],
                    ),
                });
            }
        }

        // Update particles
        emitter.particles.retain_mut(|p| {
            p.age += delta_time;
            p.position += p.velocity * delta_time;
            p.age < p.lifetime
        });

        // Update Mesh
        if let Some(Mesh3d(mesh_handle)) = mesh_opt {
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                let count = emitter.particles.len();

                let mut positions = Vec::with_capacity(count * 4);
                let mut colors = Vec::with_capacity(count * 4);
                let mut indices = Vec::with_capacity(count * 6);

                let half_size = 0.05;

                for (i, p) in emitter.particles.iter().enumerate() {
                    let t = p.age / p.lifetime;
                    // Lerp color
                    let start = p.color_start;
                    let end = p.color_end;
                    let color = LinearRgba::new(
                        start.red + (end.red - start.red) * t,
                        start.green + (end.green - start.green) * t,
                        start.blue + (end.blue - start.blue) * t,
                        start.alpha + (end.alpha - start.alpha) * t,
                    );

                    // Add 4 vertices (Quad facing +Z)
                    positions
                        .push((p.position + Vec3::new(-half_size, -half_size, 0.0)).to_array());
                    positions.push((p.position + Vec3::new(half_size, -half_size, 0.0)).to_array());
                    positions.push((p.position + Vec3::new(half_size, half_size, 0.0)).to_array());
                    positions.push((p.position + Vec3::new(-half_size, half_size, 0.0)).to_array());

                    let c = [color.red, color.green, color.blue, color.alpha];
                    colors.push(c);
                    colors.push(c);
                    colors.push(c);
                    colors.push(c);

                    let base = (i * 4) as u32;
                    indices.push(base);
                    indices.push(base + 1);
                    indices.push(base + 2);
                    indices.push(base + 2);
                    indices.push(base + 3);
                    indices.push(base);
                }

                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
            }
        }
    }
}

use bevy::render::render_asset::RenderAssets;
use bevy::render::texture::GpuImage;

pub fn frame_readback_system(
    // RenderAssets<GpuImage> maps Handle<Image> -> GpuImage
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_output: Res<crate::resources::BevyRenderOutput>,
    render_device: Res<bevy::render::renderer::RenderDevice>,
    render_queue: Res<bevy::render::renderer::RenderQueue>,
    mut buffer_cache: Local<Option<bevy::render::render_resource::Buffer>>,
) {
    if let Some(gpu_image) = gpu_images.get(&render_output.image_handle) {
        let texture = &gpu_image.texture;

        let width = gpu_image.size.width;
        let height = gpu_image.size.height;
        let block_size = gpu_image.texture_format.block_copy_size(None).unwrap();

        // bytes_per_row must be multiple of 256
        let bytes_per_pixel = block_size;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let padding = (256 - (unpadded_bytes_per_row % 256)) % 256;
        let bytes_per_row = unpadded_bytes_per_row + padding;

        let output_buffer_size = (bytes_per_row * height) as u64;

        // Ensure buffer exists and is correct size
        if buffer_cache.is_none() || buffer_cache.as_ref().unwrap().size() != output_buffer_size {
            *buffer_cache = Some(render_device.create_buffer(
                &bevy::render::render_resource::BufferDescriptor {
                    label: Some("Readback Buffer"),
                    size: output_buffer_size,
                    usage: bevy::render::render_resource::BufferUsages::MAP_READ
                        | bevy::render::render_resource::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            ));
        }

        let buffer = buffer_cache.as_ref().unwrap();

        let mut encoder = render_device.create_command_encoder(
            &bevy::render::render_resource::CommandEncoderDescriptor {
                label: Some("Readback Encoder"),
            },
        );

        encoder.copy_texture_to_buffer(
            bevy::render::render_resource::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: bevy::render::render_resource::Origin3d::ZERO,
                aspect: bevy::render::render_resource::TextureAspect::All,
            },
            bevy::render::render_resource::TexelCopyBufferInfo {
                buffer,
                layout: bevy::render::render_resource::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            bevy::render::render_resource::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        render_queue.submit(std::iter::once(encoder.finish()));

        // Blocking map
        let (tx, rx) = std::sync::mpsc::channel();
        let buffer_slice = buffer.slice(..);
        buffer_slice.map_async(bevy::render::render_resource::MapMode::Read, move |res| {
            tx.send(res).unwrap();
        });

        render_device.poll(bevy::render::render_resource::Maintain::Wait);

        if rx.recv().is_ok() {
            let data = buffer_slice.get_mapped_range();

            // Acquire lock to update shared data
            if let Ok(mut lock) = render_output.last_frame_data.lock() {
                // Remove padding if necessary
                if padding == 0 {
                    *lock = Some(data.to_vec());
                } else {
                    // Compact rows
                    let mut unpadded =
                        Vec::with_capacity((width * height * bytes_per_pixel) as usize);
                    for i in 0..height {
                        let offset = (i * bytes_per_row) as usize;
                        let end = offset + (width * bytes_per_pixel) as usize;
                        unpadded.extend_from_slice(&data[offset..end]);
                    }
                    *lock = Some(unpadded);
                }
            }
        }

        // Must unmap to use buffer again next frame for writing
        buffer.unmap();
    }
}

pub fn text_3d_system(
    mut commands: Commands,
    query: Query<(Entity, &crate::components::Bevy3DText), Changed<crate::components::Bevy3DText>>,
) {
    for (entity, config) in query.iter() {
        let justify = match config.alignment {
            crate::components::BevyTextAlignment::Left => JustifyText::Left,
            crate::components::BevyTextAlignment::Center => JustifyText::Center,
            crate::components::BevyTextAlignment::Right => JustifyText::Right,
            crate::components::BevyTextAlignment::Justify => JustifyText::Justified,
        };

        let color = Color::srgba(
            config.color[0],
            config.color[1],
            config.color[2],
            config.color[3],
        );

        commands.entity(entity).insert((
            Text2d::default(),
            Text::new(config.text.clone()),
            TextFont {
                font_size: config.font_size,
                ..default()
            },
            TextColor(color),
            TextLayout {
                justify,
                ..default()
            },
        ));
    }
}
