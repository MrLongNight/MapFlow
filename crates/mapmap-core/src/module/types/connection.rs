//!
//! Connection handling and definition.
//!

use crate::module::types::module::ModulePartId;
use serde::{Deserialize, Serialize};

/// Represents a connection between two modules/parts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConnection {
    /// Component property or field.
    pub from_part: ModulePartId,
    /// Component property or field.
    pub from_socket: usize,
    /// Component property or field.
    pub to_part: ModulePartId,
    /// Component property or field.
    pub to_socket: usize,
}
