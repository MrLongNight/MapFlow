use crate::AppUI;
use imgui::{Condition, Ui};
use mapmap_control::ControlManager;

/// Renders the OSC server status panel.
pub fn render_osc_panel(ui: &Ui, app_ui: &mut AppUI, control_manager: &mut ControlManager) {
    if !app_ui.show_osc_panel {
        return;
    }

    ui.window("OSC Control")
        .size([400.0, 500.0], Condition::FirstUseEver)
        .build(|| {
            ui.text("OSC Server");
            ui.separator();

            // Server status
            let server_status = if control_manager.osc_server.is_some() {
                "Running"
            } else {
                "Stopped"
            };
            ui.text(format!("Status: {}", server_status));

            // Port configuration
            ui.input_text("Port", &mut app_ui.osc_port_input).build();
            if ui.button("Start Server") {
                if let Ok(port) = app_ui.osc_port_input.parse() {
                    if let Err(e) = control_manager.init_osc_server(port) {
                        tracing::error!("Failed to start OSC server: {}", e);
                    }
                }
            }

            ui.separator();

            // OSC Clients (Feedback)
            ui.text("Feedback Clients");
            let mut clients_to_remove = Vec::new();
            for client in &control_manager.osc_clients {
                ui.text(client.destination_str());
                ui.same_line();
                if ui.small_button(format!("Remove##{}", client.destination_str())) {
                    clients_to_remove.push(client.destination_str());
                }
            }

            for addr in clients_to_remove {
                control_manager.remove_osc_client(&addr);
            }

            ui.input_text("Add Client", &mut app_ui.osc_client_input)
                .build();
            if ui.button("Add") {
                if let Err(e) = control_manager.add_osc_client(&app_ui.osc_client_input) {
                    tracing::error!("Failed to add OSC client: {}", e);
                } else {
                    app_ui.osc_client_input.clear();
                }
            }

            ui.separator();

            // Mappings
            ui.text("Address Mappings");
            ui.text("(Edit osc_mappings.json for now)");

            let mut mappings_to_remove = Vec::new();
            for (addr, target) in &control_manager.osc_mapping.map {
                ui.text(format!("{} -> {:?}", addr, target));
                ui.same_line();
                if ui.small_button(format!("Remove##{}", addr)) {
                    mappings_to_remove.push(addr.clone());
                }
            }

            for addr in &mappings_to_remove {
                control_manager.osc_mapping.remove_mapping(addr);
            }
            if !mappings_to_remove.is_empty() {
                if let Err(e) = control_manager
                    .osc_mapping
                    .save("osc_mappings.json")
                {
                    tracing::error!("Failed to save OSC mappings: {}", e);
                }
            }
        });
}
