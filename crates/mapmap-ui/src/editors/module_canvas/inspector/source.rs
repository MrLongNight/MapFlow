use crate::theme::colors;
use crate::widgets::styled_slider;
use crate::UIAction;
use egui::{Color32, Ui, Vec2};
use mapmap_core::module::{BevyCameraMode, ModuleId, ModulePartId, SourceType};
use super::super::state::ModuleCanvas;
use super::common::{render_common_controls, render_transport_controls, render_timeline};
use super::super::types::MediaPlaybackCommand;

/// Renders the configuration UI for a `ModulePartType::Source`.
pub fn render_source_ui(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    source: &mut SourceType,
    part_id: ModulePartId,
    module_id: ModuleId,
    shared_media_ids: &[String],
    actions: &mut Vec<UIAction>,
) {
    ui.horizontal(|ui| {
        ui.label("Source Type:");
        let current_mode = match source {
            SourceType::MediaFile { .. } => "\u{1F4F9} Media File",
            SourceType::VideoUni { .. } => "\u{1F4F9} Video (Uni)",
            SourceType::ImageUni { .. } => "\u{1F5BC} Image (Uni)",
            SourceType::VideoMulti { .. } => "\u{1F517} Video (Multi)",
            SourceType::ImageMulti { .. } => "\u{1F517} Image (Multi)",
            SourceType::Shader { .. } => "\u{1F3A8} Shader",
            SourceType::LiveInput { .. } => "\u{1F4F9} Live Input",
            SourceType::NdiInput { .. } => "\u{1F4E1} NDI Input",
            #[cfg(target_os = "windows")]
            SourceType::SpoutInput { .. } => "\u{1F6B0} Spout Input",
            SourceType::Bevy => "\u{1F3AE} Bevy Scene",
            SourceType::BevyAtmosphere { .. } => "☁️ Atmosphere",
            SourceType::BevyHexGrid { .. } => "\u{1F6D1} Hex Grid",
            SourceType::BevyParticles { .. } => "\u{2728} Particles",
            SourceType::Bevy3DShape { .. } => "\u{1F9CA} 3D Shape",
            SourceType::Bevy3DText { .. } => "📝 3D Text",
            SourceType::BevyCamera { .. } => "\u{1F3A5} Bevy Camera",
            SourceType::Bevy3DModel { .. } => "\u{1F3AE} 3D Model",
        };

        let mut next_type = None;
        egui::ComboBox::from_id_salt(format!("{}_source_type_picker", part_id))
            .selected_text(current_mode)
            .show_ui(ui, |ui| {
                ui.label("--- File Based ---");
                if ui.selectable_label(matches!(source, SourceType::MediaFile { .. }), "\u{1F4F9} Media File").clicked() { next_type = Some("MediaFile"); }
                if ui.selectable_label(matches!(source, SourceType::VideoUni { .. }), "\u{1F4F9} Video (Uni)").clicked() { next_type = Some("VideoUni"); }
                if ui.selectable_label(matches!(source, SourceType::ImageUni { .. }), "\u{1F5BC} Image (Uni)").clicked() { next_type = Some("ImageUni"); }

                ui.label("--- Shared ---");
                if ui.selectable_label(matches!(source, SourceType::VideoMulti { .. }), "\u{1F517} Video (Multi)").clicked() { next_type = Some("VideoMulti"); }
                if ui.selectable_label(matches!(source, SourceType::ImageMulti { .. }), "\u{1F517} Image (Multi)").clicked() { next_type = Some("ImageMulti"); }
            });

        if let Some(t) = next_type {
            let path = match source {
                SourceType::MediaFile { path, .. } => path.clone(),
                SourceType::VideoUni { path, .. } => path.clone(),
                SourceType::ImageUni { path, .. } => path.clone(),
                _ => String::new(),
            };
            let shared_id = match source {
                SourceType::VideoMulti { shared_id, .. } => shared_id.clone(),
                SourceType::ImageMulti { shared_id, .. } => shared_id.clone(),
                _ => String::new(),
            };

            *source = match t {
                "MediaFile" => SourceType::new_media_file(if path.is_empty() { shared_id } else { path }),
                "VideoUni" => SourceType::VideoUni {
                    path: if path.is_empty() { shared_id } else { path },
                    speed: 1.0, loop_enabled: true, start_time: 0.0, end_time: 0.0,
                    opacity: 1.0, blend_mode: None, brightness: 0.0, contrast: 1.0, saturation: 1.0, hue_shift: 0.0,
                    scale_x: 1.0, scale_y: 1.0, rotation: 0.0, offset_x: 0.0, offset_y: 0.0,
                    target_width: None, target_height: None, target_fps: None,
                    flip_horizontal: false, flip_vertical: false, reverse_playback: false,
                },
                "ImageUni" => SourceType::ImageUni {
                    path: if path.is_empty() { shared_id } else { path },
                    opacity: 1.0, blend_mode: None, brightness: 0.0, contrast: 1.0, saturation: 1.0, hue_shift: 0.0,
                    scale_x: 1.0, scale_y: 1.0, rotation: 0.0, offset_x: 0.0, offset_y: 0.0,
                    target_width: None, target_height: None,
                    flip_horizontal: false, flip_vertical: false,
                },
                "VideoMulti" => SourceType::VideoMulti {
                    shared_id: if shared_id.is_empty() { path } else { shared_id },
                    opacity: 1.0, blend_mode: None, brightness: 0.0, contrast: 1.0, saturation: 1.0, hue_shift: 0.0,
                    scale_x: 1.0, scale_y: 1.0, rotation: 0.0, offset_x: 0.0, offset_y: 0.0,
                    flip_horizontal: false, flip_vertical: false,
                },
                "ImageMulti" => SourceType::ImageMulti {
                    shared_id: if shared_id.is_empty() { path } else { shared_id },
                    opacity: 1.0, blend_mode: None, brightness: 0.0, contrast: 1.0, saturation: 1.0, hue_shift: 0.0,
                    scale_x: 1.0, scale_y: 1.0, rotation: 0.0, offset_x: 0.0, offset_y: 0.0,
                    flip_horizontal: false, flip_vertical: false,
                },
                _ => source.clone(),
            };
        }
    });

    ui.separator();

    match source {
        SourceType::MediaFile {
            path, speed, loop_enabled, start_time, end_time, opacity, blend_mode,
            brightness, contrast, saturation, hue_shift, scale_x, scale_y, rotation,
            offset_x, offset_y, flip_horizontal, flip_vertical, reverse_playback, ..
        } | SourceType::VideoUni {
            path, speed, loop_enabled, start_time, end_time, opacity, blend_mode,
            brightness, contrast, saturation, hue_shift, scale_x, scale_y, rotation,
            offset_x, offset_y, flip_horizontal, flip_vertical, reverse_playback, ..
        } => {
            // Media Picker (common for file-based video)
            if path.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    if ui.button("\u{1F4C2} Select Media File").clicked() {
                        actions.push(UIAction::PickMediaFile(module_id, part_id, "".to_string()));
                    }
                    ui.label(egui::RichText::new("No media loaded").weak().italics());
                    ui.add_space(10.0);
                });
            } else {
                ui.collapsing("📁 File Info", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.add(egui::TextEdit::singleline(path).desired_width(160.0));
                        if ui.button("\u{1F4C2}").on_hover_text("Select Media File").clicked() {
                            actions.push(UIAction::PickMediaFile(module_id, part_id, "".to_string()));
                        }
                    });
                });
            }

            // Playback Info
            let player_info = canvas.player_info.get(&part_id).cloned().unwrap_or_default();
            let video_duration = player_info.duration.max(1.0) as f32;
            let current_pos = player_info.current_time as f32;
            let is_playing = player_info.is_playing;

            // Timecode
            let current_min = (current_pos / 60.0) as u32;
            let current_sec = (current_pos % 60.0) as u32;
            let current_frac = ((current_pos * 100.0) % 100.0) as u32;
            let duration_min = (video_duration / 60.0) as u32;
            let duration_sec = (video_duration % 60.0) as u32;
            let duration_frac = ((video_duration * 100.0) % 100.0) as u32;

            ui.add_space(5.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{:02}:{:02}.{:02} / {:02}:{:02}.{:02}",
                        current_min, current_sec, current_frac,
                        duration_min, duration_sec, duration_frac
                    ))
                    .monospace().size(22.0).strong()
                    .color(if is_playing { Color32::from_rgb(100, 255, 150) } else { Color32::from_rgb(200, 200, 200) })
                );
            });
            ui.add_space(10.0);

            render_transport_controls(canvas, ui, part_id, is_playing, current_pos, loop_enabled, reverse_playback);

            ui.add_space(10.0);

            // Preview
            if let Some(tex_id) = canvas.node_previews.get(&(module_id, part_id)) {
                let size = Vec2::new(ui.available_width(), ui.available_width() * 9.0 / 16.0);
                ui.image((*tex_id, size));
            }
            ui.add_space(4.0);

            render_timeline(canvas, ui, part_id, video_duration, current_pos, start_time, end_time);

            // Safe Reset Clip (Mary StyleUX)
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);
                if crate::widgets::hold_to_action_button(
                    ui,
                    "\u{27F2} Reset Clip",
                    colors::WARN_COLOR,
                ) {
                    *start_time = 0.0;
                    *end_time = 0.0;
                }
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Playback Speed:");
                let speed_slider = styled_slider(ui, speed, 0.1..=4.0, 1.0);
                ui.label("x");
                if speed_slider.changed() {
                    actions.push(UIAction::MediaCommand(part_id, MediaPlaybackCommand::SetSpeed(*speed)));
                }
            });
            ui.separator();

            // === VIDEO OPTIONS ===
            ui.collapsing("\u{1F3AC} Video Options", |ui| {
                let mut reverse = *reverse_playback;
                if ui.checkbox(&mut reverse, "⏪ Reverse Playback").changed() {
                    actions.push(crate::UIAction::MediaCommand(part_id, MediaPlaybackCommand::SetReverse(reverse)));
                }

                ui.separator();
                ui.label("Seek Position:");
                // Note: Actual seek requires video duration from player
                // For now, just show the control - needs integration with player state
                let mut seek_pos: f64 = 0.0;
                let seek_slider = ui.add(
                    egui::Slider::new(&mut seek_pos, 0.0..=100.0)
                        .text("Position")
                        .suffix("%")
                        .show_value(true)
                );
                if seek_slider.drag_stopped() && seek_slider.changed() {
                    // Convert percentage to duration-based seek
                    // This will need actual video duration from player
                    canvas.pending_playback_commands.push((part_id, MediaPlaybackCommand::Seek(seek_pos / 100.0 * 300.0)));
                }
            });
            ui.separator();

            render_common_controls(
                ui, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical
            );
        }
        SourceType::ImageUni {
            path, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
            scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical, ..
        } => {
            // Image Picker
            if path.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    if ui.button("\u{1F4C2} Select Image File").clicked() {
                        actions.push(crate::UIAction::PickMediaFile(module_id, part_id, "".to_string()));
                    }
                    ui.label(egui::RichText::new("No image loaded").weak().italics());
                    ui.add_space(10.0);
                });
            } else {
                ui.collapsing("📁 File Info", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.add(egui::TextEdit::singleline(path).desired_width(160.0));
                        if ui.button("\u{1F4C2}").on_hover_text("Select Image File").clicked() {
                            actions.push(crate::UIAction::PickMediaFile(module_id, part_id, "".to_string()));
                        }
                    });
                });
            }

            ui.separator();
            render_common_controls(
                ui, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical
            );

        }
        SourceType::VideoMulti {
            shared_id, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
            scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical, ..
        } => {
            ui.label("\u{1F517} Shared Video Source");
            ui.horizontal(|ui| {
                ui.label("Shared ID:");
                ui.add(egui::TextEdit::singleline(shared_id).hint_text("Enter ID...").desired_width(140.0));

                egui::ComboBox::from_id_salt("shared_media_video")
                    .selected_text("Select Existing")
                    .show_ui(ui, |ui| {
                        for id in shared_media_ids {
                            if ui.selectable_label(shared_id == id, id).clicked() {
                                *shared_id = id.clone();
                            }
                        }
                    });
            });
            ui.label(egui::RichText::new("Use the same ID to sync multiple nodes.").weak().small());

            ui.separator();
            render_common_controls(
                ui, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical
            );
        }
        SourceType::ImageMulti {
            shared_id, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
            scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical, ..
        } => {
            ui.label("\u{1F517} Shared Image Source");
            ui.horizontal(|ui| {
                ui.label("Shared ID:");
                ui.add(egui::TextEdit::singleline(shared_id).hint_text("Enter ID...").desired_width(140.0));

                egui::ComboBox::from_id_salt("shared_media_image")
                    .selected_text("Select Existing")
                    .show_ui(ui, |ui| {
                        for id in shared_media_ids {
                            if ui.selectable_label(shared_id == id, id).clicked() {
                                *shared_id = id.clone();
                            }
                        }
                    });
            });
            ui.label(egui::RichText::new("Use the same ID to sync multiple nodes.").weak().small());

            ui.separator();
            render_common_controls(
                ui, opacity, blend_mode, brightness, contrast, saturation, hue_shift,
                scale_x, scale_y, rotation, offset_x, offset_y, flip_horizontal, flip_vertical
            );
        }
        SourceType::Shader { name, params: _ } => {
            ui.label("\u{1F3A8} Shader");
            egui::Grid::new("shader_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(name);
                    ui.end_row();
                });
        }
        SourceType::LiveInput { device_id } => {
            ui.label("\u{1F4F9} Live Input");
            egui::Grid::new("live_input_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Device ID:");
                    ui.add(egui::Slider::new(device_id, 0..=10));
                    ui.end_row();
                });
        }
        #[cfg(feature = "ndi")]
        SourceType::NdiInput { source_name } => {
            ui.label("\u{1F4E1} NDI Input");

            // Smart Empty State
            if source_name.is_none()
                && canvas.ndi_sources.is_empty()
                && canvas.ndi_discovery_rx.is_none()
            {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    if ui.button("🔍 Discover Sources").clicked()
                    {
                        // Start async discovery
                        let (tx, rx) =
                            std::sync::mpsc::channel();
                        canvas.ndi_discovery_rx = Some(rx);
                        mapmap_io::ndi::NdiReceiver::discover_sources_async(tx);
                        canvas.ndi_sources.clear();
                        ui.ctx().request_repaint();
                    }
                    ui.label(egui::RichText::new("No NDI source selected").weak().italics());
                    ui.add_space(10.0);
                });
            } else {
                // Display current source
                let display_name = source_name
                    .clone()
                    .unwrap_or_else(|| {
                        "Not Connected".to_string()
                    });
                ui.label(format!("Current: {}", display_name));

                // Discover button
                ui.horizontal(|ui| {
                    if ui
                        .button("🔍 Discover Sources")
                        .clicked()
                    {
                        // Start async discovery
                        let (tx, rx) =
                            std::sync::mpsc::channel();
                        canvas.ndi_discovery_rx = Some(rx);
                        mapmap_io::ndi::NdiReceiver::discover_sources_async(tx);
                        canvas.ndi_sources.clear();
                        ui.ctx().request_repaint();
                    }

                    // Check for discovery results
                    if let Some(rx) = &canvas.ndi_discovery_rx {
                        if let Ok(sources) = rx.try_recv() {
                            canvas.ndi_sources = sources;
                            canvas.ndi_discovery_rx = None;
                        }
                    }

                    // Show spinner if discovering
                    if canvas.ndi_discovery_rx.is_some() {
                        ui.spinner();
                        ui.label("Searching...");
                    }
                });

                // Source selection dropdown
                if !canvas.ndi_sources.is_empty() {
                    ui.separator();
                    ui.label("Available Sources:");

                    egui::ComboBox::from_id_salt(
                        "ndi_source_select",
                    )
                    .selected_text(display_name.clone())
                    .show_ui(ui, |ui| {
                        // Option to disconnect
                        if ui
                            .selectable_label(
                                source_name.is_none(),
                                "❌ None (Disconnect)",
                            )
                            .clicked()
                        {
                            *source_name = None;
                            actions.push(UIAction::DisconnectNdiSource { part_id });
                        }

                        // Available sources
                        for ndi_source in &canvas.ndi_sources {
                            let selected = source_name.as_ref()
                                == Some(&ndi_source.name);
                            if ui
                                .selectable_label(
                                    selected,
                                    &ndi_source.name,
                                )
                                .clicked()
                            {
                                *source_name = Some(
                                    ndi_source.name.clone(),
                                );

                                // Trigger connection action
                                actions.push(UIAction::ConnectNdiSource {
                                    part_id,
                                    source: ndi_source.clone(),
                                });
                            }
                        }
                    });

                    ui.label(format!(
                        "Found {} source(s)",
                        canvas.ndi_sources.len()
                    ));
                } else if canvas.ndi_discovery_rx.is_none() {
                    ui.label(
                        "Click 'Discover' to find NDI sources",
                    );
                }
            }
        }
        #[cfg(not(feature = "ndi"))]
        SourceType::NdiInput { .. } => {
            ui.label("\u{1F4E1} NDI Input (Feature Disabled)");
        }
        #[cfg(target_os = "windows")]
        SourceType::SpoutInput { sender_name } => {
            ui.label("\u{1F6B0} Spout Input");
            ui.horizontal(|ui| {
                ui.label("Sender:");
                ui.text_edit_singleline(sender_name);
            });
        }
        SourceType::Bevy3DText {
            text,
            font_size,
            color,
            position,
            rotation,
            alignment,
        } => {
            ui.label("📝 3D Text");
            ui.add(
                egui::TextEdit::multiline(text)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );

            ui.horizontal(|ui| {
                ui.label("Size:");
                ui.add(egui::Slider::new(font_size, 1.0..=200.0));
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                ui.color_edit_button_rgba_unmultiplied(color);
            });

            ui.horizontal(|ui| {
                ui.label("Align:");
                egui::ComboBox::from_id_salt("text_align")
                    .selected_text(alignment.as_str())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            alignment,
                            "Left".to_string(),
                            "Left",
                        );
                        ui.selectable_value(
                            alignment,
                            "Center".to_string(),
                            "Center",
                        );
                        ui.selectable_value(
                            alignment,
                            "Right".to_string(),
                            "Right",
                        );
                        ui.selectable_value(
                            alignment,
                            "Justify".to_string(),
                            "Justify",
                        );
                    });
            });

            ui.separator();
            ui.label("📐 Transform 3D");

            ui.horizontal(|ui| {
                ui.label("Pos:");
                ui.add(egui::DragValue::new(&mut position[0]).prefix("X:"));
                ui.add(egui::DragValue::new(&mut position[1]).prefix("Y:"));
                ui.add(egui::DragValue::new(&mut position[2]).prefix("Z:"));
            });

            ui.horizontal(|ui| {
                ui.label("Rot:");
                ui.add(
                    egui::DragValue::new(&mut rotation[0])
                        .prefix("X:")
                        .suffix("°"),
                );
                ui.add(
                    egui::DragValue::new(&mut rotation[1])
                        .prefix("Y:")
                        .suffix("°"),
                );
                ui.add(
                    egui::DragValue::new(&mut rotation[2])
                        .prefix("Z:")
                        .suffix("°"),
                );
            });
        }
        SourceType::BevyCamera { mode, fov, active } => {
            ui.label("\u{1F3A5} Bevy Camera");
            ui.checkbox(active, "Active Control");
            ui.add(egui::Slider::new(fov, 10.0..=120.0).text("FOV"));

            ui.separator();
            ui.label("Mode:");

            egui::ComboBox::from_id_salt("camera_mode")
                .selected_text(match mode {
                    BevyCameraMode::Orbit { .. } => "Orbit",
                    BevyCameraMode::Fly { .. } => "Fly",
                    BevyCameraMode::Static { .. } => "Static",
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            matches!(mode, BevyCameraMode::Orbit { .. }),
                            "Orbit",
                        )
                        .clicked()
                    {
                        *mode = BevyCameraMode::default(); // Default is Orbit
                    }
                    if ui
                        .selectable_label(
                            matches!(mode, BevyCameraMode::Fly { .. }),
                            "Fly",
                        )
                        .clicked()
                    {
                        *mode = BevyCameraMode::Fly {
                            speed: 5.0,
                            sensitivity: 1.0,
                        };
                    }
                    if ui
                        .selectable_label(
                            matches!(mode, BevyCameraMode::Static { .. }),
                            "Static",
                        )
                        .clicked()
                    {
                        *mode = BevyCameraMode::Static {
                            position: [0.0, 5.0, 10.0],
                            look_at: [0.0, 0.0, 0.0],
                        };
                    }
                });

            ui.separator();
            match mode {
                BevyCameraMode::Orbit {
                    radius,
                    speed,
                    target,
                    height,
                } => {
                    ui.label("Orbit Settings");
                    ui.add(egui::Slider::new(radius, 1.0..=50.0).text("Radius"));
                    ui.add(egui::Slider::new(speed, -90.0..=90.0).text("Speed (°/s)"));
                    ui.add(egui::Slider::new(height, -10.0..=20.0).text("Height"));

                    ui.label("Target:");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut target[0]).prefix("X:").speed(0.1));
                        ui.add(egui::DragValue::new(&mut target[1]).prefix("Y:").speed(0.1));
                        ui.add(egui::DragValue::new(&mut target[2]).prefix("Z:").speed(0.1));
                    });
                }
                BevyCameraMode::Fly {
                    speed,
                    sensitivity: _,
                } => {
                    ui.label("Fly Settings");
                    ui.add(egui::Slider::new(speed, 0.0..=50.0).text("Speed"));
                    ui.label("Direction: Forward (Z-)");
                }
                BevyCameraMode::Static { position, look_at } => {
                    ui.label("Static Settings");
                    ui.label("Position:");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut position[0]).prefix("X:").speed(0.1));
                        ui.add(egui::DragValue::new(&mut position[1]).prefix("Y:").speed(0.1));
                        ui.add(egui::DragValue::new(&mut position[2]).prefix("Z:").speed(0.1));
                    });
                    ui.label("Look At:");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut look_at[0]).prefix("X:").speed(0.1));
                        ui.add(egui::DragValue::new(&mut look_at[1]).prefix("Y:").speed(0.1));
                        ui.add(egui::DragValue::new(&mut look_at[2]).prefix("Z:").speed(0.1));
                    });
                }
            }
        }
        SourceType::BevyAtmosphere {
            turbidity,
            rayleigh,
            mie_coeff,
            mie_directional_g,
            sun_position,
            exposure,
        } => {
            ui.label("☁️ Atmosphere Settings");
            ui.separator();

            ui.add(egui::Slider::new(turbidity, 0.0..=10.0).text("Turbidity"));
            ui.add(egui::Slider::new(rayleigh, 0.0..=10.0).text("Rayleigh"));
            ui.add(egui::Slider::new(mie_coeff, 0.0..=0.1).text("Mie Coeff"));
            ui.add(egui::Slider::new(mie_directional_g, 0.0..=1.0).text("Mie Dir G"));
            ui.add(egui::Slider::new(exposure, 0.0..=10.0).text("Exposure"));

            ui.label("Sun Position (Azimuth, Elevation):");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut sun_position.0).prefix("Az:").speed(0.1));
                ui.add(egui::DragValue::new(&mut sun_position.1).prefix("El:").speed(0.1));
            });
        }
        SourceType::BevyHexGrid {
            radius,
            rings,
            pointy_top,
            spacing,
            position,
            rotation,
            scale,
        } => {
            ui.label("\u{2B22} Hex Grid Settings");
            ui.separator();

            ui.add(egui::DragValue::new(radius).prefix("Radius:").speed(0.1));
            ui.add(egui::DragValue::new(rings).prefix("Rings:"));
            ui.add(egui::DragValue::new(spacing).prefix("Spacing:").speed(0.1));
            ui.checkbox(pointy_top, "Pointy Top");

            ui.label("Position:");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut position[0]).prefix("X:").speed(0.1));
                ui.add(egui::DragValue::new(&mut position[1]).prefix("Y:").speed(0.1));
                ui.add(egui::DragValue::new(&mut position[2]).prefix("Z:").speed(0.1));
            });

            ui.label("Rotation:");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut rotation[0]).prefix("X:").speed(1.0));
                ui.add(egui::DragValue::new(&mut rotation[1]).prefix("Y:").speed(1.0));
                ui.add(egui::DragValue::new(&mut rotation[2]).prefix("Z:").speed(1.0));
            });

            ui.add(egui::DragValue::new(scale).prefix("Scale:").speed(0.1));
        }
        SourceType::BevyParticles {
            rate,
            lifetime,
            speed,
            color_start,
            color_end,
            position,
            rotation,
        } => {
            ui.label("\u{2728} Particle System Settings");
            ui.separator();

            ui.add(egui::DragValue::new(rate).prefix("Rate:").speed(1.0));
            ui.add(egui::DragValue::new(lifetime).prefix("Lifetime:").speed(0.1));
            ui.add(egui::DragValue::new(speed).prefix("Speed:").speed(0.1));

            ui.horizontal(|ui| {
                ui.label("Start Color:");
                ui.color_edit_button_rgba_unmultiplied(color_start);
            });
            ui.horizontal(|ui| {
                ui.label("End Color:");
                ui.color_edit_button_rgba_unmultiplied(color_end);
            });

            ui.label("Position:");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut position[0]).prefix("X:").speed(0.1));
                ui.add(egui::DragValue::new(&mut position[1]).prefix("Y:").speed(0.1));
                ui.add(egui::DragValue::new(&mut position[2]).prefix("Z:").speed(0.1));
            });

            ui.label("Rotation:");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut rotation[0]).prefix("X:").speed(1.0));
                ui.add(egui::DragValue::new(&mut rotation[1]).prefix("Y:").speed(1.0));
                ui.add(egui::DragValue::new(&mut rotation[2]).prefix("Z:").speed(1.0));
            });
        }
        SourceType::Bevy3DShape {
            shape_type,
            position,
            rotation,
            scale,
            color,
            unlit,
            outline_width,
            outline_color,
            ..
        } => {
            ui.label("\u{1F9CA} Bevy 3D Shape");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Shape:");
                egui::ComboBox::from_id_salt("shape_type_select")
                    .selected_text(format!("{:?}", shape_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(shape_type, mapmap_core::module::BevyShapeType::Cube, "Cube");
                        ui.selectable_value(shape_type, mapmap_core::module::BevyShapeType::Sphere, "Sphere");
                        ui.selectable_value(shape_type, mapmap_core::module::BevyShapeType::Capsule, "Capsule");
                        ui.selectable_value(shape_type, mapmap_core::module::BevyShapeType::Torus, "Torus");
                        ui.selectable_value(shape_type, mapmap_core::module::BevyShapeType::Cylinder, "Cylinder");
                        ui.selectable_value(shape_type, mapmap_core::module::BevyShapeType::Plane, "Plane");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                ui.color_edit_button_rgba_unmultiplied(color);
            });

            ui.checkbox(unlit, "Unlit (No Shading)");

            ui.separator();

            ui.collapsing("📐 Transform (3D)", |ui| {
                ui.label("Position:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut position[0]).speed(0.1).prefix("X: "));
                    ui.add(egui::DragValue::new(&mut position[1]).speed(0.1).prefix("Y: "));
                    ui.add(egui::DragValue::new(&mut position[2]).speed(0.1).prefix("Z: "));
                });

                ui.label("Rotation:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut rotation[0]).speed(1.0).prefix("X: ").suffix("°"));
                    ui.add(egui::DragValue::new(&mut rotation[1]).speed(1.0).prefix("Y: ").suffix("°"));
                    ui.add(egui::DragValue::new(&mut rotation[2]).speed(1.0).prefix("Z: ").suffix("°"));
                });

                ui.label("Scale:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut scale[0]).speed(0.01).prefix("X: "));
                    ui.add(egui::DragValue::new(&mut scale[1]).speed(0.01).prefix("Y: "));
                    ui.add(egui::DragValue::new(&mut scale[2]).speed(0.01).prefix("Z: "));
                });
            });

            ui.separator();
            ui.collapsing("Outline", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(egui::Slider::new(outline_width, 0.0..=10.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.color_edit_button_rgba_unmultiplied(outline_color);
                });
            });
        }
        SourceType::Bevy3DModel { .. } => {
            ui.label("\u{1F3AE} Bevy 3D Model");
            ui.label("Model controls not yet implemented.");
        }
        SourceType::Bevy => {
            ui.label("\u{1F3AE} Bevy Scene");
            ui.label(egui::RichText::new("Rendering Internal 3D Scene").weak().italics());
            ui.small("The scene is rendered internally and available as 'bevy_output'");
        }
    }
}
