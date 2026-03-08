use egui::{Color32, Context, RichText, Window};
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

    Window::new(
        RichText::new(format!("⚙ {}", i18n.t("settings").to_uppercase()))
            .strong()
            .color(Color32::from_rgb(0, 255, 255)),
    )
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
                let current_theme = context.ui_state.user_config.theme.theme;
                egui::ComboBox::from_id_salt("theme_selector")
                    .selected_text(format!("{:?}", current_theme))
                    .show_ui(ui, |ui| {
                        use mapmap_ui::core::theme::Theme;
                        for theme in [
                            Theme::Dark, Theme::Light, Theme::Resolume,
                            Theme::Synthwave, Theme::Cyber, Theme::Midnight,
                            Theme::Purple, Theme::Pink, Theme::HighContrast,
                        ] {
                            if ui.selectable_label(current_theme == theme, format!("{:?}", theme)).clicked() {
                                context.ui_state.user_config.theme.theme = theme;
                                context.ui_state.user_config.theme.apply(ctx);
                                let _ = context.ui_state.user_config.save();
                            }
                        }
                    });
            });

            ui.add_space(10.0);
            ui.separator();

            // --- PERFORMANCE & GRAPHICS ---
            ui.heading(RichText::new(format!("{} & {}", i18n.t("graphics"), i18n.t("performance"))).color(Color32::WHITE));
            ui.add_space(4.0);

            egui::Grid::new("perf_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                ui.label(format!("{}:", i18n.t("hw-accel")));
                ui.label("Enabled");
                ui.end_row();

                ui.label(format!("{}:", i18n.t("target-fps")));
                let mut fps = context.ui_state.user_config.target_fps.unwrap_or(60.0);
                if ui.add(egui::Slider::new(&mut fps, 24.0..=144.0).suffix(" FPS")).changed() {
                    context.ui_state.actions.push(UIAction::SetTargetFps(fps));
                }
                ui.end_row();

                ui.label("VSync Mode:");
                let vsync = context.ui_state.user_config.vsync_mode;
                egui::ComboBox::from_id_salt("vsync_select")
                    .selected_text(vsync.to_string())
                    .show_ui(ui, |ui| {
                        use mapmap_ui::core::config::VSyncMode;
                        for mode in [VSyncMode::Auto, VSyncMode::On, VSyncMode::Off] {
                            if ui.selectable_label(vsync == mode, mode.to_string()).clicked() {
                                context.ui_state.actions.push(UIAction::SetVsyncMode(mode));
                            }
                        }
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

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Level Meter Style:");
                let meter = context.ui_state.user_config.meter_style;
                egui::ComboBox::from_id_salt("meter_select")
                    .selected_text(format!("{:?}", meter))
                    .show_ui(ui, |ui| {
                        use mapmap_ui::core::config::AudioMeterStyle;
                        for style in [AudioMeterStyle::Retro, AudioMeterStyle::Digital] {
                            if ui.selectable_label(meter == style, format!("{:?}", style)).clicked() {
                                context.ui_state.actions.push(UIAction::SetMeterStyle(style));
                            }
                        }
                    });
            });

            ui.add_space(20.0);
            ui.separator();

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
