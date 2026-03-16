//! Bevy integration for MapFlow.
//!
//! This crate integrates the Bevy engine into MapFlow to provide advanced 3D rendering capabilities.
//! It bridges MapFlow's core data structures with Bevy's ECS (Entity Component System), allowing
//! for the creation of complex 3D scenes, particle systems, and audio-reactive visuals.
//!
//! ## Features
//!
//! - **Seamless ECS Integration**: Run Bevy systems alongside MapFlow's main loop.
//! - **Audio Reactivity**: Bind 3D transform properties (scale, rotation, position) to audio analysis data.
//! - **Particle Systems**: GPU-accelerated particle emitters with customizable lifetime, speed, and color gradients.
//! - **Atmospheric Rendering**: Realistic sky and atmosphere simulation.
//! - **3D Models**: Load and display 3D models (glTF) with optional outline effects.
//! - **Hex Grid Generation**: Procedural generation of 3D hexagonal grids.
//! - **Shared Rendering**: Efficiently share the rendered Bevy frame with MapFlow's WGPU context.
//!
//! ## Usage
//!
//! This crate is primarily used by the main application to initialize the Bevy runner and update it
//! with fresh data from the MapFlow engine.
//!
//! ```rust,no_run
//! use mapmap_bevy::BevyRunner;
//!
//! // Initialize the Bevy runner
//! let mut runner = BevyRunner::new();
//!
//! // In the main loop:
//! // runner.update(&audio_data, &node_triggers);
//! // let frame = runner.get_image_data();
//! ```

pub mod components;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
use components::*;
use resources::*;
use systems::*;
use tracing::info;

/// The main entry point for running Bevy within MapFlow.
///
/// This struct manages the Bevy `App` instance, handles initialization of plugins and resources,
/// and provides methods to update the simulation and retrieve rendered frames.
pub struct BevyRunner {
    app: App,
}

impl Default for BevyRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl BevyRunner {
    /// Create a new BevyRunner instance.
    ///
    /// This initializes the Bevy `App` with a minimal set of plugins required for 3D rendering
    /// and logic, without opening a separate window. It registers all custom components and resources
    /// needed for MapFlow integration.
    pub fn new() -> Self {
        info!("Initializing Bevy integration (Headless 3D Mode)...");

        let mut app = App::new();

        // Use DefaultPlugins but disable the window for headless rendering
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        }));

        // Add essential rendering extensions
        app.add_plugins(bevy_atmosphere::prelude::AtmospherePlugin);
        app.add_plugins(bevy_mod_outline::OutlinePlugin);

        // Register resources
        app.init_resource::<AudioInputResource>();
        app.init_resource::<BevyNodeMapping>();
        app.init_resource::<MapFlowTriggerResource>();
        app.init_resource::<crate::resources::BevyRenderOutput>();

        // Register components
        app.register_type::<AudioReactive>();
        app.register_type::<BevyAtmosphere>();
        app.register_type::<BevyHexGrid>();
        app.register_type::<BevyParticles>();
        app.register_type::<Bevy3DShape>();
        app.register_type::<Bevy3DModel>();
        app.register_type::<Bevy3DText>();
        app.register_type::<BevyCamera>();

        // Register systems
        app.add_systems(Startup, setup_3d_scene);
        app.add_systems(Update, print_status_system);
        app.add_systems(
            Update,
            (
                audio_reaction_system,
                camera_control_system,
                hex_grid_system,
                model_system,
                shape_system,
                text_3d_system,
                node_reactivity_system,
                particle_system,
                frame_readback_system,
            ),
        );

        Self { app }
    }

    /// Update the Bevy simulation with new data from MapFlow.
    ///
    /// This method should be called once per frame. It updates the internal resources with
    /// the latest audio analysis data and trigger values, then steps the Bevy `App`.
    ///
    /// # Arguments
    ///
    /// * `audio_data` - The current audio analysis data (FFT bands, volume, etc.).
    /// * `node_triggers` - A map of trigger values for specific nodes.
    pub fn update(
        &mut self,
        audio_data: &mapmap_core::audio_reactive::AudioTriggerData,
        node_triggers: &std::collections::HashMap<(u64, u64), f32>,
    ) {
        if let Some(mut res) = self
            .app
            .world_mut()
            .get_resource_mut::<AudioInputResource>()
        {
            res.band_energies = audio_data.band_energies;
            res.rms_volume = audio_data.rms_volume;
            res.peak_volume = audio_data.peak_volume;
            res.beat_detected = audio_data.beat_detected;
        }

        if let Some(mut res) = self
            .app
            .world_mut()
            .get_resource_mut::<crate::resources::MapFlowTriggerResource>()
        {
            res.trigger_values = node_triggers.clone();
        }

        self.app.update();
    }

    /// Retrieve the rendered image data from the last frame.
    ///
    /// Returns a tuple containing:
    /// - The raw byte data of the image (BGRA8 format).
    /// - The width of the image.
    /// - The height of the image.
    ///
    /// Returns `None` if no frame has been rendered yet.
    pub fn get_image_data(&self) -> Option<(Vec<u8>, u32, u32)> {
        let render_output = self
            .app
            .world()
            .get_resource::<crate::resources::BevyRenderOutput>()?;
        if let Ok(lock) = render_output.last_frame_data.lock() {
            if let Some(data) = lock.as_ref() {
                return Some((data.clone(), render_output.width, render_output.height));
            }
        }
        None
    }

    /// Update the Bevy scene based on the MapFlow graph state.
    ///
    /// This synchronizes the Bevy entities with the configuration of a `MapFlowModule`.
    /// It handles spawning new entities for new nodes and updating properties of existing ones.
    pub fn apply_graph_state(&mut self, module: &mapmap_core::module::MapFlowModule) {
        use mapmap_core::module::{ModulePartType, SourceType};
        let module_id = module.id;

        self.app
            .world_mut()
            .resource_scope(|world, mut mapping: Mut<BevyNodeMapping>| {
                for part in &module.parts {
                    if let ModulePartType::Source(source_type) = &part.part_type {
                        let key = (module_id, part.id);
                        match source_type {
                            SourceType::BevyAtmosphere {
                                turbidity,
                                rayleigh,
                                mie_coeff,
                                mie_directional_g,
                                sun_position,
                                ..
                            } => {
                                let entity = *mapping.entities.entry(key).or_insert_with(|| {
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
                                let entity = *mapping.entities.entry(key).or_insert_with(|| {
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
                                let entity = *mapping.entities.entry(key).or_insert_with(|| {
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
                            SourceType::Bevy3DShape {
                                shape_type,
                                color,
                                unlit,
                                position,
                                rotation,
                                scale,
                                outline_width,
                                outline_color,
                                ..
                            } => {
                                let entity = *mapping.entities.entry(key).or_insert_with(|| {
                                    world
                                        .spawn((
                                            crate::components::Bevy3DShape::default(),
                                            Transform::default(),
                                            Visibility::default(),
                                        ))
                                        .id()
                                });

                                if let Some(mut shape) =
                                    world.get_mut::<crate::components::Bevy3DShape>(entity)
                                {
                                    shape.shape_type = *shape_type;
                                    shape.color = *color;
                                    shape.unlit = *unlit;
                                    shape.outline_width = *outline_width;
                                    shape.outline_color = *outline_color;
                                }

                                if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                                    transform.translation = Vec3::from(*position);
                                    transform.rotation = Quat::from_euler(
                                        EulerRot::XYZ,
                                        rotation[0].to_radians(),
                                        rotation[1].to_radians(),
                                        rotation[2].to_radians(),
                                    );
                                    transform.scale = Vec3::from(*scale);
                                }
                            }
                            SourceType::Bevy3DModel {
                                path,
                                position,
                                rotation,
                                scale,
                                outline_width,
                                outline_color,
                                ..
                            } => {
                                let entity = *mapping.entities.entry(key).or_insert_with(|| {
                                    world
                                        .spawn((
                                            Bevy3DModel::default(),
                                            Transform::default(),
                                            Visibility::default(),
                                        ))
                                        .id()
                                });

                                if let Some(mut model) = world.get_mut::<Bevy3DModel>(entity) {
                                    if model.path != *path {
                                        model.path = path.clone();
                                    }
                                    model.position = *position;
                                    model.rotation = *rotation;
                                    model.scale = *scale;
                                    model.outline_width = *outline_width;
                                    model.outline_color = *outline_color;
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
                                let entity = *mapping.entities.entry(key).or_insert_with(|| {
                                    world
                                        .spawn((
                                            crate::components::Bevy3DText::default(),
                                            Transform::default(),
                                            Visibility::default(),
                                        ))
                                        .id()
                                });
                                if let Some(mut t) =
                                    world.get_mut::<crate::components::Bevy3DText>(entity)
                                {
                                    t.text = text.clone();
                                    t.font_size = *font_size;
                                    t.color = *color;
                                    t.alignment = match alignment.as_str() {
                                        "Center" => crate::components::BevyTextAlignment::Center,
                                        "Right" => crate::components::BevyTextAlignment::Right,
                                        "Justify" => crate::components::BevyTextAlignment::Justify,
                                        _ => crate::components::BevyTextAlignment::Left,
                                    };
                                }
                                if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                                    transform.translation = Vec3::from(*position);
                                    transform.rotation = Quat::from_euler(
                                        EulerRot::XYZ,
                                        rotation[0].to_radians(),
                                        rotation[1].to_radians(),
                                        rotation[2].to_radians(),
                                    );
                                }
                            }
                            SourceType::BevyCamera { mode, fov, active } => {
                                let entity = *mapping.entities.entry(key).or_insert_with(|| {
                                    world
                                        .spawn((
                                            crate::components::BevyCamera::default(),
                                            Transform::default(),
                                            Visibility::default(),
                                        ))
                                        .id()
                                });
                                if let Some(mut c) =
                                    world.get_mut::<crate::components::BevyCamera>(entity)
                                {
                                    // Convert BevyCameraMode (Core) to BevyCameraMode (Component)
                                    c.mode = match mode {
                                        mapmap_core::module::BevyCameraMode::Orbit {
                                            radius,
                                            speed,
                                            target,
                                            height,
                                        } => crate::components::BevyCameraMode::Orbit {
                                            radius: *radius,
                                            speed: *speed,
                                            target: Vec3::from(*target),
                                            height: *height,
                                        },
                                        mapmap_core::module::BevyCameraMode::Fly {
                                            speed,
                                            sensitivity,
                                        } => crate::components::BevyCameraMode::Fly {
                                            speed: *speed,
                                            sensitivity: *sensitivity,
                                        },
                                        mapmap_core::module::BevyCameraMode::Static {
                                            position,
                                            look_at,
                                        } => crate::components::BevyCameraMode::Static {
                                            position: Vec3::from(*position),
                                            look_at: Vec3::from(*look_at),
                                        },
                                    };
                                    c.fov = *fov;
                                    c.active = *active;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });
    }
}
