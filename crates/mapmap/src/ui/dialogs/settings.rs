use egui::{Context, Window, Color32, RichText};
use mapmap_control::hue::controller::HueController;
use mapmap_core::AppState;
use mapmap_render::WgpuBackend;
use mapmap_ui::{AppUI, UIAction};

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

    Window::new(RichText::new("⚙ SETTINGS").strong().color(Color32::from_rgb(0, 255, 255)))
        .open(&mut show_settings)
        .resizable(true)
        .default_width(450.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                
                // --- GENERAL ---
                ui.heading(RichText::new("General").color(Color32::WHITE));
                ui.add_space(4.0);
                
                ui.horizontal(|ui| {
                    ui.label("Language:");
                    let current_lang = context.ui_state.user_config.language.clone();
                    egui::ComboBox::from_id_salt("lang_selector")
                        .selected_text(if current_lang == "de" { "Deutsch" } else { "English" })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(current_lang == "de", "Deutsch").clicked() {
                                context.ui_state.actions.push(UIAction::SetLanguage("de".to_string()));
                            }
                            if ui.selectable_label(current_lang == "en", "English").clicked() {
                                context.ui_state.actions.push(UIAction::SetLanguage("en".to_string()));
                            }
                        });
                });

                ui.add_space(10.0);
                ui.separator();

                // --- AUDIO ---
                ui.heading(RichText::new("Audio").color(Color32::WHITE));
                ui.add_space(4.0);
                
                ui.horizontal(|ui| {
                    ui.label("Device:");
                    let current_device = context.ui_state.selected_audio_device.clone().unwrap_or_else(|| "None".to_string());
                    egui::ComboBox::from_id_salt("audio_device_selector")
                        .selected_text(&current_device)
                        .show_ui(ui, |ui| {
                            for device in &context.ui_state.audio_devices {
                                if ui.selectable_label(Some(device) == context.ui_state.selected_audio_device.as_ref(), device).clicked() {
                                    context.ui_state.actions.push(UIAction::SelectAudioDevice(device.clone()));
                                }
                            }
                        });
                });

                ui.add_space(10.0);
                ui.separator();

                // --- NDI & NETWORK ---
                ui.heading(RichText::new("Network & NDI").color(Color32::WHITE));
                ui.add_space(4.0);
                
                ui.checkbox(&mut context.ui_state.user_config.ndi_discovery, "Enable NDI Discovery");
                ui.add_space(4.0);
                
                ui.horizontal(|ui| {
                    ui.label("OSC Input Port:");
                    ui.text_edit_singleline(&mut context.ui_state.osc_port_input);
                });
                ui.horizontal(|ui| {
                    ui.label("OSC Client Address:");
                    ui.text_edit_singleline(&mut context.ui_state.osc_client_input);
                });

                ui.add_space(10.0);
                ui.separator();

                // --- HUE ---
                ui.heading(RichText::new("Philips Hue").color(Color32::from_rgb(255, 200, 0)));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(format!("Status: {}", if context.hue_controller.is_connected() { "CONNECTED" } else { "DISCONNECTED" }));
                    if ui.button("Discover Bridges").clicked() {
                        context.ui_state.actions.push(UIAction::DiscoverHueBridges);
                    }
                });

                ui.add_space(10.0);
                ui.separator();

                // --- THEME ---
                ui.heading(RichText::new("Appearance").color(Color32::WHITE));
                ui.add_space(4.0);
                if ui.button("Toggle Theme (Dark/Light)").clicked() {
                    // Logic for theme toggle
                }

                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    if ui.button(RichText::new("Restart Application").color(Color32::RED)).clicked() {
                        *context.restart_requested = true;
                        *context.exit_requested = true;
                    }
                });
            });
        });

    context.ui_state.show_settings = show_settings;
}
