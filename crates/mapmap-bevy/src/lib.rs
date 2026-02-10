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

/// Struct to manage the Bevy application instance.
pub struct BevyRunner {
    app: App,
}

impl Default for BevyRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl BevyRunner {
    /// Creates a new BevyRunner instance.
    pub fn new() -> Self {
        info!("Initializing Bevy integration...");

        let mut app = App::new();

        // Use DefaultPlugins but disable windowing and input loop to avoid Winit panic
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    close_when_requested: false,
                })
                .set(bevy::render::RenderPlugin {
                    render_creation: bevy::render::settings::RenderCreation::Automatic(
                        bevy::render::settings::WgpuSettings {
                            // Inherit backend preferences if possible, or default
                            ..default()
                        },
                    ),
                    synchronous_pipeline_compilation: false,
                    ..default()
                })
                // CRITICAL: Disable WinitPlugin to prevent it from taking over the event loop!
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
        app.register_type::<BevyCameraState>();

        // Register systems
        app.add_systems(Update, print_status_system);
        app.add_systems(
            Update,
            (
                audio_reaction_system,
                hex_grid_system,
                camera_control_system,
            ),
        );

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

    /// Get the rendered image data.
    /// Returns (data, width, height).
    pub fn get_image_data(&self) -> Option<(Vec<u8>, u32, u32)> {
        // Access shared data from resource
        if let Some(render_output) = self.app.world().get_resource::<BevyRenderOutput>() {
            if let Ok(lock) = render_output.last_frame_data.lock() {
                if let Some(data) = &*lock {
                    return Some((data.clone(), render_output.width, render_output.height));
                }
            }
        }

        None
    }

    /// Update the Bevy scene based on the MapFlow graph state.
    pub fn apply_graph_state(&mut self, module: &mapmap_core::module::MapFlowModule) {
        use mapmap_core::module::{ModulePartType, SourceType};

        self.app
            .world_mut()
            .resource_scope(|world, mut mapping: Mut<BevyNodeMapping>| {
                for part in &module.parts {
                    if let ModulePartType::Source(source_type) = &part.part_type {
                        match source_type {
                            SourceType::BevyAtmosphere {
                                turbidity,
                                rayleigh,
                                mie_coeff,
                                mie_directional_g,
                                sun_position,
                                ..
                            } => {
                                let entity =
                                    *mapping.entities.entry(part.id).or_insert_with(|| {
                                        world
                                            .spawn(crate::components::BevyAtmosphere::default())
                                            .id()
                                    });
                                if let Some(mut atmosphere) =
                                    world.get_mut::<crate::components::BevyAtmosphere>(entity)
                                {
                                    atmosphere.turbidity = *turbidity;
                                    atmosphere.rayleigh = *rayleigh;
                                    atmosphere.mie_coeff = *mie_coeff;
                                    atmosphere.mie_directional_g = *mie_directional_g;
                                    atmosphere.sun_position = *sun_position;
                                }
                            }
                            SourceType::BevyHexGrid {
                                radius,
                                rings,
                                pointy_top,
                                spacing,
                                ..
                            } => {
                                let entity =
                                    *mapping.entities.entry(part.id).or_insert_with(|| {
                                        world.spawn(crate::components::BevyHexGrid::default()).id()
                                    });
                                if let Some(mut hex) =
                                    world.get_mut::<crate::components::BevyHexGrid>(entity)
                                {
                                    hex.radius = *radius;
                                    hex.rings = *rings;
                                    hex.pointy_top = *pointy_top;
                                    hex.spacing = *spacing;
                                }
                            }
                            SourceType::BevyParticles {
                                rate,
                                lifetime,
                                speed,
                                color_start,
                                color_end,
                                ..
                            } => {
                                let entity =
                                    *mapping.entities.entry(part.id).or_insert_with(|| {
                                        world
                                            .spawn(crate::components::BevyParticles::default())
                                            .id()
                                    });
                                if let Some(mut p) =
                                    world.get_mut::<crate::components::BevyParticles>(entity)
                                {
                                    p.rate = *rate;
                                    p.lifetime = *lifetime;
                                    p.speed = *speed;
                                    p.color_start = *color_start;
                                    p.color_end = *color_end;
                                }
                            }
                            SourceType::BevyCamera {
                                mode,
                                target,
                                position,
                                distance,
                                speed,
                                direction,
                            } => {
                                // Map Core Mode to Bevy Mode
                                let bevy_mode = match mode {
                                    mapmap_core::module::BevyCameraMode::Orbit => BevyCameraMode::Orbit,
                                    mapmap_core::module::BevyCameraMode::Fly => BevyCameraMode::Fly,
                                    mapmap_core::module::BevyCameraMode::Static => BevyCameraMode::Static,
                                };

                                let target = Vec3::from(*target);
                                let position = Vec3::from(*position);
                                let direction = Vec3::from(*direction);

                                let entity = *mapping.entities.entry(part.id).or_insert_with(|| {
                                    world
                                        .spawn((
                                            BevyCamera {
                                                mode: bevy_mode,
                                                target,
                                                position,
                                                distance: *distance,
                                                speed: *speed,
                                                direction,
                                            },
                                            BevyCameraState {
                                                current_pos: position,
                                                current_angle: 0.0,
                                            },
                                        ))
                                        .id()
                                });

                                if let Some(mut cam) = world.get_mut::<BevyCamera>(entity) {
                                    cam.mode = bevy_mode;
                                    cam.target = target;
                                    cam.position = position;
                                    cam.distance = *distance;
                                    cam.speed = *speed;
                                    cam.direction = direction;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });
    }
}

fn print_status_system() {
    // Placeholder system
}