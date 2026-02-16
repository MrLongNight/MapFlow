//! Bevy integration for MapFlow.

pub mod components;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
use components::*;
use resources::*;
use systems::*;
use tracing::info;

pub struct BevyRunner {
    app: App,
}

impl Default for BevyRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl BevyRunner {
    pub fn new() -> Self {
        info!("Initializing Bevy integration (Full Asset Mode)...");

        let mut app = App::new();

        // Load essential plugins for 3D assets without opening a window
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.add_plugins(bevy::hierarchy::HierarchyPlugin);
        app.add_plugins(bevy::transform::TransformPlugin);

        // Load PBR infrastructure so StandardMaterial and Mesh assets exist
        // We use the headless configuration parts of PbrPlugin
        app.add_plugins(bevy::pbr::PbrPlugin { ..default() });
        app.add_plugins(bevy::render::RenderPlugin { ..default() });
        app.add_plugins(bevy::core_pipeline::CorePipelinePlugin);

        // Register Extensions
        app.add_plugins(bevy_mod_outline::OutlinePlugin);

        // Register resources
        app.init_resource::<AudioInputResource>();
        app.init_resource::<BevyNodeMapping>();

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
            ),
        );

        // Add readback system to the RENDER APP, not the main app
        if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render_app.add_systems(bevy::render::Render, frame_readback_system);
        }

        Self { app }
    }

    pub fn update(&mut self, audio_data: &mapmap_core::audio_reactive::AudioTriggerData) {
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
        self.app.update();
    }

    pub fn get_image_data(&self) -> Option<(Vec<u8>, u32, u32)> {
        // Dummy for now, real readback needs RenderDevice synchronization
        Some((vec![0, 0, 0, 0], 1, 1))
    }

    /// Update the Bevy scene based on the MapFlow graph state.
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
                                    if model.position != *position {
                                        model.position = *position;
                                    }
                                    if model.rotation != *rotation {
                                        model.rotation = *rotation;
                                    }
                                    if model.scale != *scale {
                                        model.scale = *scale;
                                    }
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
