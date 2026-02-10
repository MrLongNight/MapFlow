//! Bevy integration for MapFlow.
//!
//! This crate provides a bridge between MapFlow's orchestration and the Bevy game engine
//! for high-performance 3D rendering and audio reactivity.

pub mod components;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
use components::*;
use resources::*;
use systems::*;
use tracing::info;
use wgpu;

/// Struct to manage the Bevy application instance.
pub struct BevyRunner {
    app: App,
}

impl BevyRunner {
    /// Creates a new BevyRunner instance with a shared WGPU context.
    pub fn new(
        instance: std::sync::Arc<wgpu::Instance>,
        adapter: std::sync::Arc<wgpu::Adapter>,
        device: std::sync::Arc<wgpu::Device>,
        queue: std::sync::Arc<wgpu::Queue>,
    ) -> Self {
        info!("Initializing Bevy integration with shared WGPU context...");

        use bevy::render::renderer::{WgpuWrapper, RenderInstance, RenderAdapter, RenderDevice, RenderQueue, RenderAdapterInfo};

        let mut app = App::new();

        // Wrap wgpu resources into Bevy's wrapper types (WgpuWrapper is required for Send/Sync)
        let info = adapter.get_info();
        let render_instance = RenderInstance(std::sync::Arc::new(WgpuWrapper::new((*instance).clone())));
        let render_adapter = RenderAdapter(std::sync::Arc::new(WgpuWrapper::new((*adapter).clone())));

        // Use unsafe transmute to bypass potential type mismatch between workspace wgpu and Bevy's wgpu
        // This assumes binary compatibility (same wgpu version). Bevy 0.16 uses wgpu 24.
        let render_device: RenderDevice = unsafe { std::mem::transmute((*device).clone()) };
        let render_queue: RenderQueue = unsafe { std::mem::transmute((*queue).clone()) };

        let adapter_info = RenderAdapterInfo(WgpuWrapper::new(info));

        // Use DefaultPlugins but with shared WGPU resources.
        app.add_plugins(
            DefaultPlugins
                .set(bevy::render::RenderPlugin {
                    render_creation: bevy::render::settings::RenderCreation::manual(
                        render_device,
                        render_queue,
                        adapter_info,
                        render_adapter,
                        render_instance,
                    ),
                    synchronous_pipeline_compilation: true,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    close_when_requested: false,
                })
                .disable::<bevy::winit::WinitPlugin>(),
        );

        // Register resources
        app.init_resource::<AudioInputResource>();
        app.init_resource::<BevyRenderOutput>();
        app.init_resource::<BevyNodeMapping>();

        // Setup ExtractResourcePlugin to sync BevyRenderOutput to Render App
        app.add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            BevyRenderOutput,
        >::default());

        // Register components
        app.register_type::<AudioReactive>();
        app.register_type::<BevyAtmosphere>();
        app.register_type::<BevyHexGrid>();
        app.register_type::<BevyParticles>();
        app.register_type::<Bevy3DShape>();

        // Register systems
        app.add_systems(Update, print_status_system);
        app.add_systems(Update, (audio_reaction_system, hex_grid_system, shape_system));

        let render_app = app.sub_app_mut(bevy::render::RenderApp);
        render_app.add_systems(bevy::render::Render, frame_readback_system);

        Self { app }
    }

    /// Update the Bevy world manually.
    pub fn update(&mut self, audio_data: &mapmap_core::audio_reactive::AudioTriggerData) {
        // Update resource from input data
        let mut res = self.app.world_mut().resource_mut::<AudioInputResource>();
        res.band_energies = audio_data.band_energies;
        res.rms_volume = audio_data.rms_volume;
        res.peak_volume = audio_data.peak_volume;
        res.beat_detected = audio_data.beat_detected;

        // Run schedule
        self.app.update();
    }

    /// Sync the Bevy world with the MapFlow graph state.
    pub fn apply_graph_state(&mut self, module: &mapmap_core::module::MapFlowModule) {
        let module_id = module.id;
        let mut needed_entities = std::collections::HashSet::new();
        let mut to_spawn = Vec::new();

        // Scope for resource borrow
        {
            let world = self.app.world_mut();
            let mapping = world.resource::<BevyNodeMapping>();

            for part in &module.parts {
                if let mapmap_core::module::ModulePartType::Source(source) = &part.part_type {
                    if is_bevy_source(source) {
                        let key = (module_id, part.id);
                        needed_entities.insert(key);
                        if !mapping.entities.contains_key(&key) {
                            to_spawn.push((part.id, source.clone()));
                        }
                    }
                }
            }
        }

        // Spawn new entities
        for (part_id, _) in to_spawn {
            let entity = self
                .app
                .world_mut()
                .spawn((Transform::default(), Visibility::default()))
                .id();

            let mut mapping = self.app.world_mut().resource_mut::<BevyNodeMapping>();
            mapping.entities.insert((module_id, part_id), entity);
        }

        // Despawn removed entities for this module
        let to_despawn: Vec<((u64, u64), Entity)> = {
            let mapping = self.app.world().resource::<BevyNodeMapping>();
            mapping
                .entities
                .iter()
                .filter(|(&(mid, _), _)| mid == module_id)
                .filter(|(key, _)| !needed_entities.contains(key))
                .map(|(&k, &e)| (k, e))
                .collect()
        };

        for (key, entity) in to_despawn {
            self.app.world_mut().despawn(entity);
            self.app
                .world_mut()
                .resource_mut::<BevyNodeMapping>()
                .entities
                .remove(&key);
        }

        // Update components
        // We clone the map keys/values we need to avoid borrowing issues during iteration
        let entity_map: Vec<(u64, Entity)> = {
            let mapping = self.app.world().resource::<BevyNodeMapping>();
            mapping
                .entities
                .iter()
                .filter(|(&(mid, _), _)| mid == module_id)
                .map(|(&(_, pid), &e)| (pid, e))
                .collect()
        };
        let entity_lookup: std::collections::HashMap<u64, Entity> =
            entity_map.into_iter().collect();

        for part in &module.parts {
            if let mapmap_core::module::ModulePartType::Source(source) = &part.part_type {
                if let Some(&entity) = entity_lookup.get(&part.id) {
                    sync_bevy_part(self.app.world_mut(), entity, source);
                }
            }
        }
    }
}

fn is_bevy_source(source: &mapmap_core::module::SourceType) -> bool {
    matches!(
        source,
        mapmap_core::module::SourceType::Bevy
            | mapmap_core::module::SourceType::BevyAtmosphere { .. }
            | mapmap_core::module::SourceType::BevyHexGrid { .. }
            | mapmap_core::module::SourceType::BevyParticles { .. }
            | mapmap_core::module::SourceType::Bevy3DShape { .. }
    )
}

fn sync_bevy_part(world: &mut World, entity: Entity, source: &mapmap_core::module::SourceType) {
    use mapmap_core::module::SourceType;

    // 1. Sync Transform
    let (pos, rot, scale) = match source {
        SourceType::BevyHexGrid {
            position,
            rotation,
            scale,
            ..
        } => (
            Vec3::from(*position),
            Vec3::from(*rotation),
            Vec3::splat(*scale),
        ),
        SourceType::BevyParticles {
            position, rotation, ..
        } => (Vec3::from(*position), Vec3::from(*rotation), Vec3::ONE),
        SourceType::Bevy3DShape {
            position,
            rotation,
            scale,
            ..
        } => (
            Vec3::from(*position),
            Vec3::from(*rotation),
            Vec3::from(*scale),
        ),
        _ => (Vec3::ZERO, Vec3::ZERO, Vec3::ONE),
    };

    if let Some(mut transform) = world.get_mut::<Transform>(entity) {
        transform.translation = pos;
        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            rot.x.to_radians(),
            rot.y.to_radians(),
            rot.z.to_radians(),
        );
        transform.scale = scale;
    }

    // 2. Sync Specific Components
    match source {
        SourceType::Bevy3DShape {
            shape_type,
            color,
            unlit,
            ..
        } => {
            world.entity_mut(entity).insert(Bevy3DShape {
                shape_type: *shape_type,
                color: *color,
                unlit: *unlit,
            });
        }
        SourceType::BevyHexGrid {
            radius,
            rings,
            pointy_top,
            spacing,
            ..
        } => {
            world.entity_mut(entity).insert(BevyHexGrid {
                radius: *radius,
                rings: *rings,
                pointy_top: *pointy_top,
                spacing: *spacing,
            });
        }
        SourceType::BevyParticles {
            rate,
            lifetime,
            speed,
            color_start,
            color_end,
            ..
        } => {
            world.entity_mut(entity).insert(BevyParticles {
                rate: *rate,
                lifetime: *lifetime,
                speed: *speed,
                color_start: *color_start,
                color_end: *color_end,
            });
        }
        _ => {}
    }
}

fn print_status_system() {
    // Placeholder system
}
