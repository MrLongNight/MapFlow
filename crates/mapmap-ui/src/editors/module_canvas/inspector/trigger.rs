use crate::widgets::styled_slider;
use egui::Ui;
use mapmap_core::module::{ModulePart, ModulePartId, TriggerMappingMode, TriggerTarget, TriggerType};
use super::super::state::ModuleCanvas;

/// Renders the trigger configuration UI for mapping module inputs.
pub fn render_trigger_config_ui(canvas: &mut ModuleCanvas, ui: &mut Ui, part: &mut ModulePart) {
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
            for (idx, _socket) in part.inputs.iter().enumerate() {
                ui.push_id(idx, |ui| {
                    ui.separator();
                    ui.label(format!("Input {}", idx));

                    // Get config
                    let mut config = part.trigger_targets.entry(idx).or_default().clone();
                    let original_config = config.clone();

                    // Target Selector
                    egui::ComboBox::from_id_salt("target")
                        .selected_text(format!("{:?}", config.target))
                        .show_ui(ui, |ui| {
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

/// Renders the configuration UI for a `ModulePartType::Trigger`.
pub fn render_trigger_ui(canvas: &mut ModuleCanvas, ui: &mut Ui, trigger: &mut TriggerType, part_id: ModulePartId) {
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
            ui.checkbox(&mut output_config.bpm_output, "⏱️ BPM");
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
                    toggle_invert(ui, "BPM Out", "⏱️ BPM Out");
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
            ui.label("⏱️ Fixed Timer");
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
                            ui.label(egui::RichText::new("No MIDI devices").weak().italics());
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
