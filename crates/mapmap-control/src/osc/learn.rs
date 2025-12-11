//! OSC Learn Mode
//!
//! This module provides functionality for an OSC "learn" mode, where the application
//! can automatically map incoming OSC messages to control targets.

use crate::osc::address::parse_osc_address;
use crate::ControlTarget;
use rosc::{OscMessage, OscPacket};
use std::sync::{Arc, Mutex};

/// OSC learn state
#[derive(Debug, Clone, Default)]
pub struct OscLearn {
    active: Arc<Mutex<bool>>,
    last_address: Arc<Mutex<Option<String>>>,
}

impl OscLearn {
    /// Create a new `OscLearn` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether learn mode is active.
    pub fn set_active(&self, active: bool) {
        *self.active.lock().unwrap() = active;
    }

    /// Check if learn mode is active.
    pub fn is_active(&self) -> bool {
        *self.active.lock().unwrap()
    }

    /// Get the last learned OSC address.
    pub fn last_address(&self) -> Option<String> {
        self.last_address.lock().unwrap().clone()
    }

    /// Process an OSC packet. If learn mode is active, it captures the address
    /// of the first valid message and deactivates itself.
    ///
    /// Returns `true` if the packet was consumed by learn mode.
    pub fn process_packet(&self, packet: &OscPacket) -> bool {
        if !self.is_active() {
            return false;
        }

        if let OscPacket::Message(msg) = packet {
            // Check if the address is a valid control target
            if parse_osc_address(&msg.addr).is_ok() {
                *self.last_address.lock().unwrap() = Some(msg.addr.clone());
                self.set_active(false); // Deactivate after learning
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rosc::{OscMessage, OscPacket, OscType};

    #[test]
    fn test_osc_learn_activation() {
        let learn = OscLearn::new();
        assert!(!learn.is_active());
        learn.set_active(true);
        assert!(learn.is_active());
    }

    #[test]
    fn test_osc_learn_process_packet() {
        let learn = OscLearn::new();
        learn.set_active(true);

        let msg = OscMessage {
            addr: "/mapmap/layer/1/opacity".to_string(),
            args: vec![OscType::Float(0.5)],
        };
        let packet = OscPacket::Message(msg);

        assert!(learn.process_packet(&packet));
        assert!(!learn.is_active());
        assert_eq!(
            learn.last_address(),
            Some("/mapmap/layer/1/opacity".to_string())
        );
    }
}
