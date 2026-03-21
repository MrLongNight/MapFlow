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
            if src_outputs.iter().all(|s| s.id != conn.from_socket) {
                issues.push(ModuleIssue {
                    severity: IssueSeverity::Error,
                    message: format!("Connection #{} references invalid socket index {} on Source Part {} (max {})",
                        idx, conn.from_socket, src.id, src_outputs.len().saturating_sub(1)),
                    part_id: Some(src.id),
                });
            }

            let (dst_inputs, _) = dst.compute_sockets();
            if dst_inputs.iter().all(|s| s.id != conn.to_socket) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::{MapFlowModule, ModulePlaybackMode, PartType};

    #[test]
    fn test_check_module_integrity_invalid_from_part() {
        let mut module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [0.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        // Add a connection with an invalid from_part and to_part
        module.connections.push(crate::module::ModuleConnection {
            from_part: 999,
            from_socket: "0".to_string(),
            to_part: 1000,
            to_socket: "0".to_string(),
        });

        let issues = check_module_integrity(&module);
        assert_eq!(issues.len(), 2); // missing from and to parts
        assert_eq!(issues[0].severity, IssueSeverity::Error);
        assert!(issues[0].message.contains("invalid FROM Part ID"));
    }

    #[test]
    fn test_check_module_integrity_unconnected_output() {
        let mut module = MapFlowModule {
            id: 1,
            name: "Test2".to_string(),
            color: [0.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        // Add an output part using the builder
        module.add_part(PartType::Output, (0.0, 0.0));

        let issues = check_module_integrity(&module);
        assert_eq!(issues.len(), 1); // 1 Warning for disconnected output
        assert_eq!(issues[0].severity, IssueSeverity::Warning);
        assert!(issues[0].message.contains("Output Node is not connected"));
    }

    #[test]
    fn test_check_module_integrity_empty_source_path() {
        let mut module = MapFlowModule {
            id: 1,
            name: "Test3".to_string(),
            color: [0.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        // Add an empty source node
        module.add_part(PartType::Source, (0.0, 0.0));

        let issues = check_module_integrity(&module);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, IssueSeverity::Warning);
        assert!(issues[0].message.contains("no file selected"));
    }
}
