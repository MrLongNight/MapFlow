use mapmap_core::module::*;

#[test]
fn test_add_part_defaults() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    // 1. Trigger -> Should default to Beat
    let id = module.add_part(PartType::Trigger, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == id).unwrap();
    if let ModulePartType::Trigger(TriggerType::Beat) = &part.part_type {
        // OK
    } else {
        panic!("Expected Trigger(Beat), got {:?}", part.part_type);
    }

    // 2. Source -> Should default to empty MediaFile
    let id = module.add_part(PartType::Source, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == id).unwrap();
    if let ModulePartType::Source(SourceType::MediaFile { path, speed, .. }) = &part.part_type {
        assert!(path.is_empty());
        assert_eq!(*speed, 1.0);
    } else {
        panic!("Expected Source(MediaFile), got {:?}", part.part_type);
    }

    // 3. BevyParticles -> Specific defaults
    let id = module.add_part(PartType::BevyParticles, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == id).unwrap();
    if let ModulePartType::Source(SourceType::BevyParticles { rate, lifetime, .. }) =
        &part.part_type
    {
        assert_eq!(*rate, 100.0);
        assert_eq!(*lifetime, 2.0);
    } else {
        panic!("Expected Source(BevyParticles), got {:?}", part.part_type);
    }

    // 4. Bevy3DShape -> Cube default
    let id = module.add_part(PartType::Bevy3DShape, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == id).unwrap();
    if let ModulePartType::Source(SourceType::Bevy3DShape { shape_type, .. }) = &part.part_type {
        assert_eq!(*shape_type, BevyShapeType::Cube);
    } else {
        panic!("Expected Source(Bevy3DShape), got {:?}", part.part_type);
    }

    // 5. Mask -> Rectangle default
    let id = module.add_part(PartType::Mask, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == id).unwrap();
    if let ModulePartType::Mask(MaskType::Shape(MaskShape::Rectangle)) = &part.part_type {
        // OK
    } else {
        panic!("Expected Mask(Rectangle), got {:?}", part.part_type);
    }

    // 6. Modulator -> Blur default
    let id = module.add_part(PartType::Modulator, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == id).unwrap();
    if let ModulePartType::Modulizer(ModulizerType::Effect { effect_type, .. }) = &part.part_type {
        assert_eq!(*effect_type, EffectType::Blur);
    } else {
        panic!("Expected Modulizer(Effect::Blur), got {:?}", part.part_type);
    }

    // 7. Mesh -> Grid 10x10 default
    let id = module.add_part(PartType::Mesh, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == id).unwrap();
    if let ModulePartType::Mesh(MeshType::Grid { cols, rows }) = &part.part_type {
        assert_eq!(*cols, 10);
        assert_eq!(*rows, 10);
    } else {
        panic!("Expected Mesh(Grid 10x10), got {:?}", part.part_type);
    }

    // 8. Layer -> Single Layer 1 default
    let id = module.add_part(PartType::Layer, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == id).unwrap();
    if let ModulePartType::Layer(LayerType::Single { name, .. }) = &part.part_type {
        assert_eq!(name, "Layer 1");
    } else {
        panic!("Expected Layer(Single), got {:?}", part.part_type);
    }

    // 9. Hue -> Single Lamp default
    let id = module.add_part(PartType::Hue, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == id).unwrap();
    if let ModulePartType::Hue(HueNodeType::SingleLamp { name, .. }) = &part.part_type {
        assert_eq!(name, "New Lamp");
    } else {
        panic!("Expected Hue(SingleLamp), got {:?}", part.part_type);
    }

    // 10. Output -> Projector default with auto-incrementing ID
    // We already have some parts, so let's see what ID we get.
    // The implementation scans existing parts for OutputType::Projector IDs.
    // Currently no outputs exist, so it should start at 1.
    let id = module.add_part(PartType::Output, (0.0, 0.0));
    let part = module.parts.iter().find(|p| p.id == id).unwrap();
    if let ModulePartType::Output(OutputType::Projector {
        id: output_id,
        name,
        ..
    }) = &part.part_type
    {
        assert_eq!(*output_id, 1);
        assert_eq!(name, "Output 1");
    } else {
        panic!("Expected Output(Projector 1), got {:?}", part.part_type);
    }

    // Add another output to test increment
    let id2 = module.add_part(PartType::Output, (0.0, 0.0));
    let part2 = module.parts.iter().find(|p| p.id == id2).unwrap();
    if let ModulePartType::Output(OutputType::Projector {
        id: output_id,
        name,
        ..
    }) = &part2.part_type
    {
        assert_eq!(*output_id, 2);
        assert_eq!(name, "Output 2");
    } else {
        panic!("Expected Output(Projector 2), got {:?}", part2.part_type);
    }
}

#[test]
fn test_socket_consistency() {
    // 1. Trigger (AudioFFT)
    let fft = ModulePartType::Trigger(TriggerType::AudioFFT {
        band: AudioBand::Bass,
        threshold: 0.5,
        output_config: AudioTriggerOutputConfig::default(),
    });
    let (ins, outs) = fft.get_default_sockets();
    assert!(ins.is_empty());
    assert!(!outs.is_empty()); // Beat Out default

    // 2. BevyCamera (via SourceType, not directly exposed via PartType enum for add_part)
    let camera = ModulePartType::Source(SourceType::BevyCamera {
        mode: BevyCameraMode::default(),
        fov: 60.0,
        active: true,
    });
    let (ins, outs) = camera.get_default_sockets();
    assert_eq!(ins.len(), 1);
    assert_eq!(ins[0].name, "Trigger In");
    assert_eq!(outs.len(), 1);
    assert_eq!(outs[0].name, "Media Out");

    // 3. BevyParticles
    let particles = ModulePartType::Source(SourceType::BevyParticles {
        rate: 10.0,
        lifetime: 1.0,
        speed: 1.0,
        color_start: [1.0; 4],
        color_end: [1.0; 4],
        position: [0.0; 3],
        rotation: [0.0; 3],
    });
    let (ins, outs) = particles.get_default_sockets();
    assert_eq!(ins[0].name, "Spawn Trigger"); // Special name for particles
    assert_eq!(outs[0].name, "Media Out");
}

#[test]
fn test_hue_mapping_mode_serialization() {
    let mode = HueMappingMode::Spatial;
    let serialized = serde_json::to_string(&mode).unwrap();
    let deserialized: HueMappingMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(mode, deserialized);

    let mode = HueMappingMode::Trigger;
    let serialized = serde_json::to_string(&mode).unwrap();
    let deserialized: HueMappingMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(mode, deserialized);
}

#[test]
fn test_next_part_id_default() {
    // Create JSON for MapFlowModule missing "next_part_id"
    let json = r#"{
        "id": 1,
        "name": "Test",
        "color": [1.0, 1.0, 1.0, 1.0],
        "parts": [],
        "connections": [],
        "playback_mode": "LoopUntilManualSwitch"
    }"#;

    let module: MapFlowModule = serde_json::from_str(json).expect("Deserialization failed");
    assert_eq!(module.next_part_id, 1);
}
