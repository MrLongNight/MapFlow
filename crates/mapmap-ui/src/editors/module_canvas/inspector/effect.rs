use egui::{Color32, Ui};
use mapmap_core::module::{BlendModeType, EffectType, ModulizerType, ModulePartId};

/// Sets default parameters for a given effect type.
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

/// Renders the configuration UI for a `ModulePartType::Modulizer`.
pub fn render_effect_ui(ui: &mut Ui, mod_type: &mut ModulizerType, part_id: ModulePartId) {
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
