use crate::audio_reactive::AudioTriggerData;
use crate::module::{VorceModule, ModulePartId, SharedMediaState, TriggerType};
use crate::module_eval::types::TriggerState;
use crate::module_eval::ModuleEvaluator;
use rand::RngExt;
use std::collections::HashMap;
use std::time::Instant;

impl ModuleEvaluator {
    /// Evaluate a trigger node and write output values to the provided buffer
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute_trigger_output(
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

    pub(crate) fn compute_trigger_inputs(
        &self,
        module: &VorceModule,
        trigger_values: &HashMap<ModulePartId, Vec<f32>>,
    ) -> HashMap<ModulePartId, f32> {
        let mut inputs = HashMap::new();
        for conn in &module.connections {
            if let Some(values) = trigger_values.get(&conn.from_part) {
                // Find index of the output socket ID
                if let Some(from_part) = module.parts.iter().find(|p| p.id == conn.from_part) {
                    if let Some(idx) = from_part
                        .outputs
                        .iter()
                        .position(|s| s.id == conn.from_socket)
                    {
                        if let Some(&value) = values.get(idx) {
                            let current = inputs.entry(conn.to_part).or_insert(0.0);
                            *current = f32::max(*current, value);
                        }
                    }
                }
            }
        }

        inputs
    }

    pub(crate) fn compute_socket_inputs(
        &self,
        module: &VorceModule,
        trigger_values: &HashMap<ModulePartId, Vec<f32>>,
    ) -> HashMap<ModulePartId, HashMap<usize, f32>> {
        let mut inputs: HashMap<ModulePartId, HashMap<usize, f32>> = HashMap::new();

        for conn in &module.connections {
            if let Some(values) = trigger_values.get(&conn.from_part) {
                // Look up source part to find output socket index
                let from_part = module.parts.iter().find(|p| p.id == conn.from_part);
                let to_part = module.parts.iter().find(|p| p.id == conn.to_part);

                if let (Some(src), Some(dst)) = (from_part, to_part) {
                    let from_idx = src.outputs.iter().position(|s| s.id == conn.from_socket);
                    let to_idx = dst.inputs.iter().position(|s| s.id == conn.to_socket);

                    if let (Some(f_idx), Some(t_idx)) = (from_idx, to_idx) {
                        if let Some(&value) = values.get(f_idx) {
                            let part_inputs = inputs.entry(conn.to_part).or_default();
                            let current = part_inputs.entry(t_idx).or_insert(0.0);
                            *current = f32::max(*current, value);
                        }
                    }
                }
            }
        }
        inputs
    }
}
