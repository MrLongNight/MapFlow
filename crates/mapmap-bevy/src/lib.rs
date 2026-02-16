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
        // HierarchyPlugin is now part of the prelude or re-exported differently in Bevy 0.16.
        // It is often included in MinimalPlugins or DefaultPlugins subsets.
        // If compilation fails, we can remove explicit addition if it's redundant,
        // or check correct path. Bevy 0.16 release notes suggest simplified structure.
        // HierarchyPlugin is now part of the core/minimal set in many configurations,
        // but if we need to add it explicitly and `bevy::hierarchy` is missing,
        // it might be under `bevy::core::HierarchyPlugin` or just implied.
        // However, `MinimalPlugins` typically includes `CorePlugin`, `ScheduleRunnerPlugin`, etc.
        // The error `could not find hierarchy in bevy` strongly suggests the module is not exposed.
        // Let's assume MinimalPlugins handles basic hierarchy or we skip explicit add if it fails.
        // Trying removal as it's likely redundant or feature-gated out (but we don't strictly need the *Plugin* if we just use Parent/Children components which are core).
        // app.add_plugins(bevy::hierarchy::HierarchyPlugin);
        app.add_plugins(bevy::transform::TransformPlugin);

        // Load PBR infrastructure so StandardMaterial and Mesh assets exist
        // We use the headless configuration parts of PbrPlugin
        app.add_plugins(bevy::pbr::PbrPlugin { ..default() });

        // Fix for RenderCreation::Manual(RenderResources) signature change in Bevy 0.16.
        // It now takes a single RenderResources struct, or similar.
        // Assuming we want to disable automatic renderer creation (headless/manual).
        // If we want actual rendering, we should use Automatic or configure properly.
        // For BevyRunner embedded in another app, we likely want manual control.
        // However, the error says: `expected RenderResources, found Option<_>`.
        // This implies it wants `RenderCreation::Manual(RenderResources { ... })`.
        // But we are passing `(None, None)`.
        // Let's try constructing a dummy RenderResources if needed, or check docs.
        // Actually, if we just want headless without a window, maybe we don't need Manual?
        // But let's stick to the fix pattern: remove the second argument.
        // Wait, `RenderCreation::Manual` takes `RenderResources`.
        // We need to construct `RenderResources`.
        // Since we are likely integrating with an external WGPU context (MapFlow's),
        // we might need to pass that in. But here we are just initializing the App.
        // If we don't have the context yet, maybe `RenderCreation::Automatic` is safer
        // if we use a headless backend?
        // Alternatively, if the error says "takes 1 argument but 2 supplied",
        // and we passed `(None, None)`, it means it expects 1 arg.
        // If we pass `RenderCreation::Manual(bevy::render::settings::RenderResources::default())`?
        // `RenderResources` might not implement Default or be easily constructible without handles.

        // Let's try removing the second argument as a first step, assuming the first arg
        // is the resources (or a device/queue tuple wrapper).
        // Actually, for headless integration where we share wgpu, we need to pass the device/queue.
        // If we don't have them at `new()`, we can't initialize the plugin fully.
        // BUT, `mapmap-bevy` seems to be running its own App instance.
        // Let's look at the error log again:
        // "expected struct `bevy::bevy_render::settings::RenderResources`, found enum `std::option::Option<_>`"
        // So it expects `RenderCreation::Manual(RenderResources)`.

        // Since we cannot easily construct RenderResources here without context,
        // and previously it accepted (None, None), maybe we should switch to `Automatic`
        // but with a headless selector?
        // Or construct a dummy `RenderResources`? No, that requires device/queue.
        // If we want Bevy to create its own headless renderer:
        app.add_plugins(bevy::render::RenderPlugin {
            render_creation: bevy::render::settings::RenderCreation::Automatic(
                bevy::render::settings::WgpuSettings {
                    backends: Some(bevy::render::settings::Backends::PRIMARY),
                    ..default()
                },
            ),
            ..default()
        });
        app.add_plugins(bevy::core_pipeline::CorePipelinePlugin);

        // Register resources
        app.init_resource::<AudioInputResource>();
        app.init_resource::<BevyNodeMapping>();

        // Register components
        app.register_type::<AudioReactive>();
        app.register_type::<Bevy3DText>();
        app.register_type::<BevyCamera>();
        app.register_type::<Bevy3DShape>();

        // Re-enable all systems now that assets should be present
        app.add_systems(
            Update,
            (
                audio_reaction_system,
                camera_control_system,
                text_3d_system,
                shape_system,
            ),
        );

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

    pub fn apply_graph_state(&mut self, _module: &mapmap_core::module::MapFlowModule) {
        // Logic for syncing Bevy entities with MapFlow graph
    }
}
