mod smoothing;
mod traversal;
mod triggers;

use crate::audio::analyzer_v2::AudioAnalysisV2;
use crate::audio_reactive::AudioTriggerData;
use crate::module::{
    BlendModeType, LayerType, LinkBehavior, LinkMode, VorceModule, MaskType, MeshType,
    ModulePartId, ModulePartType, ModulizerType, OutputType, SharedMediaState, SourceType,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use super::types::*;

/// The evaluator traverses the module graph and computes output values.
pub struct ModuleEvaluator {
    /// Current trigger data from audio analysis
    pub(crate) audio_trigger_data: AudioTriggerData,
    /// Creation time for timing calculations
    pub(crate) start_time: Instant,
    /// Per-node state for stateful triggers (e.g., Random)
    #[allow(dead_code)]
    pub(crate) trigger_states: HashMap<ModulePartId, TriggerState>,
    /// Reusable result buffer to avoid allocations
    pub cached_result: ModuleEvalResult,

    /// Cached indices per module ID to support multi-module switching
    pub(crate) indices_cache: HashMap<crate::module::ModuleId, Arc<ModuleGraphIndices>>,

    /// Currently active keyboard keys (for Shortcut triggers)
    pub(crate) active_keys: std::collections::HashSet<String>,

    /// State for smoothed trigger inputs: (PartId, SocketIdx) -> (Current Value, Last Updated Frame)
    pub(crate) trigger_smoothing_state: RefCell<HashMap<(ModulePartId, usize), (f32, u64)>>,

    /// Manually fired triggers for the current frame
    pub(crate) manual_triggers: std::collections::HashSet<ModulePartId>,

    /// MIDI notes/CCs received this frame: (channel, note/cc)
    pub(crate) midi_triggers: std::collections::HashSet<(u8, u8)>,

    /// OSC addresses received this frame
    pub(crate) osc_triggers: std::collections::HashSet<String>,

    /// Current evaluation frame count (used to prevent smoothing multiple times per frame)
    pub(crate) current_frame: u64,

    /// Time of the last evaluation (used for delta time calculation)
    pub(crate) last_eval_time: Instant,

    /// The delta time calculated at the start of the current evaluation frame
    pub(crate) current_dt: f32,
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
        module: &VorceModule,
        shared_state: &SharedMediaState,
        graph_revision: u64,
    ) -> &mut ModuleEvalResult {
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

        &mut self.cached_result
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

#[cfg(test)]
mod evaluator_tests {
    use super::*;
    use crate::audio::analyzer_v2::AudioAnalysisV2;
    use crate::module::{
        AudioTriggerOutputConfig, VorceModule, ModulePartType, SourceType, TriggerType,
    };
    use crate::module_eval::ModuleEvaluator;
    use std::time::Duration;

    fn create_test_module() -> VorceModule {
        VorceModule {
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
        module.add_connection(
            t_id,
            "trigger_out".to_string(),
            s_id,
            "trigger_in".to_string(),
        ); // Trigger Out -> Source Trigger In

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
        module.remove_connection(
            t_id,
            "trigger_out".to_string(),
            s_id,
            "trigger_in".to_string(),
        );

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
        module.add_connection(
            t_id,
            "trigger_out".to_string(),
            s_id,
            "trigger_in".to_string(),
        );
        module.add_connection(s_id, "media_out".to_string(), l_id, "media_in".to_string());
        module.add_connection(l_id, "layer_out".to_string(), o_id, "layer_in".to_string());

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
        module.add_connection(
            t_id,
            "trigger_out".to_string(),
            l_id,
            "trigger_in".to_string(),
        );
        module.add_connection(s_id, "media_out".to_string(), l_id, "media_in".to_string());
        module.add_connection(l_id, "layer_out".to_string(), o_id, "layer_in".to_string());

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
            part.outputs.push(crate::module::ModuleSocket::output(
                "link_out",
                "Link Out",
                crate::module::ModuleSocketType::Link,
            ));
            // Also needs Trigger In socket if enabled
            part.inputs.push(
                crate::module::ModuleSocket::input_mappable(
                    "trigger_vis_in",
                    "Trigger In (Vis)",
                    crate::module::ModuleSocketType::Trigger,
                )
                .multi_input(),
            );
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
        module.add_connection(
            t_id,
            "trigger_out".to_string(),
            m_id,
            "trigger_vis_in".to_string(),
        );

        // Slave Node (Layer)
        let s_id = module.add_part(crate::module::PartType::Layer, (100.0, 0.0));
        // Configure as Slave
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == s_id) {
            part.link_data.mode = LinkMode::Slave;
            part.inputs.push(crate::module::ModuleSocket::input(
                "link_in",
                "Link In",
                crate::module::ModuleSocketType::Link,
            ));
        }

        // Connect Master Link Out -> Slave Link In
        module.add_connection(m_id, "link_out".to_string(), s_id, "link_in".to_string());

        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default(), 0);

        // Master ID in trigger_values should have 2 values: Trigger Out (1.0) and Link Out (1.0)
        let m_values = &result.trigger_values[&m_id];
        assert!(m_values.len() >= 2);
        // We need to find the link_out value.
        // The trigger_values are stored by socket index.
        // Link out was pushed last, trigger_out is first.
        assert_eq!(m_values[1], 1.0); // Link Out should be active
    }

    #[test]
    fn test_render_op_pooling() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        // 1. Layer -> Output
        let l_id = module.add_part(crate::module::PartType::Layer, (0.0, 0.0));
        let o_id = module.add_part(crate::module::PartType::Output, (100.0, 0.0));
        module.add_connection(l_id, "layer_out".to_string(), o_id, "layer_in".to_string());

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
        module.remove_connection(l_id, "layer_out".to_string(), o_id, "layer_in".to_string());
        evaluator.evaluate(&module, &shared, 1);

        // render_ops should be empty
        assert_eq!(evaluator.cached_result.render_ops.len(), 0);
        // spare_render_ops should contain the recycled one
        assert_eq!(evaluator.cached_result.spare_render_ops.len(), 1);
    }
}
