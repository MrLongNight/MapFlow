use vorce_core::{Layer, OutputConfig, Transform};

/// Represents the current selection context for the inspector
pub enum InspectorContext<'a> {
    /// No selection
    None,
    /// A layer is selected
    Layer {
        layer: &'a Layer,
        transform: &'a Transform,
        index: usize,
        first_mapping: Option<&'a vorce_core::mapping::Mapping>,
    },
    /// An output is selected
    Output(&'a OutputConfig),
    /// A module part is selected
    Module {
        canvas: &'a mut crate::editors::module_canvas::state::ModuleCanvas,
        module: &'a mut vorce_core::module::MapFlowModule,
        part_id: vorce_core::module::ModulePartId,
        shared_media_ids: Vec<String>,
    },
}

/// Actions that can be triggered from the Inspector
#[derive(Debug, Clone)]
pub enum InspectorAction {
    /// Update layer transform
    UpdateTransform(u64, Transform),
    /// Update layer opacity
    UpdateOpacity(u64, f32),
    /// Update the mesh of a mapping
    UpdateMappingMesh(u64, vorce_core::Mesh),
    /// Request to close the inspector panel
    RequestClose,
}
