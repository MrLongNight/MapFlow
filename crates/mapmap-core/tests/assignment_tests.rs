use mapmap_core::{Assignment, AssignmentManager, ControlSource, ControlTarget};
use uuid::Uuid;

#[test]
fn test_assignment_creation() {
    let source = ControlSource::Midi {
        channel: 0,
        note: 60,
    };
    let target = ControlTarget::LayerOpacity { layer_id: 1 };
    let assignment = Assignment::new(source.clone(), target.clone());

    assert_eq!(assignment.source, source);
    assert_eq!(assignment.target, target);
    assert!(assignment.enabled);
    // UUID should be random/unique (not nil)
    assert_ne!(assignment.id, Uuid::nil());
}

#[test]
fn test_assignment_manager_crud() {
    let mut manager = AssignmentManager::new();
    assert!(manager.assignments().is_empty());

    let a1 = Assignment::new(
        ControlSource::Osc {
            address: "/test/1".into(),
        },
        ControlTarget::LayerOpacity { layer_id: 1 },
    );
    let id1 = a1.id;

    manager.add(a1);
    assert_eq!(manager.assignments().len(), 1);

    let a2 = Assignment::new(
        ControlSource::Midi {
            channel: 1,
            note: 10,
        },
        ControlTarget::EffectParamF32 {
            layer_id: 1,
            effect_id: Uuid::new_v4(),
            param_name: "Speed".into(),
        },
    );
    let id2 = a2.id;
    manager.add(a2);
    assert_eq!(manager.assignments().len(), 2);

    manager.remove(id1);
    assert_eq!(manager.assignments().len(), 1);
    assert_eq!(manager.assignments()[0].id, id2);

    manager.remove(id2);
    assert!(manager.assignments().is_empty());
}

#[test]
fn test_assignment_serialization() {
    let mut manager = AssignmentManager::new();
    manager.add(Assignment::new(
        ControlSource::Dmx {
            universe: 1,
            channel: 1,
        },
        ControlTarget::LayerOpacity { layer_id: 99 },
    ));

    let json = serde_json::to_string(&manager).unwrap();
    let deserialized: AssignmentManager = serde_json::from_str(&json).unwrap();

    assert_eq!(manager, deserialized);
}

#[test]
fn test_control_source_variants() {
    let midi = ControlSource::Midi {
        channel: 1,
        note: 64,
    };
    let osc = ControlSource::Osc {
        address: "/test".into(),
    };
    let dmx = ControlSource::Dmx {
        universe: 0,
        channel: 255,
    };

    // Ensure equality works
    assert_eq!(
        midi,
        ControlSource::Midi {
            channel: 1,
            note: 64
        }
    );
    assert_ne!(midi, osc);
    assert_ne!(osc, dmx);
}
