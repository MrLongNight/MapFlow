use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

/// Resource to store current audio analysis data from MapFlow.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct AudioInputResource {
    pub band_energies: [f32; 9],
    pub rms_volume: f32,
    pub peak_volume: f32,
    pub beat_detected: bool,
}

impl AudioInputResource {
    /// Helper to get combined band energy for simpler sources
    pub fn get_energy(&self, source: &crate::components::AudioReactiveSource) -> f32 {
        use crate::components::AudioReactiveSource::*;
        match source {
            Bass => (self.band_energies[0] + self.band_energies[1]) * 0.5,
            LowMid => (self.band_energies[2] + self.band_energies[3]) * 0.5,
            Mid => (self.band_energies[4] + self.band_energies[5]) * 0.5,
            HighMid => (self.band_energies[6] + self.band_energies[7]) * 0.5,
            High => self.band_energies[8],
            Rms => self.rms_volume,
            Peak => self.peak_volume,
        }
    }
}

#[derive(Resource, Clone, Default, ExtractResource)]
pub struct BevyRenderOutput {
    pub image_handle: Handle<Image>,
    /// Last extracted frame data (BGRA8) - Shared between Main and Render worlds
    pub last_frame_data: std::sync::Arc<std::sync::Mutex<Option<Vec<u8>>>>,
    pub width: u32,
    pub height: u32,
}

#[derive(Resource)]
pub struct ReadbackBuffer {
    pub buffer: bevy::render::render_resource::Buffer,
    pub size: u64,
}
/// Maps MapFlow Node IDs to Bevy Entities
#[derive(Resource, Default)]
pub struct BevyNodeMapping {
    pub entities: std::collections::HashMap<(u64, u64), Entity>,
}

/// Stores evaluated trigger values from MapFlow for each node
#[derive(Resource, Default)]
pub struct MapFlowTriggerResource {
    pub trigger_values: std::collections::HashMap<(u64, u64), f32>,
}




