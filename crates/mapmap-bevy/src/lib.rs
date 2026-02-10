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
        let render_device = RenderDevice::from((*device).clone());
        let render_queue = RenderQueue(std::sync::Arc::new(WgpuWrapper::new((*queue).clone())));
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
        app.register_type::<BevyCamera>();

        // Register systems
        app.add_systems(Update, print_status_system);
        app.add_systems(Update, (audio_reaction_system, hex_grid_system, camera_control_system));

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

    /// Sync the Bevy world with the MapFlow module graph
    pub fn apply_graph_state(&mut self, module: &mapmap_core::module::MapFlowModule) {
        let world = self.app.world_mut();

        // Track which entities are still valid/active in this frame
        let mut active_ids = std::collections::HashSet::new();

        // Use a scope to access mapping, then spawn outside or handle borrow carefully
        // We can just iterate twice or collect needed actions.
        // Actually, world.spawn doesn't conflict with world.resource_mut if we drop the resource ref.
        // But we need the mapping to check existence.
        // We can't hold `mapping` (resource) and call `world.spawn` (mutable world access).

        // Strategy: Collect actions (Spawn vs Update)
        enum Action {
            UpdateCamera(Entity, [f32; 3], [f32; 3]),
            SpawnCamera(u64, [f32; 3], [f32; 3]),
            UpdateHexGrid(Entity, f32, u32, bool, f32),
            SpawnHexGrid(u64, f32, u32, bool, f32),
        }

        let mut actions = Vec::new();

        {
            let mapping = world.resource::<BevyNodeMapping>();

            for part in &module.parts {
                use mapmap_core::module::{ModulePartType, SourceType};
                match &part.part_type {
                    ModulePartType::Source(SourceType::BevyCamera { position, rotation }) => {
                        active_ids.insert(part.id);
                        if let Some(&entity) = mapping.entities.get(&part.id) {
                            actions.push(Action::UpdateCamera(entity, *position, *rotation));
                        } else {
                            actions.push(Action::SpawnCamera(part.id, *position, *rotation));
                        }
                    },
                    ModulePartType::Source(SourceType::BevyHexGrid { radius, rings, pointy_top, spacing, .. }) => {
                        active_ids.insert(part.id);
                        if let Some(&entity) = mapping.entities.get(&part.id) {
                            actions.push(Action::UpdateHexGrid(entity, *radius, *rings, *pointy_top, *spacing));
                        } else {
                            actions.push(Action::SpawnHexGrid(part.id, *radius, *rings, *pointy_top, *spacing));
                        }
                    },
                    _ => {}
                }
            }
        }

        // Execute actions
        for action in actions {
            match action {
                Action::UpdateCamera(e, pos, rot) => {
                    if let Some(mut comp) = world.get_mut::<BevyCamera>(e) {
                        comp.position = pos;
                        comp.rotation = rot;
                    }
                },
                Action::SpawnCamera(id, pos, rot) => {
                    let entity = world.spawn((
                        BevyCamera { position: pos, rotation: rot },
                        Transform::default(),
                        Visibility::default(),
                    )).id();
                    world.resource_mut::<BevyNodeMapping>().entities.insert(id, entity);
                },
                Action::UpdateHexGrid(e, rad, rings, pointy, spacing) => {
                    if let Some(mut comp) = world.get_mut::<BevyHexGrid>(e) {
                        comp.radius = rad;
                        comp.rings = rings;
                        comp.pointy_top = pointy;
                        comp.spacing = spacing;
                    }
                },
                Action::SpawnHexGrid(id, rad, rings, pointy, spacing) => {
                    let entity = world.spawn((
                        BevyHexGrid { radius: rad, rings: rings, pointy_top: pointy, spacing: spacing },
                        Transform::default(),
                        Visibility::default(),
                    )).id();
                    world.resource_mut::<BevyNodeMapping>().entities.insert(id, entity);
                },
            }
        }

        // Cleanup
        let mut to_remove = Vec::new();
        {
            let mapping = world.resource::<BevyNodeMapping>();
            for k in mapping.entities.keys() {
                // Garbage collect entities that belong to this module but are no longer active
                if module.parts.iter().any(|p| p.id == *k) && !active_ids.contains(k) {
                    to_remove.push(*k);
                }
            }
        }

        let mut entities_to_despawn = Vec::new();
        {
            let mut mapping = world.resource_mut::<BevyNodeMapping>();
            for k in to_remove {
                if let Some(entity) = mapping.entities.remove(&k) {
                    entities_to_despawn.push(entity);
                }
            }
        }

        for entity in entities_to_despawn {
            world.despawn(entity);
        }
    }
}

fn print_status_system() {
    // Placeholder system
}
