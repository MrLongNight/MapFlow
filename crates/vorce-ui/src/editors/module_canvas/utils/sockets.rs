use vorce_core::module::{ModulePartType, ModuleSocket};

pub fn get_sockets_for_part_type(
    part_type: &ModulePartType,
) -> (Vec<ModuleSocket>, Vec<ModuleSocket>) {
    let temp_part = vorce_core::module::ModulePart {
        id: 0,
        part_type: part_type.clone(),
        position: (0.0, 0.0),
        size: None,
        link_data: vorce_core::module::NodeLinkData::default(),
        inputs: vec![],
        outputs: vec![],
        trigger_targets: std::collections::HashMap::new(),
    };
    temp_part.compute_sockets()
}
