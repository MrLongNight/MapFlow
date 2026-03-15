//! Modular Timeline orchestration.

use egui::Context;
use mapmap_core::AppState;
use mapmap_ui::AppUI;

/// Context required to render the timeline.
pub struct TimelineContext<'a> {
    /// Reference to the UI state.
    pub ui_state: &'a mut AppUI,
    /// Reference to the app state.
    pub state: &'a mut AppState,
}

/// Renders the timeline panel.
pub fn show(ctx: &Context, mut context: TimelineContext) {
    if !context.ui_state.show_timeline {
        return;
    }

    egui::TopBottomPanel::bottom("timeline_panel")
        .resizable(true)
        .default_height(180.0)
        .min_height(100.0)
        .max_height(350.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Timeline");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        context.ui_state.show_timeline = false;
                    }
                });
            });
            ui.separator();

            let state = &mut context.state;
            let animator = std::sync::Arc::make_mut(&mut state.effect_animator);
            let timeline_modules = state
                .module_manager
                .modules()
                .iter()
                .map(|m| mapmap_ui::TimelineModule {
                    id: m.id,
                    // Optimization: Borrow name string to prevent allocation overhead in UI hot loop.
                    name: &m.name,
                })
                .collect::<Vec<_>>();

            if let Some(action) =
                context
                    .ui_state
                    .timeline_panel
                    .ui(ui, animator, &timeline_modules)
            {
                use mapmap_ui::TimelineAction;
                match action {
                    TimelineAction::Play => animator.play(),
                    TimelineAction::Pause => animator.pause(),
                    TimelineAction::Stop => animator.stop(),
                    TimelineAction::Seek(t) => animator.seek(t as f64),
                    TimelineAction::SelectModule(module_id) => {
                        context
                            .ui_state
                            .module_canvas
                            .set_active_module(Some(module_id));
                    }
                    TimelineAction::AddMarker(t) => {
                        let name = format!("Marker {:.1}s", t);
                        animator.add_marker(mapmap_core::animation::Marker::new(t as f64, name));
                    }
                    TimelineAction::RemoveMarker(t) => {
                        animator.remove_marker(t as f64);
                    }
                    TimelineAction::ToggleMarkerPause(t) => {
                        animator.toggle_marker_pause(t as f64);
                    }
                    TimelineAction::JumpNextMarker => animator.jump_next_marker(),
                    TimelineAction::JumpPrevMarker => animator.jump_prev_marker(),
                }
            }
        });
}
