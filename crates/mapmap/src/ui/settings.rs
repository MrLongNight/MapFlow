//! Settings window UI.
//!
//! Handles the rendering and logic of the application settings window.

use mapmap_control::hue::controller::HueController;
#[cfg(feature = "midi")]
use mapmap_control::midi::MidiInputHandler;
use mapmap_core::AppState;
use mapmap_render::WgpuBackend;
use mapmap_ui::AppUI;
use tracing::{error, info};

/// Context struct to pass dependencies to the settings window.
pub struct SettingsContext<'a> {
    /// The UI state.
    pub ui_state: &'a mut AppUI,
    /// The application state.
    pub state: &'a mut AppState,
    /// The WGPU backend.
    pub backend: &'a WgpuBackend,
    /// The Hue controller.
    pub hue_controller: &'a mut HueController,
    /// The MIDI handler.
    #[cfg(feature = "midi")]
    pub midi_handler: &'a mut Option<MidiInputHandler>,
    /// Available MIDI ports.
    #[cfg(feature = "midi")]
    pub midi_ports: &'a mut Vec<String>,
    /// The selected MIDI port.
    #[cfg(feature = "midi")]
    pub selected_midi_port: &'a mut Option<usize>,
    /// Whether a restart was requested.
    pub restart_requested: &'a mut bool,
    /// Whether an exit was requested.
    pub exit_requested: &'a mut bool,
    /// The Tokio runtime.
    pub tokio_runtime: &'a tokio::runtime::Runtime,
}

/// Renders the settings window.
pub fn show(ctx: &egui::Context, mut context: SettingsContext) {
    let mut show_settings = context.ui_state.show_settings;
    let mut explicit_close = false;

    if show_settings {
        egui::Window::new(context.ui_state.i18n.t("menu-file-settings"))
            .id(egui::Id::new("app_settings_window"))
            .collapsible(false)
            .resizable(true)
            .default_size([400.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut show_settings)
            .show(ctx, |ui| {
                // Project Settings
                egui::CollapsingHeader::new(format!(
                    "🎬 {}",
                    context.ui_state.i18n.t("settings-project")
                ))
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{}:",
                            context.ui_state.i18n.t("settings-frame-rate")
                        ));
                        let fps_options = [
                            (24.0, "24"),
                            (25.0, "25"),
                            (30.0, "30"),
                            (50.0, "50"),
                            (60.0, "60"),
                            (120.0, "120"),
                        ];
                        let current = context.ui_state.user_config.target_fps.unwrap_or(60.0);
                        egui::ComboBox::from_id_salt("fps_select")
                            .selected_text(format!("{:.0} FPS", current))
                            .show_ui(ui, |ui| {
                                for (fps, label) in fps_options {
                                    if ui
                                        .selectable_label((current - fps).abs() < 0.1, label)
                                        .clicked()
                                    {
                                        context.ui_state.user_config.target_fps = Some(fps);
                                        let _ = context.ui_state.user_config.save();
                                    }
                                }
                            });
                    });
                });

                ui.separator();

                // App Settings
                egui::CollapsingHeader::new(format!("⚙️ {}", context.ui_state.i18n.t("settings-app")))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "{}:",
                                context.ui_state.i18n.t("settings-language")
                            ));
                            if ui.button("English").clicked() {
                                context
                                    .ui_state
                                    .actions
                                    .push(mapmap_ui::UIAction::SetLanguage("en".to_string()));
                            }
                            if ui.button("Deutsch").clicked() {
                                context
                                    .ui_state
                                    .actions
                                    .push(mapmap_ui::UIAction::SetLanguage("de".to_string()));
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Theme:");
                            let current_theme = context.ui_state.user_config.theme.theme;
                            let theme_name = match current_theme {
                                mapmap_ui::theme::Theme::Resolume => "Cyber Dark",
                                mapmap_ui::theme::Theme::Dark => "Professional Dark",
                                mapmap_ui::theme::Theme::Light => "Light",
                                mapmap_ui::theme::Theme::Synthwave => "Synthwave",
                                mapmap_ui::theme::Theme::HighContrast => "High Contrast",
                                mapmap_ui::theme::Theme::Custom => "Custom",
                            };

                            egui::ComboBox::from_id_salt("theme_select")
                                .selected_text(theme_name)
                                .show_ui(ui, |ui| {
                                    let themes = [
                                        (mapmap_ui::theme::Theme::Resolume, "Cyber Dark"),
                                        (mapmap_ui::theme::Theme::Dark, "Professional Dark"),
                                        (mapmap_ui::theme::Theme::Light, "Light"),
                                        (mapmap_ui::theme::Theme::Synthwave, "Synthwave"),
                                        (mapmap_ui::theme::Theme::HighContrast, "High Contrast"),
                                    ];

                                    for (theme, name) in themes {
                                        if ui
                                            .selectable_value(
                                                &mut context.ui_state.user_config.theme.theme,
                                                theme,
                                                name,
                                            )
                                            .clicked()
                                        {
                                            let _ = context.ui_state.user_config.save();
                                        }
                                    }
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Audio Meter:");
                            let current = context.ui_state.user_config.meter_style;
                            egui::ComboBox::from_id_salt("meter_style_select")
                                .selected_text(format!("{}", current))
                                .show_ui(ui, |ui| {
                                    let styles = [
                                        mapmap_ui::config::AudioMeterStyle::Retro,
                                        mapmap_ui::config::AudioMeterStyle::Digital,
                                    ];
                                    for style in styles {
                                        if ui
                                            .selectable_value(
                                                &mut context.ui_state.user_config.meter_style,
                                                style,
                                                format!("{}", style),
                                            )
                                            .clicked()
                                        {
                                            let _ = context.ui_state.user_config.save();
                                        }
                                    }
                                });
                        });
                    });

                ui.separator();

                // Graphics / Performance
                egui::CollapsingHeader::new(format!("🖥️ {}", "Graphics"))
                    .default_open(true)
                    .show(ui, |ui| {
                        // Adapter Selection
                        ui.horizontal(|ui| {
                            ui.label("GPU Adapter:");
                            let current_gpu = context
                                .ui_state
                                .user_config
                                .preferred_gpu
                                .clone()
                                .unwrap_or_else(|| "Auto".to_string());

                            egui::ComboBox::from_id_salt("gpu_select")
                                .selected_text(&current_gpu)
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_label(
                                            context.ui_state.user_config.preferred_gpu.is_none(),
                                            "Auto",
                                        )
                                        .clicked()
                                    {
                                        context.ui_state.user_config.preferred_gpu = None;
                                        let _ = context.ui_state.user_config.save();
                                    }

                                    let adapters = context
                                        .backend
                                        .instance
                                        .enumerate_adapters(wgpu::Backends::all());
                                    for adapter in adapters {
                                        let info = adapter.get_info();
                                        let name = info.name;
                                        let is_selected = context
                                            .ui_state
                                            .user_config
                                            .preferred_gpu
                                            .as_ref()
                                            == Some(&name);
                                        if ui.selectable_label(is_selected, &name).clicked() {
                                            context.ui_state.user_config.preferred_gpu = Some(name);
                                            let _ = context.ui_state.user_config.save();
                                            *context.restart_requested = true;
                                            *context.exit_requested = true;
                                        }
                                    }
                                });
                        });
                        ui.label(
                            egui::RichText::new("⚠️ GPU change will restart the app automatically")
                                .color(egui::Color32::YELLOW)
                                .small(),
                        );

                        // VSync
                        ui.horizontal(|ui| {
                            ui.label("VSync Mode:");
                            egui::ComboBox::from_id_salt("vsync_select")
                                .selected_text(format!(
                                    "{}",
                                    context.ui_state.user_config.vsync_mode
                                ))
                                .show_ui(ui, |ui| {
                                    use mapmap_ui::config::VSyncMode;
                                    let modes = [VSyncMode::Auto, VSyncMode::On, VSyncMode::Off];
                                    for mode in modes {
                                        if ui
                                            .selectable_value(
                                                &mut context.ui_state.user_config.vsync_mode,
                                                mode,
                                                format!("{}", mode),
                                            )
                                            .clicked()
                                        {
                                            let _ = context.ui_state.user_config.save();
                                        }
                                    }
                                });
                        });
                        ui.label(
                            egui::RichText::new("Note: VSync change might require restart").small(),
                        );

                        ui.separator();
                        ui.label(egui::RichText::new("Performance Metrics").strong());
                        ui.label(format!(
                            "Active GPU: {} ({:?})",
                            context.backend.adapter_info.name, context.backend.adapter_info.backend
                        ));
                        ui.label(format!("CPU Usage: {:.1}%", context.ui_state.cpu_usage));
                        ui.label(format!(
                            "RAM Usage: {:.1} MB",
                            context.ui_state.ram_usage_mb
                        ));
                    });

                ui.separator();

                // Output/Projector Settings
                egui::CollapsingHeader::new(format!(
                    "📽️ {}",
                    context.ui_state.i18n.t("settings-outputs")
                ))
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Number of Outputs (Projectors):");
                        let mut output_count = context.state.settings.output_count;
                        if ui
                            .add(egui::DragValue::new(&mut output_count).speed(1).range(1..=8))
                            .changed()
                        {
                            context.state.settings.output_count = output_count;
                            context.state.dirty = true;
                        }
                    });

                    ui.label("💡 Each output can be assigned to a different screen/projector");

                    // List current outputs if any
                    let output_count = context.state.output_manager.outputs().len();
                    if output_count > 0 {
                        ui.add_space(8.0);
                        ui.label(format!("Currently configured: {} outputs", output_count));
                        for output in context.state.output_manager.outputs() {
                            ui.label(format!("  • {} (ID: {})", output.name, output.id));
                        }
                    } else {
                        ui.add_space(8.0);
                        ui.label(
                            "⚠️ No outputs configured yet. Add an Output node in the Module Canvas.",
                        );
                    }
                });

                ui.separator();

                // Logging Settings
                egui::CollapsingHeader::new(format!(
                    "📝 {}",
                    context.ui_state.i18n.t("settings-logging")
                ))
                .default_open(false)
                .show(ui, |ui| {
                    let log_config = &mut context.state.settings.log_config;

                    ui.horizontal(|ui| {
                        ui.label("Log Level:");
                        let levels = ["trace", "debug", "info", "warn", "error"];
                        egui::ComboBox::from_id_salt("log_level_select")
                            .selected_text(&log_config.level)
                            .show_ui(ui, |ui| {
                                for level in levels {
                                    if ui
                                        .selectable_label(log_config.level == level, level)
                                        .clicked()
                                    {
                                        log_config.level = level.to_string();
                                        context.state.dirty = true;
                                    }
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Log Path:");
                        let path_str = log_config.log_path.to_string_lossy().to_string();
                        let mut path_edit = path_str.clone();
                        if ui.text_edit_singleline(&mut path_edit).changed() {
                            log_config.log_path = std::path::PathBuf::from(path_edit);
                            context.state.dirty = true;
                        }
                        if ui.button("📂").clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                log_config.log_path = folder;
                                context.state.dirty = true;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Max Log Files:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut log_config.max_files)
                                    .speed(1)
                                    .range(1..=100),
                            )
                            .changed()
                        {
                            context.state.dirty = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui
                            .checkbox(&mut log_config.console_output, "Console Output")
                            .changed()
                        {
                            context.state.dirty = true;
                        }
                        if ui
                            .checkbox(&mut log_config.file_output, "File Output")
                            .changed()
                        {
                            context.state.dirty = true;
                        }
                    });
                });

                ui.separator();

                // Philips Hue Settings
                let body_returned =
                    egui::CollapsingHeader::new(format!("💡 {}", "Philips Hue"))
                        .default_open(true)
                        .show(ui, |ui| {
                            let mut changed = false;
                            let mut connect_clicked = false;
                            let mut register_clicked = false;
                            let disconnect_clicked = false;
                            let mut discover_clicked = false;
                            let hue_conf = &mut context.ui_state.user_config.hue_config;

                            ui.horizontal(|ui| {
                                ui.label("Bridge IP:");
                                if ui.text_edit_singleline(&mut hue_conf.bridge_ip).changed() {
                                    changed = true;
                                }

                                if ui.button("Discover").clicked() {
                                    discover_clicked = true;
                                }
                            });

                            if !context.ui_state.discovered_hue_bridges.is_empty() {
                                egui::ComboBox::from_id_salt("discovered_bridges")
                                    .selected_text("Select Discovered Bridge...")
                                    .show_ui(ui, |ui| {
                                        for bridge in &context.ui_state.discovered_hue_bridges {
                                            if ui
                                                .selectable_label(
                                                    hue_conf.bridge_ip == bridge.ip,
                                                    format!("{} ({})", bridge.ip, bridge.id),
                                                )
                                                .clicked()
                                            {
                                                hue_conf.bridge_ip = bridge.ip.clone();
                                                changed = true;
                                            }
                                        }
                                    });
                            }

                            ui.horizontal(|ui| {
                                ui.label("App Key (User):");
                                ui.add_enabled(
                                    false,
                                    egui::TextEdit::singleline(&mut hue_conf.username),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Client Key:");
                                ui.add_enabled(
                                    false,
                                    egui::TextEdit::singleline(&mut hue_conf.client_key)
                                        .password(true),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Entertainment Group:");
                                let current_id = &mut hue_conf.entertainment_area;

                                // ComboBox for Group Selection
                                let selected_text = context
                                    .ui_state
                                    .available_hue_groups
                                    .iter()
                                    .find(|(id, _)| id == current_id)
                                    .map(|(_, name)| name.as_str())
                                    .unwrap_or(current_id.as_str());

                                egui::ComboBox::from_id_salt("hue_group_select")
                                    .selected_text(selected_text)
                                    .show_ui(ui, |ui| {
                                        for (id, name) in &context.ui_state.available_hue_groups {
                                            if ui
                                                .selectable_label(current_id == id, name)
                                                .clicked()
                                            {
                                                *current_id = id.clone();
                                                changed = true;
                                            }
                                        }
                                    });

                                // Refresh Button
                                if ui.button("🔄").on_hover_text("Refresh Groups").clicked() {
                                    context
                                        .ui_state
                                        .actions
                                        .push(mapmap_ui::UIAction::FetchHueGroups);
                                }
                            });

                            ui.horizontal(|ui| {
                                if ui
                                    .checkbox(
                                        &mut hue_conf.auto_connect,
                                        "Auto Connect via Main Settings",
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            });

                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                if ui.button("Verbinden (Sync)").clicked() {
                                    connect_clicked = true;
                                }

                                // Link Bridge Button
                                if ui.button("Link Bridge").on_hover_text("Press the button on your Hue Bridge, then click this to link.").clicked() {
                                    register_clicked = true;
                                }

                                // Connection Status Display
                                if context.hue_controller.is_connected() {
                                     ui.colored_label(egui::Color32::GREEN, "Connected");
                                } else {
                                     ui.colored_label(egui::Color32::RED, "Disconnected");
                                }
                            });

                            ui.label(egui::RichText::new("Note: Press Link Button on Bridge before linking/connecting for the first time.").small());
                            (changed, connect_clicked, disconnect_clicked, discover_clicked, register_clicked)
                        })
                        .body_returned;

                if let Some((changed, connect, disconnect, discover, register)) = body_returned {
                    if register {
                        context
                            .ui_state
                            .actions
                            .push(mapmap_ui::UIAction::RegisterHue);
                    }
                    if changed {
                        let _ = context.ui_state.user_config.save();
                        // Update controller config with correct types
                        let ui_hue = &context.ui_state.user_config.hue_config;
                        let control_hue = mapmap_control::hue::models::HueConfig {
                            bridge_ip: ui_hue.bridge_ip.clone(),
                            username: ui_hue.username.clone(),
                            client_key: ui_hue.client_key.clone(),
                            application_id: String::new(),
                            entertainment_group_id: ui_hue.entertainment_area.clone(),
                        };
                        context.hue_controller.update_config(control_hue);
                    }
                    if connect {
                        context
                            .ui_state
                            .actions
                            .push(mapmap_ui::UIAction::ConnectHue);
                    }
                    if disconnect {
                        context
                            .ui_state
                            .actions
                            .push(mapmap_ui::UIAction::DisconnectHue);
                    }
                    if discover {
                        context
                            .ui_state
                            .actions
                            .push(mapmap_ui::UIAction::DiscoverHueBridges);
                    }
                }

                ui.separator();

                // MIDI Settings
                #[cfg(feature = "midi")]
                egui::CollapsingHeader::new(format!("🎹 {}", "MIDI"))
                    .default_open(false)
                    .show(ui, |ui| {
                        // Connection status
                        let is_connected = context
                            .midi_handler
                            .as_ref()
                            .map(|h| h.is_connected())
                            .unwrap_or(false);
                        ui.horizontal(|ui| {
                            ui.label("Status:");
                            if is_connected {
                                ui.colored_label(egui::Color32::GREEN, "🟢 Connected");
                            } else {
                                ui.colored_label(egui::Color32::RED, "🔴 Disconnected");
                            }
                        });

                        // Port selection
                        ui.horizontal(|ui| {
                            ui.label("MIDI Port:");
                            let current_port = context
                                .selected_midi_port
                                .and_then(|i| context.midi_ports.get(i))
                                .cloned()
                                .unwrap_or_else(|| "None".to_string());

                            egui::ComboBox::from_id_salt("midi_port_select")
                                .selected_text(&current_port)
                                .show_ui(ui, |ui| {
                                    for (i, port) in context.midi_ports.iter().enumerate() {
                                        if ui
                                            .selectable_label(*context.selected_midi_port == Some(i), port)
                                            .clicked()
                                        {
                                            *context.selected_midi_port = Some(i);
                                            // Connect to the selected port
                                            if let Some(handler) = &mut context.midi_handler {
                                                handler.disconnect();
                                                match handler.connect(i) {
                                                    Ok(()) => {
                                                        info!("Connected to MIDI port: {}", port);
                                                    }
                                                    Err(e) => {
                                                        error!("Failed to connect to MIDI port: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                });
                        });

                        // Refresh ports button
                        if ui.button("🔄 Refresh Ports").clicked() {
                            if let Ok(ports) = MidiInputHandler::list_ports() {
                                *context.midi_ports = ports;
                                info!("Refreshed MIDI ports: {:?}", context.midi_ports);
                            }
                        }

                        // Show available ports count
                        ui.label(format!("{} port(s) available", context.midi_ports.len()));
                    });

                ui.separator();
                if ui.button("✕ Schließen").clicked() {
                    explicit_close = true;
                }
            });
    }

    if explicit_close {
        context.ui_state.show_settings = false;
    } else {
        context.ui_state.show_settings = show_settings;
    }
}
