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
    active_ids: std::collections::HashSet<u64>,
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
        app.register_type::<Bevy3DText>();

        // Register systems
        app.add_systems(Update, print_status_system);
        app.add_systems(
            Update,
            (audio_reaction_system, hex_grid_system, text_3d_system),
        );

        let render_app = app.sub_app_mut(bevy::render::RenderApp);
        render_app.add_systems(bevy::render::Render, frame_readback_system);

        Self {
            app,
            active_ids: std::collections::HashSet::new(),
        }
    }

    /// Update the Bevy world manually.
    pub fn update(&mut self, audio_data: &mapmap_core::audio_reactive::AudioTriggerData) {
        // Update resource from input data
        let mut res = self.app.world_mut().resource_mut::<AudioInputResource>();
        res.band_energies = audio_data.band_energies;
        res.rms_volume = audio_data.rms_volume;
        res.peak_volume = audio_data.peak_volume;
        res.beat_detected = audio_data.beat_detected;

        // Perform cleanup of stale entities
        // We do this here because apply_graph_state might be called multiple times (once per module)
        // and we need to aggregate active IDs across all modules before cleanup.
        let world = self.app.world_mut();
        let mut stale_keys = Vec::new();

        {
            let mapping = world.resource::<BevyNodeMapping>();
            for key in mapping.entities.keys() {
                if !self.active_ids.contains(key) {
                    stale_keys.push(*key);
                }
            }
        }

        for key in stale_keys {
            if let Some(entity) = world.resource_mut::<BevyNodeMapping>().entities.remove(&key) {
                world.despawn(entity);
            }
        }

        // Reset active IDs for next frame
        self.active_ids.clear();

        // Run schedule
        self.app.update();
    }

    /// Sync the Bevy world with the MapFlow module graph.
    pub fn apply_graph_state(&mut self, module: &mapmap_core::module::MapFlowModule) {
        use mapmap_core::module::{ModulePartType, SourceType};

        // We need to work with the world mutably
        let world = self.app.world_mut();

        // Spawn or Update entities
        for part in &module.parts {
            if let ModulePartType::Source(source) = &part.part_type {
                match source {
                    SourceType::BevyParticles {
                        rate,
                        lifetime,
                        speed,
                        color_start,
                        color_end,
                        position,
                        rotation: _,
                    } => {
                        self.active_ids.insert(part.id);
                        let mapping = world.resource::<BevyNodeMapping>();
                        let entity = mapping.entities.get(&part.id).copied();

                        if let Some(e) = entity {
                            // Update existing (optimize by checking equality if possible, but for simple types assign is fast)
                            if let Some(mut component) = world.get_mut::<BevyParticles>(e) {
                                component.rate = *rate;
                                component.lifetime = *lifetime;
                                component.speed = *speed;
                                component.color_start = *color_start;
                                component.color_end = *color_end;
                            }
                            if let Some(mut transform) = world.get_mut::<Transform>(e) {
                                let new_pos = Vec3::new(position[0], position[1], position[2]);
                                if transform.translation != new_pos {
                                    transform.translation = new_pos;
                                }
                            }
                        } else {
                            // Spawn new
                            let id = part.id;
                            let new_entity = world
                                .spawn((
                                    BevyParticles {
                                        rate: *rate,
                                        lifetime: *lifetime,
                                        speed: *speed,
                                        color_start: *color_start,
                                        color_end: *color_end,
                                    },
                                    Transform::from_xyz(position[0], position[1], position[2]),
                                    Visibility::default(),
                                ))
                                .id();
                            world
                                .resource_mut::<BevyNodeMapping>()
                                .entities
                                .insert(id, new_entity);
                        }
                    }
                    SourceType::Bevy3DText {
                        text,
                        font_size,
                        color,
                        position,
                        rotation,
                        alignment,
                    } => {
                        self.active_ids.insert(part.id);
                        let mapping = world.resource::<BevyNodeMapping>();
                        let entity = mapping.entities.get(&part.id).copied();

                        if let Some(e) = entity {
                            // Update existing
                            if let Some(mut component) = world.get_mut::<Bevy3DText>(e) {
                                // Simple equality check to avoid dirty marking if not changed
                                if component.text != *text
                                    || (component.font_size - *font_size).abs() > f32::EPSILON
                                    || component.color != *color
                                    || component.position != *position
                                    || component.rotation != *rotation
                                    || component.alignment != *alignment
                                {
                                    component.text = text.clone();
                                    component.font_size = *font_size;
                                    component.color = *color;
                                    component.position = *position;
                                    component.rotation = *rotation;
                                    component.alignment = *alignment;
                                }
                            }
                        } else {
                            // Spawn new
                            let id = part.id;
                            let new_entity = world
                                .spawn((
                                    Bevy3DText {
                                        text: text.clone(),
                                        font_size: *font_size,
                                        color: *color,
                                        position: *position,
                                        rotation: *rotation,
                                        alignment: *alignment,
                                    },
                                    Transform::default(), // System handles transform
                                    Visibility::default(),
                                ))
                                .id();
                            world
                                .resource_mut::<BevyNodeMapping>()
                                .entities
                                .insert(id, new_entity);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn print_status_system() {
    // Placeholder system
}
