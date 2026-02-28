use super::types::*;
use egui::{Color32, Pos2, Rect, TextureHandle, Vec2};
use mapmap_core::module::{
    AudioBand, AudioTriggerOutputConfig, BevyCameraMode, BlendModeType, EffectType, HueNodeType,
    LayerType, MaskShape, MaskType, ModulePart, ModulePartType, ModuleSocket, ModuleSocketType,
    ModulizerType, OutputType, PartType, SourceType, TriggerType,
};

pub struct NodeCatalogItem {
    pub label: &'static str,
    pub search_tags: &'static str,
    pub part_type: ModulePartType,
}

pub fn build_node_catalog() -> Vec<NodeCatalogItem> {
    vec![
        // Triggers
        NodeCatalogItem {
            label: "🥁 Beat",
            search_tags: "trigger time rhythm",
            part_type: ModulePartType::Trigger(TriggerType::Beat),
        },
        NodeCatalogItem {
            label: "🔊 Audio FFT",
            search_tags: "trigger sound music reactive",
            part_type: ModulePartType::Trigger(TriggerType::AudioFFT {
                band: AudioBand::Bass,
                threshold: 0.5,
                output_config: AudioTriggerOutputConfig::default(),
            }),
        },
        NodeCatalogItem {
            label: "🎲 Random",
            search_tags: "trigger chance stochastic",
            part_type: ModulePartType::Trigger(TriggerType::Random {
                min_interval_ms: 500,
                max_interval_ms: 2000,
                probability: 0.5,
            }),
        },
        NodeCatalogItem {
            label: "⏱️ Fixed Timer",
            search_tags: "trigger time clock loop",
            part_type: ModulePartType::Trigger(TriggerType::Fixed {
                interval_ms: 1000,
                offset_ms: 0,
            }),
        },
        NodeCatalogItem {
            label: "🎹 MIDI",
            search_tags: "trigger control note cc",
            part_type: ModulePartType::Trigger(TriggerType::Midi {
                channel: 1,
                note: 60,
                device: String::new(),
            }),
        },
        NodeCatalogItem {
            label: "📡 OSC",
            search_tags: "trigger network control open sound control",
            part_type: ModulePartType::Trigger(TriggerType::Osc {
                address: "/trigger".to_string(),
            }),
        },
        NodeCatalogItem {
            label: "⌨️ Shortcut",
            search_tags: "trigger keyboard key input",
            part_type: ModulePartType::Trigger(TriggerType::Shortcut {
                key_code: "Space".to_string(),
                modifiers: 0,
            }),
        },
        // Sources
        NodeCatalogItem {
            label: "📁 Media File",
            search_tags: "source video image movie picture",
            part_type: ModulePartType::Source(SourceType::new_media_file(String::new())),
        },
        NodeCatalogItem {
            label: "🎨 Shader",
            search_tags: "source glsl generator procedural",
            part_type: ModulePartType::Source(SourceType::Shader {
                name: "Default".to_string(),
                params: Vec::new(),
            }),
        },
        #[cfg(feature = "ndi")]
        NodeCatalogItem {
            label: "📡 NDI Input",
            search_tags: "source network video stream",
            part_type: ModulePartType::Source(SourceType::NdiInput { source_name: None }),
        },
        #[cfg(target_os = "windows")]
        NodeCatalogItem {
            label: "🚀 Spout Input",
            search_tags: "source texture share windows",
            part_type: ModulePartType::Source(SourceType::SpoutInput {
                sender_name: String::new(),
            }),
        },
        // Bevy Sources
        NodeCatalogItem {
            label: "📝 3D Text",
            search_tags: "source bevy font label",
            part_type: ModulePartType::Source(SourceType::Bevy3DText {
                text: "Hello 3D".to_string(),
                font_size: 20.0,
                color: [1.0, 1.0, 1.0, 1.0],
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                alignment: "Center".to_string(),
            }),
        },
        NodeCatalogItem {
            label: "🧊 3D Shape",
            search_tags: "source bevy cube sphere geometry",
            part_type: ModulePartType::Source(SourceType::Bevy3DShape {
                shape_type: mapmap_core::module::BevyShapeType::Cube,
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                color: [1.0, 0.5, 0.0, 1.0],
                unlit: false,
                outline_width: 0.0,
                outline_color: [1.0, 1.0, 1.0, 1.0],
            }),
        },
        NodeCatalogItem {
            label: "📹 Camera",
            search_tags: "source bevy view perspective orbit fly",
            part_type: ModulePartType::Source(SourceType::BevyCamera {
                mode: BevyCameraMode::Orbit {
                    radius: 10.0,
                    speed: 10.0,
                    target: [0.0, 0.0, 0.0],
                    height: 5.0,
                },
                fov: 60.0,
                active: true,
            }),
        },
        // Masks
        NodeCatalogItem {
            label: "⚪ Shape Mask",
            search_tags: "mask circle rectangle alpha",
            part_type: ModulePartType::Mask(MaskType::Shape(MaskShape::Circle)),
        },
        NodeCatalogItem {
            label: "🌈 Gradient Mask",
            search_tags: "mask fade transition alpha",
            part_type: ModulePartType::Mask(MaskType::Gradient {
                angle: 0.0,
                softness: 0.5,
            }),
        },
        // Modulators
        NodeCatalogItem {
            label: "🎚️ Blend Mode",
            search_tags: "modulator mix composite add multiply screen",
            part_type: ModulePartType::Modulizer(ModulizerType::BlendMode(BlendModeType::Normal)),
        },
    ]
    .into_iter()
    .chain(
        // Effects
        EffectType::all().iter().map(|effect| NodeCatalogItem {
            label: effect.name(),
            search_tags: "modulator effect filter fx",
            part_type: ModulePartType::Modulizer(ModulizerType::Effect {
                effect_type: *effect,
                params: std::collections::HashMap::new(),
            }),
        }),
    )
    .chain(vec![
        // Layers
        NodeCatalogItem {
            label: "📄 Single Layer",
            search_tags: "layer composition",
            part_type: ModulePartType::Layer(LayerType::Single {
                id: 0,
                name: "New Layer".to_string(),
                opacity: 1.0,
                blend_mode: None,
                mesh: mapmap_core::module::MeshType::default(),
                mapping_mode: false,
            }),
        },
        NodeCatalogItem {
            label: "📁 Layer Group",
            search_tags: "layer folder collection",
            part_type: ModulePartType::Layer(LayerType::Group {
                name: "New Group".to_string(),
                opacity: 1.0,
                blend_mode: None,
                mesh: mapmap_core::module::MeshType::default(),
                mapping_mode: false,
            }),
        },
        NodeCatalogItem {
            label: "📚 All Layers",
            search_tags: "layer master global",
            part_type: ModulePartType::Layer(LayerType::All {
                opacity: 1.0,
                blend_mode: None,
            }),
        },
        // Hue
        NodeCatalogItem {
            label: "💡 Single Lamp",
            search_tags: "hue light smart home philips",
            part_type: ModulePartType::Hue(HueNodeType::SingleLamp {
                id: String::new(),
                name: "New Lamp".to_string(),
                brightness: 1.0,
                color: [1.0, 1.0, 1.0],
                effect: None,
                effect_active: false,
            }),
        },
        // Output
        NodeCatalogItem {
            label: "🖥️ Projector Output",
            search_tags: "output display screen beamer",
            part_type: ModulePartType::Output(OutputType::Projector {
                id: 1,
                name: "Projector 1".to_string(),
                hide_cursor: false,
                target_screen: 0,
                show_in_preview_panel: true,
                extra_preview_window: false,
                output_width: 0,
                output_height: 0,
                output_fps: 60.0,
                ndi_enabled: false,
                ndi_stream_name: String::new(),
            }),
        },
    ])
    .collect()
}

pub fn ensure_icons_loaded(
    plug_icons: &mut std::collections::HashMap<String, TextureHandle>,
    ctx: &egui::Context,
) {
    if !plug_icons.is_empty() {
        return;
    }

    let paths = [
        "resources/stecker_icons",
        "../resources/stecker_icons",
        r"C:\Users\Vinyl\Desktop\VJMapper\VjMapper\resources\stecker_icons",
    ];

    let files = [
        "audio-jack.svg",
        "audio-jack_2.svg",
        "plug.svg",
        "power-plug.svg",
        "usb-cable.svg",
    ];

    for path_str in paths {
        let base_path = std::path::Path::new(path_str);
        if base_path.exists() {
            for file in files {
                let path = base_path.join(file);
                if let Some(texture) = load_svg_icon(&path, ctx) {
                    plug_icons.insert(file.to_string(), texture);
                }
            }
            if !plug_icons.is_empty() {
                break;
            }
        }
    }
}

fn load_svg_icon(path: &std::path::Path, ctx: &egui::Context) -> Option<TextureHandle> {
    let svg_data = std::fs::read(path).ok()?;
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&svg_data, &options).ok()?;
    let size = tree.size();
    let width = size.width().round() as u32;
    let height = size.height().round() as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let mut pixels = Vec::with_capacity((width * height) as usize);
    for pixel in pixmap.pixels() {
        // Preserve original RGBA from SVG
        pixels.push(egui::Color32::from_rgba_premultiplied(
            pixel.red(),
            pixel.green(),
            pixel.blue(),
            pixel.alpha(),
        ));
    }

    let image = egui::ColorImage {
        size: [width as usize, height as usize],
        pixels,
        source_size: egui::Vec2::new(width as f32, height as f32),
    };

    Some(ctx.load_texture(
        path.file_name()?.to_string_lossy(),
        image,
        egui::TextureOptions {
            magnification: egui::TextureFilter::Linear,
            minification: egui::TextureFilter::Linear,
            wrap_mode: egui::TextureWrapMode::ClampToEdge,
            mipmap_mode: None,
        },
    ))
}

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
        ModuleSocketType::Trigger => Color32::from_rgb(180, 100, 220),
        ModuleSocketType::Media => Color32::from_rgb(100, 180, 220),
        ModuleSocketType::Effect => Color32::from_rgb(220, 180, 100),
        ModuleSocketType::Layer => Color32::from_rgb(100, 220, 140),
        ModuleSocketType::Output => Color32::from_rgb(220, 100, 100),
        ModuleSocketType::Link => Color32::from_rgb(200, 200, 200),
    }
}

pub fn get_part_property_text(part_type: &ModulePartType) -> String {
    match part_type {
        ModulePartType::Trigger(trigger_type) => match trigger_type {
            TriggerType::AudioFFT { band, .. } => format!("\u{1F50A} Audio: {:?}", band),
            TriggerType::Random { .. } => "\u{1F3B2} Random".to_string(),
            TriggerType::Fixed { interval_ms, .. } => format!("⏱️ï¸  {}ms", interval_ms),
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
            use mapmap_core::module::LayerType;
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

pub fn auto_layout_parts(parts: &mut [ModulePart]) {
    // Sort parts by type category for left-to-right flow
    let type_order = |pt: &ModulePartType| -> usize {
        match pt {
            ModulePartType::Trigger(_) => 0,
            ModulePartType::Source(_) => 1,
            ModulePartType::Mask(_) => 2,
            ModulePartType::Modulizer(_) => 3,
            ModulePartType::Mesh(_) => 4,
            ModulePartType::Layer(_) => 5,
            ModulePartType::Output(_) => 6,
            ModulePartType::Hue(_) => 7,
        }
    };

    // Group parts by type
    let mut columns: [Vec<usize>; 8] = Default::default();
    for (i, part) in parts.iter().enumerate() {
        let col = type_order(&part.part_type);
        columns[col].push(i);
    }

    // Layout parameters
    let node_width = 200.0;
    let node_height = 120.0;
    let h_spacing = 100.0;
    let v_spacing = 60.0;
    let start_x = 50.0;
    let start_y = 50.0;

    // Position each column
    let mut x = start_x;
    for col in &columns {
        if col.is_empty() {
            continue;
        }

        let mut y = start_y;
        for &part_idx in col {
            parts[part_idx].position = (x, y);
            y += node_height + v_spacing;
        }

        x += node_width + h_spacing;
    }
}

pub fn find_free_position(parts: &[ModulePart], preferred: (f32, f32)) -> (f32, f32) {
    let node_width = 200.0;
    let node_height = 130.0;
    let grid_step = 30.0;

    let mut pos = preferred;
    let mut attempts = 0;

    loop {
        let new_rect =
            Rect::from_min_size(Pos2::new(pos.0, pos.1), Vec2::new(node_width, node_height));

        let has_collision = parts.iter().any(|part| {
            let part_height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
            let part_rect = Rect::from_min_size(
                Pos2::new(part.position.0, part.position.1),
                Vec2::new(node_width, part_height),
            );
            new_rect.intersects(part_rect)
        });

        if !has_collision {
            return pos;
        }

        attempts += 1;
        if attempts > 100 {
            return (preferred.0, preferred.1 + (parts.len() as f32) * 150.0);
        }

        pos.1 += grid_step;
        if pos.1 > preferred.1 + 500.0 {
            pos.1 = preferred.1;
            pos.0 += node_width + 20.0;
        }
    }
}

pub fn default_presets() -> Vec<ModulePreset> {
    vec![
        ModulePreset {
            name: "Simple Media Chain".to_string(),
            parts: vec![
                (
                    ModulePartType::Trigger(TriggerType::Beat),
                    (50.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Source(SourceType::new_media_file(String::new())),
                    (350.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Output(OutputType::Projector {
                        id: 1,
                        name: "Projector 1".to_string(),

                        hide_cursor: true,
                        target_screen: 0,
                        show_in_preview_panel: true,
                        extra_preview_window: false,
                        output_width: 0,
                        output_height: 0,
                        output_fps: 60.0,
                        ndi_enabled: false,
                        ndi_stream_name: String::new(),
                    }),
                    (650.0, 100.0),
                    None,
                ),
            ],
            connections: vec![(0, 0, 1, 0), (1, 0, 2, 0)],
        },
        ModulePreset {
            name: "Effect Chain".to_string(),
            parts: vec![
                (
                    ModulePartType::Trigger(TriggerType::Beat),
                    (50.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Source(SourceType::new_media_file(String::new())),
                    (350.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Modulizer(ModulizerType::Effect {
                        effect_type: EffectType::Blur,
                        params: std::collections::HashMap::new(),
                    }),
                    (650.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Output(OutputType::Projector {
                        id: 1,
                        name: "Projector 1".to_string(),

                        hide_cursor: true,
                        target_screen: 0,
                        show_in_preview_panel: true,
                        extra_preview_window: false,
                        output_width: 0,
                        output_height: 0,
                        output_fps: 60.0,
                        ndi_enabled: false,
                        ndi_stream_name: String::new(),
                    }),
                    (950.0, 100.0),
                    None,
                ),
            ],
            connections: vec![(0, 0, 1, 0), (1, 0, 2, 0), (2, 0, 3, 0)],
        },
        ModulePreset {
            name: "Audio Reactive".to_string(),
            parts: vec![
                (
                    ModulePartType::Trigger(TriggerType::AudioFFT {
                        band: AudioBand::Bass,
                        threshold: 0.5,
                        output_config: AudioTriggerOutputConfig::default(),
                    }),
                    (50.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Source(SourceType::new_media_file(String::new())),
                    (350.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Modulizer(ModulizerType::Effect {
                        effect_type: EffectType::Glitch,
                        params: std::collections::HashMap::new(),
                    }),
                    (650.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Layer(LayerType::All {
                        opacity: 1.0,
                        blend_mode: None,
                    }),
                    (950.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Output(OutputType::Projector {
                        id: 1,
                        name: "Projector 1".to_string(),

                        hide_cursor: true,
                        target_screen: 0,
                        show_in_preview_panel: true,
                        extra_preview_window: false,
                        output_width: 0,
                        output_height: 0,
                        output_fps: 60.0,
                        ndi_enabled: false,
                        ndi_stream_name: String::new(),
                    }),
                    (1250.0, 100.0),
                    None,
                ),
            ],
            connections: vec![(0, 0, 1, 0), (1, 0, 2, 0), (2, 0, 3, 0), (3, 0, 4, 0)],
        },
        ModulePreset {
            name: "Masked Media".to_string(),
            parts: vec![
                (
                    ModulePartType::Trigger(TriggerType::Beat),
                    (50.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Source(SourceType::new_media_file(String::new())),
                    (350.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Mask(MaskType::Shape(MaskShape::Circle)),
                    (650.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Output(OutputType::Projector {
                        id: 1,
                        name: "Projector 1".to_string(),

                        hide_cursor: true,
                        target_screen: 0,
                        show_in_preview_panel: true,
                        extra_preview_window: false,
                        output_width: 0,
                        output_height: 0,
                        output_fps: 60.0,
                        ndi_enabled: false,
                        ndi_stream_name: String::new(),
                    }),
                    (950.0, 100.0),
                    None,
                ),
            ],
            connections: vec![(0, 0, 1, 0), (1, 0, 2, 0), (2, 0, 3, 0)],
        },
        ModulePreset {
            name: "NDI Source".to_string(),
            parts: vec![
                (
                    ModulePartType::Trigger(TriggerType::Beat),
                    (50.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Source(SourceType::NdiInput { source_name: None }),
                    (350.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Output(OutputType::Projector {
                        id: 1,
                        name: "Projector 1".to_string(),

                        hide_cursor: true,
                        target_screen: 0,
                        show_in_preview_panel: true,
                        extra_preview_window: false,
                        output_width: 0,
                        output_height: 0,
                        output_fps: 60.0,
                        ndi_enabled: false,
                        ndi_stream_name: String::new(),
                    }),
                    (650.0, 100.0),
                    None,
                ),
            ],
            connections: vec![(0, 0, 1, 0), (1, 0, 2, 0)],
        },
        ModulePreset {
            name: "NDI Output".to_string(),
            parts: vec![
                (
                    ModulePartType::Trigger(TriggerType::Beat),
                    (50.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Source(SourceType::new_media_file(String::new())),
                    (350.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Output(OutputType::NdiOutput {
                        name: "MapFlow NDI".to_string(),
                    }),
                    (650.0, 100.0),
                    None,
                ),
            ],
            connections: vec![(0, 0, 1, 0), (1, 0, 2, 0)],
        },
        #[cfg(target_os = "windows")]
        ModulePreset {
            name: "Spout Source".to_string(),
            parts: vec![
                (
                    ModulePartType::Trigger(TriggerType::Beat),
                    (50.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Source(SourceType::SpoutInput {
                        sender_name: String::new(),
                    }),
                    (350.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Output(OutputType::Projector {
                        id: 1,
                        name: "Projector 1".to_string(),

                        hide_cursor: true,
                        target_screen: 0,
                        show_in_preview_panel: true,
                        extra_preview_window: false,
                        output_width: 0,
                        output_height: 0,
                        output_fps: 60.0,
                        ndi_enabled: false,
                        ndi_stream_name: String::new(),
                    }),
                    (650.0, 100.0),
                    None,
                ),
            ],
            connections: vec![(0, 0, 1, 0), (1, 0, 2, 0)],
        },
        #[cfg(target_os = "windows")]
        ModulePreset {
            name: "Spout Output".to_string(),
            parts: vec![
                (
                    ModulePartType::Trigger(TriggerType::Beat),
                    (50.0, 100.0),
                    None,
                ),
                (
                    ModulePartType::Source(SourceType::new_media_file(String::new())),
                    (350.0, 100.0),
                    None,
                ),
                #[cfg(target_os = "windows")]
                (
                    ModulePartType::Output(OutputType::Spout {
                        name: "MapFlow Spout".to_string(),
                    }),
                    (650.0, 100.0),
                    None,
                ),
            ],
            connections: vec![(0, 0, 1, 0), (1, 0, 2, 0)],
        },
    ]
}

pub fn get_sockets_for_part_type(
    part_type: &ModulePartType,
) -> (Vec<ModuleSocket>, Vec<ModuleSocket>) {
    match part_type {
        ModulePartType::Trigger(_) => (
            vec![],
            vec![ModuleSocket {
                name: "Trigger Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            }],
        ),
        ModulePartType::Source(_) => (
            vec![ModuleSocket {
                name: "Trigger In".to_string(),
                socket_type: ModuleSocketType::Trigger,
            }],
            vec![ModuleSocket {
                name: "Media Out".to_string(),
                socket_type: ModuleSocketType::Media,
            }],
        ),
        ModulePartType::Mask(_) => (
            vec![
                ModuleSocket {
                    name: "Media In".to_string(),
                    socket_type: ModuleSocketType::Media,
                },
                ModuleSocket {
                    name: "Mask In".to_string(),
                    socket_type: ModuleSocketType::Media,
                },
            ],
            vec![ModuleSocket {
                name: "Media Out".to_string(),
                socket_type: ModuleSocketType::Media,
            }],
        ),
        ModulePartType::Modulizer(_) => (
            vec![
                ModuleSocket {
                    name: "Media In".to_string(),
                    socket_type: ModuleSocketType::Media,
                },
                ModuleSocket {
                    name: "Trigger In".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                },
            ],
            vec![ModuleSocket {
                name: "Media Out".to_string(),
                socket_type: ModuleSocketType::Media,
            }],
        ),
        ModulePartType::Mesh(_) => (vec![], vec![]),
        ModulePartType::Layer(_) => (
            vec![ModuleSocket {
                name: "Media In".to_string(),
                socket_type: ModuleSocketType::Media,
            }],
            vec![ModuleSocket {
                name: "Layer Out".to_string(),
                socket_type: ModuleSocketType::Layer,
            }],
        ),
        ModulePartType::Output(_) => (
            vec![ModuleSocket {
                name: "Layer In".to_string(),
                socket_type: ModuleSocketType::Layer,
            }],
            vec![],
        ),
        ModulePartType::Hue(_) => (
            vec![
                ModuleSocket {
                    name: "Brightness".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                },
                ModuleSocket {
                    name: "Color (RGB)".to_string(),
                    socket_type: ModuleSocketType::Media,
                },
                ModuleSocket {
                    name: "Strobe".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                },
            ],
            vec![],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_catalog_not_empty() {
        let catalog = build_node_catalog();
        assert!(!catalog.is_empty(), "Node catalog should not be empty");
    }

    #[test]
    fn test_node_catalog_coverage() {
        let catalog = build_node_catalog();

        let has_trigger = catalog.iter().any(|item| {
            matches!(
                item.part_type,
                mapmap_core::module::ModulePartType::Trigger(_)
            )
        });
        let has_source = catalog.iter().any(|item| {
            matches!(
                item.part_type,
                mapmap_core::module::ModulePartType::Source(_)
            )
        });
        let has_effect = catalog.iter().any(|item| {
            matches!(
                item.part_type,
                mapmap_core::module::ModulePartType::Modulizer(
                    mapmap_core::module::ModulizerType::Effect { .. }
                )
            )
        });
        let has_output = catalog.iter().any(|item| {
            matches!(
                item.part_type,
                mapmap_core::module::ModulePartType::Output(_)
            )
        });

        assert!(has_trigger, "Catalog missing Triggers");
        assert!(has_source, "Catalog missing Sources");
        assert!(has_effect, "Catalog missing Effects");
        assert!(has_output, "Catalog missing Outputs");
    }

    #[test]
    fn test_node_catalog_search_tags() {
        let catalog = build_node_catalog();
        let beat_trigger = catalog
            .iter()
            .find(|item| item.label.contains("Beat"))
            .expect("Beat trigger not found");
        assert!(
            beat_trigger.search_tags.contains("rhythm"),
            "Beat trigger missing search tag"
        );
    }
}