use crate::module::{
    MeshType, ModulePartId, ModulePartType, ModulizerType, SourceType, VorceModule,
};
use crate::module_eval::evaluator::RenderOp;
use crate::module_eval::types::{
    primary_render_connection_idx, ModuleGraphIndices, SourceProperties,
};
use crate::module_eval::ModuleEvaluator;

impl ModuleEvaluator {
    pub(crate) fn trace_chain_into(
        &self,
        start_node_id: ModulePartId,
        module: &VorceModule,
        op: &mut RenderOp,
        default_mesh: &MeshType,
        indices: &ModuleGraphIndices,
    ) {
        op.effects.clear();
        op.masks.clear();
        op.source_part_id = None;
        op.source_props = SourceProperties::default_identity();

        let mut override_mesh = None;
        let mut current_id = start_node_id;

        // Cycle detection
        let mut visited = std::collections::HashSet::with_capacity(16);
        visited.insert(start_node_id);

        // Optimization: Use the part index cache that was already built in evaluate()
        // This avoids an O(N) allocation and iteration for every layer being rendered.
        let _part_index = &indices.part_index_cache;

        tracing::debug!(
            "trace_chain: Starting from node {} in module {}",
            start_node_id,
            module.name
        );

        let trigger_values = &self.cached_result.trigger_values;

        // Safety limit to prevent infinite loops in cyclic graphs
        for _iteration in 0..50 {
            // Apply Trigger Targets for the current node
            // We need to find if any input sockets have triggers active and targets mapped
            if let Some(&part_idx) = indices.part_index_cache.get(&current_id) {
                let part = &module.parts[part_idx];

                // If this is a source, load its base properties first
                if let ModulePartType::Source(source_type) = &part.part_type {
                    op.source_part_id = Some(part.id);
                    if let SourceType::MediaFile {
                        opacity,
                        brightness,
                        contrast,
                        saturation,
                        hue_shift,
                        scale_x,
                        scale_y,
                        rotation,
                        offset_x,
                        offset_y,
                        flip_horizontal,
                        flip_vertical,
                        ..
                    } = source_type
                    {
                        op.source_props = SourceProperties {
                            opacity: *opacity,
                            brightness: *brightness,
                            contrast: *contrast,
                            saturation: *saturation,
                            hue_shift: *hue_shift,
                            scale_x: *scale_x,
                            scale_y: *scale_y,
                            rotation: *rotation,
                            offset_x: *offset_x,
                            offset_y: *offset_y,
                            flip_horizontal: *flip_horizontal,
                            flip_vertical: *flip_vertical,
                        };
                    }
                }

                if !part.trigger_targets.is_empty() {
                    tracing::debug!(
                        "Part {} has {} trigger targets",
                        part.id,
                        part.trigger_targets.len()
                    );
                }
                for (target_idx, config) in &part.trigger_targets {
                    // Mappable socket index in the part's input vector
                    let socket_idx = *target_idx;
                    let socket_id = if let Some(s) = part.inputs.get(socket_idx) {
                        &s.id
                    } else {
                        continue;
                    };

                    // Find connection to this socket
                    let mut trigger_val = 0.0;
                    if let Some(conn_indices) = indices.conn_index_cache.get(&current_id) {
                        for &conn_idx in conn_indices {
                            let conn = &module.connections[conn_idx];
                            if conn.to_socket == *socket_id {
                                if let Some(from_values) = trigger_values.get(&conn.from_part) {
                                    // Look up the value by finding the source socket ID's index
                                    if let Some(&part_idx) =
                                        indices.part_index_cache.get(&conn.from_part)
                                    {
                                        let src_part = &module.parts[part_idx];
                                        if let Some(idx) = src_part
                                            .outputs
                                            .iter()
                                            .position(|s| s.id == conn.from_socket)
                                        {
                                            if let Some(val) = from_values.get(idx) {
                                                trigger_val = *val;
                                            }
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                    let val = self.apply_smoothing(
                        part.id,
                        socket_idx,
                        config.apply(trigger_val),
                        &config.mode,
                    );
                    match &config.target {
                        crate::module::TriggerTarget::Opacity => op.source_props.opacity = val,
                        crate::module::TriggerTarget::Brightness => {
                            op.source_props.brightness = val
                        }
                        crate::module::TriggerTarget::Contrast => op.source_props.contrast = val,
                        crate::module::TriggerTarget::Saturation => {
                            op.source_props.saturation = val
                        }
                        crate::module::TriggerTarget::HueShift => op.source_props.hue_shift = val,
                        crate::module::TriggerTarget::ScaleX => op.source_props.scale_x = val,
                        crate::module::TriggerTarget::ScaleY => op.source_props.scale_y = val,
                        crate::module::TriggerTarget::Rotation => op.source_props.rotation = val,
                        crate::module::TriggerTarget::OffsetX => op.source_props.offset_x = val,
                        crate::module::TriggerTarget::OffsetY => op.source_props.offset_y = val,
                        crate::module::TriggerTarget::FlipH => {
                            op.source_props.flip_horizontal = val > 0.5
                        }
                        crate::module::TriggerTarget::FlipV => {
                            op.source_props.flip_vertical = val > 0.5
                        }
                        crate::module::TriggerTarget::Param(name) => {
                            if let Some(ModulizerType::Effect { params, .. }) =
                                op.effects.last_mut()
                            {
                                params.insert(name.clone(), val);
                            }
                        }
                        _ => {}
                    }
                }
            }

            // 2. Find PREVIOUS node in chain
            if let Some(conn_idx) = primary_render_connection_idx(module, indices, current_id) {
                let conn = &module.connections[conn_idx];

                // Cycle detection
                if !visited.insert(conn.from_part) {
                    tracing::warn!(
                        "Cycle detected in module graph chain starting at node {}",
                        start_node_id
                    );
                    break;
                }

                if let Some(&part_idx) = indices.part_index_cache.get(&conn.from_part) {
                    let part = &module.parts[part_idx];
                    match &part.part_type {
                        ModulePartType::Source(source_type) => {
                            op.source_part_id = Some(part.id);

                            // Helper to extract props from any source variant that has them
                            let mut extracted_props = None;

                            match source_type {
                                SourceType::MediaFile {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                }
                                | SourceType::VideoUni {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                }
                                | SourceType::ImageUni {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                } => {
                                    extracted_props = Some(SourceProperties {
                                        opacity: *opacity,
                                        brightness: *brightness,
                                        contrast: *contrast,
                                        saturation: *saturation,
                                        hue_shift: *hue_shift,
                                        scale_x: *scale_x,
                                        scale_y: *scale_y,
                                        rotation: *rotation,
                                        offset_x: *offset_x,
                                        offset_y: *offset_y,
                                        flip_horizontal: *flip_horizontal,
                                        flip_vertical: *flip_vertical,
                                    });
                                }
                                SourceType::VideoMulti {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                }
                                | SourceType::ImageMulti {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                } => {
                                    extracted_props = Some(SourceProperties {
                                        opacity: *opacity,
                                        brightness: *brightness,
                                        contrast: *contrast,
                                        saturation: *saturation,
                                        hue_shift: *hue_shift,
                                        scale_x: *scale_x,
                                        scale_y: *scale_y,
                                        rotation: *rotation,
                                        offset_x: *offset_x,
                                        offset_y: *offset_y,
                                        flip_horizontal: *flip_horizontal,
                                        flip_vertical: *flip_vertical,
                                    });
                                }
                                _ => {}
                            }

                            if let Some(mut props) = extracted_props {
                                // Re-apply overrides
                                for (target_idx, config) in &part.trigger_targets {
                                    let socket_idx = *target_idx;
                                    let socket_id = if let Some(s) = part.inputs.get(socket_idx) {
                                        &s.id
                                    } else {
                                        continue;
                                    };

                                    // Find connection to this socket
                                    let mut trigger_val = 0.0;
                                    if let Some(conn_indices) =
                                        indices.conn_index_cache.get(&part.id)
                                    {
                                        for &conn_idx in conn_indices {
                                            let conn = &module.connections[conn_idx];
                                            if conn.to_socket == *socket_id {
                                                if let Some(from_values) =
                                                    trigger_values.get(&conn.from_part)
                                                {
                                                    if let Some(&src_idx) = indices
                                                        .part_index_cache
                                                        .get(&conn.from_part)
                                                    {
                                                        let src_part = &module.parts[src_idx];
                                                        if let Some(idx) = src_part
                                                            .outputs
                                                            .iter()
                                                            .position(|s| s.id == conn.from_socket)
                                                        {
                                                            if let Some(val) = from_values.get(idx)
                                                            {
                                                                trigger_val = *val;
                                                            }
                                                        }
                                                    }
                                                }
                                                break;
                                            }
                                        }
                                    }

                                    // Apply config if value is significant
                                    match &config.target {
                                        crate::module::TriggerTarget::None => {}
                                        target => {
                                            // Apply mapping
                                            let raw_final_val = config.apply(trigger_val);
                                            let final_val = self.apply_smoothing(
                                                part.id,
                                                socket_idx,
                                                raw_final_val,
                                                &config.mode,
                                            );

                                            match target {
                                                crate::module::TriggerTarget::Opacity => {
                                                    props.opacity = final_val;
                                                }
                                                crate::module::TriggerTarget::Brightness => {
                                                    props.brightness = final_val;
                                                }
                                                crate::module::TriggerTarget::Contrast => {
                                                    props.contrast = final_val;
                                                }
                                                crate::module::TriggerTarget::Saturation => {
                                                    props.saturation = final_val;
                                                }
                                                crate::module::TriggerTarget::HueShift => {
                                                    props.hue_shift = final_val;
                                                }
                                                crate::module::TriggerTarget::ScaleX => {
                                                    props.scale_x = final_val;
                                                }
                                                crate::module::TriggerTarget::ScaleY => {
                                                    props.scale_y = final_val;
                                                }
                                                crate::module::TriggerTarget::Rotation => {
                                                    props.rotation = final_val;
                                                }
                                                crate::module::TriggerTarget::OffsetX => {
                                                    props.offset_x = final_val;
                                                }
                                                crate::module::TriggerTarget::OffsetY => {
                                                    props.offset_y = final_val;
                                                }
                                                crate::module::TriggerTarget::FlipH => {
                                                    props.flip_horizontal = final_val > 0.5;
                                                }
                                                crate::module::TriggerTarget::FlipV => {
                                                    props.flip_vertical = final_val > 0.5;
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }

                                op.source_props = props;
                            }
                            break;
                        }
                        ModulePartType::Modulizer(mod_type) => {
                            op.effects.push(mod_type.clone());
                            current_id = part.id;
                        }
                        ModulePartType::Mask(mask_type) => {
                            op.masks.push(mask_type.clone());
                            current_id = part.id;
                        }
                        ModulePartType::Mesh(mesh_type) => {
                            if override_mesh.is_none() {
                                override_mesh = Some(mesh_type.clone());
                            }
                            current_id = part.id;
                        }
                        _ => {
                            break;
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        op.effects.reverse();
        op.masks.reverse();

        op.mesh = override_mesh.unwrap_or_else(|| default_mesh.clone());
    }
}
