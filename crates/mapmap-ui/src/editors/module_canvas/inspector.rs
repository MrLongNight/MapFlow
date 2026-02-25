use super::mesh;
use super::state::ModuleCanvas;
use super::types::MediaPlaybackCommand;
use crate::theme::colors;
use crate::widgets::{styled_drag_value, styled_slider};
use crate::UIAction;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::module::{
    BevyCameraMode, BlendModeType, EffectType, HueMappingMode, LayerType, MaskShape, MaskType,
    ModuleId, ModulePart, ModulePartId, ModulePartType, ModulizerType, OutputType, SourceType,
    TriggerMappingMode, TriggerTarget, TriggerType,
};

/// Sets default parameters for a given effect type
pub fn set_default_effect_params(
    effect_type: EffectType,
    params: &mut std::collections::HashMap<String, f32>,
) {
    params.clear();
    match effect_type {
        EffectType::Blur => {
            params.insert("radius".to_string(), 5.0);
            params.insert("samples".to_string(), 9.0);
        }
        EffectType::Pixelate => {
            params.insert("pixel_size".to_string(), 8.0);
        }
        EffectType::FilmGrain => {
            params.insert("amount".to_string(), 0.1);
            params.insert("speed".to_string(), 1.0);
        }
        EffectType::Vignette => {
            params.insert("radius".to_string(), 0.5);
            params.insert("softness".to_string(), 0.5);
        }
        EffectType::ChromaticAberration => {
            params.insert("amount".to_string(), 0.01);
        }
        EffectType::EdgeDetect => {
            // Usually no params, or threshold?
        }
        EffectType::Brightness | EffectType::Contrast | EffectType::Saturation => {
            params.insert("brightness".to_string(), 0.0);
            params.insert("contrast".to_string(), 1.0);
            params.insert("saturation".to_string(), 1.0);
        }
        _ => {}
    }
}

pub fn render_inspector_for_part(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    part: &mut ModulePart,
    actions: &mut Vec<UIAction>,
    module_id: ModuleId,
    shared_media_ids: &[String],
) {
    // Sync mesh editor state if needed
    mesh::sync_mesh_editor_to_current_selection(canvas, part);

    let part_id = part.id;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // --- Input Configuration ---
            render_trigger_config_ui(canvas, ui, part);
            ui.separator();

            match &mut part.part_type {
                ModulePartType::Trigger(trigger) => {
                    ui.label("Trigger Type:");
                    match trigger {
                        TriggerType::Beat => {
                            ui.label("🥁 Beat Sync");
                            ui.label("Triggers on BPM beat.");
                        }
                        TriggerType::AudioFFT { band: _band, threshold, output_config } => {
                            ui.label("\u{1F50A} Audio FFT");
                            ui.label("Outputs 9 frequency bands, plus volume and beat.");
                            ui.add(
                                egui::Slider::new(threshold, 0.0..=1.0)
                                    .text("Threshold"),
                            );

                            ui.separator();
                            ui.label("\u{1F4E4} Output Configuration:");
                            ui.checkbox(&mut output_config.beat_output, "🥁 Beat Detection");
                            ui.checkbox(&mut output_config.bpm_output, "⏱️ï¸  BPM");
                            ui.checkbox(&mut output_config.volume_outputs, "\u{1F4CA} Volume (RMS, Peak)");
                            ui.checkbox(&mut output_config.frequency_bands, "\u{1F3B5} Frequency Bands (9)");

                            ui.separator();
                            ui.collapsing("\u{1F504} Invert Signals (NOT Logic)", |ui| {
                                ui.label("Select signals to invert (Active = 0.0):");

                                let mut toggle_invert = |ui: &mut Ui, name: &str, label: &str| {
                                    let name_string = name.to_string();
                                    let mut invert = output_config.inverted_outputs.contains(&name_string);
                                    if ui.checkbox(&mut invert, label).changed() {
                                        if invert {
                                            output_config.inverted_outputs.insert(name_string);
                                        } else {
                                            output_config.inverted_outputs.remove(&name_string);
                                        }
                                    }
                                };

                                if output_config.beat_output {
                                    toggle_invert(ui, "Beat Out", "🥁 Beat Out");
                                }
                                if output_config.bpm_output {
                                    toggle_invert(ui, "BPM Out", "⏱️ï¸  BPM Out");
                                }
                                if output_config.volume_outputs {
                                    toggle_invert(ui, "RMS Volume", "\u{1F4CA} RMS Volume");
                                    toggle_invert(ui, "Peak Volume", "\u{1F4CA} Peak Volume");
                                }
                                if output_config.frequency_bands {
                                    ui.label("Bands:");
                                    toggle_invert(ui, "SubBass Out", "SubBass (20-60Hz)");
                                    toggle_invert(ui, "Bass Out", "Bass (60-250Hz)");
                                    toggle_invert(ui, "LowMid Out", "LowMid (250-500Hz)");
                                    toggle_invert(ui, "Mid Out", "Mid (500-1kHz)");
                                    toggle_invert(ui, "HighMid Out", "HighMid (1-2kHz)");
                                    toggle_invert(ui, "UpperMid Out", "UpperMid (2-4kHz)");
                                    toggle_invert(ui, "Presence Out", "Presence (4-6kHz)");
                                    toggle_invert(ui, "Brilliance Out", "Brilliance (6-12kHz)");
                                    toggle_invert(ui, "Air Out", "Air (12-20kHz)");
                                }
                            });

                            ui.label(
                                "Threshold is used for the node's visual glow effect.",
                            );
                        }
                        TriggerType::Random {
                            min_interval_ms,
                            max_interval_ms,
                            probability,
                        } => {
                            ui.label("\u{1F3B2} Random");
                            ui.add(
                                egui::Slider::new(min_interval_ms, 50..=5000)
                                    .text("Min (ms)"),
                            );
                            ui.add(
                                egui::Slider::new(max_interval_ms, 100..=10000)
                                    .text("Max (ms)"),
                            );
                            ui.add(
                                egui::Slider::new(probability, 0.0..=1.0)
                                    .text("Probability"),
                            );
                        }
                        TriggerType::Fixed {
                            interval_ms,
                            offset_ms,
                            ..
                        } => {
                            ui.label("⏱️ï¸  Fixed Timer");
                            ui.add(
                                egui::Slider::new(interval_ms, 16..=10000)
                                    .text("Interval (ms)"),
                            );
                            ui.add(
                                egui::Slider::new(offset_ms, 0..=5000)
                                    .text("Offset (ms)"),
                            );
                        }
                        TriggerType::Midi { channel, note, device: _ } => {
                            ui.label("\u{1F3B9} MIDI Trigger");

                            // Available MIDI ports dropdown
                            ui.horizontal(|ui| {
                                ui.label("Device:");
                                #[cfg(feature = "midi")]
                                {
                                    if let Ok(ports) =
                                        mapmap_control::midi::MidiInputHandler::list_ports()
                                    {
                                        if ports.is_empty() {
                                            ui.label("No MIDI devices");
                                        } else {
                                            egui::ComboBox::from_id_salt(
                                                "midi_device",
                                            )
                                            .selected_text(
                                                ports.first().cloned().unwrap_or_default(),
                                            )
                                            .show_ui(ui, |ui| {
                                                for port in &ports {
                                                    let _ = ui.selectable_label(false, port);
                                                }
                                            });
                                        }
                                    } else {
                                        ui.label("MIDI unavailable");
                                    }
                                }
                                #[cfg(not(feature = "midi"))]
                                {
                                    ui.label("(MIDI disabled)");
                                }
                            });

                            ui.add(
                                egui::Slider::new(channel, 1..=16)
                                    .text("Channel"),
                            );
                            ui.add(
                                egui::Slider::new(note, 0..=127).text("Note"),
                            );

                            // MIDI Learn button
                            let is_learning =
                                canvas.midi_learn_part_id == Some(part_id);
                            let learn_text = if is_learning {
                                "â ³ Waiting for MIDI..."
                            } else {
                                "🎯 MIDI Learn"
                            };
                            if ui.button(learn_text).clicked() {
                                if is_learning {
                                    canvas.midi_learn_part_id = None;
                                } else {
                                    canvas.midi_learn_part_id = Some(part_id);
                                }
                            }
                            if is_learning {
                                ui.label("Press any MIDI key/knob...");
                            }
                        }
                        TriggerType::Osc { address } => {
                            ui.label("\u{1F4E1} OSC Trigger");
                            ui.horizontal(|ui| {
                                ui.label("Address:");
                                ui.add(
                                    egui::TextEdit::singleline(address)
                                        .desired_width(150.0),
                                );
                            });
                            ui.label("Format: /path/to/trigger");
                            ui.label("Default port: 8000");
                        }
                        TriggerType::Shortcut {
                            key_code,
                            modifiers,
                        } => {
                            ui.label("âŒ¨ï¸  Shortcut");
                            ui.horizontal(|ui| {
                                ui.label("Key:");
                                ui.text_edit_singleline(key_code);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Mods:");
                                ui.label(format!(
                                    "Ctrl={} Shift={} Alt={}",
                                    *modifiers & 1 != 0,
                                    *modifiers & 2 != 0,
                                    *modifiers & 4 != 0
                                ));
                            });
                        }
                    }
                }
                ModulePartType::Source(source) => {
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
                            SourceType::BevyAtmosphere { .. } => "â˜ ï¸  Atmosphere",
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
                                    if ui.add(egui::Button::new("\u{1F4C2} Select Media File").min_size(egui::vec2(150.0, 30.0))).clicked() {
                                        actions.push(UIAction::PickMediaFile(module_id, part_id, "".to_string()));
                                    }
                                    ui.label(egui::RichText::new("No media loaded").weak());
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
                                if ui.checkbox(&mut reverse, "â ª Reverse Playback").changed() {
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
                                    if ui.add(egui::Button::new("\u{1F4C2} Select Image File").min_size(egui::vec2(150.0, 30.0))).clicked() {
                                        actions.push(crate::UIAction::PickMediaFile(module_id, part_id, "".to_string()));
                                    }
                                    ui.label(egui::RichText::new("No image loaded").weak());
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
                        }                                            SourceType::Shader { name, params: _ } => {
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
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                "🔍 Discover Sources",
                                            )
                                            .min_size(egui::vec2(150.0, 30.0)),
                                        )
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
                                    ui.label(
                                        egui::RichText::new(
                                            "No NDI source selected",
                                        )
                                        .weak(),
                                    );
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
                                                "â Œ None (Disconnect)",
                                            )
                                            .clicked()
                                        {
                                            *source_name = None;
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
                                                canvas.pending_ndi_connect =
                                                    Some((
                                                        part_id,
                                                        ndi_source.clone(),
                                                    ));
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
                                        .suffix("Â°"),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut rotation[1])
                                        .prefix("Y:")
                                        .suffix("Â°"),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut rotation[2])
                                        .prefix("Z:")
                                        .suffix("Â°"),
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
                                    ui.add(egui::Slider::new(speed, -90.0..=90.0).text("Speed (Â°/s)"));
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
                        SourceType::BevyAtmosphere { .. }
                        | SourceType::BevyHexGrid { .. }
                        | SourceType::BevyParticles { .. } => {
                            ui.label("Controls for this Bevy node are not yet implemented in UI.");
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
                                    ui.add(egui::DragValue::new(&mut rotation[0]).speed(1.0).prefix("X: ").suffix("Â°"));
                                    ui.add(egui::DragValue::new(&mut rotation[1]).speed(1.0).prefix("Y: ").suffix("Â°"));
                                    ui.add(egui::DragValue::new(&mut rotation[2]).speed(1.0).prefix("Z: ").suffix("Â°"));
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
                            ui.label(egui::RichText::new("Rendering Internal 3D Scene").weak());
                            ui.small("The scene is rendered internally and available as 'bevy_output'");
                        }

                    }
                }
                ModulePartType::Mask(mask) => {
                    ui.label("Mask Type:");
                    match mask {
                        MaskType::File { path } => {
                            ui.label("📁 Mask File");
                            if path.is_empty() {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(10.0);
                                    if ui.add(egui::Button::new("\u{1F4C2} Select Mask File")
                                        .min_size(egui::vec2(150.0, 30.0)))
                                        .clicked()
                                    {
                                        if let Some(picked) = rfd::FileDialog::new()
                                            .add_filter(
                                                "Image",
                                                &[
                                                    "png", "jpg", "jpeg", "webp",
                                                    "bmp",
                                                ],
                                            )
                                            .pick_file()
                                        {
                                            *path = picked.display().to_string();
                                        }
                                    }
                                    ui.label(egui::RichText::new("No mask loaded").weak());
                                    ui.add_space(10.0);
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(path)
                                            .desired_width(120.0),
                                    );
                                    if ui.button("\u{1F4C2}").on_hover_text("Select Mask File").clicked() {
                                        if let Some(picked) = rfd::FileDialog::new()
                                            .add_filter(
                                                "Image",
                                                &[
                                                    "png", "jpg", "jpeg", "webp",
                                                    "bmp",
                                                ],
                                            )
                                            .pick_file()
                                        {
                                            *path = picked.display().to_string();
                                        }
                                    }
                                });
                            }
                        }
                        MaskType::Shape(shape) => {
                            ui.label("\u{1F537} Shape Mask");
                            egui::ComboBox::from_id_salt("mask_shape")
                                .selected_text(format!("{:?}", shape))
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_label(
                                            matches!(shape, MaskShape::Circle),
                                            "Circle",
                                        )
                                        .clicked()
                                    {
                                        *shape = MaskShape::Circle;
                                    }
                                    if ui
                                        .selectable_label(
                                            matches!(
                                                shape,
                                                MaskShape::Rectangle
                                            ),
                                            "Rectangle",
                                        )
                                        .clicked()
                                    {
                                        *shape = MaskShape::Rectangle;
                                    }
                                    if ui
                                        .selectable_label(
                                            matches!(
                                                shape,
                                                MaskShape::Triangle
                                            ),
                                            "Triangle",
                                        )
                                        .clicked()
                                    {
                                        *shape = MaskShape::Triangle;
                                    }
                                    if ui
                                        .selectable_label(
                                            matches!(shape, MaskShape::Star),
                                            "Star",
                                        )
                                        .clicked()
                                    {
                                        *shape = MaskShape::Star;
                                    }
                                    if ui
                                        .selectable_label(
                                            matches!(shape, MaskShape::Ellipse),
                                            "Ellipse",
                                        )
                                        .clicked()
                                    {
                                        *shape = MaskShape::Ellipse;
                                    }
                                });
                        }
                        MaskType::Gradient { angle, softness } => {
                            ui.label("\u{1F308} Gradient Mask");
                            ui.add(
                                egui::Slider::new(angle, 0.0..=360.0)
                                    .text("Angle Â°"),
                            );
                            ui.add(
                                egui::Slider::new(softness, 0.0..=1.0)
                                    .text("Softness"),
                            );
                        }
                    }
                }
                ModulePartType::Modulizer(mod_type) => {
                    ui.label("Modulator:");
                    match mod_type {
                        ModulizerType::Effect { effect_type: effect, params } => {
                            // === LIVE HEADER ===
                            ui.add_space(5.0);

                            // 1. Big Title
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new(effect.name())
                                        .size(22.0)
                                        .color(Color32::from_rgb(100, 200, 255))
                                        .strong(),
                                );
                            });
                            ui.add_space(10.0);

                            // 2. Safe Reset Button (Prominent)
                            ui.vertical_centered(|ui| {
                                if crate::widgets::hold_to_action_button(
                                    ui,
                                    "\u{27F2} Safe Reset",
                                    Color32::from_rgb(255, 180, 0),
                                ) {
                                    set_default_effect_params(
                                        *effect, params,
                                    );
                                }
                            });

                            ui.add_space(10.0);
                            ui.separator();

                            let mut changed_type = None;

                            egui::ComboBox::from_id_salt(format!("{}_effect", part_id))
                                .selected_text(effect.name())
                                .show_ui(ui, |ui| {
                                    ui.label("--- Basic ---");
                                    if ui.selectable_label(matches!(effect, EffectType::Blur), "Blur").clicked() { changed_type = Some(EffectType::Blur); }
                                    if ui.selectable_label(matches!(effect, EffectType::Invert), "Invert").clicked() { changed_type = Some(EffectType::Invert); }
                                    if ui.selectable_label(matches!(effect, EffectType::Sharpen), "Sharpen").clicked() { changed_type = Some(EffectType::Sharpen); }
                                    if ui.selectable_label(matches!(effect, EffectType::Threshold), "Threshold").clicked() { changed_type = Some(EffectType::Threshold); }

                                    ui.label("--- Color ---");
                                    if ui.selectable_label(matches!(effect, EffectType::Brightness), "Brightness").clicked() { changed_type = Some(EffectType::Brightness); }
                                    if ui.selectable_label(matches!(effect, EffectType::Contrast), "Contrast").clicked() { changed_type = Some(EffectType::Contrast); }
                                    if ui.selectable_label(matches!(effect, EffectType::Saturation), "Saturation").clicked() { changed_type = Some(EffectType::Saturation); }
                                    if ui.selectable_label(matches!(effect, EffectType::HueShift), "Hue Shift").clicked() { changed_type = Some(EffectType::HueShift); }
                                    if ui.selectable_label(matches!(effect, EffectType::Colorize), "Colorize").clicked() { changed_type = Some(EffectType::Colorize); }

                                    ui.label("--- Distortion ---");
                                    if ui.selectable_label(matches!(effect, EffectType::Wave), "Wave").clicked() { changed_type = Some(EffectType::Wave); }
                                    if ui.selectable_label(matches!(effect, EffectType::Spiral), "Spiral").clicked() { changed_type = Some(EffectType::Spiral); }
                                    if ui.selectable_label(matches!(effect, EffectType::Kaleidoscope), "Kaleidoscope").clicked() { changed_type = Some(EffectType::Kaleidoscope); }

                                    ui.label("--- Stylize ---");
                                    if ui.selectable_label(matches!(effect, EffectType::Pixelate), "Pixelate").clicked() { changed_type = Some(EffectType::Pixelate); }
                                    if ui.selectable_label(matches!(effect, EffectType::EdgeDetect), "Edge Detect").clicked() { changed_type = Some(EffectType::EdgeDetect); }

                                    ui.label("--- Composite ---");
                                    if ui.selectable_label(matches!(effect, EffectType::RgbSplit), "RGB Split").clicked() { changed_type = Some(EffectType::RgbSplit); }
                                    if ui.selectable_label(matches!(effect, EffectType::ChromaticAberration), "Chromatic").clicked() { changed_type = Some(EffectType::ChromaticAberration); }
                                    if ui.selectable_label(matches!(effect, EffectType::FilmGrain), "Film Grain").clicked() { changed_type = Some(EffectType::FilmGrain); }
                                    if ui.selectable_label(matches!(effect, EffectType::Vignette), "Vignette").clicked() { changed_type = Some(EffectType::Vignette); }
                                });

                            if let Some(new_type) = changed_type {
                                *effect = new_type;
                                set_default_effect_params(new_type, params);
                            }

                            ui.separator();
                            match effect {
                                EffectType::Blur => {
                                    let val = params.entry("radius".to_string()).or_insert(5.0);
                                    ui.add(egui::Slider::new(val, 0.0..=50.0).text("Radius"));
                                    let samples = params.entry("samples".to_string()).or_insert(9.0);
                                    ui.add(egui::Slider::new(samples, 1.0..=20.0).text("Samples"));
                                }
                                EffectType::Pixelate => {
                                    let val = params.entry("pixel_size".to_string()).or_insert(8.0);
                                    ui.add(egui::Slider::new(val, 1.0..=100.0).text("Pixel Size"));
                                }
                                EffectType::FilmGrain => {
                                    let amt = params.entry("amount".to_string()).or_insert(0.1);
                                    ui.add(egui::Slider::new(amt, 0.0..=1.0).text("Amount"));
                                    let spd = params.entry("speed".to_string()).or_insert(1.0);
                                    ui.add(egui::Slider::new(spd, 0.0..=5.0).text("Speed"));
                                }
                                EffectType::Vignette => {
                                    let rad = params.entry("radius".to_string()).or_insert(0.5);
                                    ui.add(egui::Slider::new(rad, 0.0..=1.0).text("Radius"));
                                    let soft = params.entry("softness".to_string()).or_insert(0.5);
                                    ui.add(egui::Slider::new(soft, 0.0..=1.0).text("Softness"));
                                }
                                EffectType::ChromaticAberration => {
                                    let amt = params.entry("amount".to_string()).or_insert(0.01);
                                    ui.add(egui::Slider::new(amt, 0.0..=0.1).text("Amount"));
                                }
                                EffectType::Brightness | EffectType::Contrast | EffectType::Saturation => {
                                    let bri = params.entry("brightness".to_string()).or_insert(0.0);
                                    ui.add(egui::Slider::new(bri, -1.0..=1.0).text("Brightness"));
                                    let con = params.entry("contrast".to_string()).or_insert(1.0);
                                    ui.add(egui::Slider::new(con, 0.0..=2.0).text("Contrast"));
                                    let sat = params.entry("saturation".to_string()).or_insert(1.0);
                                    ui.add(egui::Slider::new(sat, 0.0..=2.0).text("Saturation"));
                                }
                                _ => {
                                    ui.label("No configurable parameters");
                                }
                            }
                        }
                        ModulizerType::BlendMode(blend) => {
                            ui.label("\u{1F3A8} Blend Mode");
                            egui::ComboBox::from_id_salt("blend_mode")
                                .selected_text(format!("{:?}", blend))
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(matches!(blend, BlendModeType::Normal), "Normal").clicked() { *blend = BlendModeType::Normal; }
                                    if ui.selectable_label(matches!(blend, BlendModeType::Add), "Add").clicked() { *blend = BlendModeType::Add; }
                                    if ui.selectable_label(matches!(blend, BlendModeType::Multiply), "Multiply").clicked() { *blend = BlendModeType::Multiply; }
                                    if ui.selectable_label(matches!(blend, BlendModeType::Screen), "Screen").clicked() { *blend = BlendModeType::Screen; }
                                    if ui.selectable_label(matches!(blend, BlendModeType::Overlay), "Overlay").clicked() { *blend = BlendModeType::Overlay; }
                                    if ui.selectable_label(matches!(blend, BlendModeType::Difference), "Difference").clicked() { *blend = BlendModeType::Difference; }
                                    if ui.selectable_label(matches!(blend, BlendModeType::Exclusion), "Exclusion").clicked() { *blend = BlendModeType::Exclusion; }
                                });
                            ui.add(
                                egui::Slider::new(&mut 1.0_f32, 0.0..=1.0)
                                    .text("Opacity"),
                            );
                        }
                        ModulizerType::AudioReactive { source } => {
                            ui.label("\u{1F50A} Audio Reactive");
                            ui.horizontal(|ui| {
                                ui.label("Source:");
                                egui::ComboBox::from_id_salt("audio_source")
                                    .selected_text(source.as_str())
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_label(source == "SubBass", "SubBass").clicked() { *source = "SubBass".to_string(); }
                                        if ui.selectable_label(source == "Bass", "Bass").clicked() { *source = "Bass".to_string(); }
                                        if ui.selectable_label(source == "LowMid", "LowMid").clicked() { *source = "LowMid".to_string(); }
                                        if ui.selectable_label(source == "Mid", "Mid").clicked() { *source = "Mid".to_string(); }
                                        if ui.selectable_label(source == "HighMid", "HighMid").clicked() { *source = "HighMid".to_string(); }
                                        if ui.selectable_label(source == "Presence", "Presence").clicked() { *source = "Presence".to_string(); }
                                        if ui.selectable_label(source == "Brilliance", "Brilliance").clicked() { *source = "Brilliance".to_string(); }
                                        if ui.selectable_label(source == "RMS", "RMS Volume").clicked() { *source = "RMS".to_string(); }
                                        if ui.selectable_label(source == "Peak", "Peak").clicked() { *source = "Peak".to_string(); }
                                        if ui.selectable_label(source == "BPM", "BPM").clicked() { *source = "BPM".to_string(); }
                                    });
                            });
                            ui.add(
                                egui::Slider::new(&mut 0.1_f32, 0.0..=1.0)
                                    .text("Smoothing"),
                            );
                        }
                    }
                }
                ModulePartType::Layer(layer) => {
                    ui.label("📋 Layer:");

                    // Helper to render mesh UI
                    let mut render_mesh_ui = |ui: &mut Ui, mesh: &mut mapmap_core::module::MeshType, id_salt: u64| {
                        mesh::render_mesh_editor_ui(canvas, ui, mesh, part_id, id_salt);
                    };

                    match layer {
                        LayerType::Single { id, name, opacity, blend_mode, mesh, mapping_mode } => {
                            ui.label("🔳 Single Layer");
                            ui.horizontal(|ui| { ui.label("ID:"); ui.add(egui::DragValue::new(id)); });
                            ui.text_edit_singleline(name);
                            ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));

                            // Blend mode
                            let blend_text = blend_mode.as_ref().map(|b| format!("{:?}", b)).unwrap_or_else(|| "None".to_string());
                            egui::ComboBox::from_id_salt("layer_blend").selected_text(blend_text).show_ui(ui, |ui| {
                                if ui.selectable_label(blend_mode.is_none(), "None").clicked() { *blend_mode = None; }
                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Normal)), "Normal").clicked() { *blend_mode = Some(BlendModeType::Normal); }
                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Add)), "Add").clicked() { *blend_mode = Some(BlendModeType::Add); }
                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Multiply)), "Multiply").clicked() { *blend_mode = Some(BlendModeType::Multiply); }
                            });

                            ui.checkbox(mapping_mode, "Mapping Mode (Grid)");

                            render_mesh_ui(ui, mesh, *id);
                        }
                        LayerType::Group { name, opacity, mesh, mapping_mode, .. } => {
                            ui.label("📂 Group");
                            ui.text_edit_singleline(name);
                            ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
                            ui.checkbox(mapping_mode, "Mapping Mode (Grid)");
                            render_mesh_ui(ui, mesh, 9999); // Dummy ID
                        }
                        LayerType::All { opacity, .. } => {
                            ui.label("🎚️ Master");
                            ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
                        }
                    }
                }
                ModulePartType::Mesh(mesh) => {
                    ui.label("🕸️ Mesh Node");
                    ui.separator();

                    mesh::render_mesh_editor_ui(canvas, ui, mesh, part_id, part_id);
                }
                ModulePartType::Output(output) => {
                    ui.label("Output:");
                    match output {
                        OutputType::Projector {
                            id,
                            name,
                            hide_cursor,
                            target_screen,
                            show_in_preview_panel,
                            extra_preview_window,
                            ndi_enabled: _ndi_enabled,
                            ndi_stream_name: _ndi_stream_name,
                            ..
                        } => {
                            ui.label("📽️ï¸  Projector Output");

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
                            ui.label("🖥️ï¸  Window Settings:");

                            // Target screen selection
                            ui.horizontal(|ui| {
                                ui.label("Target Screen:");
                                egui::ComboBox::from_id_salt("target_screen_select")
                                    .selected_text(format!("Monitor {}", target_screen))
                                    .show_ui(ui, |ui| {
                                        for i in 0..=3u8 {
                                            let label = if i == 0 { "Primary".to_string() } else { format!("Monitor {}", i) };
                                            if ui.selectable_label(*target_screen == i, &label).clicked() {
                                                *target_screen = i;
                                            }
                                        }
                                    });
                            });

                            ui.checkbox(hide_cursor, "🖱️ï¸  Hide Mouse Cursor");

                            ui.separator();
                            ui.label("👁️ï¸  Preview:");
                            ui.checkbox(show_in_preview_panel, "Show in Preview Panel");
                            ui.checkbox(extra_preview_window, "Extra Preview Window");

                            ui.separator();
                            ui.label("\u{1F4E1} NDI Broadcast");
                            #[cfg(feature = "ndi")]
                            {
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
                            }
                            #[cfg(not(feature = "ndi"))]
                            {
                                ui.label("NDI feature disabled in build");
                            }
                        }
                        #[cfg(feature = "ndi")]
                        OutputType::NdiOutput { name } => {
                            ui.label("\u{1F4E1} NDI Output");
                            ui.horizontal(|ui| {
                                ui.label("Stream Name:");
                                ui.text_edit_singleline(name);
                            });
                        }
                        #[cfg(not(feature = "ndi"))]
                        OutputType::NdiOutput { .. } => {
                            ui.label("\u{1F4E1} NDI Output (Feature Disabled)");
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
                            ui.collapsing("âš™ï¸  Setup (Bridge & Pairing)", |ui| {
                                // Discovery status
                                if let Some(msg) = &canvas.hue_status_message {
                                    ui.label(format!("Status: {}", msg));
                                }

                                // Handle discovery results
                                if let Some(rx) = &canvas.hue_discovery_rx {
                                    if let Ok(result) = rx.try_recv() {
                                        canvas.hue_discovery_rx = None;
                                        // Explicit type annotation for the result to help inference
                                        let result: Result<Vec<mapmap_control::hue::api::discovery::DiscoveredBridge>, _> = result;
                                        match result {
                                            Ok(bridges) => {
                                                canvas.hue_bridges = bridges;
                                                canvas.hue_status_message = Some(format!("Found {} bridges", canvas.hue_bridges.len()));
                                            }
                                            Err(e) => {
                                                canvas.hue_status_message = Some(format!("Discovery failed: {}", e));
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
                                if ui.button("\u{1F517} Pair with Bridge").on_hover_text("Press button on Bridge then click this").clicked() {
                                    // TODO: Implement pairing logic
                                    // This requires async call to `register_user`
                                    // Similar pattern to discovery
                                }

                                if !username.is_empty() {
                                    ui.label("\u{2705} Paired");
                                    // ui.label(format!("User: {}", username)); // Keep secret?
                                } else {
                                    ui.label("â Œ Not Paired");
                                }
                            });

                            ui.collapsing("\u{1F3AD} Area & Mode", |ui| {
                                    ui.label("Entertainment Area:");
                                    ui.text_edit_singleline(entertainment_area);
                                    // TODO: Fetch areas from bridge if paired

                                    ui.separator();
                                    ui.label("Mapping Mode:");
                                    ui.radio_value(mapping_mode, HueMappingMode::Ambient, "Ambient (Average Color)");
                                    ui.radio_value(mapping_mode, HueMappingMode::Spatial, "Spatial (2D Map)");
                                    ui.radio_value(mapping_mode, HueMappingMode::Trigger, "Trigger (Strobe/Pulse)");
                            });

                            if *mapping_mode == HueMappingMode::Spatial {
                                ui.collapsing("🗺️ï¸  Spatial Editor", |ui| {
                                    ui.label("Position lamps in the virtual room:");
                                    // Render 2D room editor
                                    mesh::render_hue_spatial_editor(ui, lamp_positions);
                                });
                            }
                        }
                    }
                }
                ModulePartType::Hue(_) => {
                    ui.label("Hue Node Configuration");
                }
            }
        });
}

fn render_hue_bridge_discovery(canvas: &mut ModuleCanvas, ui: &mut Ui, current_ip: &mut String) {
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

fn render_trigger_config_ui(canvas: &mut ModuleCanvas, ui: &mut Ui, part: &mut ModulePart) {
    // Only show for parts with input sockets
    if part.inputs.is_empty() {
        return;
    }

    ui.add_space(5.0);
    egui::CollapsingHeader::new("\u{26A1} Trigger & Automation")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("MIDI Assignment:");
                let is_learning = canvas.midi_learn_part_id == Some(part.id);
                let btn_text = if is_learning {
                    "\u{1F6D1} Stop Learning"
                } else {
                    "\u{1F3B9} MIDI Learn"
                };
                if ui.selectable_label(is_learning, btn_text).clicked() {
                    if is_learning {
                        canvas.midi_learn_part_id = None;
                    } else {
                        canvas.midi_learn_part_id = Some(part.id);
                    }
                }
            });

            ui.separator();

            // Iterate over inputs
            for (idx, socket) in part.inputs.iter().enumerate() {
                ui.push_id(idx, |ui| {
                    ui.separator();
                    ui.label(format!("Input {}: {}", idx, socket.name));

                    // Get config
                    let mut config = part.trigger_targets.entry(idx).or_default().clone();
                    let original_config = config.clone();

                    // Target Selector
                    egui::ComboBox::from_id_salt("target")
                        .selected_text(format!("{:?}", config.target))
                        .show_ui(ui, |ui| {
                            use mapmap_core::module::TriggerTarget;
                            ui.selectable_value(&mut config.target, TriggerTarget::None, "None");
                            ui.selectable_value(
                                &mut config.target,
                                TriggerTarget::Opacity,
                                "Opacity",
                            );
                            ui.selectable_value(
                                &mut config.target,
                                TriggerTarget::Brightness,
                                "Brightness",
                            );
                            ui.selectable_value(
                                &mut config.target,
                                TriggerTarget::Contrast,
                                "Contrast",
                            );
                            ui.selectable_value(
                                &mut config.target,
                                TriggerTarget::Saturation,
                                "Saturation",
                            );
                            ui.selectable_value(
                                &mut config.target,
                                TriggerTarget::HueShift,
                                "Hue Shift",
                            );
                            ui.selectable_value(
                                &mut config.target,
                                TriggerTarget::ScaleX,
                                "Scale X",
                            );
                            ui.selectable_value(
                                &mut config.target,
                                TriggerTarget::ScaleY,
                                "Scale Y",
                            );
                            ui.selectable_value(
                                &mut config.target,
                                TriggerTarget::Rotation,
                                "Rotation",
                            );
                        });

                    // Only show options if target is not None
                    if config.target != TriggerTarget::None {
                        // Mode Selector
                        ui.horizontal(|ui| {
                            ui.label("Mode:");
                            // Helper to display mode name without fields
                            let mode_name = match config.mode {
                                TriggerMappingMode::Direct => "Direct",
                                TriggerMappingMode::Fixed => "Fixed",
                                TriggerMappingMode::RandomInRange => "Random",
                                TriggerMappingMode::Smoothed { .. } => "Smoothed",
                            };

                            egui::ComboBox::from_id_salt("mode")
                                .selected_text(mode_name)
                                .show_ui(ui, |ui| {
                                    use mapmap_core::module::TriggerMappingMode;
                                    ui.selectable_value(
                                        &mut config.mode,
                                        TriggerMappingMode::Direct,
                                        "Direct",
                                    );
                                    ui.selectable_value(
                                        &mut config.mode,
                                        TriggerMappingMode::Fixed,
                                        "Fixed",
                                    );
                                    ui.selectable_value(
                                        &mut config.mode,
                                        TriggerMappingMode::RandomInRange,
                                        "Random",
                                    );
                                    // For smoothed, we preserve existing params if already smoothed, else default
                                    let default_smoothed = TriggerMappingMode::Smoothed {
                                        attack: 0.1,
                                        release: 0.1,
                                    };
                                    ui.selectable_value(
                                        &mut config.mode,
                                        default_smoothed,
                                        "Smoothed",
                                    );
                                });
                        });

                        // Params based on Mode
                        match &mut config.mode {
                            TriggerMappingMode::Fixed => {
                                ui.horizontal(|ui| {
                                    ui.label("Threshold:");
                                    styled_slider(ui, &mut config.threshold, 0.0..=1.0, 0.5);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Off:");
                                    styled_slider(ui, &mut config.min_value, -5.0..=5.0, 0.0);
                                    ui.label("On:");
                                    styled_slider(ui, &mut config.max_value, -5.0..=5.0, 1.0);
                                });
                            }
                            TriggerMappingMode::RandomInRange => {
                                ui.horizontal(|ui| {
                                    ui.label("Range:");
                                    ui.label("Min:");
                                    styled_slider(ui, &mut config.min_value, -5.0..=5.0, 0.0);
                                    ui.label("Max:");
                                    styled_slider(ui, &mut config.max_value, -5.0..=5.0, 1.0);
                                });
                            }
                            TriggerMappingMode::Smoothed { attack, release } => {
                                ui.horizontal(|ui| {
                                    ui.label("Range:");
                                    ui.label("Min:");
                                    styled_slider(ui, &mut config.min_value, -5.0..=5.0, 0.0);
                                    ui.label("Max:");
                                    styled_slider(ui, &mut config.max_value, -5.0..=5.0, 1.0);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Attack:");
                                    styled_slider(ui, attack, 0.0..=2.0, 0.1);
                                    ui.label("s");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Release:");
                                    styled_slider(ui, release, 0.0..=2.0, 0.1);
                                    ui.label("s");
                                });
                            }
                            _ => {
                                // Direct
                                ui.horizontal(|ui| {
                                    ui.label("Range:");
                                    ui.label("Min:");
                                    styled_slider(ui, &mut config.min_value, -5.0..=5.0, 0.0);
                                    ui.label("Max:");
                                    styled_slider(ui, &mut config.max_value, -5.0..=5.0, 1.0);
                                });
                            }
                        }

                        ui.checkbox(&mut config.invert, "Invert Input");
                    }

                    // Save back if changed
                    if config != original_config {
                        part.trigger_targets.insert(idx, config);
                    }
                });
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn render_common_controls(
    ui: &mut Ui,
    opacity: &mut f32,
    blend_mode: &mut Option<BlendModeType>,
    brightness: &mut f32,
    contrast: &mut f32,
    saturation: &mut f32,
    hue_shift: &mut f32,
    scale_x: &mut f32,
    scale_y: &mut f32,
    rotation: &mut f32,
    offset_x: &mut f32,
    offset_y: &mut f32,
    flip_horizontal: &mut bool,
    flip_vertical: &mut bool,
) {
    // === APPEARANCE ===
    ui.collapsing("\u{1F3A8} Appearance", |ui| {
        egui::Grid::new("appearance_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Opacity:");
                styled_slider(ui, opacity, 0.0..=1.0, 1.0);
                ui.end_row();

                ui.label("Blend Mode:");
                egui::ComboBox::from_id_salt("blend_mode_selector")
                    .selected_text(match blend_mode {
                        Some(BlendModeType::Normal) => "Normal",
                        Some(BlendModeType::Add) => "Add",
                        Some(BlendModeType::Multiply) => "Multiply",
                        Some(BlendModeType::Screen) => "Screen",
                        Some(BlendModeType::Overlay) => "Overlay",
                        Some(BlendModeType::Difference) => "Difference",
                        Some(BlendModeType::Exclusion) => "Exclusion",
                        None => "Normal",
                    })
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(blend_mode.is_none(), "Normal")
                            .clicked()
                        {
                            *blend_mode = None;
                        }
                        if ui
                            .selectable_label(*blend_mode == Some(BlendModeType::Add), "Add")
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Add);
                        }
                        if ui
                            .selectable_label(
                                *blend_mode == Some(BlendModeType::Multiply),
                                "Multiply",
                            )
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Multiply);
                        }
                        if ui
                            .selectable_label(*blend_mode == Some(BlendModeType::Screen), "Screen")
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Screen);
                        }
                        if ui
                            .selectable_label(
                                *blend_mode == Some(BlendModeType::Overlay),
                                "Overlay",
                            )
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Overlay);
                        }
                        if ui
                            .selectable_label(
                                *blend_mode == Some(BlendModeType::Difference),
                                "Difference",
                            )
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Difference);
                        }
                        if ui
                            .selectable_label(
                                *blend_mode == Some(BlendModeType::Exclusion),
                                "Exclusion",
                            )
                            .clicked()
                        {
                            *blend_mode = Some(BlendModeType::Exclusion);
                        }
                    });
                ui.end_row();
            });
    });

    // === COLOR CORRECTION ===
    if crate::widgets::collapsing_header_with_reset(ui, "\u{1F308} Color Correction", false, |ui| {
        egui::Grid::new("color_correction_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Brightness:");
                styled_slider(ui, brightness, -1.0..=1.0, 0.0);
                ui.end_row();

                ui.label("Contrast:");
                styled_slider(ui, contrast, 0.0..=2.0, 1.0);
                ui.end_row();

                ui.label("Saturation:");
                styled_slider(ui, saturation, 0.0..=2.0, 1.0);
                ui.end_row();

                ui.label("Hue Shift:");
                styled_slider(ui, hue_shift, -180.0..=180.0, 0.0);
                ui.end_row();
            });
    }) {
        *brightness = 0.0;
        *contrast = 1.0;
        *saturation = 1.0;
        *hue_shift = 0.0;
    }

    // === TRANSFORM ===
    if crate::widgets::collapsing_header_with_reset(ui, "📐 Transform", false, |ui| {
        egui::Grid::new("transform_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Scale:");
                ui.horizontal(|ui| {
                    styled_drag_value(ui, scale_x, 0.01, 0.0..=10.0, 1.0, "X: ", "");
                    styled_drag_value(ui, scale_y, 0.01, 0.0..=10.0, 1.0, "Y: ", "");
                });
                ui.end_row();

                ui.label("Offset:");
                ui.horizontal(|ui| {
                    styled_drag_value(ui, offset_x, 1.0, -2000.0..=2000.0, 0.0, "X: ", "px");
                    styled_drag_value(ui, offset_y, 1.0, -2000.0..=2000.0, 0.0, "Y: ", "px");
                });
                ui.end_row();

                ui.label("Rotation:");
                styled_slider(ui, rotation, -180.0..=180.0, 0.0);
                ui.end_row();

                ui.label("Mirror:");
                ui.horizontal(|ui| {
                    ui.checkbox(flip_horizontal, "X");
                    ui.checkbox(flip_vertical, "Y");
                });
                ui.end_row();
            });
    }) {
        *scale_x = 1.0;
        *scale_y = 1.0;
        *rotation = 0.0;
        *offset_x = 0.0;
        *offset_y = 0.0;
        *flip_horizontal = false;
        *flip_vertical = false;
    }
}

fn render_transport_controls(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    part_id: ModulePartId,
    is_playing: bool,
    current_pos: f32,
    loop_enabled: &mut bool,
    reverse_playback: &mut bool,
) {
    // 2. CONSOLIDATED TRANSPORT BAR (UX Improved)
    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing.x = 8.0;
        let button_height = 42.0;
        let big_btn_size = Vec2::new(70.0, button_height);
        let small_btn_size = Vec2::new(40.0, button_height);

        // PLAY (Primary Action - Green)
        let play_btn = egui::Button::new(egui::RichText::new("\u{25B6}").size(24.0))
            .min_size(big_btn_size)
            .fill(if is_playing {
                Color32::from_rgb(40, 180, 60)
            } else {
                Color32::from_gray(50)
            });
        if ui.add(play_btn).on_hover_text("Play").clicked() {
            canvas
                .pending_playback_commands
                .push((part_id, MediaPlaybackCommand::Play));
        }

        // PAUSE (Secondary Action - Yellow)
        let pause_btn = egui::Button::new(egui::RichText::new("â ¸").size(24.0))
            .min_size(big_btn_size)
            .fill(if !is_playing && current_pos > 0.1 {
                Color32::from_rgb(200, 160, 40)
            } else {
                Color32::from_gray(50)
            });
        if ui.add(pause_btn).on_hover_text("Pause").clicked() {
            canvas
                .pending_playback_commands
                .push((part_id, MediaPlaybackCommand::Pause));
        }

        // Safety Spacer
        ui.add_space(24.0);
        ui.separator();
        ui.add_space(8.0);

        // STOP (Destructive Action - Separated)
        // Mary StyleUX: Use hold-to-confirm for safety
        if crate::widgets::hold_to_action_button(ui, "â ¹", Color32::from_rgb(255, 80, 80)) {
            canvas
                .pending_playback_commands
                .push((part_id, MediaPlaybackCommand::Stop));
        }

        // LOOP
        let loop_color = if *loop_enabled {
            Color32::from_rgb(80, 150, 255)
        } else {
            Color32::from_gray(45)
        };
        if ui
            .add(
                egui::Button::new(egui::RichText::new("🔁").size(18.0))
                    .min_size(small_btn_size)
                    .fill(loop_color),
            )
            .on_hover_text("Toggle Loop")
            .clicked()
        {
            *loop_enabled = !*loop_enabled;
            canvas
                .pending_playback_commands
                .push((part_id, MediaPlaybackCommand::SetLoop(*loop_enabled)));
        }

        // REVERSE
        let rev_color = if *reverse_playback {
            Color32::from_rgb(200, 80, 80)
        } else {
            Color32::from_gray(45)
        };
        if ui
            .add(
                egui::Button::new(egui::RichText::new("â ª").size(18.0))
                    .min_size(small_btn_size)
                    .fill(rev_color),
            )
            .on_hover_text("Toggle Reverse Playback")
            .clicked()
        {
            *reverse_playback = !*reverse_playback;
        }
    });
}

fn render_timeline(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    part_id: ModulePartId,
    video_duration: f32,
    current_pos: f32,
    start_time: &mut f32,
    end_time: &mut f32,
) {
    let (response, painter) = ui.allocate_painter(
        Vec2::new(ui.available_width(), 32.0),
        Sense::click_and_drag(),
    );
    let rect = response.rect;

    // Background (Full Track)
    painter.rect_filled(rect, 0.0, Color32::from_gray(30));
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0 * canvas.zoom, Color32::from_gray(60)),
        egui::StrokeKind::Middle,
    );

    // Data normalization
    let effective_end = if *end_time > 0.0 {
        *end_time
    } else {
        video_duration
    };
    let start_x = rect.min.x + (*start_time / video_duration).clamp(0.0, 1.0) * rect.width();
    let end_x = rect.min.x + (effective_end / video_duration).clamp(0.0, 1.0) * rect.width();

    // Active Region Highlight
    let region_rect =
        Rect::from_min_max(Pos2::new(start_x, rect.min.y), Pos2::new(end_x, rect.max.y));
    painter.rect_filled(
        region_rect,
        0.0,
        Color32::from_rgba_unmultiplied(60, 180, 100, 80),
    );
    painter.rect_stroke(
        region_rect,
        0.0,
        Stroke::new(1.0, Color32::from_rgb(60, 180, 100)),
        egui::StrokeKind::Middle,
    );

    // INTERACTION LOGIC
    let mut handled = false;

    // 1. Handles (Prioritize resizing)
    let handle_width = 8.0;
    let start_handle_rect = Rect::from_center_size(
        Pos2::new(start_x, rect.center().y),
        Vec2::new(handle_width, rect.height()),
    );
    let end_handle_rect = Rect::from_center_size(
        Pos2::new(end_x, rect.center().y),
        Vec2::new(handle_width, rect.height()),
    );

    let start_resp = ui.interact(start_handle_rect, response.id.with("start"), Sense::drag());
    let end_resp = ui.interact(end_handle_rect, response.id.with("end"), Sense::drag());

    if start_resp.hovered() || end_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }

    if start_resp.dragged() {
        let delta_s = (start_resp.drag_delta().x / rect.width()) * video_duration;
        *start_time = (*start_time + delta_s).clamp(0.0, effective_end - 0.1);
        handled = true;
    } else if end_resp.dragged() {
        let delta_s = (end_resp.drag_delta().x / rect.width()) * video_duration;
        let mut new_end = (effective_end + delta_s).clamp(*start_time + 0.1, video_duration);
        // Snap to end (0.0) if close
        if (video_duration - new_end).abs() < 0.1 {
            new_end = 0.0;
        }
        *end_time = new_end;
        handled = true;
    }

    // 2. Body Interaction (Slide or Seek)
    if !handled && response.hovered() {
        if ui.input(|i| i.modifiers.shift)
            && region_rect.contains(response.hover_pos().unwrap_or_default())
        {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    if !handled && response.dragged() {
        if ui.input(|i| i.modifiers.shift) {
            // Slide Region
            let delta_s = (response.drag_delta().x / rect.width()) * video_duration;
            let duration_s = effective_end - *start_time;

            let new_start = (*start_time + delta_s).clamp(0.0, video_duration - duration_s);
            let new_end = new_start + duration_s;

            *start_time = new_start;
            *end_time = if (video_duration - new_end).abs() < 0.1 {
                0.0
            } else {
                new_end
            };
        } else {
            // Seek
            if let Some(pos) = response.interact_pointer_pos() {
                let seek_norm = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                let seek_s = seek_norm * video_duration;
                canvas
                    .pending_playback_commands
                    .push((part_id, MediaPlaybackCommand::Seek(seek_s as f64)));
            }
        }
    }

    // Draw Handles
    painter.rect_filled(start_handle_rect.shrink(2.0), 2.0, Color32::WHITE);
    painter.rect_filled(end_handle_rect.shrink(2.0), 2.0, Color32::WHITE);

    // Draw Playhead
    let cursor_norm = (current_pos / video_duration).clamp(0.0, 1.0);
    let cursor_x = rect.min.x + cursor_norm * rect.width();
    painter.line_segment(
        [
            Pos2::new(cursor_x, rect.min.y),
            Pos2::new(cursor_x, rect.max.y),
        ],
        Stroke::new(2.0, Color32::from_rgb(255, 200, 50)),
    );
    // Playhead triangle top
    let tri_size = 6.0;
    painter.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(cursor_x - tri_size, rect.min.y),
            Pos2::new(cursor_x + tri_size, rect.min.y),
            Pos2::new(cursor_x, rect.min.y + tri_size * 1.5),
        ],
        Color32::from_rgb(255, 200, 50),
        Stroke::NONE,
    ));
}
