use super::super::utils;
use egui::Ui;
use mapmap_core::module::{
    BevyCameraMode, BlendModeType, EffectType, HueNodeType, LayerType, MaskShape, MaskType,
    ModuleManager, ModulePartType, ModulizerType, OutputType, SourceType, TriggerType,
};

pub fn render_add_node_menu_content(
    ui: &mut Ui,
    manager: &mut ModuleManager,
    pos_override: Option<(f32, f32)>,
    active_module_id: Option<u64>,
) {
    let mut module = if let Some(id) = active_module_id {
        manager.get_module_mut(id)
    } else {
        None
    };

    if let Some(module) = &mut module {
        let mut add_node = |part_type: ModulePartType| {
            let preferred_pos = pos_override.unwrap_or((200.0, 200.0));
            let pos = utils::find_free_position(&module.parts, preferred_pos);
            module.add_part_with_type(part_type, pos);
        };

        ui.menu_button("\u{26A1} Triggers", |ui| {
            if ui.button("🥁 Beat").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Beat));
                ui.close();
            }
            if ui.button("\u{1F50A} Audio FFT").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::AudioFFT {
                    band: mapmap_core::module::AudioBand::Bass,
                    threshold: 0.5,
                    output_config: mapmap_core::module::AudioTriggerOutputConfig::default(),
                }));
                ui.close();
            }
            if ui.button("\u{1F3B2} Random").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Random {
                    min_interval_ms: 500,
                    max_interval_ms: 2000,
                    probability: 0.5,
                }));
                ui.close();
            }
            if ui.button("⏱️ Fixed Timer").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Fixed {
                    interval_ms: 1000,
                    offset_ms: 0,
                }));
                ui.close();
            }
            if ui.button("\u{1F3B9} MIDI").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Midi {
                    channel: 1,
                    note: 60,
                    device: String::new(),
                }));
                ui.close();
            }
            if ui.button("\u{1F4E1} OSC").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Osc {
                    address: "/trigger".to_string(),
                }));
                ui.close();
            }
            if ui.button("⌨️ Shortcut").clicked() {
                add_node(ModulePartType::Trigger(TriggerType::Shortcut {
                    key_code: "Space".to_string(),
                    modifiers: 0,
                }));
                ui.close();
            }
        });

        ui.menu_button("\u{1F4F9} Sources", |ui| {
            if ui.button("📁 Media File").clicked() {
                add_node(ModulePartType::Source(SourceType::new_media_file(
                    String::new(),
                )));
                ui.close();
            }
            if ui.button("\u{1F3A8} Shader").clicked() {
                add_node(ModulePartType::Source(SourceType::Shader {
                    name: "Default".to_string(),
                    params: Vec::new(),
                }));
                ui.close();
            }
            #[cfg(feature = "ndi")]
            if ui.button("\u{1F4E1} NDI Input").clicked() {
                add_node(ModulePartType::Source(SourceType::NdiInput {
                    source_name: None,
                }));
                ui.close();
            }
            #[cfg(target_os = "windows")]
            if ui.button("\u{1F6B0} Spout Input").clicked() {
                add_node(ModulePartType::Source(SourceType::SpoutInput {
                    sender_name: String::new(),
                }));
                ui.close();
            }
            ui.separator();
            ui.label("Bevy 3D:");
            if ui.button("📝 3D Text").clicked() {
                add_node(ModulePartType::Source(SourceType::Bevy3DText {
                    text: "Hello 3D".to_string(),
                    font_size: 20.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                    position: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    alignment: "Center".to_string(),
                }));
                ui.close();
            }
            if ui.button("\u{1F9CA} 3D Shape").clicked() {
                add_node(ModulePartType::Source(SourceType::Bevy3DShape {
                    shape_type: mapmap_core::module::BevyShapeType::Cube,
                    position: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                    color: [1.0, 0.5, 0.0, 1.0],
                    unlit: false,
                    outline_width: 0.0,
                    outline_color: [1.0, 1.0, 1.0, 1.0],
                }));
                ui.close();
            }
            if ui.button("\u{1F3A5} Camera").clicked() {
                add_node(ModulePartType::Source(SourceType::BevyCamera {
                    mode: BevyCameraMode::Orbit {
                        radius: 10.0,
                        speed: 10.0,
                        target: [0.0, 0.0, 0.0],
                        height: 5.0,
                    },
                    fov: 60.0,
                    active: true,
                }));
                ui.close();
            }
        });

        ui.menu_button("\u{1F3AD} Masks", |ui| {
            if ui.button("\u{2B55} Shape").clicked() {
                add_node(ModulePartType::Mask(MaskType::Shape(MaskShape::Circle)));
                ui.close();
            }
            if ui.button("\u{1F308} Gradient").clicked() {
                add_node(ModulePartType::Mask(MaskType::Gradient {
                    angle: 0.0,
                    softness: 0.5,
                }));
                ui.close();
            }
        });

        ui.menu_button("🎛️ Modulators", |ui| {
            if ui.button("🎚️ Blend Mode").clicked() {
                add_node(ModulePartType::Modulizer(ModulizerType::BlendMode(
                    BlendModeType::Normal,
                )));
                ui.close();
            }
            ui.separator();
            for effect in [
                EffectType::LoadLUT,
                EffectType::Blur,
                EffectType::Pixelate,
                EffectType::Glitch,
                EffectType::Kaleidoscope,
                EffectType::EdgeDetect,
                EffectType::Colorize,
                EffectType::HueShift,
            ] {
                if ui.button(effect.name()).clicked() {
                    add_node(ModulePartType::Modulizer(ModulizerType::Effect {
                        effect_type: effect,
                        params: std::collections::HashMap::new(),
                    }));
                    ui.close();
                }
            }
        });

        ui.menu_button("\u{1F4D1} Layers", |ui| {
            if ui.button("\u{1F4D1} Single Layer").clicked() {
                add_node(ModulePartType::Layer(LayerType::Single {
                    id: 0,
                    name: "New Layer".to_string(),
                    opacity: 1.0,
                    blend_mode: None,
                    mesh: mapmap_core::module::MeshType::default(),
                    mapping_mode: false,
                }));
                ui.close();
            }
            if ui.button("📁 Layer Group").clicked() {
                add_node(ModulePartType::Layer(LayerType::Group {
                    name: "New Group".to_string(),
                    opacity: 1.0,
                    blend_mode: None,
                    mesh: mapmap_core::module::MeshType::default(),
                    mapping_mode: false,
                }));
                ui.close();
            }
            if ui.button("\u{1F4D1} All Layers").clicked() {
                add_node(ModulePartType::Layer(LayerType::All {
                    opacity: 1.0,
                    blend_mode: None,
                }));
                ui.close();
            }
        });

        ui.menu_button("\u{1F4A1} Philips Hue", |ui| {
            if ui.button("\u{1F4A1} Single Lamp").clicked() {
                add_node(ModulePartType::Hue(HueNodeType::SingleLamp {
                    id: String::new(),
                    name: "New Lamp".to_string(),
                    brightness: 1.0,
                    color: [1.0, 1.0, 1.0],
                    effect: None,
                    effect_active: false,
                }));
                ui.close();
            }
        });

        ui.separator();
        if ui.button("\u{1F5BC} Output").clicked() {
            add_node(ModulePartType::Output(OutputType::Projector {
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
            }));
            ui.close();
        }
    }
}
