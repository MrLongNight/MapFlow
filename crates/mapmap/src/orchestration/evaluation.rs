use crate::app::core::app_struct::App;
use mapmap_core::audio::AudioAnalysis;
use std::collections::HashMap;

/// Orchestrates the evaluation of the module graph and synchronizes with the Bevy engine.
pub fn perform_evaluation(
    app: &mut App,
    modules_for_eval: &[u64],
    analysis: &AudioAnalysis,
    graph_dirty: bool,
) {
    app.render_ops.clear();
    let mut node_triggers = HashMap::new();

    for module_id in modules_for_eval {
        if let Some(module_ref) = app.state.module_manager.get_module(*module_id) {
            // Only sync graph structure to Bevy if it actually changed
            if graph_dirty {
                if let Some(runner) = &mut app.bevy_runner {
                    runner.apply_graph_state(module_ref);
                }
            }

            let eval_result = app.module_evaluator.evaluate(
                module_ref,
                &app.state.module_manager.shared_media,
                app.state.module_manager.graph_revision,
            );

            for (part_id, values) in &eval_result.trigger_values {
                let max_val = values.iter().cloned().fold(0.0, f32::max);
                node_triggers.insert((*module_id, *part_id), max_val);
            }

            app.render_ops.extend(
                eval_result
                    .render_ops
                    .iter()
                    .cloned()
                    .map(|op| (*module_id, op)),
            );
        }
    }

    // Sync with Bevy (only if runner exists)
    if let Some(runner) = &mut app.bevy_runner {
        let trigger_data = mapmap_core::audio_reactive::AudioTriggerData {
            band_energies: analysis.band_energies,
            rms_volume: analysis.rms_volume,
            peak_volume: analysis.peak_volume,
            beat_detected: analysis.beat_detected,
            beat_strength: analysis.beat_strength,
            bpm: analysis.tempo_bpm,
        };
        runner.update(&trigger_data, &node_triggers);
    }
}
