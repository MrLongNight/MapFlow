use super::super::types::*;
use mapmap_core::module::{
    AudioBand, AudioTriggerOutputConfig, EffectType, LayerType, MaskShape, MaskType,
    ModulePartType, ModulizerType, OutputType, SourceType, TriggerType,
};

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
            connections: vec![
                (0, "0".to_string(), 1, "0".to_string()),
                (1, "0".to_string(), 2, "0".to_string()),
            ],
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
            connections: vec![
                (0, "0".to_string(), 1, "0".to_string()),
                (1, "0".to_string(), 2, "0".to_string()),
                (2, "0".to_string(), 3, "0".to_string()),
            ],
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
                    ModulePartType::Layer(LayerType::Single {
                        id: 0,
                        name: "Audio Layer".to_string(),
                        opacity: 1.0,
                        blend_mode: None,
                        mesh: mapmap_core::module::MeshType::default(),
                        mapping_mode: false,
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
            connections: vec![
                (0, "0".to_string(), 1, "0".to_string()),
                (1, "0".to_string(), 2, "0".to_string()),
                (2, "0".to_string(), 3, "0".to_string()),
                (3, "0".to_string(), 4, "0".to_string()),
            ],
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
            connections: vec![
                (0, "0".to_string(), 1, "0".to_string()),
                (1, "0".to_string(), 2, "0".to_string()),
                (2, "0".to_string(), 3, "0".to_string()),
            ],
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
            connections: vec![
                (0, "0".to_string(), 1, "0".to_string()),
                (1, "0".to_string(), 2, "0".to_string()),
            ],
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
            connections: vec![
                (0, "0".to_string(), 1, "0".to_string()),
                (1, "0".to_string(), 2, "0".to_string()),
            ],
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
            connections: vec![
                (0, "0".to_string(), 1, "0".to_string()),
                (1, "0".to_string(), 2, "0".to_string()),
            ],
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
            connections: vec![
                (0, "0".to_string(), 1, "0".to_string()),
                (1, "0".to_string(), 2, "0".to_string()),
            ],
        },
    ]
}
