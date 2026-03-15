//! Module Graph Evaluator
//!
//! Traverses the module graph and computes output values.
//! This handles the full pipeline: Trigger -> Source -> Mask -> Effect -> Layer(Mesh) -> Output.

use crate::audio::analyzer_v2::AudioAnalysisV2;
use crate::audio_reactive::AudioTriggerData;
use crate::module::LinkMode;
use crate::module::{
    BlendModeType, LayerType, LinkBehavior, MapFlowModule, MaskType, MeshType, ModulePartId,
    ModulePartType, ModulizerType, OutputType, SharedMediaState, SourceType, TriggerType,
};
use rand::RngExt;
use std::cell::RefCell;
use std::collections::HashMap;

use std::sync::Arc;
use std::time::Instant;

/// State for individual trigger nodes, stored in the evaluator
#[derive(Debug, Clone, Default)]
pub enum TriggerState {
    #[default]
    /// No state
    None,
    /// Random trigger state
    Random {
        /// The timestamp (in ms since start) when the next trigger is scheduled.
        next_fire_time_ms: u64,
    },
}

/// Source-specific rendering properties (from MediaFile)
#[derive(Debug, Clone, Default)]
pub struct SourceProperties {
    /// Source opacity (multiplied with layer opacity)
    pub opacity: f32,
    /// Color correction: Brightness (-1.0 to 1.0)
    pub brightness: f32,
    /// Color correction: Contrast (0.0 to 2.0, 1.0 = normal)
    pub contrast: f32,
    /// Color correction: Saturation (0.0 to 2.0, 1.0 = normal)
    pub saturation: f32,
    /// Color correction: Hue shift (-180 to 180 degrees)
    pub hue_shift: f32,
    /// Transform: Scale X
    pub scale_x: f32,
    /// Transform: Scale Y
    pub scale_y: f32,
    /// Transform: Rotation in degrees
    pub rotation: f32,
    /// Transform: Offset X
    pub offset_x: f32,
    /// Transform: Offset Y
    pub offset_y: f32,
    /// Flip horizontally
    pub flip_horizontal: bool,
    /// Flip vertically
    pub flip_vertical: bool,
}

impl SourceProperties {
    /// Default source properties (no modifications)
    pub fn default_identity() -> Self {
        Self {
            opacity: 1.0,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            hue_shift: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
}

/// Cached indices for a specific module graph revision
#[derive(Debug, Clone)]
pub struct ModuleGraphIndices {
    /// Cached map from Part ID to index in `module.parts`
    pub part_index_cache: HashMap<ModulePartId, usize>,
    /// Cached map from Part ID to list of indices in `module.connections` (incoming connections)
    pub conn_index_cache: HashMap<ModulePartId, Vec<usize>>,
    /// The graph revision this cache corresponds to
    pub last_revision: u64,
}

fn primary_render_connection_idx(
    module: &MapFlowModule,
    indices: &ModuleGraphIndices,
    part_id: ModulePartId,
) -> Option<usize> {
    indices
        .conn_index_cache
        .get(&part_id)?
        .iter()
        .copied()
        // Socket 0 is the primary visual input of the render chain.
        .find(|&conn_idx| module.connections[conn_idx].to_socket == 0)
}

#[cfg(test)]
mod tests_evaluator {
    use super::*;
    use crate::audio::analyzer_v2::AudioAnalysisV2;
    use crate::module::{
        AudioTriggerOutputConfig, MapFlowModule, ModulePartType, SourceType, TriggerType,
    };
    use std::time::Duration;

    fn create_test_module() -> MapFlowModule {
        MapFlowModule {
            id: 1,
            name: "Test Module".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: crate::module::ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        }
    }

    #[test]
    fn test_evaluator_initialization() {
        let evaluator = ModuleEvaluator::new();
        assert_eq!(evaluator.audio_trigger_data.rms_volume, 0.0);
    }

    #[test]
    fn test_trigger_fixed_interval() {
        let mut evaluator = ModuleEvaluator::new();
        let shared = crate::module::SharedMediaState::default();

        let mut module = create_test_module();
        let trigger_part = ModulePartType::Trigger(TriggerType::Fixed {
            interval_ms: 100,
            offset_ms: 0,
        });
        let part_id = module.add_part_with_type(trigger_part, (0.0, 0.0));

        // Initial eval - t=0, phase=0, duration=10, 0 < 10 -> 1.0
        let result = evaluator.evaluate(&module, &shared, 0);
        let values = &result.trigger_values[&part_id];
        assert_eq!(values[0], 1.0);

        // We can't easily mock time passage without refactoring ModuleEvaluator to accept a clock,
        // or sleeping. Sleeping in unit tests is generally bad, but for 15ms it's acceptable-ish
        // for this specific scenario if we want to test the time logic.
        // Ideally we'd refactor to inject time.
        std::thread::sleep(Duration::from_millis(20));

        let result = evaluator.evaluate(&module, &shared, 0);
        let values = &result.trigger_values[&part_id];
        // 20ms > 10ms pulse duration -> 0.0
        assert_eq!(values[0], 0.0);
    }

    #[test]
    fn test_trigger_audio_fft() {
        let mut evaluator = ModuleEvaluator::new();

        // Setup Audio Analysis with a "Beat"
        let analysis = AudioAnalysisV2 {
            beat_detected: true,
            rms_volume: 0.8,
            ..Default::default()
        };

        evaluator.update_audio(&analysis);

        let mut module = create_test_module();

        // Setup AudioFFT Trigger expecting Beat Output and Volume
        let config = AudioTriggerOutputConfig {
            beat_output: true,
            volume_outputs: true,
            ..Default::default()
        };
        let trigger_part = ModulePartType::Trigger(TriggerType::AudioFFT {
            band: crate::module::AudioBand::Bass, // irrelevant here
            threshold: 0.5,
            output_config: config,
        });

        let part_id = module.add_part_with_type(trigger_part, (0.0, 0.0));

        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default(), 0);
        let values = &result.trigger_values[&part_id];

        // Order in generate_outputs:
        // if volume_outputs: RMS, Peak
        // if beat_output: Beat Out
        // So indices: 0: RMS, 1: Peak, 2: Beat Out

        assert_eq!(values.len(), 3);
        assert_eq!(values[0], 0.8); // RMS
        assert_eq!(values[2], 1.0); // Beat detected
    }

    #[test]
    fn test_evaluator_propagation() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        // 1. Trigger (Fixed -> always 1.0 at start)
        let t_type = ModulePartType::Trigger(TriggerType::Fixed {
            interval_ms: 0,
            offset_ms: 0,
        }); // 0 interval = always on
        let t_id = module.add_part_with_type(t_type, (0.0, 0.0));

        // 2. Source (Target)
        let s_id = module.add_part(crate::module::PartType::Source, (200.0, 0.0));
        module.add_connection(t_id, 0, s_id, 0); // Trigger Out -> Source Trigger In

        let shared = crate::module::SharedMediaState::default();
        let _result = evaluator.evaluate(&module, &shared, 0);

        // Should produce a SourceCommand because trigger > 0.1
        // (Source defaults to "MediaFile" with empty path, create_source_command checks empty path)
        // Wait, SourceType::new_media_file("") -> default empty path.
        // create_source_command returns None if path is empty.
        // Let's set a path.
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == s_id) {
            if let ModulePartType::Source(SourceType::MediaFile { path, .. }) = &mut part.part_type
            {
                *path = "test.mp4".to_string();
            }
        }

        let result = evaluator.evaluate(&module, &shared, 0);
        assert!(result.source_commands.contains_key(&s_id));

        // Now remove connection
        module.remove_connection(t_id, 0, s_id, 0);

        let result = evaluator.evaluate(&module, &shared, 1);

        // A disconnected source defaults to trigger = 1.0, so it SHOULD generate a SourceCommand
        assert!(result.source_commands.contains_key(&s_id));
    }

    #[test]
    fn test_full_evaluation_pipeline() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        // Graph: FixedTrigger(Always) -> Source -> Layer -> Output

        // 1. Trigger
        let t_type = ModulePartType::Trigger(TriggerType::Fixed {
            interval_ms: 0,
            offset_ms: 0,
        });
        let t_id = module.add_part_with_type(t_type, (0.0, 0.0));

        // 2. Source
        let s_id = module.add_part(crate::module::PartType::Source, (100.0, 0.0));
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == s_id) {
            if let ModulePartType::Source(SourceType::MediaFile { path, .. }) = &mut part.part_type
            {
                *path = "test.mp4".to_string();
            }
        }

        // 3. Layer
        let l_id = module.add_part(crate::module::PartType::Layer, (200.0, 0.0));

        // 4. Output
        let o_id = module.add_part(crate::module::PartType::Output, (300.0, 0.0));

        // Connections
        module.add_connection(t_id, 0, s_id, 0); // Trigger -> Source Trigger
        module.add_connection(s_id, 0, l_id, 0); // Source Media -> Layer Input
        module.add_connection(l_id, 0, o_id, 0); // Layer Output -> Output Layer In

        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default(), 0);

        // Verify RenderOp
        assert_eq!(result.render_ops.len(), 1);
        let op = &result.render_ops[0];
        assert_eq!(op.output_part_id, o_id);
        assert_eq!(op.layer_part_id, l_id);
        assert_eq!(op.source_part_id, Some(s_id));

        // Verify SourceCommand
        assert!(result.source_commands.contains_key(&s_id));
        if let Some(SourceCommand::PlayMedia { path, .. }) = result.source_commands.get(&s_id) {
            assert_eq!(path, "test.mp4");
        } else {
            panic!("Expected PlayMedia command");
        }
    }

    #[test]
    fn test_render_trace_prefers_layer_visual_input_over_trigger_input() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        let t_id = module.add_part_with_type(
            ModulePartType::Trigger(TriggerType::Fixed {
                interval_ms: 0,
                offset_ms: 0,
            }),
            (0.0, 0.0),
        );

        let s_id = module.add_part(crate::module::PartType::Source, (100.0, 0.0));
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == s_id) {
            if let ModulePartType::Source(SourceType::MediaFile { path, .. }) = &mut part.part_type
            {
                *path = "test.mp4".to_string();
            }
        }

        let l_id = module.add_part(crate::module::PartType::Layer, (200.0, 0.0));
        let o_id = module.add_part(crate::module::PartType::Output, (300.0, 0.0));

        // Repro: if the trigger connection is inserted first, the render trace must
        // still follow layer socket 0 (visual chain) rather than socket 1 (trigger).
        module.add_connection(t_id, 0, l_id, 1);
        module.add_connection(s_id, 0, l_id, 0);
        module.add_connection(l_id, 0, o_id, 0);

        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default(), 0);

        assert_eq!(result.render_ops.len(), 1);
        let op = &result.render_ops[0];
        assert_eq!(op.output_part_id, o_id);
        assert_eq!(op.layer_part_id, l_id);
        assert_eq!(op.source_part_id, Some(s_id));
    }

    #[test]
    fn test_link_system_master_slave() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        // Master Node (Trigger Type for simplicity, acting as master)
        let m_type = ModulePartType::Trigger(TriggerType::Fixed {
            interval_ms: 0,
            offset_ms: 0,
        });
        let m_id = module.add_part_with_type(m_type, (0.0, 0.0));

        // Configure as Master
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == m_id) {
            part.link_data.mode = LinkMode::Master;
            part.link_data.trigger_input_enabled = true; // Use trigger input to drive link
            part.outputs.push(crate::module::ModuleSocket {
                name: "Link Out".to_string(),
                socket_type: crate::module::ModuleSocketType::Link,
            });
            // Also needs Trigger In socket if enabled
            part.inputs.push(crate::module::ModuleSocket {
                name: "Trigger In (Vis)".to_string(),
                socket_type: crate::module::ModuleSocketType::Trigger,
            });
        }

        // Driving Trigger
        let t_id = module.add_part_with_type(
            ModulePartType::Trigger(TriggerType::Fixed {
                interval_ms: 0,
                offset_ms: 0,
            }),
            (-100.0, 0.0),
        );

        // Connect Driving Trigger -> Master Trigger In (Vis)
        // Master Trigger In index: 0 (since Triggers usually have 0 inputs)
        module.add_connection(t_id, 0, m_id, 0);

        // Slave Node (Layer)
        let s_id = module.add_part(crate::module::PartType::Layer, (100.0, 0.0));
        // Configure as Slave
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == s_id) {
            part.link_data.mode = LinkMode::Slave;
            part.inputs.push(crate::module::ModuleSocket {
                name: "Link In".to_string(),
                socket_type: crate::module::ModuleSocketType::Link,
            });
        }

        // Connect Master Link Out -> Slave Link In
        // Master Link Out index: 1 (0 is Trigger Out)
        // Slave Link In index: 2 (0=Media, 1=Trigger)
        module.add_connection(m_id, 1, s_id, 2);

        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default(), 0);

        // Master ID in trigger_values should have 2 values: Trigger Out (1.0) and Link Out (1.0)
        let m_values = &result.trigger_values[&m_id];
        assert!(m_values.len() >= 2);
        assert_eq!(m_values[1], 1.0); // Link Out should be active
    }

    #[test]
    fn test_render_op_pooling() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        // 1. Layer -> Output
        let l_id = module.add_part(crate::module::PartType::Layer, (0.0, 0.0));
        let o_id = module.add_part(crate::module::PartType::Output, (100.0, 0.0));
        module.add_connection(l_id, 0, o_id, 0);

        let shared = crate::module::SharedMediaState::default();

        // Pass 1: Should create one RenderOp
        evaluator.evaluate(&module, &shared, 0);
        assert_eq!(evaluator.cached_result.render_ops.len(), 1);
        assert_eq!(evaluator.cached_result.spare_render_ops.len(), 0);

        // Pass 2: Should recycle the RenderOp
        // Note: evaluate() calls clear() at the start, moving render_ops to spare.
        // Then it pops one.
        evaluator.evaluate(&module, &shared, 0);
        assert_eq!(evaluator.cached_result.render_ops.len(), 1);
        // Spare should be 0 because we popped the one that was recycled
        assert_eq!(evaluator.cached_result.spare_render_ops.len(), 0);

        // Pass 3: Reduce workload (no output connection)
        module.remove_connection(l_id, 0, o_id, 0);
        evaluator.evaluate(&module, &shared, 1);

        // render_ops should be empty
        assert_eq!(evaluator.cached_result.render_ops.len(), 0);
        // spare_render_ops should contain the recycled one
        assert_eq!(evaluator.cached_result.spare_render_ops.len(), 1);
    }
}

/// Render operation containing all info needed to render a layer to an output
#[derive(Debug, Clone)]
pub struct RenderOp {
    /// The output node ID (Part ID)
    pub output_part_id: ModulePartId,
    /// The specific output type configuration
    pub output_type: OutputType,

    /// The layer node ID calling for this render
    pub layer_part_id: ModulePartId,
    /// The mesh geometry to use
    pub mesh: MeshType,
    /// Layer opacity
    pub opacity: f32,
    /// Layer blend mode
    pub blend_mode: Option<BlendModeType>,
    /// Mapping mode active (render grid)
    pub mapping_mode: bool,

    /// Source part ID (if any)
    pub source_part_id: Option<ModulePartId>,
    /// Source-specific properties (color, transform, flip)
    pub source_props: SourceProperties,
    /// Applied effects in order (Source -> Effect1 -> Effect2 -> ...)
    pub effects: Vec<ModulizerType>,
    /// Applied masks
    pub masks: Vec<MaskType>,
}

/// Evaluation result for a single frame
#[derive(Debug, Clone, Default)]
pub struct ModuleEvalResult {
    /// Trigger values: part_id -> (output_index -> value)
    pub trigger_values: HashMap<ModulePartId, Vec<f32>>,
    /// Source commands: part_id -> SourceCommand
    pub source_commands: HashMap<ModulePartId, SourceCommand>,
    /// Render operations to specific outputs
    pub render_ops: Vec<RenderOp>,
    /// Spare render operations for reuse (object pooling)
    pub spare_render_ops: Vec<RenderOp>,
}

impl ModuleEvalResult {
    /// Clears the result for reuse, preserving capacity where possible
    pub fn clear(&mut self) {
        // Clear trigger values but keep the vectors to reuse their capacity
        for values in self.trigger_values.values_mut() {
            values.clear();
        }
        // Note: We don't remove keys from trigger_values map to reuse map capacity and vectors.
        // However, if the graph changes, we might accumulate stale keys.
        // For a fixed graph (most of the time), this is fine.
        // To be safe against memory leaks on graph changes, we could occasionally prune.
        // For now, simple reuse is a huge win.

        // Source commands are typically small (one per source), but we can clear the map
        self.source_commands.clear();

        // Recycle render ops instead of clearing (which drops/frees internal vectors)
        self.spare_render_ops.append(&mut self.render_ops);
    }
}

/// Command for a source node
#[derive(Debug, Clone)]
pub enum SourceCommand {
    /// Play media from a local path.
    PlayMedia {
        /// Path to the media file.
        path: String,
        /// Current trigger value.
        trigger_value: f32,
    },
    /// Play media from the shared library.
    PlaySharedMedia {
        /// Unique identifier for the shared media.
        id: String,
        /// Path to the media file.
        path: String,
        /// Current trigger value.
        trigger_value: f32,
    },
    /// Render a shader with the given parameters.
    PlayShader {
        /// Name of the shader.
        name: String,
        /// List of (parameter name, value) tuples.
        params: Vec<(String, f32)>,
        /// Current trigger value.
        trigger_value: f32,
    },
    /// Receive frames from an NDI source.
    NdiInput {
        /// Name of the NDI source.
        source_name: Option<String>,
        /// Current trigger value.
        trigger_value: f32,
    },
    /// Receive frames from a live video device.
    LiveInput {
        /// ID of the capture device.
        device_id: u32,
        /// Current trigger value.
        trigger_value: f32,
    },
    #[cfg(target_os = "windows")]
    /// Receive frames from a Spout sender (Windows only).
    SpoutInput {
        /// Name of the Spout sender.
        sender_name: String,
        /// Current trigger value.
        trigger_value: f32,
    },
    /// Input from the Bevy game engine.
    BevyInput {
        /// Current trigger value.
        trigger_value: f32,
    },
    /// Render a 3D model via Bevy.
    Bevy3DModel {
        /// Path to the 3D model.
        path: String,
        /// Position in 3D space.
        position: [f32; 3],
        /// Rotation in degrees.
        rotation: [f32; 3],
        /// Scale factor.
        scale: [f32; 3],
        /// Current trigger value.
        trigger_value: f32,
    },
    /// Control Philips Hue smart lights.
    HueOutput {
        /// Brightness level (0.0 - 1.0).
        brightness: f32,
        /// Hue value (0.0 - 1.0, optional).
        hue: Option<f32>,
        /// Saturation level (0.0 - 1.0, optional).
        saturation: Option<f32>,
        /// Strobe speed (0.0 - 1.0, optional).
        strobe: Option<f32>,
        /// List of light IDs to control.
        ids: Option<Vec<String>>,
    },
}

/// Module graph evaluator
pub struct ModuleEvaluator {
    /// Current trigger data from audio analysis
    audio_trigger_data: AudioTriggerData,
    /// Creation time for timing calculations
    start_time: Instant,
    /// Per-node state for stateful triggers (e.g., Random)
    #[allow(dead_code)]
    trigger_states: HashMap<ModulePartId, TriggerState>,
    /// Reusable result buffer to avoid allocations
    cached_result: ModuleEvalResult,

    /// Cached indices per module ID to support multi-module switching
    indices_cache: HashMap<crate::module::ModuleId, Arc<ModuleGraphIndices>>,

    /// Currently active keyboard keys (for Shortcut triggers)
    active_keys: std::collections::HashSet<String>,

    /// State for smoothed trigger inputs: (PartId, SocketIdx) -> (Current Value, Last Updated Frame)
    trigger_smoothing_state: RefCell<HashMap<(ModulePartId, usize), (f32, u64)>>,

    /// Manually fired triggers for the current frame
    manual_triggers: std::collections::HashSet<ModulePartId>,

    /// MIDI notes/CCs received this frame: (channel, note/cc)
    midi_triggers: std::collections::HashSet<(u8, u8)>,

    /// OSC addresses received this frame
    osc_triggers: std::collections::HashSet<String>,

    /// Current evaluation frame count (used to prevent smoothing multiple times per frame)
    current_frame: u64,

    /// Time of the last evaluation (used for delta time calculation)
    last_eval_time: Instant,

    /// The delta time calculated at the start of the current evaluation frame
    current_dt: f32,
}

impl Default for ModuleEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleEvaluator {
    /// Create a new module evaluator
    pub fn new() -> Self {
        Self {
            audio_trigger_data: AudioTriggerData::default(),
            start_time: Instant::now(),
            trigger_states: HashMap::new(),
            cached_result: ModuleEvalResult::default(),
            indices_cache: HashMap::new(),
            active_keys: std::collections::HashSet::new(),
            trigger_smoothing_state: RefCell::new(HashMap::new()),
            manual_triggers: std::collections::HashSet::new(),
            midi_triggers: std::collections::HashSet::new(),
            osc_triggers: std::collections::HashSet::new(),
            current_frame: 0,
            last_eval_time: Instant::now(),
            current_dt: 0.0,
        }
    }

    /// Inject the frame delta reported by the outer app loop.
    pub fn set_delta_time(&mut self, dt: f32) {
        let clamped = dt.clamp(0.0, 0.5);
        self.current_dt = clamped;
        self.last_eval_time = Instant::now() - std::time::Duration::from_secs_f32(clamped);
    }

    /// Manually fire a trigger node for the next evaluation frame
    pub fn trigger_node(&mut self, part_id: ModulePartId) {
        self.manual_triggers.insert(part_id);
    }

    /// Record an OSC message for the next evaluation frame
    pub fn record_osc(&mut self, address: &str) {
        self.osc_triggers.insert(address.to_string());
    }

    /// Update audio trigger data from analysis
    pub fn update_audio(&mut self, analysis: &AudioAnalysisV2) {
        self.audio_trigger_data.band_energies = analysis.band_energies;
        self.audio_trigger_data.rms_volume = analysis.rms_volume;
        self.audio_trigger_data.peak_volume = analysis.peak_volume;
        self.audio_trigger_data.beat_detected = analysis.beat_detected;
        self.audio_trigger_data.beat_strength = analysis.beat_strength;
        self.audio_trigger_data.bpm = analysis.tempo_bpm;
    }

    /// Update active keyboard keys for evaluation.
    pub fn update_keys(&mut self, keys: &std::collections::HashSet<String>) {
        self.active_keys = keys.clone();
    }

    fn apply_smoothing(
        &self,
        part_id: ModulePartId,
        socket_idx: usize,
        target_val: f32,
        mode: &crate::module::TriggerMappingMode,
    ) -> f32 {
        if let crate::module::TriggerMappingMode::Smoothed { attack, release } = mode {
            let state_key = (part_id, socket_idx);
            let mut cache = self.trigger_smoothing_state.borrow_mut();
            let (mut current_val, last_frame) =
                cache.get(&state_key).copied().unwrap_or((target_val, 0));
            if last_frame != self.current_frame {
                let time_constant = if target_val > current_val {
                    *attack
                } else {
                    *release
                };
                if time_constant > 0.001 {
                    let alpha = 1.0 - (-self.current_dt / time_constant).exp();
                    current_val = current_val + (target_val - current_val) * alpha;
                } else {
                    current_val = target_val;
                }
                cache.insert(state_key, (current_val, self.current_frame));
            }
            current_val
        } else {
            target_val
        }
    }

    /// Get a spare RenderOp from the cache or create a new one (Object Pooling)
    fn get_spare_render_op(&mut self) -> RenderOp {
        self.cached_result
            .spare_render_ops
            .pop()
            .unwrap_or_else(|| RenderOp {
                output_part_id: 0,
                output_type: OutputType::Projector {
                    id: 0,
                    name: String::new(),
                    hide_cursor: false,
                    target_screen: 0,
                    show_in_preview_panel: true,
                    extra_preview_window: false,
                    output_width: 1920,
                    output_height: 1080,
                    output_fps: 60.0,
                    ndi_enabled: false,
                    ndi_stream_name: String::new(),
                },
                layer_part_id: 0,
                mesh: MeshType::default(),
                opacity: 1.0,
                blend_mode: None,
                mapping_mode: false,
                source_part_id: None,
                source_props: SourceProperties::default_identity(),
                effects: Vec::new(),
                masks: Vec::new(),
            })
    }

    /// Evaluate the module graph for the current frame.
    pub fn evaluate(
        &mut self,
        module: &MapFlowModule,
        shared_state: &SharedMediaState,
        graph_revision: u64,
    ) -> &ModuleEvalResult {
        let mut rng = rand::rng();
        let now = Instant::now();

        self.current_dt = now
            .duration_since(self.last_eval_time)
            .as_secs_f32()
            .min(0.5);
        self.last_eval_time = now;
        self.current_frame = self.current_frame.wrapping_add(1);

        // Clear previous result for reuse
        self.cached_result.clear();
        let indices_valid = if let Some(cache) = self.indices_cache.get(&module.id) {
            cache.last_revision == graph_revision
        } else {
            false
        };
        if !indices_valid {
            // Rebuild cache
            let mut part_index_cache = HashMap::new();
            let mut conn_index_cache = HashMap::new();
            for (idx, part) in module.parts.iter().enumerate() {
                part_index_cache.insert(part.id, idx);
            }
            for (idx, conn) in module.connections.iter().enumerate() {
                conn_index_cache
                    .entry(conn.to_part)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
            self.indices_cache.insert(
                module.id,
                Arc::new(ModuleGraphIndices {
                    part_index_cache,
                    conn_index_cache,
                    last_revision: graph_revision,
                }),
            );
        }

        // Clone the Arc to avoid borrowing self.indices_cache while borrowing self mutably later
        let indices = self.indices_cache[&module.id].clone();

        // === DIAGNOSTICS: Log module structure ===
        // (Diagnostic logging code removed for brevity/performance in hot path unless feature enabled?
        // keeping it as it was but maybe less frequently? leaving as is per instructions to preserve functionality)

        // Step 1: Evaluate all trigger nodes
        for part in &module.parts {
            if let ModulePartType::Trigger(trigger_type) = &part.part_type {
                let state = self.trigger_states.entry(part.id).or_default();
                let values = self
                    .cached_result
                    .trigger_values
                    .entry(part.id)
                    .or_default();
                values.clear();

                let manual_fired = self.manual_triggers.contains(&part.id);
                Self::compute_trigger_output(
                    trigger_type,
                    state,
                    &self.audio_trigger_data,
                    self.start_time,
                    shared_state,
                    &self.active_keys,
                    manual_fired,
                    values,
                    &mut rng,
                );
            }
        }
        let mut trigger_inputs =
            self.compute_trigger_inputs(module, &self.cached_result.trigger_values);
        for part in &module.parts {
            if part.link_data.mode == LinkMode::Master {
                let mut activity = 1.0;
                if part.link_data.trigger_input_enabled {
                    activity = trigger_inputs.get(&part.id).copied().unwrap_or(0.0);
                }
                if !part.outputs.is_empty() {
                    let output_count = part.outputs.len();
                    let values = self
                        .cached_result
                        .trigger_values
                        .entry(part.id)
                        .or_default();
                    values.clear();
                    values.resize(output_count, 0.0);
                    values[output_count - 1] = activity;
                }
            }
        }

        // Step 4: Second propagation (Master Link Out -> Slave Link In)
        trigger_inputs = self.compute_trigger_inputs(module, &self.cached_result.trigger_values);

        // Step 5: Process Slave Behaviors (Invert Link Input)
        for part in &module.parts {
            if part.link_data.mode == LinkMode::Slave {
                if let Some(val) = trigger_inputs.get_mut(&part.id) {
                    if part.link_data.behavior == LinkBehavior::Inverted {
                        *val = 1.0 - (*val).clamp(0.0, 1.0);
                    }
                }
            }
        }

        // Step 6: Generate source commands
        let socket_inputs = self.compute_socket_inputs(module, &self.cached_result.trigger_values);

        for part in &module.parts {
            if let ModulePartType::Source(source_type) = &part.part_type {
                // Default to 1.0 (playing) so media files play even if no trigger is attached
                let trigger_value = trigger_inputs.get(&part.id).copied().unwrap_or(1.0);
                if let Some(mut cmd) =
                    self.create_source_command(source_type, trigger_value, shared_state)
                {
                    for (socket_idx, config) in &part.trigger_targets {
                        if let Some(socket_vals) = socket_inputs.get(&part.id) {
                            if let Some(&raw_val) = socket_vals.get(socket_idx) {
                                let val = self.apply_smoothing(
                                    part.id,
                                    *socket_idx,
                                    config.apply(raw_val),
                                    &config.mode,
                                );
                                match (&mut cmd, &config.target) {
                                    (
                                        SourceCommand::Bevy3DModel { position, .. },
                                        crate::module::TriggerTarget::Position3D,
                                    ) => {
                                        position[0] = val;
                                        position[1] = val;
                                        position[2] = val;
                                    }
                                    (
                                        SourceCommand::Bevy3DModel { scale, .. },
                                        crate::module::TriggerTarget::Scale3D,
                                    ) => {
                                        scale[0] = val;
                                        scale[1] = val;
                                        scale[2] = val;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    self.cached_result.source_commands.insert(part.id, cmd);
                }
            }
            // Generate output commands for Hue (which acts like a Sink/Output)
            if let ModulePartType::Output(OutputType::Hue { .. }) = &part.part_type {
                let trigger_value = trigger_inputs.get(&part.id).copied().unwrap_or(0.0);
                self.cached_result.source_commands.insert(
                    part.id,
                    SourceCommand::HueOutput {
                        brightness: trigger_value,
                        hue: None,
                        saturation: None,
                        strobe: None,
                        ids: None,
                    },
                );
            }

            // Generate output commands for New Hue Nodes
            if let ModulePartType::Hue(hue_node) = &part.part_type {
                // We need per-socket inputs here
                let socket_inputs =
                    self.compute_socket_inputs(module, &self.cached_result.trigger_values);

                let brightness = socket_inputs
                    .get(&part.id)
                    .and_then(|m| m.get(&0))
                    .copied()
                    .unwrap_or(0.0);
                let hue = socket_inputs.get(&part.id).and_then(|m| m.get(&1)).copied(); // Socket 1: Color(Hue)
                let strobe = socket_inputs.get(&part.id).and_then(|m| m.get(&2)).copied(); // Socket 2: Strobe
                                                                                           // Note: If we added Saturation later it would be index 3? For now assume inputs from get_default_sockets:
                                                                                           // 0: Brightness, 1: Color(Hue), 2: Strobe. (Wait, previous view showed 3 sockets)

                // Extract IDs from node type
                use crate::module::HueNodeType;
                let ids = match hue_node {
                    HueNodeType::SingleLamp { id, .. } => Some(vec![id.clone()]),
                    HueNodeType::MultiLamp { ids, .. } => Some(ids.clone()),
                    HueNodeType::EntertainmentGroup { .. } => None, // Broadcast to group
                };
                self.cached_result.source_commands.insert(
                    part.id,
                    SourceCommand::HueOutput {
                        brightness,
                        hue,
                        saturation: None,
                        strobe,
                        ids,
                    },
                );
            }
        }

        // Step 4: Trace Render Pipeline
        for part in &module.parts {
            if let ModulePartType::Output(output_type) = &part.part_type {
                if let Some(conn_idx) = primary_render_connection_idx(module, &indices, part.id) {
                    let conn = &module.connections[conn_idx];

                    // Look up the layer part
                    if let Some(&layer_idx) = indices.part_index_cache.get(&conn.from_part) {
                        let layer_part = &module.parts[layer_idx];
                        let link_opacity =
                            trigger_inputs.get(&layer_part.id).copied().unwrap_or(1.0);
                        if let ModulePartType::Layer(layer_type) = &layer_part.part_type {
                            let (mesh, opacity, blend_mode, mapping_mode) = match layer_type {
                                LayerType::Single {
                                    mesh,
                                    opacity,
                                    blend_mode,
                                    mapping_mode,
                                    ..
                                } => (mesh, opacity, blend_mode, mapping_mode),
                                LayerType::Group {
                                    mesh,
                                    opacity,
                                    blend_mode,
                                    mapping_mode,
                                    ..
                                } => (mesh, opacity, blend_mode, mapping_mode),
                                _ => continue,
                            };
                            let mut op = self.get_spare_render_op();
                            op.output_part_id = part.id;
                            op.output_type = output_type.clone();
                            op.layer_part_id = layer_part.id;
                            op.opacity = *opacity * link_opacity;
                            op.blend_mode = *blend_mode;
                            op.mapping_mode = *mapping_mode;
                            self.trace_chain_into(layer_part.id, module, &mut op, mesh, &indices);
                            self.cached_result.render_ops.push(op);
                        }
                    } else {
                        tracing::warn!(
                            "ModuleEval: Output {} connected to non-Layer node {}",
                            part.id,
                            conn.from_part
                        );
                    }
                }
            }
        }

        // Final step: Clear triggers for next frame
        self.manual_triggers.clear();
        self.midi_triggers.clear();
        self.osc_triggers.clear();

        &self.cached_result
    }

    fn trace_chain_into(
        &self,
        start_node_id: ModulePartId,
        module: &MapFlowModule,
        op: &mut RenderOp,
        default_mesh: &MeshType,
        indices: &ModuleGraphIndices,
    ) {
        op.effects.clear();
        op.masks.clear();
        op.source_part_id = None;
        op.source_props = SourceProperties::default_identity();

        let mut override_mesh = None;
        let mut current_id = start_node_id;

        // Cycle detection
        let mut visited = std::collections::HashSet::with_capacity(16);
        visited.insert(start_node_id);

        // Optimization: Use the part index cache that was already built in evaluate()
        // This avoids an O(N) allocation and iteration for every layer being rendered.
        let _part_index = &indices.part_index_cache;

        tracing::debug!(
            "trace_chain: Starting from node {} in module {}",
            start_node_id,
            module.name
        );

        let trigger_values = &self.cached_result.trigger_values;

        // Safety limit to prevent infinite loops in cyclic graphs
        for _iteration in 0..50 {
            // Apply Trigger Targets for the current node
            // We need to find if any input sockets have triggers active and targets mapped
            if let Some(&part_idx) = indices.part_index_cache.get(&current_id) {
                let part = &module.parts[part_idx];

                // If this is a source, load its base properties first
                if let ModulePartType::Source(source_type) = &part.part_type {
                    op.source_part_id = Some(part.id);
                    if let SourceType::MediaFile {
                        opacity,
                        brightness,
                        contrast,
                        saturation,
                        hue_shift,
                        scale_x,
                        scale_y,
                        rotation,
                        offset_x,
                        offset_y,
                        flip_horizontal,
                        flip_vertical,
                        ..
                    } = source_type
                    {
                        op.source_props = SourceProperties {
                            opacity: *opacity,
                            brightness: *brightness,
                            contrast: *contrast,
                            saturation: *saturation,
                            hue_shift: *hue_shift,
                            scale_x: *scale_x,
                            scale_y: *scale_y,
                            rotation: *rotation,
                            offset_x: *offset_x,
                            offset_y: *offset_y,
                            flip_horizontal: *flip_horizontal,
                            flip_vertical: *flip_vertical,
                        };
                    }
                }

                if !part.trigger_targets.is_empty() {
                    tracing::debug!(
                        "Part {} has {} trigger targets",
                        part.id,
                        part.trigger_targets.len()
                    );
                }
                for (socket_idx, config) in &part.trigger_targets {
                    // Find connection to this socket
                    let mut trigger_val = 0.0;
                    if let Some(conn_indices) = indices.conn_index_cache.get(&current_id) {
                        for &conn_idx in conn_indices {
                            let conn = &module.connections[conn_idx];
                            if conn.to_socket == *socket_idx {
                                if let Some(from_values) = trigger_values.get(&conn.from_part) {
                                    if let Some(val) = from_values.get(conn.from_socket) {
                                        trigger_val = *val;
                                    }
                                }
                                break;
                            }
                        }
                    }
                    let val = self.apply_smoothing(
                        part.id,
                        *socket_idx,
                        config.apply(trigger_val),
                        &config.mode,
                    );
                    match &config.target {
                        crate::module::TriggerTarget::Opacity => op.source_props.opacity = val,
                        crate::module::TriggerTarget::Brightness => {
                            op.source_props.brightness = val
                        }
                        crate::module::TriggerTarget::Contrast => op.source_props.contrast = val,
                        crate::module::TriggerTarget::Saturation => {
                            op.source_props.saturation = val
                        }
                        crate::module::TriggerTarget::HueShift => op.source_props.hue_shift = val,
                        crate::module::TriggerTarget::ScaleX => op.source_props.scale_x = val,
                        crate::module::TriggerTarget::ScaleY => op.source_props.scale_y = val,
                        crate::module::TriggerTarget::Rotation => op.source_props.rotation = val,
                        crate::module::TriggerTarget::OffsetX => op.source_props.offset_x = val,
                        crate::module::TriggerTarget::OffsetY => op.source_props.offset_y = val,
                        crate::module::TriggerTarget::FlipH => {
                            op.source_props.flip_horizontal = val > 0.5
                        }
                        crate::module::TriggerTarget::FlipV => {
                            op.source_props.flip_vertical = val > 0.5
                        }
                        crate::module::TriggerTarget::Param(name) => {
                            if let Some(ModulizerType::Effect { params, .. }) =
                                op.effects.last_mut()
                            {
                                params.insert(name.clone(), val);
                            }
                        }
                        _ => {}
                    }
                }
            }

            // 2. Find PREVIOUS node in chain
            if let Some(conn_idx) = primary_render_connection_idx(module, indices, current_id) {
                let conn = &module.connections[conn_idx];

                // Cycle detection
                if !visited.insert(conn.from_part) {
                    tracing::warn!(
                        "Cycle detected in module graph chain starting at node {}",
                        start_node_id
                    );
                    break;
                }

                if let Some(&part_idx) = indices.part_index_cache.get(&conn.from_part) {
                    let part = &module.parts[part_idx];
                    match &part.part_type {
                        ModulePartType::Source(source_type) => {
                            op.source_part_id = Some(part.id);

                            // Helper to extract props from any source variant that has them
                            let mut extracted_props = None;

                            match source_type {
                                SourceType::MediaFile {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                }
                                | SourceType::VideoUni {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                }
                                | SourceType::ImageUni {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                } => {
                                    extracted_props = Some(SourceProperties {
                                        opacity: *opacity,
                                        brightness: *brightness,
                                        contrast: *contrast,
                                        saturation: *saturation,
                                        hue_shift: *hue_shift,
                                        scale_x: *scale_x,
                                        scale_y: *scale_y,
                                        rotation: *rotation,
                                        offset_x: *offset_x,
                                        offset_y: *offset_y,
                                        flip_horizontal: *flip_horizontal,
                                        flip_vertical: *flip_vertical,
                                    });
                                }
                                SourceType::VideoMulti {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                }
                                | SourceType::ImageMulti {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                } => {
                                    extracted_props = Some(SourceProperties {
                                        opacity: *opacity,
                                        brightness: *brightness,
                                        contrast: *contrast,
                                        saturation: *saturation,
                                        hue_shift: *hue_shift,
                                        scale_x: *scale_x,
                                        scale_y: *scale_y,
                                        rotation: *rotation,
                                        offset_x: *offset_x,
                                        offset_y: *offset_y,
                                        flip_horizontal: *flip_horizontal,
                                        flip_vertical: *flip_vertical,
                                    });
                                }
                                _ => {}
                            }

                            if let Some(mut props) = extracted_props {
                                // Re-apply overrides since we just replaced with defaults
                                // (This structure is slightly inefficient, re-doing logic)
                                // Better: Apply overrides TO props.

                                // .. Re-run target logic ..
                                for (socket_idx, config) in &part.trigger_targets {
                                    // Find connection to this socket
                                    let mut trigger_val = 0.0;
                                    // L556 replacement
                                    if let Some(conn_indices) =
                                        indices.conn_index_cache.get(&part.id)
                                    {
                                        for &conn_idx in conn_indices {
                                            let conn = &module.connections[conn_idx];
                                            if conn.to_socket == *socket_idx {
                                                if let Some(from_values) =
                                                    trigger_values.get(&conn.from_part)
                                                {
                                                    if let Some(val) =
                                                        from_values.get(conn.from_socket)
                                                    {
                                                        trigger_val = *val;
                                                    }
                                                }
                                                break;
                                            }
                                        }
                                    }

                                    // Apply config if value is significant (or if fixed/random mode which might trigger on 0?)
                                    // Actually we just apply the mapping.
                                    match &config.target {
                                        crate::module::TriggerTarget::None => {}
                                        target => {
                                            // Apply mapping
                                            let raw_final_val = config.apply(trigger_val);
                                            let final_val = self.apply_smoothing(
                                                part.id,
                                                *socket_idx,
                                                raw_final_val,
                                                &config.mode,
                                            );

                                            tracing::debug!(
                                                "Trigger applying: part={}, socket={}, target={:?}, raw={}, final={}",
                                                part.id, socket_idx, target, trigger_val, final_val
                                            );

                                            match target {
                                                crate::module::TriggerTarget::Opacity => {
                                                    props.opacity = final_val;
                                                }
                                                crate::module::TriggerTarget::Brightness => {
                                                    props.brightness = final_val;
                                                }
                                                crate::module::TriggerTarget::Contrast => {
                                                    props.contrast = final_val;
                                                }
                                                crate::module::TriggerTarget::Saturation => {
                                                    props.saturation = final_val;
                                                }
                                                crate::module::TriggerTarget::HueShift => {
                                                    props.hue_shift = final_val;
                                                }
                                                crate::module::TriggerTarget::ScaleX => {
                                                    props.scale_x = final_val;
                                                }
                                                crate::module::TriggerTarget::ScaleY => {
                                                    props.scale_y = final_val;
                                                }
                                                crate::module::TriggerTarget::Rotation => {
                                                    props.rotation = final_val;
                                                }
                                                crate::module::TriggerTarget::OffsetX => {
                                                    props.offset_x = final_val;
                                                }
                                                crate::module::TriggerTarget::OffsetY => {
                                                    props.offset_y = final_val;
                                                }
                                                crate::module::TriggerTarget::FlipH => {
                                                    props.flip_horizontal = final_val > 0.5;
                                                }
                                                crate::module::TriggerTarget::FlipV => {
                                                    props.flip_vertical = final_val > 0.5;
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }

                                op.source_props = props;
                            }
                            break;
                        }
                        ModulePartType::Modulizer(mod_type) => {
                            op.effects.push(mod_type.clone());
                            current_id = part.id;
                        }
                        ModulePartType::Mask(mask_type) => {
                            op.masks.push(mask_type.clone());
                            current_id = part.id;
                        }
                        ModulePartType::Mesh(mesh_type) => {
                            if override_mesh.is_none() {
                                override_mesh = Some(mesh_type.clone());
                            }
                            current_id = part.id;
                        }
                        _ => {
                            break;
                        }
                    }
                } else {
                    break;
                }
            } else {
                warn_once!(
                    "trace_chain: Node {} not found in part_index, stopping traversal",
                    current_id
                );
                break;
            }
        }

        op.effects.reverse();
        op.masks.reverse();

        op.mesh = override_mesh.unwrap_or_else(|| default_mesh.clone());
    }

    /// Evaluate a trigger node and write output values to the provided buffer
    #[allow(clippy::too_many_arguments)]
    fn compute_trigger_output(
        trigger_type: &TriggerType,
        state: &mut TriggerState,
        audio_data: &AudioTriggerData,
        start_time: Instant,
        shared_state: &SharedMediaState,
        active_keys: &std::collections::HashSet<String>,
        manual_fired: bool,
        output: &mut Vec<f32>,
        rng: &mut impl rand::Rng,
    ) {
        // Helper to push and optionally invert/manual override value
        let push_val_internal = |val: f32, out: &mut Vec<f32>, invert: bool| {
            let base_val = if manual_fired { 1.0 } else { val };
            let final_val = if invert {
                1.0 - base_val.clamp(0.0, 1.0)
            } else {
                base_val
            };
            out.push(final_val);
        };

        match trigger_type {
            TriggerType::AudioFFT {
                threshold,
                output_config,
                ..
            } => {
                if output_config.frequency_bands {
                    for i in 0..9 {
                        let val = audio_data.band_energies[i];
                        let invert = output_config
                            .inverted_outputs
                            .contains(&format!("Band {}", i));
                        push_val_internal(if val > *threshold { val } else { 0.0 }, output, invert);
                    }
                }
                if output_config.volume_outputs {
                    push_val_internal(
                        audio_data.rms_volume,
                        output,
                        output_config.inverted_outputs.contains("RMS Volume"),
                    );
                    push_val_internal(
                        audio_data.peak_volume,
                        output,
                        output_config.inverted_outputs.contains("Peak Volume"),
                    );
                }
                if output_config.beat_output {
                    let invert = output_config.inverted_outputs.contains("Beat Out");
                    push_val_internal(
                        if audio_data.beat_detected { 1.0 } else { 0.0 },
                        output,
                        invert,
                    );
                }
            }
            TriggerType::Beat => push_val_internal(
                if audio_data.beat_detected { 1.0 } else { 0.0 },
                output,
                false,
            ),
            TriggerType::Random {
                min_interval_ms,
                max_interval_ms,
                probability,
            } => {
                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                let min_interval = u64::from(*min_interval_ms);
                let max_interval = u64::from((*max_interval_ms).max(*min_interval_ms));

                if !matches!(state, TriggerState::Random { .. }) {
                    *state = TriggerState::Random {
                        next_fire_time_ms: elapsed_ms
                            + rng.random_range(min_interval..=max_interval),
                    };
                }

                let mut triggered = false;
                if let TriggerState::Random { next_fire_time_ms } = state {
                    if elapsed_ms >= *next_fire_time_ms {
                        *next_fire_time_ms =
                            elapsed_ms + rng.random_range(min_interval..=max_interval);
                        if rng.random_range(0.0..=1.0) < *probability {
                            triggered = true;
                        }
                    }
                }

                push_val_internal(if triggered { 1.0 } else { 0.0 }, output, false);
            }
            TriggerType::Fixed {
                interval_ms,
                offset_ms,
            } => {
                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                let adjusted_time = elapsed_ms.saturating_sub(u64::from(*offset_ms));
                let interval = u64::from(*interval_ms);
                let val = if interval == 0 {
                    1.0
                } else {
                    let pulse_duration = (interval / 10).max(16);
                    let phase = adjusted_time % interval;
                    if phase < pulse_duration {
                        1.0
                    } else {
                        0.0
                    }
                };
                push_val_internal(val, output, false);
            }
            TriggerType::Midi { channel, note, .. } => {
                if let Some(&value) = shared_state.active_midi_cc.get(&(*channel, *note)) {
                    output.push(value as f32 / 127.0);
                } else {
                    let mut active_val = 0.0;
                    for (ev_ch, ev_note, velocity) in &shared_state.active_midi_events {
                        if ev_ch == channel && ev_note == note {
                            active_val = *velocity as f32 / 127.0;
                            break;
                        }
                    }
                    output.push(active_val);
                }
            }
            TriggerType::Osc { address } => {
                let mut active_val = 0.0;
                if let Some(values) = shared_state.active_osc_messages.get(address) {
                    active_val = values.first().copied().unwrap_or(1.0);
                }
                output.push(active_val);
            }
            TriggerType::Shortcut {
                key_code,
                modifiers,
            } => {
                let is_pressed = active_keys.contains(key_code);

                // Check modifiers bitmask (1=Shift, 2=Control, 4=Alt)
                let mut modifiers_match = true;
                if *modifiers & 1 != 0 && !active_keys.contains("Shift") {
                    modifiers_match = false;
                }
                if *modifiers & 2 != 0 && !active_keys.contains("Control") {
                    modifiers_match = false;
                }
                if *modifiers & 4 != 0 && !active_keys.contains("Alt") {
                    modifiers_match = false;
                }
                push_val_internal(
                    if is_pressed && modifiers_match {
                        1.0
                    } else {
                        0.0
                    },
                    output,
                    false,
                );
            }
        }
    }

    fn compute_trigger_inputs(
        &self,
        module: &MapFlowModule,
        trigger_values: &HashMap<ModulePartId, Vec<f32>>,
    ) -> HashMap<ModulePartId, f32> {
        let mut inputs = HashMap::new();
        for conn in &module.connections {
            if let Some(values) = trigger_values.get(&conn.from_part) {
                if let Some(&value) = values.get(conn.from_socket) {
                    let current = inputs.entry(conn.to_part).or_insert(0.0);
                    *current = f32::max(*current, value);
                }
            }
        }

        inputs
    }

    fn compute_socket_inputs(
        &self,
        module: &MapFlowModule,
        trigger_values: &HashMap<ModulePartId, Vec<f32>>,
    ) -> HashMap<ModulePartId, HashMap<usize, f32>> {
        let mut inputs: HashMap<ModulePartId, HashMap<usize, f32>> = HashMap::new();

        for conn in &module.connections {
            if let Some(values) = trigger_values.get(&conn.from_part) {
                if let Some(&value) = values.get(conn.from_socket) {
                    let part_inputs = inputs.entry(conn.to_part).or_default();
                    let current = part_inputs.entry(conn.to_socket).or_insert(0.0);
                    *current = f32::max(*current, value);
                }
            }
        }
        inputs
    }

    fn create_source_command(
        &self,
        source_type: &SourceType,
        trigger_value: f32,
        shared_state: &SharedMediaState,
    ) -> Option<SourceCommand> {
        if trigger_value < 0.1 {
            return None;
        }
        match source_type {
            SourceType::MediaFile { path, .. }
            | SourceType::VideoUni { path, .. }
            | SourceType::ImageUni { path, .. } => {
                if path.is_empty() {
                    return None;
                }
                Some(SourceCommand::PlayMedia {
                    path: path.clone(),
                    trigger_value,
                })
            }
            SourceType::VideoMulti { shared_id, .. } | SourceType::ImageMulti { shared_id, .. } => {
                shared_state
                    .get(shared_id)
                    .map(|item| SourceCommand::PlaySharedMedia {
                        id: shared_id.clone(),
                        path: item.path.clone(),
                        trigger_value,
                    })
            }
            SourceType::Shader { name, params } => Some(SourceCommand::PlayShader {
                name: name.clone(),
                params: params.clone(),
                trigger_value,
            }),
            SourceType::NdiInput { source_name } => Some(SourceCommand::NdiInput {
                source_name: source_name.clone(),
                trigger_value,
            }),
            SourceType::LiveInput { device_id } => Some(SourceCommand::LiveInput {
                device_id: *device_id,
                trigger_value,
            }),
            #[cfg(target_os = "windows")]
            SourceType::SpoutInput { sender_name } => Some(SourceCommand::SpoutInput {
                sender_name: sender_name.clone(),
                trigger_value,
            }),
            SourceType::Bevy3DModel {
                path,
                position,
                rotation,
                scale,
                ..
            } => Some(SourceCommand::Bevy3DModel {
                path: path.clone(),
                position: *position,
                rotation: *rotation,
                scale: *scale,
                trigger_value,
            }),
            _ => Some(SourceCommand::BevyInput { trigger_value }),
        }
    }
}
