use egui::Color32;
use vorce_core::module::{
    BevyCameraMode, BlendModeType, EffectType, HueNodeType, LayerType, MaskShape, MaskType,
    ModulePartType, ModuleSocketType, ModulizerType, OutputType, PartType, SourceType, TriggerType,
};

pub fn get_part_style(
    part_type: &ModulePartType,
) -> (Color32, Color32, &'static str, &'static str) {
    match part_type {
        ModulePartType::Trigger(trigger) => {
            let name = match trigger {
                TriggerType::AudioFFT { .. } => "Audio FFT",
                TriggerType::Beat => "Beat",
                TriggerType::Midi { .. } => "MIDI",
                TriggerType::Osc { .. } => "OSC",
                TriggerType::Shortcut { .. } => "Shortcut",
                TriggerType::Random { .. } => "Random",
                TriggerType::Fixed { .. } => "Fixed Timer",
            };
            (
                Color32::from_rgb(60, 50, 70),
                Color32::from_rgb(130, 80, 180),
                "\u{26A1}",
                name,
            )
        }
        ModulePartType::Source(SourceType::BevyAtmosphere { .. }) => (
            Color32::from_rgb(40, 60, 80),
            Color32::from_rgb(100, 180, 220),
            "â˜ ï¸ ",
            "Atmosphere",
        ),
        ModulePartType::Source(SourceType::BevyHexGrid { .. }) => (
            Color32::from_rgb(40, 60, 80),
            Color32::from_rgb(100, 180, 220),
            "\u{1F6D1}",
            "Hex Grid",
        ),
        ModulePartType::Source(SourceType::BevyParticles { .. }) => (
            Color32::from_rgb(40, 60, 80),
            Color32::from_rgb(100, 180, 220),
            "\u{2728}",
            "Particles",
        ),
        ModulePartType::Source(SourceType::Bevy3DText { .. }) => (
            Color32::from_rgb(40, 60, 80),
            Color32::from_rgb(100, 220, 180),
            "T",
            "3D Text",
        ),
        ModulePartType::Source(SourceType::BevyCamera { .. }) => (
            Color32::from_rgb(40, 60, 80),
            Color32::from_rgb(180, 100, 220),
            "\u{1F3A5}",
            "Camera",
        ),
        ModulePartType::Source(SourceType::Bevy3DShape { .. }) => (
            Color32::from_rgb(40, 60, 80),
            Color32::from_rgb(100, 180, 220),
            "\u{1F9CA}",
            "3D Shape",
        ),
        ModulePartType::Source(source) => {
            let name = match source {
                SourceType::MediaFile { .. } => "Media File",
                SourceType::Shader { .. } => "Shader",
                SourceType::LiveInput { .. } => "Live Input",
                SourceType::NdiInput { .. } => "NDI Input",
                #[cfg(target_os = "windows")]
                SourceType::SpoutInput { .. } => "Spout Input",
                SourceType::VideoUni { .. } => "Video (Uni)",
                SourceType::ImageUni { .. } => "Image (Uni)",
                SourceType::VideoMulti { .. } => "Video (Multi)",
                SourceType::ImageMulti { .. } => "Image (Multi)",
                SourceType::Bevy => "Bevy Scene",
                SourceType::BevyAtmosphere { .. } => "Atmosphere",
                SourceType::BevyHexGrid { .. } => "Hex Grid",
                SourceType::BevyParticles { .. } => "Particles",
                SourceType::Bevy3DText { .. } => "3D Text",
                SourceType::BevyCamera { .. } => "Camera",
                SourceType::Bevy3DShape { .. } => "3D Shape",
                SourceType::Bevy3DModel { .. } => "3D Model",
            };
            (
                Color32::from_rgb(50, 60, 70),
                Color32::from_rgb(80, 140, 180),
                "\u{1F3AC}",
                name,
            )
        }

        ModulePartType::Mask(mask) => {
            let name = match mask {
                MaskType::File { .. } => "File Mask",
                MaskType::Shape(shape) => match shape {
                    MaskShape::Circle => "Circle",
                    MaskShape::Rectangle => "Rectangle",
                    MaskShape::Triangle => "Triangle",
                    MaskShape::Star => "Star",
                    MaskShape::Ellipse => "Ellipse",
                },
                MaskType::Gradient { .. } => "Gradient",
            };
            (
                Color32::from_rgb(60, 55, 70),
                Color32::from_rgb(160, 100, 180),
                "\u{1F3AD}",
                name,
            )
        }
        ModulePartType::Modulizer(mod_type) => {
            let name = match mod_type {
                ModulizerType::Effect {
                    effect_type: effect,
                    ..
                } => match effect {
                    EffectType::Blur => "Blur",
                    EffectType::Sharpen => "Sharpen",
                    EffectType::Invert => "Invert",
                    EffectType::Threshold => "Threshold",
                    EffectType::Brightness => "Brightness",
                    EffectType::Contrast => "Contrast",
                    EffectType::Saturation => "Saturation",
                    EffectType::HueShift => "Hue Shift",
                    EffectType::Colorize => "Colorize",
                    EffectType::Wave => "Wave",
                    EffectType::Spiral => "Spiral",
                    EffectType::Pinch => "Pinch",
                    EffectType::Mirror => "Mirror",
                    EffectType::Kaleidoscope => "Kaleidoscope",
                    EffectType::Pixelate => "Pixelate",
                    EffectType::Halftone => "Halftone",
                    EffectType::EdgeDetect => "Edge Detect",
                    EffectType::Posterize => "Posterize",
                    EffectType::Glitch => "Glitch",
                    EffectType::RgbSplit => "RGB Split",
                    EffectType::ChromaticAberration => "Chromatic",
                    EffectType::VHS => "VHS",
                    EffectType::FilmGrain => "Film Grain",
                    EffectType::Vignette => "Vignette",
                    EffectType::LoadLUT => "Load 3D LUT",
                    EffectType::ShaderGraph(_) => "Custom Graph",
                },
                ModulizerType::BlendMode(blend) => match blend {
                    BlendModeType::Normal => "Normal",
                    BlendModeType::Add => "Add",
                    BlendModeType::Multiply => "Multiply",
                    BlendModeType::Screen => "Screen",
                    BlendModeType::Overlay => "Overlay",
                    BlendModeType::Difference => "Difference",
                    BlendModeType::Exclusion => "Exclusion",
                },
                ModulizerType::AudioReactive { .. } => "Audio Reactive",
            };
            (
                egui::Color32::from_rgb(60, 60, 50),
                egui::Color32::from_rgb(180, 140, 60),
                "ã€°ï¸ ",
                name,
            )
        }
        ModulePartType::Mesh(_) => (
            egui::Color32::from_rgb(60, 60, 80),
            egui::Color32::from_rgb(100, 100, 200),
            "🕸️ï¸ ",
            "Mesh",
        ),
        ModulePartType::Layer(layer) => {
            let name = match layer {
                LayerType::Single { .. } => "Single Layer",
                LayerType::Group { .. } => "Layer Group",
                LayerType::All { .. } => "All Layers",
            };
            (
                Color32::from_rgb(50, 70, 60),
                Color32::from_rgb(80, 180, 120),
                "\u{1F4D1}",
                name,
            )
        }
        ModulePartType::Output(output) => {
            let name = match output {
                OutputType::Projector { .. } => "Projector",
                OutputType::NdiOutput { .. } => "NDI Output",
                #[cfg(target_os = "windows")]
                OutputType::Spout { .. } => "Spout Output",
                OutputType::Hue { .. } => "Philips Hue",
            };
            (
                Color32::from_rgb(70, 50, 50),
                Color32::from_rgb(180, 80, 80),
                "\u{1F4FA}",
                name,
            )
        }
        ModulePartType::Hue(hue) => {
            let name = match hue {
                HueNodeType::SingleLamp { .. } => "Single Lamp",
                HueNodeType::MultiLamp { .. } => "Multi Lamp",
                HueNodeType::EntertainmentGroup { .. } => "Entertainment Group",
            };
            (
                Color32::from_rgb(60, 60, 40),
                Color32::from_rgb(200, 200, 100),
                "\u{1F4A1}",
                name,
            )
        }
    }
}

pub fn get_part_category(part_type: &ModulePartType) -> &'static str {
    match part_type {
        ModulePartType::Trigger(_) => "Trigger",
        ModulePartType::Source(_) => "Source",
        ModulePartType::Mask(_) => "Mask",
        ModulePartType::Modulizer(_) => "Modulator",
        ModulePartType::Mesh(_) => "Mesh",
        ModulePartType::Layer(_) => "Layer",
        ModulePartType::Output(_) => "Output",
        ModulePartType::Hue(_) => "Hue",
    }
}

pub fn get_socket_color(socket_type: &ModuleSocketType) -> Color32 {
    match socket_type {
        ModuleSocketType::Trigger => Color32::from_rgb(200, 50, 255), // Vibrant Purple
        ModuleSocketType::Media => Color32::from_rgb(50, 150, 255),   // Bright Blue
        ModuleSocketType::Effect => Color32::from_rgb(255, 160, 0),   // Vivid Orange
        ModuleSocketType::Layer => Color32::from_rgb(0, 230, 120),    // Emerald Green
        ModuleSocketType::Output => Color32::from_rgb(255, 60, 60),   // Bright Red
        ModuleSocketType::Link => Color32::from_rgb(180, 180, 180),   // Silver
    }
}

pub fn get_part_property_text(part_type: &ModulePartType) -> String {
    match part_type {
        ModulePartType::Trigger(trigger_type) => match trigger_type {
            TriggerType::AudioFFT { band, .. } => format!("\u{1F50A} Audio: {:?}", band),
            TriggerType::Random { .. } => "\u{1F3B2} Random".to_string(),
            TriggerType::Fixed { interval_ms, .. } => format!("⏱️ {}ms", interval_ms),
            TriggerType::Midi { channel, note, .. } => {
                format!("\u{1F3B9} Ch{} N{}", channel, note)
            }
            TriggerType::Osc { address } => format!("\u{1F4E1} {}", address),
            TriggerType::Shortcut { key_code, .. } => format!("âŒ¨ï¸  {}", key_code),
            TriggerType::Beat => "🥁 Beat".to_string(),
        },
        ModulePartType::Source(source_type) => match source_type {
            SourceType::MediaFile { path, .. } => {
                if path.is_empty() {
                    "📁 Select file...".to_string()
                } else {
                    format!("📁 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                }
            }
            SourceType::Shader { name, .. } => format!("\u{1F3A8} {}", name),
            SourceType::LiveInput { device_id } => format!("\u{1F4F9} Device {}", device_id),
            SourceType::NdiInput { source_name } => {
                format!("\u{1F4E1} {}", source_name.as_deref().unwrap_or("None"))
            }
            SourceType::Bevy => "\u{1F3AE} Bevy Scene".to_string(),
            #[cfg(target_os = "windows")]
            SourceType::SpoutInput { sender_name } => format!("\u{1F6B0} {}", sender_name),
            SourceType::VideoUni { path, .. } => {
                if path.is_empty() {
                    "📁 Select video...".to_string()
                } else {
                    format!(
                        "\u{1F4F9} {}",
                        path.split(['/', '\\']).next_back().unwrap_or(path)
                    )
                }
            }
            SourceType::ImageUni { path, .. } => {
                if path.is_empty() {
                    "\u{1F5BC} Select image...".to_string()
                } else {
                    format!(
                        "\u{1F5BC} {}",
                        path.split(['/', '\\']).next_back().unwrap_or(path)
                    )
                }
            }
            SourceType::VideoMulti { shared_id, .. } => {
                format!("\u{1F4F9} Shared: {}", shared_id)
            }
            SourceType::ImageMulti { shared_id, .. } => {
                format!("\u{1F5BC} Shared: {}", shared_id)
            }
            SourceType::BevyAtmosphere { .. } => "â˜ ï¸  Atmosphere".to_string(),
            SourceType::BevyHexGrid { .. } => "\u{1F6D1} Hex Grid".to_string(),
            SourceType::BevyParticles { .. } => "\u{2728} Particles".to_string(),
            SourceType::Bevy3DText { text, .. } => {
                format!("T: {}", text.chars().take(10).collect::<String>())
            }
            SourceType::BevyCamera { mode, .. } => match mode {
                BevyCameraMode::Orbit { .. } => "\u{1F3A5} Orbit".to_string(),
                BevyCameraMode::Fly { .. } => "\u{1F3A5} Fly".to_string(),
                BevyCameraMode::Static { .. } => "\u{1F3A5} Static".to_string(),
            },
            SourceType::Bevy3DShape { shape_type, .. } => format!("\u{1F9CA} {:?}", shape_type),
            SourceType::Bevy3DModel { path, .. } => format!("\u{1F3AE} Model: {}", path),
        },
        ModulePartType::Mask(mask_type) => match mask_type {
            MaskType::File { path } => {
                if path.is_empty() {
                    "📁 Select mask...".to_string()
                } else {
                    format!("📁 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                }
            }
            MaskType::Shape(shape) => format!("\u{1F537} {:?}", shape),
            MaskType::Gradient { angle, .. } => {
                format!("\u{1F308} Gradient {}Â°", *angle as i32)
            }
        },
        ModulePartType::Modulizer(modulizer_type) => match modulizer_type {
            ModulizerType::Effect {
                effect_type: effect,
                ..
            } => format!("\u{2728} {}", effect.name()),
            ModulizerType::BlendMode(blend) => format!("🔄 {}", blend.name()),
            ModulizerType::AudioReactive { source } => format!("\u{1F50A} {}", source),
        },
        ModulePartType::Mesh(_) => "🕸️ï¸  Mesh".to_string(),
        ModulePartType::Layer(layer_type) => {
            use vorce_core::module::LayerType;
            match layer_type {
                LayerType::Single { name, .. } => format!("\u{1F4D1} {}", name),
                LayerType::Group { name, .. } => format!("📁 {}", name),
                LayerType::All { .. } => "\u{1F4D1} All Layers".to_string(),
            }
        }
        ModulePartType::Output(output_type) => match output_type {
            OutputType::Projector { name, .. } => format!("\u{1F4FA} {}", name),
            OutputType::NdiOutput { name } => format!("\u{1F4E1} {}", name),
            #[cfg(target_os = "windows")]
            OutputType::Spout { name } => format!("\u{1F6B0} {}", name),
            OutputType::Hue { bridge_ip, .. } => {
                if bridge_ip.is_empty() {
                    "\u{1F4A1} Not Connected".to_string()
                } else {
                    format!("\u{1F4A1} {}", bridge_ip)
                }
            }
        },
        ModulePartType::Hue(hue) => match hue {
            HueNodeType::SingleLamp { name, .. } => {
                format!("\u{1F4A1} {}", name)
            }
            HueNodeType::MultiLamp { name, .. } => {
                format!("\u{1F4A1}\u{1F4A1} {}", name)
            }
            HueNodeType::EntertainmentGroup { name, .. } => {
                format!("\u{1F3AD} {}", name)
            }
        },
    }
}

pub fn part_type_from_module_part_type(mpt: &ModulePartType) -> PartType {
    match mpt {
        ModulePartType::Trigger(_) => PartType::Trigger,
        ModulePartType::Source(_) => PartType::Source,
        ModulePartType::Mask(_) => PartType::Mask,
        ModulePartType::Modulizer(_) => PartType::Modulator,
        ModulePartType::Mesh(_) => PartType::Mesh,
        ModulePartType::Layer(_) => PartType::Layer,
        ModulePartType::Output(_) => PartType::Output,
        ModulePartType::Hue(_) => PartType::Hue,
    }
}
