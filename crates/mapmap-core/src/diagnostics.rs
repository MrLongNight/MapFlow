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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::{
        MapFlowModule, ModuleConnection, ModulePartType, ModulePlaybackMode,
        PartType, SourceType,
    };

    fn create_empty_module() -> MapFlowModule {
        MapFlowModule {
            id: 1,
            name: "Test Module".to_string(),
            color: [1.0; 4],
            parts: Vec::new(),
            connections: Vec::new(),
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        }
    }

    #[test]
    fn test_valid_module_no_issues() {
        let mut module = create_empty_module();

        // Create valid graph: Source -> Layer -> Output
        let src_id = module.add_part(PartType::Source, (0.0, 0.0));
        // Fix source path so it doesn't warn
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == src_id) {
             if let ModulePartType::Source(SourceType::MediaFile { path, .. }) = &mut part.part_type {
                 *path = "valid.mp4".to_string();
             }
        }

        let layer_id = module.add_part(PartType::Layer, (100.0, 0.0));
        let out_id = module.add_part(PartType::Output, (200.0, 0.0));

        // Source(0) -> Layer(0)
        module.add_connection(src_id, 0, layer_id, 0);
        // Layer(0) -> Output(0)
        module.add_connection(layer_id, 0, out_id, 0);

        let issues = check_module_integrity(&module);
        assert!(issues.is_empty(), "Expected no issues, found {:?}", issues);
    }

    #[test]
    fn test_invalid_connection_topology() {
        let mut module = create_empty_module();

        // Add connection referring to non-existent parts
        module.connections.push(ModuleConnection {
            from_part: 999,
            from_socket: 0,
            to_part: 888,
            to_socket: 0,
        });

        let issues = check_module_integrity(&module);
        assert!(!issues.is_empty());

        // Should have errors for missing parts
        let errors = issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();
        assert!(errors >= 1);
    }

    #[test]
    fn test_invalid_socket_index() {
        let mut module = create_empty_module();
        let src_id = module.add_part(PartType::Source, (0.0, 0.0));
        let layer_id = module.add_part(PartType::Layer, (100.0, 0.0));

        // Fix source path
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == src_id) {
             if let ModulePartType::Source(SourceType::MediaFile { path, .. }) = &mut part.part_type {
                 *path = "valid.mp4".to_string();
             }
        }

        // Source has 1 output (index 0). Try connecting index 5.
        module.connections.push(ModuleConnection {
            from_part: src_id,
            from_socket: 5,
            to_part: layer_id,
            to_socket: 0,
        });

        let issues = check_module_integrity(&module);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == IssueSeverity::Error).collect();

        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("references invalid socket index 5"));
    }

    #[test]
    fn test_disconnected_output_warning() {
        let mut module = create_empty_module();
        // Add just an Output
        module.add_part(PartType::Output, (0.0, 0.0));

        let issues = check_module_integrity(&module);
        let warnings: Vec<_> = issues.iter().filter(|i| i.severity == IssueSeverity::Warning).collect();

        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("not connected"));
    }

    #[test]
    fn test_empty_source_path_warning() {
        let mut module = create_empty_module();
        // Add Source (default path is empty)
        module.add_part(PartType::Source, (0.0, 0.0));

        let issues = check_module_integrity(&module);
        let warnings: Vec<_> = issues.iter().filter(|i| i.severity == IssueSeverity::Warning).collect();

        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("no file selected"));
    }
}
