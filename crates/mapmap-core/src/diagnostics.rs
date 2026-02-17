//! Diagnostics - Module Integrity Checking
//!
//! This module provides tools to validate module connections, detect broken links,
//! and report issues (errors/warnings) to the user.
//!
//! # Features
//!
//! - **ModuleIssue**: Represents a detected problem (Error, Warning, Info).
//! - **check_module_integrity**: Main function to validate a `MapFlowModule`.

use crate::module::{MapFlowModule, ModulePartType};

/// Represents an issue found within a module
#[derive(Debug, Clone)]
pub struct ModuleIssue {
    /// Severity level of the issue
    pub severity: IssueSeverity,
    /// Human-readable description
    pub message: String,
    /// ID of the part related to the issue (if any)
    pub part_id: Option<u64>,
}

/// Severity level of a diagnostic issue
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IssueSeverity {
    /// Critical error that prevents proper functioning
    Error,
    /// Potential issue or suboptimal configuration
    Warning,
    /// Informational message
    Info,
}

/// Check a module for structural integrity and logical errors
///
/// This performs multiple checks:
/// 1. Connection validity (dangling references, out-of-bounds sockets)
/// 2. Part configuration (missing files, disconnected outputs)
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
            ModulePartType::Source(crate::module::SourceType::MediaFile { path, .. }) => {
                if path.is_empty() {
                    issues.push(ModuleIssue {
                        severity: IssueSeverity::Warning,
                        message: "Source Node has no file selected.".to_string(),
                        part_id: Some(part.id),
                    });
                }
            }
            _ => {}
        }
    }

    issues
}




