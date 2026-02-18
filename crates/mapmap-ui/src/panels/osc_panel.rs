use crate::widgets::panel::{cyber_panel_frame, render_panel_header};
use crate::AppUI;
use egui;
use mapmap_control::ControlManager;

/// Renders the OSC server status panel.
pub fn show_osc_panel(
    ctx: &egui::Context,
    app_ui: &mut AppUI,
    control_manager: &mut ControlManager,
) {
    let mut open = app_ui.show_osc_panel;
    if !open {
        return;
    }

    egui::Window::new(app_ui.i18n.t("panel-osc-title"))
        .open(&mut open)
        .default_size([400.0, 500.0])
        .frame(cyber_panel_frame(&ctx.style()))
        .show(ctx, |ui| {
            render_panel_header(
                ui,
                &app_ui.i18n.t("panel-osc-title"),
                |_| {},
            );

            ui.add_space(8.0);

            ui.heading(app_ui.i18n.t("header-osc-server"));
            ui.separator();

            // Server status
            let server_status_key = if control_manager.osc_server.is_some() {
                "status-running"
            } else {
                "status-stopped"
            };
            ui.label(format!(
                "{}: {}",
                app_ui.i18n.t("label-status"),
                app_ui.i18n.t(server_status_key)
            ));

            // Port configuration
            ui.horizontal(|ui| {
                ui.label(app_ui.i18n.t("label-port"));
                ui.text_edit_singleline(&mut app_ui.osc_port_input);
            });

            if ui.button(app_ui.i18n.t("btn-start-server")).clicked() {
                if let Ok(port) = app_ui.osc_port_input.parse() {
                    if let Err(e) = control_manager.init_osc_server(port) {
                        tracing::error!("Failed to start OSC server: {}", e);
                    }
                }
            }

            ui.separator();

            // OSC Clients (Feedback)
            ui.heading(app_ui.i18n.t("header-feedback-clients"));
            let mut clients_to_remove = Vec::new();
            for client in &control_manager.osc_clients {
                ui.horizontal(|ui| {
                    ui.label(client.destination_str());
                    if ui
                        .button(format!(
                            "{}##{}",
                            app_ui.i18n.t("btn-remove"),
                            client.destination_str()
                        ))
                        .clicked()
                    {
                        clients_to_remove.push(client.destination_str());
                    }
                });
            }

            for addr in clients_to_remove {
                control_manager.remove_osc_client(&addr);
            }

            ui.horizontal(|ui| {
                ui.label(app_ui.i18n.t("label-add-client"));
                ui.text_edit_singleline(&mut app_ui.osc_client_input);
            });

            if ui.button(app_ui.i18n.t("btn-add")).clicked() {
                if let Err(e) = control_manager.add_osc_client(&app_ui.osc_client_input) {
                    tracing::error!("Failed to add OSC client: {}", e);
                } else {
                    app_ui.osc_client_input.clear();
                }
            }

            ui.separator();

            // Mappings
            ui.heading(app_ui.i18n.t("header-address-mappings"));
            ui.label(app_ui.i18n.t("text-osc-edit-tip"));

            let mut mappings_to_remove = Vec::new();
            for (addr, target) in &control_manager.osc_mapping.map {
                ui.horizontal(|ui| {
                    ui.label(format!("{} -> {:?}", addr, target));
                    if ui
                        .button(format!("{}##{}", app_ui.i18n.t("btn-remove"), addr))
                        .clicked()
                    {
                        mappings_to_remove.push(addr.clone());
                    }
                });
            }

            for addr in &mappings_to_remove {
                control_manager.osc_mapping.remove_mapping(addr);
            }
            if !mappings_to_remove.is_empty() {
                if let Err(e) = control_manager.osc_mapping.save("osc_mappings.json") {
                    let err_msg = format!("Failed to save OSC mappings: {}", e);
                    tracing::error!("{}", err_msg);
                    eprintln!("{}", err_msg);
                    // Do not exit process, just log error.
                }
            }
        });
    app_ui.show_osc_panel = open;
}
