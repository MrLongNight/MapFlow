use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

/// Resource to store current audio analysis data from MapFlow.
///
/// This resource is updated every frame by the `BevyRunner` to reflect the latest
/// audio spectrum data (e.g., bass, mid, treble) and volume metrics. Systems can
/// query this resource to drive audio-reactive animations.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct AudioInputResource {
    /// Normalized energy levels (0.0 - 1.0) for each of the 9 frequency bands.
    pub band_energies: [f32; 9],
    /// The Root Mean Square (RMS) volume level (overall loudness).
    pub rms_volume: f32,
    /// The peak volume level.
    pub peak_volume: f32,
    /// Whether a beat was detected in the current frame.
    pub beat_detected: bool,
}

impl AudioInputResource {
    /// Helper to get the combined energy level for a specific audio source type.
    ///
    /// This method aggregates relevant frequency bands for simpler source types
    /// (e.g., `Bass` combines bands 0 and 1).
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

/// Resource for sharing the rendered frame from Bevy with MapFlow.
///
/// This resource holds the handle to the render target texture and a shared buffer
/// containing the pixel data of the last rendered frame. This allows MapFlow to
/// display the Bevy scene as a layer or texture.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct BevyRenderOutput {
    /// Handle to the image asset used as the render target.
    pub image_handle: Handle<Image>,
    /// Thread-safe container for the last extracted frame data (BGRA8 format).
    /// This is shared between the main Bevy world and the render world.
    pub last_frame_data: std::sync::Arc<std::sync::Mutex<Option<Vec<u8>>>>,
    /// Width of the render target in pixels.
    pub width: u32,
    /// Height of the render target in pixels.
    pub height: u32,
}

/// Resource managing the GPU buffer for reading back texture data.
///
/// This is used internally by the render extraction system to copy the rendered
/// frame from the GPU to CPU memory.
#[derive(Resource)]
pub struct ReadbackBuffer {
    /// The WGPU buffer used for the readback operation.
    pub buffer: bevy::render::render_resource::Buffer,
    /// The size of the buffer in bytes.
    pub size: u64,
}

/// Resource that maps MapFlow Node IDs to Bevy Entity IDs.
///
/// This allows the system to update existing Bevy entities when their corresponding
/// MapFlow node properties change, rather than recreating them every frame.
///
/// Key is `(module_id, part_id)`.
#[derive(Resource, Default)]
pub struct BevyNodeMapping {
    /// Map from (Module ID, Part ID) to the Bevy Entity ID.
    pub entities: std::collections::HashMap<(u64, u64), Entity>,
}

/// Resource that stores the current evaluated trigger values for MapFlow nodes.
///
/// This allows Bevy systems to react to logic signals from the node graph
/// (e.g., triggering a particle burst when a specific node activates).
///
/// Key is `(module_id, part_id)`.
#[derive(Resource, Default)]
pub struct MapFlowTriggerResource {
    /// Map from (Module ID, Part ID) to the trigger value (0.0 - 1.0).
    pub trigger_values: std::collections::HashMap<(u64, u64), f32>,
}
