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
    #[serde(deserialize_with = "deserialize_socket_id")]
    pub from_socket: String,
    /// Component property or field.
    pub to_part: ModulePartId,
    /// Component property or field.
    #[serde(deserialize_with = "deserialize_socket_id")]
    pub to_socket: String,
}

fn deserialize_socket_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct SocketIdVisitor;

    impl<'de> serde::de::Visitor<'de> for SocketIdVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or an integer")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.to_owned())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.to_string())
        }
    }

    deserializer.deserialize_any(SocketIdVisitor)
}
