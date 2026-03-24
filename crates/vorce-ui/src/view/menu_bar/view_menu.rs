use super::menu_item;
use crate::icons::AppIcon;
use crate::{AppUI, UIAction};

pub fn show(
    ui: &mut egui::Ui,
    ui_state: &mut AppUI,
    actions: &mut Vec<UIAction>,
    compact_menu: bool,
) {
    let menu_view_label = ui_state.i18n.t("menu-view");
    let top_label = if compact_menu {
        "👁"
    } else {
        &menu_view_label
    };
    let response = ui.menu_button(top_label, |ui| {
        ui.label(ui_state.i18n.t("view-egui-panels"));
        ui.checkbox(
            &mut ui_state.dashboard.visible,
            ui_state.i18n.t("panel-dashboard"),
        );
        ui.checkbox(
            &mut ui_state.effect_chain_panel.visible,
            ui_state.i18n.t("panel-effect-chain"),
        );
        if ui.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::M,
            ))
        }) {
            actions.push(UIAction::ToggleModuleCanvas);
        }
        ui.checkbox(
            &mut ui_state.show_module_canvas,
            ui_state.i18n.t("panel-module-canvas"),
        );
        if ui.button("Media Manager").clicked() {
            actions.push(UIAction::ToggleMediaManager);
        }
        ui.checkbox(
            &mut ui_state.show_controller_overlay,
            "MIDI Controller Overlay",
        );
        ui.separator();
        ui.label(ui_state.i18n.t("view-legacy-panels"));
        ui.checkbox(
            &mut ui_state.show_osc_panel,
            ui_state.i18n.t("check-show-osc"),
        );
        ui.checkbox(
            &mut ui_state.show_controls,
            ui_state.i18n.t("check-show-controls"),
        );
        ui.checkbox(
            &mut ui_state.show_layers,
            ui_state.i18n.t("check-show-layers"),
        );
        ui.checkbox(
            &mut ui_state.paint_panel.visible,
            ui_state.i18n.t("check-show-paints"),
        );
        ui.checkbox(
            &mut ui_state.show_mappings,
            ui_state.i18n.t("check-show-mappings"),
        );
        ui.checkbox(
            &mut ui_state.show_transforms,
            ui_state.i18n.t("check-show-transforms"),
        );
        ui.checkbox(
            &mut ui_state.show_master_controls,
            ui_state.i18n.t("check-show-master"),
        );
        ui.checkbox(
            &mut ui_state.oscillator_panel.visible,
            ui_state.i18n.t("check-show-oscillator"),
        );
        ui.checkbox(
            &mut ui_state.output_panel.visible,
            ui_state.i18n.t("check-show-outputs"),
        );
        ui.checkbox(
            &mut ui_state.edge_blend_panel.visible,
            ui_state.i18n.t("check-show-edge-blend"),
        );
        ui.checkbox(
            &mut ui_state.show_cue_panel,
            ui_state.i18n.t("check-show-cues"),
        );
        ui.checkbox(
            &mut ui_state.show_stats,
            ui_state.i18n.t("check-show-stats"),
        );
        ui.checkbox(&mut ui_state.show_timeline, "Timeline");
        if ui
            .checkbox(&mut ui_state.show_shader_graph, "Shader Graph")
            .changed()
            && ui_state.show_shader_graph
        {
            actions.push(UIAction::OpenShaderGraph(1));
        }
        ui.checkbox(&mut ui_state.show_toolbar, "Werkzeugleiste");
        ui.checkbox(&mut ui_state.icon_demo_panel.visible, "Icon Gallery");
        ui.separator();
        if ui
            .checkbox(
                &mut ui_state.user_config.global_fullscreen,
                "📽️ Projectors Fullscreen",
            )
            .changed()
        {
            actions.push(UIAction::SetGlobalFullscreen(
                ui_state.user_config.global_fullscreen,
            ));
        }
        ui.separator();
        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("btn-fullscreen"),
            Some(AppIcon::Monitor),
        ) {
            actions.push(UIAction::ToggleFullscreen);
            ui.close();
        }
        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("view-reset-layout"),
            Some(AppIcon::AppWindow),
        ) {
            actions.push(UIAction::ResetLayout);
            ui.close();
        }
    });

    if compact_menu {
        response.response.on_hover_text(menu_view_label);
    }
}
