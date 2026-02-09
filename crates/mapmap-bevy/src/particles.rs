use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use rand::Rng;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ParticleEmitter>();
        app.add_systems(Update, (
            init_particle_system,
            sync_emitter_params,
            simulate_particles,
            update_particle_mesh
        ));
    }
}

/// Component attached to the entity representing the particle emitter
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ParticleEmitter {
    // Logic parameters (synced from BevyParticles component)
    pub rate: f32,
    pub lifetime: f32,
    pub speed: f32,
    pub color_start: LinearRgba,
    pub color_end: LinearRgba,

    // Internal state
    pub accumulator: f32,
}

/// Data for a single particle
#[derive(Clone, Copy, Debug)]
struct Particle {
    position: Vec3,
    velocity: Vec3,
    age: f32,
    lifetime: f32,
    color_start: LinearRgba,
    color_end: LinearRgba,
}

/// Component holding the simulation state and mesh handle
#[derive(Component)]
pub struct ParticleSystem {
    particles: Vec<Particle>,
    mesh_handle: Handle<Mesh>,
    material_handle: Handle<StandardMaterial>,
    capacity: usize,
}

fn init_particle_system(
    mut commands: Commands,
    query: Query<(Entity, &crate::components::BevyParticles), Added<crate::components::BevyParticles>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, config) in query.iter() {
        // Create a dynamic mesh
        let capacity = 10000; // Max particles
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default()); // Dynamic usage

        let mesh_handle = meshes.add(mesh);

        // Create material (using vertex colors)
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::WHITE, // Multiply with vertex color
            alpha_mode: AlphaMode::Add,
            unlit: true, // Particles usually emit light
            ..default()
        });

        commands.entity(entity).insert((
            ParticleEmitter {
                rate: config.rate,
                lifetime: config.lifetime,
                speed: config.speed,
                color_start: LinearRgba::from_f32_array(config.color_start),
                color_end: LinearRgba::from_f32_array(config.color_end),
                accumulator: 0.0,
            },
            ParticleSystem {
                particles: Vec::with_capacity(capacity),
                mesh_handle: mesh_handle.clone(),
                material_handle: material_handle.clone(),
                capacity,
            },
            // Render components
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
        ));
    }
}

fn sync_emitter_params(
    mut query: Query<(&crate::components::BevyParticles, &mut ParticleEmitter), Changed<crate::components::BevyParticles>>,
) {
    for (config, mut emitter) in query.iter_mut() {
        emitter.rate = config.rate;
        emitter.lifetime = config.lifetime;
        emitter.speed = config.speed;
        emitter.color_start = LinearRgba::from_f32_array(config.color_start);
        emitter.color_end = LinearRgba::from_f32_array(config.color_end);
    }
}

fn simulate_particles(
    time: Res<Time>,
    mut query: Query<(&mut ParticleEmitter, &mut ParticleSystem, &GlobalTransform)>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::rng();

    for (mut emitter, mut system, _transform) in query.iter_mut() {
        // 1. Spawn new particles
        emitter.accumulator += dt * emitter.rate;
        let spawn_count = emitter.accumulator.floor() as usize;
        emitter.accumulator -= spawn_count as f32;

        let spawn_count = spawn_count.min(100); // Limit per frame spawn to avoid freeze

        for _ in 0..spawn_count {
            if system.particles.len() >= system.capacity {
                break;
            }

            let velocity = Vec3::new(
                rng.random::<f32>() - 0.5,
                rng.random::<f32>() - 0.5,
                rng.random::<f32>() - 0.5,
            ).normalize_or_zero() * emitter.speed;

            system.particles.push(Particle {
                position: Vec3::ZERO,
                velocity,
                age: 0.0,
                lifetime: emitter.lifetime,
                color_start: emitter.color_start,
                color_end: emitter.color_end,
            });
        }

        // 2. Update existing particles
        system.particles.retain_mut(|p| {
            p.age += dt;
            if p.age >= p.lifetime {
                return false;
            }

            p.position += p.velocity * dt;
            true
        });
    }
}

fn update_particle_mesh(
    query: Query<&ParticleSystem>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_query: Query<&GlobalTransform, With<crate::components::SharedEngineCamera>>,
) {
    // We need camera to billboard the particles
    let cam_transform = if let Some(t) = camera_query.iter().next() {
        t
    } else {
        return;
    };

    // Get camera basis vectors for billboarding
    let cam_right = cam_transform.right();
    let cam_up = cam_transform.up();

    for system in query.iter() {
        let Some(mesh) = meshes.get_mut(&system.mesh_handle) else { continue };

        let count = system.particles.len();
        let mut positions = Vec::with_capacity(count * 4);
        let mut colors = Vec::with_capacity(count * 4);
        let mut uvs = Vec::with_capacity(count * 4);
        let mut indices = Vec::with_capacity(count * 6);

        let size = 0.1; // configurable?

        for (i, p) in system.particles.iter().enumerate() {
            // Billboarding: Quad aligned with camera
            let center = p.position;

            // Corners
            let bl = center - cam_right * size - cam_up * size;
            let br = center + cam_right * size - cam_up * size;
            let tr = center + cam_right * size + cam_up * size;
            let tl = center - cam_right * size + cam_up * size;

            positions.push(bl);
            positions.push(br);
            positions.push(tr);
            positions.push(tl);

            // Color interpolation
            let t = p.age / p.lifetime;
            // Simple lerp for LinearRgba
            let color = LinearRgba::new(
                p.color_start.red + (p.color_end.red - p.color_start.red) * t,
                p.color_start.green + (p.color_end.green - p.color_start.green) * t,
                p.color_start.blue + (p.color_end.blue - p.color_start.blue) * t,
                p.color_start.alpha + (p.color_end.alpha - p.color_start.alpha) * t,
            );

            let c = [color.red, color.green, color.blue, color.alpha];
            colors.push(c);
            colors.push(c);
            colors.push(c);
            colors.push(c);

            // UVs
            uvs.push([0.0, 1.0]);
            uvs.push([1.0, 1.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([0.0, 0.0]);

            // Indices
            let base = i as u32 * 4;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            indices.push(base + 2);
            indices.push(base + 3);
            indices.push(base);
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));
    }
}
