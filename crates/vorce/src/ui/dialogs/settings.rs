use egui::{Color32, Context, RichText, Window};
use vorce_control::hue::controller::HueController;
use vorce_core::AppState;
use vorce_render::WgpuBackend;
use vorce_ui::core::config::{AppLogLevel, ToolbarMetricMode};
use vorce_ui::{AppUI, UIAction};

#[cfg(feature = "midi")]
use vorce_control::midi::MidiInputHandler;

/// Context required to render the settings window.
pub struct SettingsContext<'a> {
    /// UI State
    pub ui_state: &'a mut AppUI,
    /// App State
    pub state: &'a mut AppState,
    /// Wgpu Backend
    pub backend: &'a WgpuBackend,
    /// Hue Controller
    pub hue_controller: &'a mut HueController,
    /// MIDI Handler
    #[cfg(feature = "midi")]
    pub midi_handler: &'a mut Option<MidiInputHandler>,
    /// MIDI Ports
    #[cfg(feature = "midi")]
    pub midi_ports: &'a mut Vec<String>,
    /// Selected MIDI Port
    #[cfg(feature = "midi")]
    pub selected_midi_port: &'a mut Option<usize>,
    /// Restart Requested
    pub restart_requested: &'a mut bool,
    /// Exit Requested
    pub exit_requested: &'a mut bool,
    /// Tokio Runtime
    pub tokio_runtime: &'a tokio::runtime::Runtime,
}

/// Show settings dialog
pub fn show(ctx: &Context, context: SettingsContext) {
    let mut show_settings = context.ui_state.show_settings;

    Window::new(
        RichText::new(format!(
            "⚙ {}",
            context.ui_state.i18n.t("settings").to_uppercase()
        ))
        .strong()
        .color(Color32::from_rgb(0, 255, 255)),
    )
    .open(&mut show_settings)
    .resizable(true)
    .default_width(500.0)
    .show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 8.0);
            ui.style_mut().spacing.button_padding = egui::vec2(12.0, 7.0);
            ui.style_mut().spacing.interact_size = egui::vec2(30.0, 26.0);

            let tab_id = egui::Id::new("settings_active_tab");
            let mut active_tab = ctx.data_mut(|d| d.get_persisted::<usize>(tab_id).unwrap_or(0));
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut active_tab, 0, "Allgemein & Theme");
                ui.selectable_value(&mut active_tab, 1, "Animation & Layout");
                ui.selectable_value(&mut active_tab, 2, "Performance");
                ui.selectable_value(&mut active_tab, 3, "Audio & System");
            });
            ctx.data_mut(|d| d.insert_persisted(tab_id, active_tab));
            ui.separator();
            ui.add_space(4.0);

            if active_tab == 0 {
                ui.heading(RichText::new("General").color(Color32::WHITE));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", context.ui_state.i18n.t("language")));
                    let current_lang = context.ui_state.user_config.language.clone();
                    let lang_name = if current_lang == "de" {
                        "Deutsch"
                    } else {
                        "English"
                    };
                    egui::ComboBox::from_id_salt("lang_selector")
                        .selected_text(lang_name)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(current_lang == "de", "Deutsch")
                                .clicked()
                            {
                                context
                                    .ui_state
                                    .actions
                                    .push(UIAction::SetLanguage("de".to_string()));
                            }
                            if ui
                                .selectable_label(current_lang == "en", "English")
                                .clicked()
                            {
                                context
                                    .ui_state
                                    .actions
                                    .push(UIAction::SetLanguage("en".to_string()));
                            }
                        });
                });
                ui.add_space(10.0);
                ui.separator();
                ui.heading(
                    RichText::new(context.ui_state.i18n.t("appearance")).color(Color32::WHITE),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", context.ui_state.i18n.t("theme")));
                    let current_theme = context.ui_state.user_config.theme.theme;
                    egui::ComboBox::from_id_salt("theme_selector")
                        .selected_text(format!("{:?}", current_theme))
                        .show_ui(ui, |ui| {
                            use vorce_ui::core::theme::Theme;
                            for theme in [
                                Theme::Dark,
                                Theme::Light,
                                Theme::Resolume,
                                Theme::Synthwave,
                                Theme::Cyber,
                                Theme::Midnight,
                                Theme::Purple,
                                Theme::Pink,
                                Theme::HighContrast,
                            ] {
                                if ui
                                    .selectable_label(
                                        current_theme == theme,
                                        format!("{:?}", theme),
                                    )
                                    .clicked()
                                {
                                    context.ui_state.user_config.theme.theme = theme;
                                    context.ui_state.user_config.theme.apply(ctx);
                                    let _ = context.ui_state.user_config.save();
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Globaler UI-Scale:");
                    let mut ui_scale = context.ui_state.user_config.ui_scale;
                    if ui
                        .add(egui::Slider::new(&mut ui_scale, 0.8..=1.4).suffix("x"))
                        .changed()
                    {
                        context.ui_state.user_config.ui_scale = ui_scale;
                        let _ = context.ui_state.user_config.save();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Basis-Schriftgröße:");
                    let mut font_scale_percent =
                        (context.ui_state.user_config.theme.font_size / 14.0 * 100.0).round()
                            as i32;
                    if ui
                        .add(
                            egui::Slider::new(&mut font_scale_percent, 80..=140)
                                .text("%")
                                .suffix("%"),
                        )
                        .changed()
                    {
                        context.ui_state.user_config.theme.font_size =
                            14.0 * (font_scale_percent as f32 / 100.0);
                        context.ui_state.user_config.theme.apply(ctx);
                        let _ = context.ui_state.user_config.save();
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.heading(RichText::new("Toolbar-Metriken").color(Color32::WHITE));
                ui.add_space(4.0);

                let mut save_needed = false;
                let metrics = &mut context.ui_state.user_config.toolbar_metrics;

                let mut metric_row = |ui: &mut egui::Ui,
                                      label: &str,
                                      config: &mut vorce_ui::core::config::ToolbarMetricConfig,
                                      id: &str| {
                    ui.horizontal(|ui| {
                        save_needed |= ui.checkbox(&mut config.visible, label).changed();
                        ui.add_enabled_ui(config.visible, |ui| {
                            egui::ComboBox::from_id_salt(id)
                                .selected_text(match config.mode {
                                    ToolbarMetricMode::Always => "Permanent",
                                    ToolbarMetricMode::Hover => "Nur Hover/Popover",
                                })
                                .show_ui(ui, |ui| {
                                    save_needed |= ui
                                        .selectable_value(
                                            &mut config.mode,
                                            ToolbarMetricMode::Always,
                                            "Permanent",
                                        )
                                        .changed();
                                    save_needed |= ui
                                        .selectable_value(
                                            &mut config.mode,
                                            ToolbarMetricMode::Hover,
                                            "Nur Hover/Popover",
                                        )
                                        .changed();
                                });
                        });
                    });
                };

                metric_row(ui, "BPM (Tempo)", &mut metrics.bpm, "metric_bpm");
                metric_row(ui, "Audio Meter", &mut metrics.audio_meter, "metric_audio");
                metric_row(ui, "FPS (Frames/sec)", &mut metrics.fps, "metric_fps");
                metric_row(ui, "Frame Time", &mut metrics.frame_time, "metric_ft");
                metric_row(ui, "CPU Last", &mut metrics.cpu, "metric_cpu");
                metric_row(ui, "GPU Load", &mut metrics.gpu, "metric_gpu");
                metric_row(ui, "RAM Verbrauch", &mut metrics.ram, "metric_ram");
                metric_row(ui, "Status-Indikator", &mut metrics.status, "metric_status");

                if save_needed {
                    let _ = context.ui_state.user_config.save();
                }

                ui.add_space(10.0);
                ui.separator();
                ui.heading(RichText::new("Logging").color(Color32::WHITE));
                ui.add_space(4.0);

                let previous_log_level = context.ui_state.user_config.log_level;
                ui.horizontal(|ui| {
                    ui.label("Log-Level:");
                    egui::ComboBox::from_id_salt("log_level_selector")
                        .selected_text(context.ui_state.user_config.log_level.to_string())
                        .show_ui(ui, |ui| {
                            for level in [AppLogLevel::Info, AppLogLevel::Debug] {
                                if ui
                                    .selectable_label(
                                        context.ui_state.user_config.log_level == level,
                                        level.to_string(),
                                    )
                                    .clicked()
                                {
                                    context.ui_state.user_config.log_level = level;
                                }
                            }
                        });
                });
                if context.ui_state.user_config.log_level != previous_log_level {
                    context.state.settings_mut().log_config.level =
                        context.ui_state.user_config.log_level.as_str().to_string();
                    context.state.dirty = true;
                    let _ = context.ui_state.user_config.save();
                }
                ui.label(
                    RichText::new(
                        "Hinweis: Die Log-Level-Aenderung wird nach einem App-Neustart wirksam.",
                    )
                    .small()
                    .weak(),
                );

                ui.add_space(10.0);
                ui.separator();
            }

            if active_tab == 1 {
                ui.heading(RichText::new("Node-Animationen").color(Color32::WHITE));
                ui.add_space(4.0);
                let mut node_anim_changed = false;
                node_anim_changed |= ui
                    .checkbox(
                        &mut context.ui_state.user_config.node_animations_enabled,
                        "Node-Animationen aktivieren",
                    )
                    .changed();
                node_anim_changed |= ui
                    .checkbox(
                        &mut context.ui_state.user_config.short_circuit_animation_enabled,
                        "Kurzschluss-Effekt bei falschen Verbindungen",
                    )
                    .changed();
                node_anim_changed |= ui
                    .checkbox(
                        &mut context.ui_state.user_config.startup_animation_enabled,
                        "App-Start-Animation aktivieren",
                    )
                    .changed();
                ui.horizontal(|ui| {
                    ui.label("Startup-Video:");
                    node_anim_changed |= ui
                        .text_edit_singleline(
                            &mut context.ui_state.user_config.startup_animation_path,
                        )
                        .changed();
                    if ui.button("Standard").clicked() {
                        context.ui_state.user_config.startup_animation_path =
                            "resources/app_videos/MF-Mechanical_Cube_Logo_Splash_Animation.webm"
                                .to_string();
                        node_anim_changed = true;
                    }
                });
                node_anim_changed |= ui
                    .checkbox(
                        &mut context.ui_state.user_config.reduce_motion_enabled,
                        "Reduce Motion (Bewegungen reduzieren)",
                    )
                    .changed();
                node_anim_changed |= ui
                    .checkbox(
                        &mut context.ui_state.user_config.silent_startup_enabled,
                        "Silent Startup (kein Startsound)",
                    )
                    .changed();
                ui.horizontal(|ui| {
                    ui.label("Animationsprofil:");
                    use vorce_ui::core::config::AnimationProfile;
                    egui::ComboBox::from_id_salt("ui_animation_profile")
                        .selected_text(context.ui_state.user_config.animation_profile.to_string())
                        .show_ui(ui, |ui| {
                            for profile in [
                                AnimationProfile::Off,
                                AnimationProfile::Subtle,
                                AnimationProfile::Cinematic,
                            ] {
                                if ui
                                    .selectable_label(
                                        context.ui_state.user_config.animation_profile == profile,
                                        profile.to_string(),
                                    )
                                    .clicked()
                                {
                                    context.ui_state.user_config.animation_profile = profile;
                                    let _ = context.ui_state.user_config.save();
                                }
                            }
                        });
                });
                if node_anim_changed {
                    let _ = context.ui_state.user_config.save();
                }

                ui.add_space(10.0);
                ui.separator();

                ui.heading(RichText::new("Layout Profiles").color(Color32::WHITE));
                ui.add_space(4.0);

                let active_layout_before = context.ui_state.user_config.active_layout_id.clone();
                let layout_items: Vec<(String, String)> = context
                    .ui_state
                    .user_config
                    .layouts
                    .iter()
                    .map(|l| (l.id.clone(), l.name.clone()))
                    .collect();

                let mut selected_layout_id = active_layout_before.clone();
                ui.horizontal(|ui| {
                    ui.label("Aktives Layout:");
                    egui::ComboBox::from_id_salt("layout_profile_selector")
                        .selected_text(
                            layout_items
                                .iter()
                                .find(|(id, _)| id == &selected_layout_id)
                                .map(|(_, name)| name.clone())
                                .unwrap_or_else(|| selected_layout_id.clone()),
                        )
                        .show_ui(ui, |ui| {
                            for (id, name) in &layout_items {
                                if ui
                                    .selectable_label(selected_layout_id == *id, name)
                                    .clicked()
                                {
                                    selected_layout_id = id.clone();
                                }
                            }
                        });

                    if ui.button("Duplizieren").clicked() {
                        if let Some(active) = context.ui_state.user_config.active_layout().cloned()
                        {
                            let mut clone = active;
                            let next = context.ui_state.user_config.layouts.len() + 1;
                            clone.id = format!("layout-{}", next);
                            clone.name = format!("{} {}", clone.name, next);
                            context.ui_state.user_config.add_layout_profile(clone);
                            let _ = context.ui_state.user_config.save();
                        }
                    }

                    if ui.button("Zurücksetzen").clicked() {
                        if let Some(layout) = context.ui_state.user_config.active_layout_mut() {
                            let id = layout.id.clone();
                            let name = layout.name.clone();
                            *layout = vorce_ui::core::config::LayoutProfile::default_profile();
                            layout.id = id;
                            layout.name = name;
                        }
                        context.ui_state.apply_active_layout();
                        let _ = context.ui_state.user_config.save();
                    }
                });

                if selected_layout_id != active_layout_before
                    && context
                        .ui_state
                        .user_config
                        .set_active_layout(&selected_layout_id)
                {
                    context.ui_state.apply_active_layout();
                    let _ = context.ui_state.user_config.save();
                }

                ui.add_space(4.0);
                ui.label("Panel-Sichtbarkeit");
                let mut changed_visibility = false;
                changed_visibility |= ui
                    .checkbox(&mut context.ui_state.show_toolbar, "Toolbar")
                    .changed();
                changed_visibility |= ui
                    .checkbox(&mut context.ui_state.show_left_sidebar, "Left Sidebar")
                    .changed();
                changed_visibility |= ui
                    .checkbox(&mut context.ui_state.show_inspector, "Inspector")
                    .changed();
                changed_visibility |= ui
                    .checkbox(&mut context.ui_state.show_timeline, "Timeline")
                    .changed();
                changed_visibility |= ui
                    .checkbox(&mut context.ui_state.show_media_browser, "Media Browser")
                    .changed();
                changed_visibility |= ui
                    .checkbox(&mut context.ui_state.show_module_canvas, "Module Canvas")
                    .changed();

                if changed_visibility {
                    context.ui_state.sync_runtime_to_active_layout();
                    let _ = context.ui_state.user_config.save();
                }

                if let Some(layout) = context.ui_state.user_config.active_layout_mut() {
                    ui.add_space(4.0);
                    ui.label("Panel-Größen");
                    let mut changed_sizes = false;
                    changed_sizes |= ui
                        .add(
                            egui::Slider::new(
                                &mut layout.panel_sizes.left_sidebar_width,
                                220.0..=640.0,
                            )
                            .text("Sidebar Breite"),
                        )
                        .changed();
                    changed_sizes |= ui
                        .add(
                            egui::Slider::new(
                                &mut layout.panel_sizes.inspector_width,
                                260.0..=760.0,
                            )
                            .text("Inspector Breite"),
                        )
                        .changed();
                    changed_sizes |= ui
                        .add(
                            egui::Slider::new(
                                &mut layout.panel_sizes.timeline_height,
                                100.0..=500.0,
                            )
                            .text("Timeline Höhe"),
                        )
                        .changed();
                    changed_sizes |= ui
                        .checkbox(&mut layout.lock_layout, "Layout sperren")
                        .changed();

                    if changed_sizes {
                        let _ = context.ui_state.user_config.save();
                    }
                }

                ui.add_space(10.0);
                ui.separator();
            }

            if active_tab == 2 {
                ui.heading(
                    RichText::new(format!(
                        "{} & {}",
                        context.ui_state.i18n.t("graphics"),
                        context.ui_state.i18n.t("performance")
                    ))
                    .color(Color32::WHITE),
                );
                ui.add_space(4.0);
                egui::Grid::new("perf_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(format!("{}:", context.ui_state.i18n.t("hw-accel")));
                        ui.label("Enabled");
                        ui.end_row();
                        ui.label(format!("{}:", context.ui_state.i18n.t("target-fps")));
                        let mut fps = context.ui_state.user_config.target_fps.unwrap_or(60.0);
                        if ui
                            .add(egui::Slider::new(&mut fps, 24.0..=144.0).suffix(" FPS"))
                            .changed()
                        {
                            context.ui_state.actions.push(UIAction::SetTargetFps(fps));
                        }
                        ui.end_row();
                        ui.label("VSync Mode:");
                        let vsync = context.ui_state.user_config.vsync_mode;
                        egui::ComboBox::from_id_salt("vsync_select")
                            .selected_text(vsync.to_string())
                            .show_ui(ui, |ui| {
                                use vorce_ui::core::config::VSyncMode;
                                for mode in [VSyncMode::Auto, VSyncMode::On, VSyncMode::Off] {
                                    if ui
                                        .selectable_label(vsync == mode, mode.to_string())
                                        .clicked()
                                    {
                                        context.ui_state.actions.push(UIAction::SetVsyncMode(mode));
                                    }
                                }
                            });
                        ui.end_row();
                        ui.label("Preferred GPU:");
                        let current_gpu = context.ui_state.user_config.preferred_gpu.clone();
                        let gpu_text = current_gpu.unwrap_or_else(|| "Default".to_string());
                        ui.horizontal(|ui| {
                            let mut temp_gpu = gpu_text.clone();
                            if ui.text_edit_singleline(&mut temp_gpu).changed() {
                                let new_val = if temp_gpu.trim().is_empty()
                                    || temp_gpu.trim().eq_ignore_ascii_case("default")
                                {
                                    None
                                } else {
                                    Some(temp_gpu.trim().to_string())
                                };
                                context
                                    .ui_state
                                    .actions
                                    .push(UIAction::SetPreferredGpu(new_val));
                            }
                            if ui.button("Clear").clicked() {
                                context
                                    .ui_state
                                    .actions
                                    .push(UIAction::SetPreferredGpu(None));
                            }
                        });
                        ui.end_row();
                    });
                ui.add_space(10.0);
                ui.separator();
            }

            if active_tab == 3 {
                ui.heading(RichText::new(context.ui_state.i18n.t("audio")).color(Color32::WHITE));
                if cfg!(target_os = "macos") {
                    ui.add_space(8.0);
                    vorce_ui::widgets::custom::render_info_label(
                        ui,
                        "Audio input is currently feature-gated on macOS for stability.",
                    );
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", context.ui_state.i18n.t("label-device")));
                    let current_device = context
                        .ui_state
                        .selected_audio_device
                        .clone()
                        .unwrap_or_else(|| context.ui_state.i18n.t("no-device"));
                    egui::ComboBox::from_id_salt("audio_device_selector")
                        .selected_text(&current_device)
                        .show_ui(ui, |ui| {
                            for device in &context.ui_state.audio_devices {
                                let is_selected =
                                    Some(device) == context.ui_state.selected_audio_device.as_ref();
                                ui.add_enabled_ui(!cfg!(target_os = "macos"), |ui| {
                                    if ui.selectable_label(is_selected, device).clicked() {
                                        context
                                            .ui_state
                                            .actions
                                            .push(UIAction::SelectAudioDevice(device.clone()));
                                    }
                                });
                            }
                        });
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Sample Rate:");
                    let mut sample_rate = context.state.audio_config.sample_rate;
                    egui::ComboBox::from_id_salt("audio_sample_rate_selector")
                        .selected_text(format!("{} Hz", sample_rate))
                        .show_ui(ui, |ui| {
                            for rate in [22050_u32, 44100, 48000, 96000] {
                                ui.add_enabled_ui(!cfg!(target_os = "macos"), |ui| {
                                    if ui
                                        .selectable_label(
                                            sample_rate == rate,
                                            format!("{} Hz", rate),
                                        )
                                        .clicked()
                                    {
                                        sample_rate = rate;
                                    }
                                });
                            }
                        });
                    if sample_rate != context.state.audio_config.sample_rate {
                        let mut cfg = context.state.audio_config.clone();
                        cfg.sample_rate = sample_rate;
                        context
                            .ui_state
                            .actions
                            .push(UIAction::UpdateAudioConfig(cfg));
                    }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Buffer Size:");
                    let mut fft_size = context.state.audio_config.fft_size;
                    egui::ComboBox::from_id_salt("audio_buffer_size_selector")
                        .selected_text(format!("{}", fft_size))
                        .show_ui(ui, |ui| {
                            for size in [256_usize, 512, 1024, 2048, 4096] {
                                ui.add_enabled_ui(!cfg!(target_os = "macos"), |ui| {
                                    if ui
                                        .selectable_label(fft_size == size, format!("{}", size))
                                        .clicked()
                                    {
                                        fft_size = size;
                                    }
                                });
                            }
                        });
                    if fft_size != context.state.audio_config.fft_size {
                        let mut cfg = context.state.audio_config.clone();
                        cfg.fft_size = fft_size;
                        context
                            .ui_state
                            .actions
                            .push(UIAction::UpdateAudioConfig(cfg));
                    }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Level Meter Style:");
                    let meter = context.ui_state.user_config.meter_style;
                    egui::ComboBox::from_id_salt("meter_select")
                        .selected_text(format!("{:?}", meter))
                        .show_ui(ui, |ui| {
                            use vorce_ui::core::config::AudioMeterStyle;
                            for style in [AudioMeterStyle::Retro, AudioMeterStyle::Digital] {
                                ui.add_enabled_ui(!cfg!(target_os = "macos"), |ui| {
                                    if ui
                                        .selectable_label(meter == style, format!("{:?}", style))
                                        .clicked()
                                    {
                                        context
                                            .ui_state
                                            .actions
                                            .push(UIAction::SetMeterStyle(style));
                                    }
                                });
                            }
                        });
                });
                ui.add_space(20.0);
                ui.separator();
                ui.vertical_centered(|ui| {
                    if ui
                        .button(
                            RichText::new(context.ui_state.i18n.t("restart-app"))
                                .color(Color32::RED)
                                .strong(),
                        )
                        .clicked()
                    {
                        *context.restart_requested = true;
                        *context.exit_requested = true;
                    }
                });
            }
        });
    });
    context.ui_state.show_settings = show_settings;
}
