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
    let i18n = &context.ui_state.i18n;

    Window::new(RichText::new(format!("⚙ {}", i18n.t("settings").to_uppercase())).strong().color(Color32::from_rgb(0, 255, 255)))
        .open(&mut show_settings)
        .resizable(true)
        .default_width(500.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                
                // --- GENERAL ---
                ui.heading(RichText::new("General").color(Color32::WHITE));
                
                ui.add_space(4.0);
                
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", i18n.t("language")));
                    let current_lang = context.ui_state.user_config.language.clone();
                    let lang_name = if current_lang == "de" { "Deutsch" } else { "English" };
                    
                    egui::ComboBox::from_id_salt("lang_selector")
                        .selected_text(lang_name)
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

                // --- APPEARANCE & THEME ---
                ui.heading(RichText::new(i18n.t("appearance")).color(Color32::WHITE));
                ui.add_space(4.0);
                
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", i18n.t("theme")));
                    let is_dark = ctx.style().visuals.dark_mode;
                    if ui.selectable_label(is_dark, format!("🌙 {}", i18n.t("theme-dark"))).clicked() {
                        ctx.set_visuals(egui::Visuals::dark());
                    }
                    if ui.selectable_label(!is_dark, format!("☀ {}", i18n.t("theme-light"))).clicked() {
                        ctx.set_visuals(egui::Visuals::light());
                    }
                });
                
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", i18n.t("theme-accent")));
                    ui.label("Cyber Cyan (Default)");
                });

                ui.add_space(10.0);
                ui.separator();

                // --- PERFORMANCE & GRAPHICS ---
                ui.heading(RichText::new(format!("{} & {}", i18n.t("graphics"), i18n.t("performance"))).color(Color32::WHITE));
                ui.add_space(4.0);
                
                egui::Grid::new("perf_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                    ui.label(format!("{}:", i18n.t("hw-accel")));
                    ui.label("D3D11 (Enabled)");
                    ui.end_row();
                    
                    ui.label(format!("{}:", i18n.t("target-fps")));
                    let mut fps = 60;
                    ui.add(egui::Slider::new(&mut fps, 24..=144).suffix(" FPS"));
                    ui.end_row();
                    
                    ui.label(format!("{}:", i18n.t("texture-quality")));
                    let mut quality = 1; // High
                    egui::ComboBox::from_id_salt("quality_picker")
                        .selected_text(i18n.t("quality"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut quality, 0, "Low");
                            ui.selectable_value(&mut quality, 1, "High");
                        });
                    ui.end_row();
                });

                ui.add_space(10.0);
                ui.separator();

                // --- AUDIO ---
                ui.heading(RichText::new(i18n.t("audio")).color(Color32::WHITE));
                ui.add_space(4.0);
                
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", i18n.t("label-device")));
                    let current_device = context.ui_state.selected_audio_device.clone().unwrap_or_else(|| i18n.t("no-device"));
                    egui::ComboBox::from_id_salt("audio_device_selector")
                        .selected_text(&current_device)
                        .show_ui(ui, |ui| {
                            for device in &context.ui_state.audio_devices {
                                let is_selected = Some(device) == context.ui_state.selected_audio_device.as_ref();
                                if ui.selectable_label(is_selected, device).clicked() {
                                    context.ui_state.actions.push(UIAction::SelectAudioDevice(device.clone()));
                                }
                            }
                        });
                });

                ui.add_space(10.0);
                ui.separator();

                // --- HUE ---
                ui.heading(RichText::new("Philips Hue").color(Color32::from_rgb(255, 200, 0)));
                ui.add_space(4.0);
                let is_connected = context.hue_controller.is_connected();
                ui.horizontal(|ui| {
                    ui.label(format!("{}: {}", i18n.t("hue-status"), if is_connected { "CONNECTED" } else { "DISCONNECTED" }));
                    
                    if !is_connected {
                        if ui.button(i18n.t("hue-discover")).clicked() {
                            context.ui_state.actions.push(UIAction::DiscoverHueBridges);
                        }
                    } else {
                        if ui.button(i18n.t("hue-disconnect")).clicked() {
                            // Placeholder
                        }
                    }
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                ui.vertical_centered(|ui| {
                    if ui.button(RichText::new(i18n.t("restart-app")).color(Color32::RED).strong()).clicked() {
                        *context.restart_requested = true;
                        *context.exit_requested = true;
                    }
                });
            });
        });

    context.ui_state.show_settings = show_settings;
}
