use crate::app::core::app_struct::App;
use crate::app::core::app_struct::RuntimeRenderQueueItem;
use vorce_core::audio::AudioAnalysis;
use std::collections::HashMap;

/// Orchestrates the evaluation of the module graph and synchronizes with the Bevy engine.
pub fn perform_evaluation(
    app: &mut App,
    modules_for_eval: &[u64],
    analysis: &AudioAnalysis,
    graph_dirty: bool,
) {
    // Reclaim RenderOp objects from the previous frame to avoid allocations.
    // This closes the object pool loop for evaluation results.
    for items in app.render_queue.items.values_mut() {
        app.module_evaluator
            .cached_result
            .spare_render_ops
            .extend(items.drain(..).map(|item| item.render_op));
    }
    app.render_queue.items.clear();
    app.render_queue.graph_revision = app.state.module_manager.graph_revision;
    app.ui_state.module_canvas.last_trigger_values.clear();
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
                app.ui_state
                    .module_canvas
                    .last_trigger_values
                    .insert(*part_id, max_val);
            }

            #[cfg(target_os = "macos")]
            for part in &module_ref.parts {
                if let vorce_core::module::ModulePartType::Source(
                    vorce_core::module::SourceType::NdiInput { .. },
                ) = &part.part_type
                {
                    // Throttled logging to avoid spam
                    let now = std::time::Instant::now();
                    let log_key = format!("ndi_input_unsupported_{}", module_ref.name);
                    let should_log =
                        if let Some(last_log) = app.video_diagnostic_log_times.get(&log_key) {
                            now.duration_since(*last_log).as_secs_f32() > 5.0
                        } else {
                            true
                        };
                    if should_log {
                        tracing::warn!("NDI Input in module '{}' is currently unsupported/experimental on macOS.", module_ref.name);
                        app.video_diagnostic_log_times.insert(log_key, now);
                    }
                }
            }

            // Transfer RenderOps using drain to avoid clones
            // Note: We need to access eval_result fields directly because evaluate returns a reference.
            // But since ModuleEvaluator is on app, we can just drain from its cached_result.
            let render_ops: Vec<_> = app
                .module_evaluator
                .cached_result
                .render_ops
                .drain(..)
                .collect();
            for render_op in render_ops {
                let mut diagnostics = Vec::new();

                if render_op.blend_mode.is_some() {
                    diagnostics.push(crate::app::core::app_struct::RenderDiagnostic {
                        module_id: *module_id,
                        part_id: render_op.layer_part_id,
                        severity: crate::app::core::app_struct::DiagnosticSeverity::Warning,
                        code: "blend_mode_unsupported".to_string(),
                        message: "Blend modes are currently only supported via specific compositing passes.".to_string(),
                    });
                }

                if !render_op.masks.is_empty() {
                    diagnostics.push(crate::app::core::app_struct::RenderDiagnostic {
                        module_id: *module_id,
                        part_id: render_op.layer_part_id,
                        severity: crate::app::core::app_struct::DiagnosticSeverity::Warning,
                        code: "masks_unsupported".to_string(),
                        message: "Masks are not yet supported in this render path.".to_string(),
                    });
                }

                let output_id = match render_op.output_type {
                    vorce_core::module::OutputType::Projector { id, .. } => id,
                    _ => render_op.output_part_id,
                };

                let item = RuntimeRenderQueueItem {
                    module_id: *module_id,
                    render_op,
                    diagnostics,
                };
                app.render_queue
                    .items
                    .entry(output_id)
                    .or_default()
                    .push(item);
            }
        }
    }

    for ops in app.render_queue.items.values_mut() {
        ops.sort_by(|a, b| b.render_op.output_part_id.cmp(&a.render_op.output_part_id));
    }

    // Sync with Bevy (only if runner exists)
    if let Some(runner) = &mut app.bevy_runner {
        let trigger_data = vorce_core::audio_reactive::AudioTriggerData {
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
