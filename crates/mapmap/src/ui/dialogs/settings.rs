use egui::{Context, Window};
use mapmap_control::hue::controller::HueController;
use mapmap_core::AppState;
use mapmap_render::WgpuBackend;
use mapmap_ui::AppUI;

#[cfg(feature = "midi")]
use mapmap_control::midi::MidiInputHandler;

/// Context required to render the settings window.
pub struct SettingsContext<'a> {
    /// Reference to the UI state.
    pub ui_state: &'a mut AppUI,
    /// Reference to the global application state.
    pub state: &'a mut AppState,
    /// Reference to the render backend.
    pub backend: &'a WgpuBackend,
    /// Reference to the Hue controller.
    pub hue_controller: &'a mut HueController,
    /// Reference to the MIDI input handler (if enabled).
    #[cfg(feature = "midi")]
    pub midi_handler: &'a mut Option<MidiInputHandler>,
    /// List of available MIDI ports (if enabled).
    #[cfg(feature = "midi")]
    pub midi_ports: &'a mut Vec<String>,
    /// Index of the selected MIDI port (if enabled).
    #[cfg(feature = "midi")]
    pub selected_midi_port: &'a mut Option<usize>,
    /// Flag indicating if a restart was requested.
    pub restart_requested: &'a mut bool,
    /// Flag indicating if an exit was requested.
    pub exit_requested: &'a mut bool,
    /// Reference to the Tokio runtime.
    pub tokio_runtime: &'a tokio::runtime::Runtime,
}

/// Renders the settings window.
pub fn show(ctx: &Context, context: SettingsContext) {
    let mut show_settings = context.ui_state.show_settings;

    // Clone data needed for UI to avoid borrow conflicts
    let active_audio_device = context.ui_state.selected_audio_device.clone();

    Window::new("Settings")
        .open(&mut show_settings)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("General");
            ui.separator();
            ui.label("MapMap VJ Software");

            ui.add_space(10.0);

            ui.heading("Audio");
            if let Some(audio_device) = &active_audio_device {
                ui.label(format!("Active Device: {}", audio_device));
            } else {
                ui.label("No Audio Device Selected");
            }

            ui.add_space(10.0);

            ui.heading("Hue");
            ui.label(format!(
                "Hue Bridge: {}",
                if context.hue_controller.is_connected() {
                    "Connected"
                } else {
                    "Disconnected"
                }
            ));
            if ui.button("Connect Hue").clicked() {
                // Connection logic would go here
            }

            #[cfg(feature = "midi")]
            {
                ui.add_space(10.0);
                ui.heading("MIDI");
                if let Some(port_idx) = *context.selected_midi_port {
                    if let Some(port_name) = context.midi_ports.get(port_idx) {
                        ui.label(format!("Active MIDI Port: {}", port_name));
                    }
                } else {
                    ui.label("No MIDI Port Selected");
                }
            }

            ui.add_space(10.0);

            ui.heading("Theme");
            if ui.button("Switch Theme").clicked() {
                // TODO: Implement theme switching via event or state mutation
            }

            ui.separator();
            if ui.button("Restart Application").clicked() {
                *context.restart_requested = true;
                *context.exit_requested = true;
            }
        });

    context.ui_state.show_settings = show_settings;
}




