//!
//! Connection handling and definition.
//!

use crate::module::types::module::ModulePartId;
use serde::{Deserialize, Serialize};

/// Represents a connection between two modules/parts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConnection {
    /// Source part ID.
    pub from_part: ModulePartId,
    /// Source socket ID string.
    pub from_socket: String,
    /// Target part ID.
    pub to_part: ModulePartId,
    /// Target socket ID string.
    pub to_socket: String,
}
