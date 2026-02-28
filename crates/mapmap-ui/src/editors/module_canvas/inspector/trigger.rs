use super::state::ModuleCanvas;
use egui::Ui;
use mapmap_core::module::{ModulePart, TriggerType, TriggerTarget, TriggerMappingMode};
use crate::widgets::{styled_slider, styled_drag_value};

/// Renders the trigger configuration UI for a part
pub fn render_trigger_config_ui(canvas: &mut ModuleCanvas, ui: &mut Ui, part: &mut ModulePart) {
    ui.collapsing("⚡ Trigger Inputs", |ui| {
        ui.label("Map incoming triggers to parameters.");
        
        let inputs = part.inputs.clone();
        for (i, socket) in inputs.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", socket.name));
                
                let config = part.trigger_targets.entry(i).or_default();
                
                egui::ComboBox::from_id_salt(format!("trigger_target_{}_{}", part.id, i))
                    .selected_text(format!("{:?}", config.target))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.target, TriggerTarget::None, "None");
                        ui.selectable_value(&mut config.target, TriggerTarget::Opacity, "Opacity");
                        ui.selectable_value(&mut config.target, TriggerTarget::Brightness, "Brightness");
                        ui.selectable_value(&mut config.target, TriggerTarget::Contrast, "Contrast");
                        ui.selectable_value(&mut config.target, TriggerTarget::Saturation, "Saturation");
                        ui.selectable_value(&mut config.target, TriggerTarget::HueShift, "Hue Shift");
                        ui.selectable_value(&mut config.target, TriggerTarget::Rotation, "Rotation");
                    });
            });
            
            if let Some(config) = part.trigger_targets.get_mut(&i) {
                if config.target != TriggerTarget::None {
                    ui.indent(format!("trigger_details_{}", i), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Mode:");
                            egui::ComboBox::from_id_salt(format!("trigger_mode_{}_{}", part.id, i))
                                .selected_text(format!("{:?}", config.mode))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut config.mode, TriggerMappingMode::Direct, "Direct");
                                    ui.selectable_value(&mut config.mode, TriggerMappingMode::Fixed, "Fixed");
                                    ui.selectable_value(&mut config.mode, TriggerMappingMode::RandomInRange, "Random");
                                });
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Range:");
                            ui.add(egui::DragValue::new(&mut config.min_value).speed(0.01).prefix("Min:"));
                            ui.add(egui::DragValue::new(&mut config.max_value).speed(0.01).prefix("Max:"));
                        });
                        
                        ui.checkbox(&mut config.invert, "Invert Signal");
                    });
                }
            }
        }
    });
}

/// Renders the UI for a trigger part
pub fn render_trigger_ui(canvas: &mut ModuleCanvas, ui: &mut Ui, part_id: u64, trigger: &mut TriggerType) {
    ui.label("Trigger Type:");
    match trigger {
        TriggerType::Beat => {
            ui.label("🥁 Beat Sync");
            ui.label("Triggers on BPM beat.");
        }
        TriggerType::AudioFFT { threshold, output_config, .. } => {
            ui.label("🔊 Audio FFT");
            ui.add(egui::Slider::new(threshold, 0.0..=1.0).text("Threshold"));
            
            ui.separator();
            ui.label("📤 Output Configuration:");
            ui.checkbox(&mut output_config.beat_output, "🥁 Beat Detection");
            ui.checkbox(&mut output_config.bpm_output, "⏱️ BPM");
            ui.checkbox(&mut output_config.volume_outputs, "📊 Volume");
            ui.checkbox(&mut output_config.frequency_bands, "🎶 Frequency Bands");
        }
        TriggerType::Random { min_interval_ms, max_interval_ms, probability } => {
            ui.label("🎲 Random");
            ui.add(egui::Slider::new(min_interval_ms, 50..=5000).text("Min (ms)"));
            ui.add(egui::Slider::new(max_interval_ms, 100..=10000).text("Max (ms)"));
            ui.add(egui::Slider::new(probability, 0.0..=1.0).text("Probability"));
        }
        TriggerType::Fixed { interval_ms, offset_ms } => {
            ui.label("⏱️ Fixed Timer");
            ui.add(egui::Slider::new(interval_ms, 16..=10000).text("Interval (ms)"));
            ui.add(egui::Slider::new(offset_ms, 0..=5000).text("Offset (ms)"));
        }
        TriggerType::Midi { channel, note, .. } => {
            ui.label("🎹 MIDI Trigger");
            ui.add(egui::Slider::new(channel, 1..=16).text("Channel"));
            ui.add(egui::Slider::new(note, 0..=127).text("Note"));
            
            let is_learning = canvas.midi_learn_part_id == Some(part_id);
            let learn_text = if is_learning { "⌛ Waiting..." } else { "🎯 MIDI Learn" };
            if ui.button(learn_text).clicked() {
                canvas.midi_learn_part_id = if is_learning { None } else { Some(part_id) };
            }
        }
        TriggerType::Osc { address } => {
            ui.label("📡 OSC Trigger");
            ui.text_edit_singleline(address);
        }
        TriggerType::Shortcut { key_code, .. } => {
            ui.label("⌨ Shortcut");
            ui.text_edit_singleline(key_code);
        }
    }
}
