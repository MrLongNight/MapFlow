use crate::module::{MapFlowModule, ModulePartType};

#[derive(Debug, Clone)]
pub struct ModuleIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub part_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

pub fn check_module_integrity(module: &MapFlowModule) -> Vec<ModuleIssue> {
    let mut issues = Vec::new();

    // 1. Check connections validity (Topology)
    for (idx, conn) in module.connections.iter().enumerate() {
        let from_part = module.parts.iter().find(|p| p.id == conn.from_part);
        let to_part = module.parts.iter().find(|p| p.id == conn.to_part);

        if from_part.is_none() {
            issues.push(ModuleIssue {
                severity: IssueSeverity::Error,
                message: format!(
                    "Connection #{} has invalid FROM Part ID {}",
                    idx, conn.from_part
                ),
                part_id: None,
            });
        }
        if to_part.is_none() {
            issues.push(ModuleIssue {
                severity: IssueSeverity::Error,
                message: format!(
                    "Connection #{} has invalid TO Part ID {}",
                    idx, conn.to_part
                ),
                part_id: None,
            });
        }

        if let (Some(src), Some(dst)) = (from_part, to_part) {
            // Check socket bounds
            let (_src_inputs, src_outputs) = src.compute_sockets();
            if conn.from_socket >= src_outputs.len() {
                issues.push(ModuleIssue {
                    severity: IssueSeverity::Error,
                    message: format!("Connection #{} references invalid socket index {} on Source Part {} (max {})", 
                        idx, conn.from_socket, src.id, src_outputs.len().saturating_sub(1)),
                    part_id: Some(src.id),
                });
            }

            let (dst_inputs, _) = dst.compute_sockets();
            if conn.to_socket >= dst_inputs.len() {
                issues.push(ModuleIssue {
                    severity: IssueSeverity::Error,
                    message: format!("Connection #{} references invalid socket index {} on Target Part {} (max {})", 
                        idx, conn.to_socket, dst.id, dst_inputs.len().saturating_sub(1)),
                    part_id: Some(dst.id),
                });
            }
        }
    }

    // 2. Check Parts (Nodes)
    for part in &module.parts {
        match &part.part_type {
            ModulePartType::Layer(layer_type) => {
                // Verify Layer state
                // e.g. check if mesh looks reasonable (not all zeros?)
                match layer_type {
                    crate::module::LayerType::Single { .. }
                    | crate::module::LayerType::Group { .. } => {
                        // Basic mesh validation could go here
                    }
                    crate::module::LayerType::All { .. } => {
                        // Master Layer
                    }
                }
            }
            ModulePartType::Output(_) => {
                // Warning if disconnected
                let is_connected = module.connections.iter().any(|c| c.to_part == part.id);
                if !is_connected {
                    issues.push(ModuleIssue {
                        severity: IssueSeverity::Warning,
                        message: "Output Node is not connected to any Input (expects Layer)."
                            .to_string(),
                        part_id: Some(part.id),
                    });
                }
            }
            ModulePartType::Source(source) => if let crate::module::SourceType::MediaFile { path, .. } = source {
                if path.is_empty() {
                    issues.push(ModuleIssue {
                        severity: IssueSeverity::Warning,
                        message: "Source Node has no file selected.".to_string(),
                        part_id: Some(part.id),
                    });
                }
            },
            _ => {}
        }
    }

    issues
}
