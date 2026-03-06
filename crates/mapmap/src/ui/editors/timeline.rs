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
pub fn show(ctx: &Context, context: TimelineContext) {
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

            let timeline_modules = context
                .state
                .module_manager
                .modules()
                .iter()
                .map(|m| mapmap_ui::TimelineModule {
                    id: m.id,
                    name: m.name.clone(),
                })
                .collect::<Vec<_>>();

            if let Some(action) = context
                .ui_state
                .timeline_panel
                .ui(ui, context.state.effect_animator_mut(), &timeline_modules)
            {
                use mapmap_ui::TimelineAction;
                match action {
                    TimelineAction::Play => context.state.effect_animator_mut().play(),
                    TimelineAction::Pause => context.state.effect_animator_mut().pause(),
                    TimelineAction::Stop => context.state.effect_animator_mut().stop(),
                    TimelineAction::Seek(t) => context.state.effect_animator_mut().seek(t as f64),
                    TimelineAction::SelectModule(module_id) => {
                        context.ui_state.module_canvas.set_active_module(Some(module_id));
                    }
                }
            }
        });
}
