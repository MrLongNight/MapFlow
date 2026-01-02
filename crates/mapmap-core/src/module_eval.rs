//! Module Graph Evaluator
//!
//! Traverses the module graph and computes output values.
//! This is the runtime that connects:
//! - Triggers (Audio, MIDI, etc.) to Sources
//! - Sources to Effects
//! - Effects to Outputs

use crate::audio::analyzer_v2::AudioAnalysisV2;
use crate::audio_reactive::AudioTriggerData;
use crate::module::{
    LinkBehavior, LinkMode, MapFlowModule, ModulePartId, ModulePartType, OutputType, SourceType,
    TriggerType,
};
use std::collections::HashMap;

/// Evaluation result for a single frame
#[derive(Debug, Clone, Default)]
pub struct ModuleEvalResult {
    /// Trigger values: part_id -> (output_index -> value)
    pub trigger_values: HashMap<ModulePartId, Vec<f32>>,
    /// Source commands: part_id -> SourceCommand
    pub source_commands: HashMap<ModulePartId, SourceCommand>,
    /// Output assignments: output_id -> texture_id
    pub output_assignments: HashMap<u64, TextureAssignment>,
}

/// Command for a source node
#[derive(Debug, Clone)]
pub enum SourceCommand {
    /// Load and play a media file
    PlayMedia { path: String, trigger_value: f32 },
    /// Play a shader with parameters
    PlayShader {
        name: String,
        params: Vec<(String, f32)>,
        trigger_value: f32,
    },
    /// NDI input source
    NdiInput {
        source_name: Option<String>,
        trigger_value: f32,
    },
    /// Live camera input
    LiveInput { device_id: u32, trigger_value: f32 },
    #[cfg(target_os = "windows")]
    /// Spout input (Windows only)
    SpoutInput {
        sender_name: String,
        trigger_value: f32,
    },
}

/// Texture assignment for an output
#[derive(Debug, Clone)]
pub struct TextureAssignment {
    /// ID of the output (projector, NDI, etc.)
    pub output_id: u64,
    /// Type of output
    pub output_type: OutputType,
    /// Source part ID that feeds this output
    pub source_part_id: Option<ModulePartId>,
    /// Opacity/blend factor
    pub opacity: f32,
}

/// Module graph evaluator
pub struct ModuleEvaluator {
    /// Current trigger data from audio analysis
    audio_trigger_data: AudioTriggerData,
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
        }
    }

    pub fn update_audio(&mut self, analysis: &AudioAnalysisV2) {
        self.audio_trigger_data.band_energies = analysis.band_energies;
        self.audio_trigger_data.rms_volume = analysis.rms_volume;
        self.audio_trigger_data.peak_volume = analysis.peak_volume;
        self.audio_trigger_data.beat_detected = analysis.beat_detected;
        self.audio_trigger_data.beat_strength = analysis.beat_strength;
        self.audio_trigger_data.bpm = analysis.tempo_bpm;
    }

    /// Evaluate a module for one frame
    pub fn evaluate(&self, module: &MapFlowModule) -> ModuleEvalResult {
        let mut result = ModuleEvalResult::default();

        // Step 1: Evaluate all trigger nodes
        for part in &module.parts {
            if let ModulePartType::Trigger(trigger_type) = &part.part_type {
                let values = self.evaluate_trigger(trigger_type);
                result.trigger_values.insert(part.id, values);
            }
        }

        // Step 2: First propagation (Triggers -> Nodes)
        // This populates inputs for Master nodes if they use Trigger Input
        let mut trigger_inputs = self.compute_trigger_inputs(module, &result.trigger_values);

        // Step 3: Process Master Links (Nodes -> Link Out)
        for part in &module.parts {
            if part.link_data.mode == LinkMode::Master {
                // Determine Master Activity
                let mut activity = 1.0; // Default active

                if part.link_data.trigger_input_enabled {
                    // Check if we received a trigger signal
                    // If enabled but no signal connected/active, strictly it should be 0.0?
                    // "Der Layer mit master link benötigt dann ein Trigger Signal..." -> Yes.
                    if let Some(&val) = trigger_inputs.get(&part.id) {
                        activity = val;
                    } else {
                        activity = 0.0;
                    }
                }

                // Write activity to Link Out socket
                // Link Out is appended at the end of outputs list
                if !part.outputs.is_empty() {
                    let output_count = part.outputs.len();
                    // Create a vector of zeros, set last to activity
                    // Note: We assume only Link Out carries signal from a Layer/Effect node
                    // Media outputs carry textures (not modeled here in trigger_values)
                    let mut values = vec![0.0; output_count];
                    values[output_count - 1] = activity;

                    result.trigger_values.insert(part.id, values);
                }
            }
        }

        // Step 4: Second propagation (Master Link Out -> Slave Link In)
        // Re-compute to propagate Link signals to Slaves
        trigger_inputs = self.compute_trigger_inputs(module, &result.trigger_values);

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
                if let Some(cmd) = self.create_source_command(source_type, trigger_value) {
                    result.source_commands.insert(part.id, cmd);
                }
            }
        }

        // Step 7: Find output assignments
        for part in &module.parts {
            if let ModulePartType::Output(output_type) = &part.part_type {
                let output_id = match output_type {
                    OutputType::Projector { id, .. } => *id,
                    OutputType::NdiOutput { .. } => part.id,
                    #[cfg(target_os = "windows")]
                    OutputType::Spout { .. } => part.id,
                };

                // Find the source that feeds this output (trace back through connections)
                let source_part_id = self.find_source_for_output(module, part.id);

                // Opacity is determined by the trigger input (Control Signal)
                // For a Slave Layer, this 'trigger_input' is the processed Link signal.
                let opacity = trigger_inputs.get(&part.id).copied().unwrap_or(1.0);

                result.output_assignments.insert(
                    output_id,
                    TextureAssignment {
                        output_id,
                        output_type: output_type.clone(),
                        source_part_id,
                        opacity,
                    },
                );
            }
        }

        result
    }

    /// Evaluate a trigger node and return output values
    fn evaluate_trigger(&self, trigger_type: &TriggerType) -> Vec<f32> {
        match trigger_type {
            TriggerType::AudioFFT {
                band: _band,
                threshold: _threshold,
                output_config,
            } => {
                let mut values = Vec::new();

                // Helper to push and optionally invert value
                let mut push_val = |name: &str, val: f32| {
                    let inverted = output_config.inverted_outputs.contains(name);
                    let final_val = if inverted {
                        1.0 - val.clamp(0.0, 1.0)
                    } else {
                        val
                    };
                    values.push(final_val);
                };

                // Generate values based on config
                // ORDER MUST MATCH AudioTriggerOutputConfig::generate_outputs
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
                        if i < self.audio_trigger_data.band_energies.len() {
                            push_val(name, self.audio_trigger_data.band_energies[i]);
                        } else {
                            push_val(name, 0.0);
                        }
                    }
                }
                if output_config.volume_outputs {
                    push_val("RMS Volume", self.audio_trigger_data.rms_volume);
                    push_val("Peak Volume", self.audio_trigger_data.peak_volume);
                }
                if output_config.beat_output {
                    let val = if self.audio_trigger_data.beat_detected {
                        1.0
                    } else {
                        0.0
                    };
                    push_val("Beat Out", val);
                }
                if output_config.bpm_output {
                    let val = self.audio_trigger_data.bpm.unwrap_or(0.0) / 200.0;
                    push_val("BPM Out", val);
                }

                // Fallback: if empty, add beat output (matches generate_outputs fallback)
                if values.is_empty() {
                    let val = if self.audio_trigger_data.beat_detected {
                        1.0
                    } else {
                        0.0
                    };
                    // Note: generate_outputs fallback uses "Beat Out" name, so we check that
                    // But effectively we just push the value.
                    // If we want to support inversion on fallback, we need to check "Beat Out"
                    let inverted = output_config.inverted_outputs.contains("Beat Out");
                    let final_val = if inverted { 1.0 - val } else { val };
                    values.push(final_val);
                }

                values
            }
            TriggerType::Beat => {
                vec![if self.audio_trigger_data.beat_detected {
                    1.0
                } else {
                    0.0
                }]
            }
            TriggerType::Random { probability, .. } => {
                // Placeholder for random
                vec![if 0.5 < *probability { 1.0 } else { 0.0 }]
            }
            TriggerType::Fixed { .. } => {
                // Fixed triggers need timing state, return 1.0 for now
                vec![1.0]
            }
            TriggerType::Midi { .. } => {
                // MIDI triggers need external input
                vec![0.0]
            }
            TriggerType::Osc { .. } => {
                // OSC triggers need external input
                vec![0.0]
            }
            TriggerType::Shortcut { .. } => {
                // Shortcut triggers need keyboard input
                vec![0.0]
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
                    // Combine multiple inputs with max (or could use add/multiply)
                    let current = inputs.entry(conn.to_part).or_insert(0.0);
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
    ) -> Option<SourceCommand> {
        // Only activate source if trigger is above threshold (0.1)
        if trigger_value < 0.1 {
            return None;
        }

        match source_type {
            SourceType::MediaFile { path } => {
                if path.is_empty() {
                    return None;
                }
                Some(SourceCommand::PlayMedia {
                    path: path.clone(),
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
        }
    }

    /// Find the source part that ultimately feeds an output
    fn find_source_for_output(
        &self,
        module: &MapFlowModule,
        output_part_id: ModulePartId,
    ) -> Option<ModulePartId> {
        // Trace back through connections to find a source
        let mut current = output_part_id;
        let mut visited = std::collections::HashSet::new();

        while visited.insert(current) {
            // Find a connection that goes TO this part
            if let Some(conn) = module.connections.iter().find(|c| c.to_part == current) {
                // Check if from_part is a source
                if let Some(part) = module.parts.iter().find(|p| p.id == conn.from_part) {
                    match &part.part_type {
                        ModulePartType::Source(_) => return Some(part.id),
                        _ => {
                            current = conn.from_part;
                            continue;
                        }
                    }
                }
            }
            break;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::{MapFlowModule, ModulePlaybackMode};

    #[test]
    fn test_evaluator_creation() {
        let evaluator = ModuleEvaluator::new();
        assert_eq!(evaluator.audio_trigger_data.beat_detected, false);
    }

    #[test]
    fn test_evaluate_empty_module() {
        let evaluator = ModuleEvaluator::new();
        let module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0, 1.0, 1.0, 1.0],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        };

        let result = evaluator.evaluate(&module);
        assert!(result.trigger_values.is_empty());
        assert!(result.source_commands.is_empty());
        assert!(result.output_assignments.is_empty());
    }
}
