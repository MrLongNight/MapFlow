use super::super::mesh;
use super::super::state::ModuleCanvas;
#[cfg(feature = "ndi")]
use super::capabilities;
use egui::Ui;
use mapmap_core::module::{HueMappingMode, ModulePartId, OutputType};

/// Renders the hue bridge discovery UI.
pub fn render_hue_bridge_discovery(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    current_ip: &mut String,
) {
    if ui.button("🔍 Discover Bridges").clicked() {
        let (tx, rx) = std::sync::mpsc::channel();
        canvas.hue_discovery_rx = Some(rx);
        // Spawn async task
        #[cfg(feature = "tokio")]
        {
            canvas.hue_status_message = Some("Searching...".to_string());
            let task = async move {
                let result = mapmap_control::hue::api::discovery::discover_bridges()
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(result);
            };
            tokio::spawn(task);
        }
        #[cfg(not(feature = "tokio"))]
        {
            let _ = tx;
            canvas.hue_status_message = Some("Async runtime not available".to_string());
        }
    }

    if !canvas.hue_bridges.is_empty() {
        ui.separator();
        ui.label("Select Bridge:");
        for bridge in &canvas.hue_bridges {
            if ui
                .button(format!("{} ({})", bridge.id, bridge.ip))
                .clicked()
            {
                *current_ip = bridge.ip.clone();
            }
        }
    }
}

/// Renders the configuration UI for a `ModulePartType::Output`.
pub fn render_output_ui(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    output: &mut OutputType,
    _part_id: ModulePartId,
) {
    ui.label("Output:");
    match output {
        OutputType::Projector {
            id,
            name,
            hide_cursor,
            target_screen,
            show_in_preview_panel,
            extra_preview_window,
            output_width,
            output_height,
            output_fps,
            ndi_enabled: _ndi_enabled,
            ndi_stream_name: _ndi_stream_name,
            ..
        } => {
            ui.label("📽️ Projector Output");

            // Output ID selection
            ui.horizontal(|ui| {
                ui.label("Output #:");
                ui.add(egui::DragValue::new(id).range(1..=8));
            });

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(name);
            });

            ui.separator();
            ui.label("🖥️ Window Settings:");

            // Target screen selection
            ui.horizontal(|ui| {
                ui.label("Target Screen:");
                egui::ComboBox::from_id_salt("target_screen_select")
                    .selected_text(format!("Monitor {}", target_screen))
                    .show_ui(ui, |ui| {
                        for i in 0..=3u8 {
                            let label = if i == 0 {
                                "Primary".to_string()
                            } else {
                                format!("Monitor {}", i)
                            };
                            if ui.selectable_label(*target_screen == i, &label).clicked() {
                                *target_screen = i;
                            }
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Width:");
                ui.add(egui::DragValue::new(output_width).range(1..=7680));
            });

            ui.horizontal(|ui| {
                ui.label("Height:");
                ui.add(egui::DragValue::new(output_height).range(1..=4320));
            });

            ui.horizontal(|ui| {
                ui.label("FPS:");
                ui.add(egui::DragValue::new(output_fps).range(1.0..=240.0));
            });

            ui.checkbox(hide_cursor, "🖱️ Hide Mouse Cursor");

            ui.separator();
            ui.label("⚙️ Advanced Setup:");
            ui.horizontal(|ui| {
                ui.label("Resolution:");
                ui.add(
                    egui::DragValue::new(output_width)
                        .suffix(" px")
                        .range(0..=8192),
                );
                ui.label("x");
                ui.add(
                    egui::DragValue::new(output_height)
                        .suffix(" px")
                        .range(0..=8192),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Target FPS:");
                ui.add(egui::DragValue::new(output_fps).range(0.0..=240.0));
            });

            ui.separator();
            ui.label("👁️ Preview:");
            ui.checkbox(show_in_preview_panel, "Show in Preview Panel");
            ui.checkbox(extra_preview_window, "Extra Preview Window");

            ui.separator();
            ui.label("\u{1F4E1} NDI Broadcast");
            #[cfg(feature = "ndi")]
            {
                let supported = capabilities::is_output_type_enum_supported(true, false);
                if !supported {
                    #[cfg(target_os = "macos")]
                    capabilities::render_unsupported_warning(
                        ui,
                        "NDI Output is experimental/unavailable on macOS currently.",
                    );
                    #[cfg(not(target_os = "macos"))]
                    capabilities::render_unsupported_warning(
                        ui,
                        "[Experimental] NDI Output has no active runtime path currently.",
                    );
                }
                ui.add_enabled_ui(supported, |ui| {
                    ui.checkbox(_ndi_enabled, "Enable NDI Output");
                    if *_ndi_enabled {
                        ui.horizontal(|ui| {
                            ui.label("Stream Name:");
                            ui.text_edit_singleline(_ndi_stream_name);
                        });
                        if _ndi_stream_name.is_empty() {
                            ui.small(format!("Default: {}", name));
                        }
                    }
                });
            }
            #[cfg(not(feature = "ndi"))]
            {
                ui.label("NDI feature disabled in build");
            }
        }
        #[cfg(feature = "ndi")]
        OutputType::NdiOutput { name } => {
            ui.label("\u{1F4E1} NDI Output");
            capabilities::render_unsupported_warning(
                ui,
                "[Experimental] NDI Output has no active runtime path currently.",
            );
            ui.horizontal(|ui| {
                ui.label("Stream Name:");
                ui.text_edit_singleline(name);
            });
        }
        #[cfg(not(feature = "ndi"))]
        OutputType::NdiOutput { .. } => {
            ui.label("\u{1F4E1} NDI Output (Feature Disabled)");
            crate::editors::module_canvas::inspector::capabilities::render_unsupported_warning(
                ui,
                "[Experimental] NDI feature is disabled in this build.",
            );
        }
        #[cfg(target_os = "windows")]
        OutputType::Spout { name } => {
            ui.label("\u{1F6B0} Spout Output");
            ui.horizontal(|ui| {
                ui.label("Stream Name:");
                ui.text_edit_singleline(name);
            });
        }
        OutputType::Hue {
            bridge_ip,
            username,
            client_key: _client_key,
            entertainment_area,
            lamp_positions,
            mapping_mode,
        } => {
            ui.label("\u{1F4A1} Philips Hue Entertainment");
            ui.separator();

            // --- Tabs for Hue configuration ---
            ui.collapsing("⚙️ Setup (Bridge & Pairing)", |ui| {
                // Discovery status
                if let Some(msg) = &canvas.hue_status_message {
                    ui.label(format!("Status: {}", msg));
                }

                // Handle discovery results
                if let Some(rx) = &canvas.hue_discovery_rx {
                    if let Ok(result) = rx.try_recv() {
                        canvas.hue_discovery_rx = None;
                        // Explicit type annotation for the result to help inference
                        let result: Result<
                            Vec<mapmap_control::hue::api::discovery::DiscoveredBridge>,
                            _,
                        > = result;
                        match result {
                            Ok(bridges) => {
                                canvas.hue_bridges = bridges;
                                canvas.hue_status_message =
                                    Some(format!("Found {} bridges", canvas.hue_bridges.len()));
                            }
                            Err(e) => {
                                canvas.hue_status_message =
                                    Some(format!("Discovery failed: {}", e));
                            }
                        }
                    } else {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Searching for bridges...");
                        });
                    }
                }

                // Using a block to allow attributes on expression
                // Hue Discovery Logic extracted to method to satisfy rustfmt
                render_hue_bridge_discovery(canvas, ui, bridge_ip);

                ui.separator();
                ui.label("Manual IP:");
                ui.text_edit_singleline(bridge_ip);

                // Pairing (Requires bridge button press)
                if ui
                    .button("\u{1F517} Pair with Bridge")
                    .on_hover_text("Press button on Bridge then click this")
                    .clicked()
                {
                    let (tx, rx) = std::sync::mpsc::channel();
                    canvas.hue_pairing_rx = Some(rx);
                    let ip = bridge_ip.clone();

                    #[cfg(feature = "tokio")]
                    {
                        canvas.hue_status_message =
                            Some("Pairing... (Press Bridge Button)".to_string());
                        let task = async move {
                            let result =
                                mapmap_control::hue::api::client::HueClient::register_user(
                                    &ip, "MapFlow",
                                )
                                .await
                                .map_err(|e| e.to_string());
                            let _ = tx.send(result);
                        };
                        tokio::spawn(task);
                    }
                    #[cfg(not(feature = "tokio"))]
                    {
                        let _ = tx;
                        let _ = ip;
                        canvas.hue_status_message = Some("Async runtime not available".to_string());
                    }
                }

                // Handle pairing results
                if let Some(rx) = &canvas.hue_pairing_rx {
                    if let Ok(result) = rx.try_recv() {
                        canvas.hue_pairing_rx = None;
                        match result {
                            Ok(config) => {
                                *username = config.username;
                                *_client_key = config.client_key;
                                canvas.hue_status_message = Some("Pairing Successful!".to_string());
                            }
                            Err(e) => {
                                canvas.hue_status_message = Some(format!("Pairing failed: {}", e));
                            }
                        }
                    }
                }

                if !username.is_empty() {
                    ui.label("\u{2705} Paired");
                    // ui.label(format!("User: {}", username)); // Keep secret?
                } else {
                    ui.label("❌ Not Paired");
                }
            });

            ui.collapsing("\u{1F3AD} Area & Mode", |ui| {
                ui.label("Entertainment Area:");
                ui.text_edit_singleline(entertainment_area);
                // TODO: Fetch areas from bridge if paired

                ui.separator();
                ui.label("Mapping Mode:");
                ui.radio_value(
                    mapping_mode,
                    HueMappingMode::Ambient,
                    "Ambient (Average Color)",
                );
                ui.radio_value(mapping_mode, HueMappingMode::Spatial, "Spatial (2D Map)");
                ui.radio_value(
                    mapping_mode,
                    HueMappingMode::Trigger,
                    "Trigger (Strobe/Pulse)",
                );
            });

            if *mapping_mode == HueMappingMode::Spatial {
                ui.collapsing("🗺️ Spatial Editor", |ui| {
                    ui.label("Position lamps in the virtual room:");
                    // Render 2D room editor
                    mesh::render_hue_spatial_editor(ui, lamp_positions);
                });
            }
        }
    }
}
