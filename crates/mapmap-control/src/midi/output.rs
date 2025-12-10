//! MIDI output handling

use super::MidiMessage;
use crate::error::{ControlError, Result};
use midir::{MidiOutput as MidirOutput, MidiOutputConnection};
use tracing::{info, warn};

/// MIDI output handler
pub struct MidiOutputHandler {
    connection: Option<MidiOutputConnection>,
    /// Track whether MIDI backend is available on this system
    backend_available: bool,
}

impl MidiOutputHandler {
    /// Create a new MIDI output handler
    ///
    /// Returns Ok even if no MIDI backend is available (e.g., in CI environments).
    /// In this case, operations will return appropriate errors.
    pub fn new() -> Result<Self> {
        // Check if MIDI backend is available
        let backend_available = match MidirOutput::new("MapMap MIDI Output") {
            Ok(_) => true,
            Err(e) => {
                warn!(
                    "No MIDI backend available: {:?}. MIDI output will be disabled.",
                    e
                );
                false
            }
        };

        Ok(Self {
            connection: None,
            backend_available,
        })
    }

    /// List available MIDI output ports
    ///
    /// Returns an empty list if no MIDI backend is available.
    pub fn list_ports() -> Result<Vec<String>> {
        let midi_output = match MidirOutput::new("MapMap MIDI Output") {
            Ok(mo) => mo,
            Err(e) => {
                warn!("No MIDI backend available: {:?}", e);
                return Ok(Vec::new());
            }
        };

        let mut ports = Vec::new();
        for port in midi_output.ports() {
            if let Ok(name) = midi_output.port_name(&port) {
                ports.push(name);
            }
        }

        Ok(ports)
    }

    /// Connect to a MIDI output port by index
    pub fn connect(&mut self, port_index: usize) -> Result<()> {
        // Check if MIDI backend is available
        if !self.backend_available {
            return Err(ControlError::MidiError(
                "No MIDI backend available on this system".to_string(),
            ));
        }

        // Disconnect existing connection if any
        self.disconnect();

        let midi_output = match MidirOutput::new("MapMap MIDI Output") {
            Ok(mo) => mo,
            Err(e) => {
                return Err(ControlError::MidiError(format!(
                    "Failed to create MIDI output: {:?}",
                    e
                )));
            }
        };
        let ports = midi_output.ports();

        if port_index >= ports.len() {
            return Err(ControlError::InvalidParameter(format!(
                "Port index {} out of range (max: {})",
                port_index,
                ports.len()
            )));
        }

        let port = &ports[port_index];
        let port_name = midi_output
            .port_name(port)
            .unwrap_or_else(|_| "Unknown".to_string());

        info!("Connecting to MIDI output port: {}", port_name);

        let connection = midi_output
            .connect(port, "mapmap-output")
            .map_err(|e| ControlError::MidiError(format!("Connection failed: {:?}", e)))?;

        self.connection = Some(connection);

        info!("Successfully connected to MIDI output: {}", port_name);

        Ok(())
    }

    /// Disconnect from MIDI output
    pub fn disconnect(&mut self) {
        if let Some(connection) = self.connection.take() {
            drop(connection);
            info!("Disconnected from MIDI output");
        }
    }

    /// Send a MIDI message
    pub fn send_message(&mut self, message: &MidiMessage) -> Result<()> {
        if let Some(connection) = &mut self.connection {
            let bytes = message.to_bytes();
            connection.send(&bytes)?;
            Ok(())
        } else {
            Err(ControlError::MidiError("Not connected".to_string()))
        }
    }

    /// Send raw MIDI bytes
    pub fn send_raw(&mut self, bytes: &[u8]) -> Result<()> {
        if let Some(connection) = &mut self.connection {
            connection.send(bytes)?;
            Ok(())
        } else {
            Err(ControlError::MidiError("Not connected".to_string()))
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
}

impl Drop for MidiOutputHandler {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_ports() {
        let result = MidiOutputHandler::list_ports();
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_handler() {
        let handler = MidiOutputHandler::new();
        assert!(handler.is_ok());
    }
}
