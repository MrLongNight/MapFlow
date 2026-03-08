use crate::app::core::app_struct::App;
use std::collections::HashMap;
use mapmap_core::audio::AudioAnalysis;

/// Orchestrates the evaluation of the module graph and synchronizes with the Bevy engine.
pub fn perform_evaluation(
    app: &mut App,
    all_module_ids: &[u64],
    analysis: &AudioAnalysis,
    graph_dirty: bool,
) {
    app.render_ops.clear();
    let mut node_triggers = HashMap::new();

    let show_module_id = app.ui_state.timeline_panel.runtime_show_module(
        app.state.effect_animator.get_current_time() as f32,
        app.state.effect_animator.is_playing(),
        all_module_ids,
    );

    let modules_for_eval: Vec<u64> = if let Some(mid) = show_module_id {
        vec![mid]
    } else {
        all_module_ids.to_vec()
    };

    for module_id in &modules_for_eval {
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
                eval_result.render_ops.iter().cloned().map(|op| (*module_id, op)),
            );
        }
    }

    // Sync with Bevy (only if runner exists)
    if let Some(runner) = &mut app.bevy_runner {
        let trigger_data = mapmap_core::audio_reactive::AudioTriggerData {
            band_energies: {
                let mut b = [0.0; 9];
                for i in 0..9.min(analysis.band_energies.len()) { b[i] = analysis.band_energies[i]; }
                b
            },
            rms_volume: analysis.rms_volume,
            peak_volume: analysis.peak_volume,
            beat_detected: analysis.beat_detected,
            beat_strength: analysis.beat_strength,
            bpm: analysis.tempo_bpm,
        };
        runner.update(&trigger_data, &node_triggers);
    }
}
