//! Module Graph Evaluator
//!
//! Traverses the module graph and computes output values.
//! This handles the full pipeline: Trigger -> Source -> Mask -> Effect -> Layer(Mesh) -> Output.

use crate::audio::analyzer_v2::AudioAnalysisV2;
use crate::audio_reactive::AudioTriggerData;
use crate::module::{
    BlendModeType, LayerType, LinkBehavior, LinkMode, MapFlowModule, MaskType, MeshType,
    ModulePartId, ModulePartType, ModulizerType, OutputType, SharedMediaState, SourceType,
    TriggerType,
};
use rand::Rng;
use std::collections::HashMap;
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

#[cfg(test)]
mod tests_evaluator {
    use super::*;
    use crate::audio::analyzer_v2::AudioAnalysisV2;
    use crate::module::{
        AudioTriggerOutputConfig, LinkMode, MapFlowModule, ModulePartType, SourceType, TriggerType,
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
        let result = evaluator.evaluate(&module, &shared);
        let values = &result.trigger_values[&part_id];
        assert_eq!(values[0], 1.0);

        // We can't easily mock time passage without refactoring ModuleEvaluator to accept a clock,
        // or sleeping. Sleeping in unit tests is generally bad, but for 15ms it's acceptable-ish
        // for this specific scenario if we want to test the time logic.
        // Ideally we'd refactor to inject time.
        std::thread::sleep(Duration::from_millis(20));

        let result = evaluator.evaluate(&module, &shared);
        let values = &result.trigger_values[&part_id];
        // 20ms > 10ms pulse duration -> 0.0
        assert_eq!(values[0], 0.0);
    }

    #[test]
    fn test_trigger_audio_fft() {
        let mut evaluator = ModuleEvaluator::new();

        // Setup Audio Analysis with a "Beat"
        let mut analysis = AudioAnalysisV2::default();
        analysis.beat_detected = true;
        analysis.rms_volume = 0.8;

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

        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default());
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
        let _result = evaluator.evaluate(&module, &shared);

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

        let result = evaluator.evaluate(&module, &shared);
        assert!(result.source_commands.contains_key(&s_id));

        // Now remove connection
        module.remove_connection(t_id, 0, s_id, 0);
        let result = evaluator.evaluate(&module, &shared);
        assert!(!result.source_commands.contains_key(&s_id));
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

        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default());

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

        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default());

        // Master ID in trigger_values should have 2 values: Trigger Out (1.0) and Link Out (1.0)
        let m_values = &result.trigger_values[&m_id];
        assert!(m_values.len() >= 2);
        assert_eq!(m_values[1], 1.0); // Link Out should be active
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
        // Render ops is a Vec, simply clear
        self.render_ops.clear();
    }
}

/// Command for a source node
#[derive(Debug, Clone)]
pub enum SourceCommand {
    /// Load and play a media file
    PlayMedia {
        /// Path to media file
        path: String,
        /// Trigger value (opacity/intensity)
        trigger_value: f32,
    },
    /// Play a shared media resource
    PlaySharedMedia {
        /// Shared ID
        id: String,
        /// Resolved path (from registry)
        path: String,
        /// Trigger value
        trigger_value: f32,
    },
    /// Play a shader with parameters
    PlayShader {
        /// Shader name/ID
        name: String,
        /// Shader parameters
        params: Vec<(String, f32)>,
        /// Trigger value
        trigger_value: f32,
    },
    /// NDI input source
    NdiInput {
        /// NDI source name
        source_name: Option<String>,
        /// Trigger value
        trigger_value: f32,
    },
    /// Live camera input
    LiveInput {
        /// Device ID
        device_id: u32,
        /// Trigger value
        trigger_value: f32,
    },
    #[cfg(target_os = "windows")]
    /// Spout input (Windows only)
    SpoutInput {
        /// Sender name
        sender_name: String,
        /// Trigger value
        trigger_value: f32,
    },
    /// Philips Hue output (Trigger/Effect data)
    HueOutput {
        /// Brightness (0.0 - 1.0)
        brightness: f32,
        /// Hue (0.0 - 1.0)
        hue: Option<f32>,
        /// Saturation (0.0 - 1.0)
        saturation: Option<f32>,
        /// Strobe speed/intensity (0.0 - 1.0)
        strobe: Option<f32>,
        /// Target lamp/group IDs (for new Hue nodes)
        ids: Option<Vec<String>>,
    },
}

/// Helper struct for chain tracing results
struct ProcessingChain {
    source_id: Option<ModulePartId>,
    source_props: SourceProperties,
    effects: Vec<ModulizerType>,
    masks: Vec<MaskType>,
    override_mesh: Option<MeshType>,
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

    // Caching for indices to avoid re-allocation every frame
    /// Cached map from Part ID to index in `module.parts`
    part_index_cache: HashMap<ModulePartId, usize>,
    /// Cached map from Part ID to list of indices in `module.connections` (incoming connections)
    conn_index_cache: HashMap<ModulePartId, Vec<usize>>,
    /// Currently active keyboard keys (for Shortcut triggers)
    active_keys: std::collections::HashSet<String>,
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
            part_index_cache: HashMap::new(),
            conn_index_cache: HashMap::new(),
            active_keys: std::collections::HashSet::new(),
        }
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

    /// Update active keyboard keys for Shortcut triggers
    pub fn update_keys(&mut self, keys: &std::collections::HashSet<String>) {
        self.active_keys = keys.clone();
    }

    /// Evaluate a module for one frame
    /// Returns a reference to the reusable result buffer
    pub fn evaluate(
        &mut self,
        module: &MapFlowModule,
        shared_state: &SharedMediaState,
    ) -> &ModuleEvalResult {
        // Clear previous result for reuse
        self.cached_result.clear();
        // Since we cleared trigger_values via iteration (retaining keys),
        // we might have entries with empty vectors. This is fine as we will overwrite them.

        // Build acceleration indices
        self.part_index_cache.clear();
        for (idx, part) in module.parts.iter().enumerate() {
            self.part_index_cache.insert(part.id, idx);
        }

        // Clear vectors but keep capacity
        for indices in self.conn_index_cache.values_mut() {
            indices.clear();
        }
        for (idx, conn) in module.connections.iter().enumerate() {
            self.conn_index_cache
                .entry(conn.to_part)
                .or_default()
                .push(idx);
        }

        // === DIAGNOSTICS: Log module structure ===
        // (Diagnostic logging code removed for brevity/performance in hot path unless feature enabled?
        // keeping it as it was but maybe less frequently? leaving as is per instructions to preserve functionality)

        // Step 1: Evaluate all trigger nodes
        // Collect (part_id, trigger_type clone) to avoid borrow issues
        let triggers_to_eval: Vec<_> = module
            .parts
            .iter()
            .filter_map(|part| {
                if let ModulePartType::Trigger(trigger_type) = &part.part_type {
                    Some((part.id, trigger_type.clone()))
                } else {
                    None
                }
            })
            .collect();

        for (part_id, trigger_type) in triggers_to_eval {
            let mut values = Vec::new();
            self.compute_trigger_output(
                &trigger_type,
                &self.audio_trigger_data,
                self.start_time,
                &mut values,
            );
            self.cached_result.trigger_values.insert(part_id, values);
        }

        // Step 2: First propagation (Triggers -> Nodes)
        // This populates inputs for Master nodes if they use Trigger Input
        let mut trigger_inputs =
            self.compute_trigger_inputs(module, &self.cached_result.trigger_values);

        // Step 3: Process Master Links (Nodes -> Link Out)
        for part in &module.parts {
            if part.link_data.mode == LinkMode::Master {
                // Determine Master Activity
                let mut activity = 1.0; // Default active

                if part.link_data.trigger_input_enabled {
                    if let Some(&val) = trigger_inputs.get(&part.id) {
                        activity = val;
                    } else {
                        activity = 0.0;
                    }
                }

                // Write activity to Link Out socket
                if !part.outputs.is_empty() {
                    let output_count = part.outputs.len();
                    // Get/Create the buffer
                    let values = self
                        .cached_result
                        .trigger_values
                        .entry(part.id)
                        .or_default();
                    values.clear(); // Ensure clean slate even if we reused it
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
        for part in &module.parts {
            if let ModulePartType::Source(source_type) = &part.part_type {
                let trigger_value = trigger_inputs.get(&part.id).copied().unwrap_or(0.0);
                if let Some(cmd) =
                    self.create_source_command(source_type, trigger_value, shared_state)
                {
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
                        saturation: None, // Implicit 1.0 or handled by node?
                        strobe,
                        ids,
                    },
                );
            }
        }

        // Step 4: Trace Render Pipeline
        for part in &module.parts {
            if let ModulePartType::Output(output_type) = &part.part_type {
                // Look up first connection to this output
                if let Some(conn_idx) = self
                    .conn_index_cache
                    .get(&part.id)
                    .and_then(|v| v.first())
                    .copied()
                {
                    let conn = &module.connections[conn_idx];

                    // Look up the layer part
                    if let Some(&layer_idx) = self.part_index_cache.get(&conn.from_part) {
                        let layer_part = &module.parts[layer_idx];

                        let link_opacity =
                            trigger_inputs.get(&layer_part.id).copied().unwrap_or(1.0);

                        if let ModulePartType::Layer(layer_type) = &layer_part.part_type {
                            match layer_type {
                                LayerType::Single {
                                    mesh,
                                    opacity,
                                    blend_mode,
                                    ..
                                } => {
                                    let chain = self.trace_chain(layer_part.id, module);
                                    let final_mesh = chain.override_mesh.unwrap_or(mesh.clone());

                                    self.cached_result.render_ops.push(RenderOp {
                                        output_part_id: part.id,
                                        output_type: output_type.clone(),
                                        layer_part_id: layer_part.id,
                                        mesh: final_mesh,
                                        opacity: *opacity * link_opacity,
                                        blend_mode: *blend_mode,
                                        source_part_id: chain.source_id,
                                        source_props: chain.source_props,
                                        effects: chain.effects,
                                        masks: chain.masks,
                                    });
                                }
                                LayerType::Group {
                                    opacity,
                                    blend_mode,
                                    mesh,
                                    ..
                                } => {
                                    let chain = self.trace_chain(layer_part.id, module);
                                    let final_mesh = chain.override_mesh.unwrap_or(mesh.clone());

                                    self.cached_result.render_ops.push(RenderOp {
                                        output_part_id: part.id,
                                        output_type: output_type.clone(),
                                        layer_part_id: layer_part.id,
                                        mesh: final_mesh,
                                        opacity: *opacity * link_opacity,
                                        blend_mode: *blend_mode,
                                        source_part_id: chain.source_id,
                                        source_props: chain.source_props.clone(),
                                        effects: chain.effects,
                                        masks: chain.masks,
                                    });
                                }
                                LayerType::All { .. } => {
                                    // TODO: Handle global layers
                                }
                            }
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

        &self.cached_result
    }

    /// Trace the processing input chain backwards from a start node (e.g. Layer input)
    fn trace_chain(&self, start_node_id: ModulePartId, module: &MapFlowModule) -> ProcessingChain {
        let mut effects = Vec::new();
        let mut masks = Vec::new();
        let mut override_mesh = None;
        let mut source_id = None;
        let mut source_props = SourceProperties::default_identity();
        let mut current_id = start_node_id;

        tracing::debug!("trace_chain: Starting from node {}", start_node_id);

        let trigger_values = &self.cached_result.trigger_values;

        // Safety limit to prevent infinite loops in cyclic graphs
        for _iteration in 0..50 {
            // Apply Trigger Targets for the current node
            // We need to find if any input sockets have triggers active and targets mapped
            if let Some(&part_idx) = self.part_index_cache.get(&current_id) {
                let part = &module.parts[part_idx];

                // Check if any *incoming* connection determines the value?
                // Wait, trigger targets are on this part's inputs.
                // We need the input values for this part trigger sockets.
                // But inputs are pull-based or pushed?
                // We have `trigger_inputs` map computed in `evaluate` but it's not passed here.
                // `compute_trigger_inputs` returns a map part_id -> float.
                // But that's a single value per part (consolidated).
                // We need per-socket values if we want specific mapping.
                // Or we re-compute or access cached values.
                // Let's assume we can access per-socket inputs or just iterate connections to this part.

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
                    if let Some(conn_indices) = self.conn_index_cache.get(&current_id) {
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

                    // Apply mapping
                    let final_val = config.apply(trigger_val);

                    tracing::debug!(
                        "Trigger applying: part={}, socket={}, target={:?}, raw={}, final={}",
                        part.id,
                        socket_idx,
                        config.target,
                        trigger_val,
                        final_val
                    );

                    match &config.target {
                        crate::module::TriggerTarget::Opacity => source_props.opacity = final_val, // Override
                        crate::module::TriggerTarget::Brightness => {
                            source_props.brightness = final_val
                        }
                        crate::module::TriggerTarget::Contrast => source_props.contrast = final_val,
                        crate::module::TriggerTarget::Saturation => {
                            source_props.saturation = final_val
                        }
                        crate::module::TriggerTarget::HueShift => {
                            source_props.hue_shift = final_val
                        }
                        crate::module::TriggerTarget::ScaleX => source_props.scale_x = final_val,
                        crate::module::TriggerTarget::ScaleY => source_props.scale_y = final_val,
                        crate::module::TriggerTarget::Rotation => source_props.rotation = final_val,
                        crate::module::TriggerTarget::Param(_name) => {
                            // Handle effect params?
                        }
                        _ => {}
                    }
                }
            }

            // Find main input connection (to ANY socket - usually socket 0)
            // L522 replacement
            if let Some(conn_idx) = self
                .conn_index_cache
                .get(&current_id)
                .and_then(|v| v.first())
                .copied()
            {
                let conn = &module.connections[conn_idx];
                if let Some(&part_idx) = self.part_index_cache.get(&conn.from_part) {
                    let part = &module.parts[part_idx];
                    match &part.part_type {
                        ModulePartType::Source(source_type) => {
                            source_id = Some(part.id);

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
                                    if let Some(conn_indices) = self.conn_index_cache.get(&part.id)
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
                                            let final_val = config.apply(trigger_val);

                                            tracing::debug!(
                                                "Trigger applying: part={}, socket={}, target={:?}, raw={}, final={}",
                                                part.id, socket_idx, target, trigger_val, final_val
                                            );

                                            match target {
                                                crate::module::TriggerTarget::Opacity => {
                                                    props.opacity = final_val;
                                                }
                                                crate::module::TriggerTarget::Brightness => {
                                                    // Map 0..1 to -1..1 for brightness if direct, or use configured range
                                                    // With TriggerConfig, the user sets min/max.
                                                    // If user wants -1..1, they set min=-1, max=1.
                                                    // So we just assign the final_val directly if it's calibrated.
                                                    // BUT: Historical behavior was 0..1 -> -1..1.
                                                    // If we just use final_val which defaults to 0..1, brightness will be positive only.
                                                    // Let's assume the user configures the range in the UI.
                                                    // However, legacy projects might rely on the hardcoded mapping.
                                                    // To support legacy, we could check if config is default?
                                                    // But we want to empower the user.
                                                    // For now, let's trust the configured min/max.
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
                                                _ => {}
                                            }
                                        }
                                    }
                                }

                                source_props = props;
                            }
                            break;
                        }
                        ModulePartType::Modulizer(mod_type) => {
                            effects.push(mod_type.clone());
                            current_id = part.id;
                        }
                        ModulePartType::Mask(mask_type) => {
                            masks.push(mask_type.clone());
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
                tracing::warn!(
                    "trace_chain: Node {} not found in part_index, stopping traversal",
                    current_id
                );
                break;
            }
        }

        // Warn if we hit the iteration limit (possible cycle in graph)
        // Note: We can't easily detect this inside the loop, but if source_id is still None
        // after 50 iterations, something may be wrong.
        if source_id.is_none() && !effects.is_empty() {
            tracing::warn!(
                "trace_chain: Completed 50 iterations starting from {} but found no source. Possible cycle or broken chain.",
                start_node_id
            );
        }

        // Fix order (we pushed back-to-front, so reverse to get execution order)
        effects.reverse();
        masks.reverse();

        ProcessingChain {
            source_id,
            source_props,
            effects,
            masks,
            override_mesh,
        }
    }

    /// Evaluate a trigger node and write output values to the provided buffer
    fn compute_trigger_output(
        &self,
        trigger_type: &TriggerType,
        audio_data: &AudioTriggerData,
        start_time: Instant,
        output: &mut Vec<f32>,
    ) {
        match trigger_type {
            TriggerType::AudioFFT {
                band: _band,
                threshold: _threshold,
                output_config,
            } => {
                // Helper to push and optionally invert value
                let push_val = |name: &str, val: f32, out: &mut Vec<f32>| {
                    let inverted = output_config.inverted_outputs.contains(name);
                    let final_val = if inverted {
                        1.0 - val.clamp(0.0, 1.0)
                    } else {
                        val
                    };
                    out.push(final_val);
                };

                // Generate values based on config
                if output_config.frequency_bands {
                    let bands = [
                        "SubBass Out",
                        "Bass Out",
                        "LowMid Out",
                        "Mid Out",
                        "HighMid Out",
                        "UpperMid Out",
                        "Presence Out",
                        "Brilliance Out",
                        "Air Out",
                    ];
                    for (i, name) in bands.iter().enumerate() {
                        if i < audio_data.band_energies.len() {
                            push_val(name, audio_data.band_energies[i], output);
                        } else {
                            push_val(name, 0.0, output);
                        }
                    }
                }
                if output_config.volume_outputs {
                    push_val("RMS Volume", audio_data.rms_volume, output);
                    push_val("Peak Volume", audio_data.peak_volume, output);
                }
                if output_config.beat_output {
                    let val = if audio_data.beat_detected { 1.0 } else { 0.0 };
                    push_val("Beat Out", val, output);
                }
                if output_config.bpm_output {
                    let val = audio_data.bpm.unwrap_or(0.0) / 200.0;
                    push_val("BPM Out", val, output);
                }

                // Fallback
                if output.is_empty() {
                    let val = if audio_data.beat_detected { 1.0 } else { 0.0 };
                    let inverted = output_config.inverted_outputs.contains("Beat Out");
                    let final_val = if inverted { 1.0 - val } else { val };
                    output.push(final_val);
                }
            }
            TriggerType::Beat => {
                output.push(if audio_data.beat_detected { 1.0 } else { 0.0 });
            }
            TriggerType::Random { probability, .. } => {
                let random_value: f32 = rand::rng().random();
                output.push(if random_value < *probability {
                    1.0
                } else {
                    0.0
                });
            }
            TriggerType::Fixed {
                interval_ms,
                offset_ms,
            } => {
                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                let adjusted_time = elapsed_ms.saturating_sub(*offset_ms as u64);
                let interval = *interval_ms as u64;
                if interval == 0 {
                    output.push(1.0);
                } else {
                    let pulse_duration = (interval / 10).max(16);
                    let phase = adjusted_time % interval;
                    output.push(if phase < pulse_duration { 1.0 } else { 0.0 });
                }
            }
            TriggerType::Midi { .. } => {
                output.push(0.0);
            }
            TriggerType::Osc { .. } => {
                output.push(0.0);
            }
            TriggerType::Shortcut {
                key_code,
                modifiers,
            } => {
                // Check if the key is currently pressed
                // key_code is stored as the winit KeyCode debug format (e.g., "KeyA")
                let is_pressed = self.active_keys.contains(key_code);
                // TODO: Check modifiers (Ctrl, Shift, Alt) if needed
                let _ = modifiers; // Suppress unused warning for now
                output.push(if is_pressed { 1.0 } else { 0.0 });
            }
        }
    }

    /// Compute trigger input values for each part by propagating through connections
    fn compute_trigger_inputs(
        &self,
        module: &MapFlowModule,
        trigger_values: &HashMap<ModulePartId, Vec<f32>>,
    ) -> HashMap<ModulePartId, f32> {
        let mut inputs: HashMap<ModulePartId, f32> = HashMap::new();

        // For each connection, propagate the trigger value
        for conn in &module.connections {
            if let Some(values) = trigger_values.get(&conn.from_part) {
                if let Some(&value) = values.get(conn.from_socket) {
                    // Combine multiple inputs with max
                    let current = inputs.entry(conn.to_part).or_insert(0.0);
                    *current = current.max(value);
                }
            }
        }

        inputs
    }

    /// Compute raw inputs per socket index
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
                    *current = current.max(value);
                }
            }
        }
        inputs
    }

    /// Create a source command based on source type and trigger value
    fn create_source_command(
        &self,
        source_type: &SourceType,
        trigger_value: f32,
        shared_state: &SharedMediaState,
    ) -> Option<SourceCommand> {
        // Only activate source if trigger is above threshold (0.1)
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
                // Resolve path from shared state
                if let Some(item) = shared_state.get(shared_id) {
                    Some(SourceCommand::PlaySharedMedia {
                        id: shared_id.clone(),
                        path: item.path.clone(),
                        trigger_value,
                    })
                } else {
                    None // ID not found
                }
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
        }
    }
}

#[cfg(test)]
mod tests_logic {
    use super::*;
    use crate::audio_reactive::AudioTriggerData;
    use crate::module::{
        AudioBand, AudioTriggerOutputConfig, MapFlowModule, ModuleConnection, ModulePart,
        ModulePartType, ModulePlaybackMode, SourceType, TriggerType,
    };
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    fn create_audio_data(beat: bool) -> AudioTriggerData {
        let mut data = AudioTriggerData::default();
        data.beat_detected = beat;
        data.rms_volume = 0.5;
        data.peak_volume = 0.8;
        data.band_energies = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
        data
    }

    fn create_test_module() -> MapFlowModule {
        MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        }
    }

    #[test]
    fn test_compute_trigger_output_beat() {
        let mut output = Vec::new();
        let data_true = create_audio_data(true);
        let evaluator = ModuleEvaluator::new();
        evaluator.compute_trigger_output(
            &TriggerType::Beat,
            &data_true,
            Instant::now(),
            &mut output,
        );
        assert_eq!(output, vec![1.0]);

        output.clear();
        let data_false = create_audio_data(false);
        evaluator.compute_trigger_output(
            &TriggerType::Beat,
            &data_false,
            Instant::now(),
            &mut output,
        );
        assert_eq!(output, vec![0.0]);
    }

    #[test]
    fn test_compute_trigger_output_audio_fft() {
        let mut output = Vec::new();
        let data = create_audio_data(true);

        let config = AudioTriggerOutputConfig {
            frequency_bands: true,
            volume_outputs: true,
            beat_output: true,
            bpm_output: false,
            inverted_outputs: vec!["Bass Out".to_string()].into_iter().collect(),
        };

        let evaluator = ModuleEvaluator::new();
        evaluator.compute_trigger_output(
            &TriggerType::AudioFFT {
                band: AudioBand::Bass,
                threshold: 0.5,
                output_config: config,
            },
            &data,
            Instant::now(),
            &mut output,
        );

        // Expected output order:
        // Bands (9 values):
        // "SubBass Out" -> 0.1
        // "Bass Out" -> 0.2 (INVERTED -> 0.8)
        // ...
        // "Air Out" -> 0.9
        // Volume (2 values): RMS (0.5), Peak (0.8)
        // Beat (1 value): 1.0

        assert!(output.len() >= 9 + 2 + 1);
        assert!((output[0] - 0.1).abs() < 1e-6); // SubBass
        assert!((output[1] - 0.8).abs() < 1e-6); // Bass (Inverted: 1.0 - 0.2)
        assert!((output[9] - 0.5).abs() < 1e-6); // RMS
        assert!((output[11] - 1.0).abs() < 1e-6); // Beat
    }

    #[test]
    fn test_compute_trigger_output_fixed() {
        let mut output = Vec::new();
        let data = create_audio_data(false);

        // Interval 1000ms. Pulse duration 100ms.
        // At 50ms: Phase 50 < 100 -> 1.0
        // At 150ms: Phase 150 > 100 -> 0.0

        // Emulate 50ms elapsed
        let start_past_50 = Instant::now() - Duration::from_millis(50);
        output.clear();
        let evaluator = ModuleEvaluator::new();
        evaluator.compute_trigger_output(
            &TriggerType::Fixed {
                interval_ms: 1000,
                offset_ms: 0,
            },
            &data,
            start_past_50,
            &mut output,
        );
        assert_eq!(output[0], 1.0);

        // 150ms
        let start_past_150 = Instant::now() - Duration::from_millis(150);
        output.clear();
        evaluator.compute_trigger_output(
            &TriggerType::Fixed {
                interval_ms: 1000,
                offset_ms: 0,
            },
            &data,
            start_past_150,
            &mut output,
        );
        assert_eq!(output[0], 0.0);
    }

    #[test]
    fn test_compute_trigger_inputs_propagation() {
        let evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        // Connection: Part 1 (Socket 0) -> Part 2
        // Connection: Part 3 (Socket 1) -> Part 2

        module.connections.push(ModuleConnection {
            from_part: 1,
            from_socket: 0,
            to_part: 2,
            to_socket: 0,
        });
        module.connections.push(ModuleConnection {
            from_part: 3,
            from_socket: 1,
            to_part: 2,
            to_socket: 0,
        });

        let mut trigger_values = HashMap::new();
        trigger_values.insert(1, vec![0.5]);
        trigger_values.insert(3, vec![0.0, 0.8]); // Socket 1 has 0.8

        let inputs = evaluator.compute_trigger_inputs(&module, &trigger_values);

        // Should take max(0.5, 0.8) = 0.8
        assert_eq!(inputs.get(&2), Some(&0.8));
    }

    #[test]
    fn test_create_source_command() {
        let evaluator = ModuleEvaluator::new();
        let shared = crate::module::SharedMediaState::default();

        // Threshold check (< 0.1)
        let cmd_low = evaluator.create_source_command(
            &SourceType::LiveInput { device_id: 0 },
            0.05,
            &shared,
        );
        assert!(cmd_low.is_none());

        // Valid command
        let cmd_valid = evaluator.create_source_command(
            &SourceType::LiveInput { device_id: 1 },
            0.5,
            &shared,
        );
        match cmd_valid {
            Some(SourceCommand::LiveInput {
                device_id,
                trigger_value,
            }) => {
                assert_eq!(device_id, 1);
                assert_eq!(trigger_value, 0.5);
            }
            _ => panic!("Expected LiveInput"),
        }
    }

    #[test]
    fn test_trace_chain_limit() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        // Create a cycle: Part 1 -> Part 2 -> Part 1
        module.parts.push(ModulePart {
            id: 1,
            part_type: ModulePartType::Modulizer(crate::module::ModulizerType::Effect {
                effect_type: crate::module::EffectType::Blur,
                params: HashMap::new(),
            }),
            position: (0.0, 0.0),
            size: None,
            link_data: Default::default(),
            inputs: vec![],
            outputs: vec![],
            trigger_targets: HashMap::new(),
        });
        module.parts.push(ModulePart {
            id: 2,
            part_type: ModulePartType::Modulizer(crate::module::ModulizerType::Effect {
                effect_type: crate::module::EffectType::Blur,
                params: HashMap::new(),
            }),
            position: (0.0, 0.0),
            size: None,
            link_data: Default::default(),
            inputs: vec![],
            outputs: vec![],
            trigger_targets: HashMap::new(),
        });

        module.connections.push(ModuleConnection {
            from_part: 2,
            from_socket: 0,
            to_part: 1,
            to_socket: 0,
        });
        module.connections.push(ModuleConnection {
            from_part: 1,
            from_socket: 0,
            to_part: 2,
            to_socket: 0,
        });

        // Run evaluate to populate internal caches
        evaluator.evaluate(&module, &crate::module::SharedMediaState::default());

        // Start trace from 1 using cached indices
        let chain = evaluator.trace_chain(1, &module);

        // Should not panic or hang, but finish with limited effects
        // The limit is 50.
        // It should just return safely.
        assert!(chain.effects.len() <= 50);
    }

    #[test]
    fn test_trace_chain_order() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        // 1. Source
        let source_id = module.add_part(crate::module::PartType::Source, (0.0, 0.0));
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == source_id) {
            if let ModulePartType::Source(SourceType::MediaFile { path, .. }) = &mut part.part_type
            {
                *path = "test.mp4".to_string();
            }
        }

        // 2. Effect A (Blur)
        let effect_a_id = module.add_part(crate::module::PartType::Modulator, (100.0, 0.0));
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == effect_a_id) {
            part.part_type = ModulePartType::Modulizer(crate::module::ModulizerType::Effect {
                effect_type: crate::module::EffectType::Blur,
                params: HashMap::new(),
            });
        }

        // 3. Effect B (Invert)
        let effect_b_id = module.add_part(crate::module::PartType::Modulator, (200.0, 0.0));
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == effect_b_id) {
            part.part_type = ModulePartType::Modulizer(crate::module::ModulizerType::Effect {
                effect_type: crate::module::EffectType::Invert,
                params: HashMap::new(),
            });
        }

        // 4. Layer
        let layer_id = module.add_part(crate::module::PartType::Layer, (300.0, 0.0));

        // 5. Output
        let output_id = module.add_part(crate::module::PartType::Output, (400.0, 0.0));

        // Connections: Source -> Effect A -> Effect B -> Layer -> Output
        // Source(0) -> Effect A(0) (Input)
        module.add_connection(source_id, 0, effect_a_id, 0);
        // Effect A(0) -> Effect B(0)
        module.add_connection(effect_a_id, 0, effect_b_id, 0);
        // Effect B(0) -> Layer(0)
        module.add_connection(effect_b_id, 0, layer_id, 0);
        // Layer(0) -> Output(0)
        module.add_connection(layer_id, 0, output_id, 0);

        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default());

        assert_eq!(result.render_ops.len(), 1);
        let op = &result.render_ops[0];
        assert_eq!(op.effects.len(), 2);

        // Expected Order: Source -> Effect A (Blur) -> Effect B (Invert) -> Layer
        // op.effects should be [Blur, Invert]

        if let crate::module::ModulizerType::Effect { effect_type, .. } = &op.effects[0] {
            assert_eq!(*effect_type, crate::module::EffectType::Blur);
        } else {
            panic!("First effect should be Blur");
        }

        if let crate::module::ModulizerType::Effect { effect_type, .. } = &op.effects[1] {
            assert_eq!(*effect_type, crate::module::EffectType::Invert);
        } else {
            panic!("Second effect should be Invert");
        }
    }
}
