//! Tests for socket generation across all ModulePartType variants

#[cfg(test)]
mod tests {
    use crate::module::types::hue::HueNodeType;
    use crate::module::types::layer::LayerType;
    use crate::module::types::mask::{MaskShape, MaskType};
    use crate::module::types::mesh::MeshType;
    use crate::module::types::modulizer::{EffectType, ModulizerType};
    use crate::module::types::output::{HueMappingMode, OutputType};
    use crate::module::types::part::ModulePartType;
    use crate::module::types::source::SourceType;
    use crate::module::types::trigger::{AudioTriggerOutputConfig, TriggerType};
    use std::collections::HashMap;

    #[test]
    fn test_all_part_type_variants_socket_generation() {
        // Trigger - AudioFFT
        let trigger_fft = ModulePartType::Trigger(TriggerType::AudioFFT {
            band: crate::module::types::trigger::AudioBand::Bass,
            threshold: 0.5,
            output_config: AudioTriggerOutputConfig {
                frequency_bands: true,
                ..Default::default()
            },
        });
        let (in_sockets, out_sockets) = trigger_fft.get_default_sockets();
        assert_eq!(in_sockets.len(), 0);
        assert!(!out_sockets.is_empty());

        // Trigger - Others
        let trigger_random = ModulePartType::Trigger(TriggerType::Random {
            min_interval_ms: 100,
            max_interval_ms: 200,
            probability: 1.0,
        });
        let (in_sockets, out_sockets) = trigger_random.get_default_sockets();
        assert_eq!(in_sockets.len(), 0);
        assert_eq!(out_sockets.len(), 1);

        // Source - Camera
        let source_camera = ModulePartType::Source(SourceType::BevyCamera {
            active: true,
            mode: crate::module::types::source::BevyCameraMode::Orbit {
                radius: 10.0,
                speed: 1.0,
                target: [0.0, 0.0, 0.0],
                height: 5.0,
            },
            fov: 90.0,
        });
        let (in_sockets, out_sockets) = source_camera.get_default_sockets();
        assert_eq!(in_sockets.len(), 1);
        assert_eq!(out_sockets.len(), 1);

        // Source - Particles
        let source_particles = ModulePartType::Source(SourceType::BevyParticles {
            rate: 1.0,
            lifetime: 1.0,
            speed: 1.0,
            color_start: [1.0, 1.0, 1.0, 1.0],
            color_end: [0.0, 0.0, 0.0, 0.0],
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
        });
        let (in_sockets, out_sockets) = source_particles.get_default_sockets();
        assert_eq!(in_sockets.len(), 1);
        assert_eq!(out_sockets.len(), 1);

        // Source - Normal
        let source_media = ModulePartType::Source(SourceType::MediaFile {
            path: "test.mp4".into(),
            speed: 1.0,
            loop_enabled: false,
            start_time: 0.0,
            end_time: 0.0,
            opacity: 1.0,
            blend_mode: None,
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            hue_shift: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            target_width: None,
            target_height: None,
            target_fps: None,
            flip_horizontal: false,
            flip_vertical: false,
            reverse_playback: false,
        });
        let (in_sockets, out_sockets) = source_media.get_default_sockets();
        assert_eq!(in_sockets.len(), 1);
        assert_eq!(out_sockets.len(), 1);

        // Mask
        let mask = ModulePartType::Mask(MaskType::Shape(MaskShape::Circle));
        let (in_sockets, out_sockets) = mask.get_default_sockets();
        assert_eq!(in_sockets.len(), 2);
        assert_eq!(out_sockets.len(), 1);

        // Modulizer
        let modulizer = ModulePartType::Modulizer(ModulizerType::Effect {
            effect_type: EffectType::Blur,
            params: HashMap::new(),
        });
        let (in_sockets, out_sockets) = modulizer.get_default_sockets();
        assert_eq!(in_sockets.len(), 2);
        assert_eq!(out_sockets.len(), 1);

        // Layer
        let layer = ModulePartType::Layer(LayerType::Single {
            id: 1,
            name: "Layer".into(),
            opacity: 1.0,
            blend_mode: None,
            mesh: MeshType::Quad {
                tl: (0., 0.),
                tr: (1., 0.),
                br: (1., 1.),
                bl: (0., 1.),
            },
            mapping_mode: false,
        });
        let (in_sockets, out_sockets) = layer.get_default_sockets();
        assert_eq!(in_sockets.len(), 2);
        assert_eq!(out_sockets.len(), 1);

        // Mesh
        let mesh = ModulePartType::Mesh(MeshType::Quad {
            tl: (0., 0.),
            tr: (1., 0.),
            br: (1., 1.),
            bl: (0., 1.),
        });
        let (in_sockets, out_sockets) = mesh.get_default_sockets();
        assert_eq!(in_sockets.len(), 2);
        assert_eq!(out_sockets.len(), 1);

        // Hue
        let hue = ModulePartType::Hue(HueNodeType::SingleLamp {
            id: "1".into(),
            name: "Lamp".into(),
            brightness: 1.0,
            color: [1., 1., 1.],
            effect: None,
            effect_active: false,
        });
        let (in_sockets, out_sockets) = hue.get_default_sockets();
        assert_eq!(in_sockets.len(), 3);
        assert_eq!(out_sockets.len(), 0);

        // Output - Hue
        let output_hue = ModulePartType::Output(OutputType::Hue {
            bridge_ip: "1".into(),
            username: "1".into(),
            client_key: "1".into(),
            entertainment_area: "1".into(),
            lamp_positions: HashMap::new(),
            mapping_mode: HueMappingMode::Ambient,
        });
        let (in_sockets, out_sockets) = output_hue.get_default_sockets();
        assert_eq!(in_sockets.len(), 2);
        assert_eq!(out_sockets.len(), 0);

        // Output - Normal
        let output_projector = ModulePartType::Output(OutputType::NdiOutput {
            name: "test".into(),
        });
        let (in_sockets, out_sockets) = output_projector.get_default_sockets();
        assert_eq!(in_sockets.len(), 1);
        assert_eq!(out_sockets.len(), 0);
    }

    #[test]
    fn test_standard_socket_builders() {
        use crate::module::types::socket::{ModuleSocket, ModuleSocketDirection, ModuleSocketType};

        let media_in = ModuleSocket::standard_media_in();
        assert_eq!(media_in.id, "media_in");
        assert_eq!(media_in.direction, ModuleSocketDirection::Input);
        assert_eq!(media_in.socket_type, ModuleSocketType::Media);
        assert!(media_in.is_primary);

        let media_out = ModuleSocket::standard_media_out();
        assert_eq!(media_out.id, "media_out");
        assert_eq!(media_out.direction, ModuleSocketDirection::Output);
        assert_eq!(media_out.socket_type, ModuleSocketType::Media);
        assert!(media_out.is_primary);

        let trigger_in = ModuleSocket::standard_trigger_in();
        assert_eq!(trigger_in.id, "trigger_in");
        assert_eq!(trigger_in.direction, ModuleSocketDirection::Input);
        assert_eq!(trigger_in.socket_type, ModuleSocketType::Trigger);
        assert!(!trigger_in.is_primary);
        assert!(trigger_in.supports_trigger_mapping);

        let trigger_out = ModuleSocket::standard_trigger_out();
        assert_eq!(trigger_out.id, "trigger_out");
        assert_eq!(trigger_out.direction, ModuleSocketDirection::Output);
        assert_eq!(trigger_out.socket_type, ModuleSocketType::Trigger);
        assert!(!trigger_out.is_primary);

        let layer_in = ModuleSocket::standard_layer_in();
        assert_eq!(layer_in.id, "layer_in");
        assert_eq!(layer_in.direction, ModuleSocketDirection::Input);
        assert_eq!(layer_in.socket_type, ModuleSocketType::Layer);
        assert!(layer_in.is_primary);

        let layer_out = ModuleSocket::standard_layer_out();
        assert_eq!(layer_out.id, "layer_out");
        assert_eq!(layer_out.direction, ModuleSocketDirection::Output);
        assert_eq!(layer_out.socket_type, ModuleSocketType::Layer);
        assert!(layer_out.is_primary);
    }
}
