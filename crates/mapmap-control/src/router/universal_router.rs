//! Universal Trigger Router
//!
//! Handles routing of triggers from various sources (MIDI, OSC, GPIO, etc.)
//! to timeline actions for the Trackline Mode.

use crate::target::{ControlTarget, ControlValue};
use std::collections::HashMap;

/// Source of a trigger
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TriggerSource {
    MidiNote(u8, u8),   // Channel, Note
    MidiCc(u8, u8),     // Channel, Controller
    OscAddress(String), // Path
    GpioPin(u8),        // Pin Number
    Custom(String),     // Internal or generic action
}

/// A mapping from an incoming source event to an action target
#[derive(Debug, Clone)]
pub struct TriggerRoute {
    pub target: ControlTarget,
    pub expected_value: Option<ControlValue>, // Optional condition
}

/// Universal Trigger Router for Trackline mode
#[derive(Debug, Default)]
pub struct UniversalTriggerRouter {
    /// Configured routes mapping arbitrary sources to control targets
    pub routes: HashMap<TriggerSource, TriggerRoute>,
}

impl UniversalTriggerRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Add a route from a source to a target
    pub fn add_route(&mut self, source: TriggerSource, route: TriggerRoute) {
        self.routes.insert(source, route);
    }

    /// Remove a route for a source
    pub fn remove_route(&mut self, source: &TriggerSource) {
        self.routes.remove(source);
    }

    /// Process an incoming target request and potentially map it to a new timeline target.
    /// In the Mapflow `ControlManager` architecture, incoming hardware events are already
    /// mapped to `ControlTarget` and `ControlValue`. The router can intercept these and
    /// emit Timeline specific targets if they match routing rules, or return them as-is.
    pub fn route_control(
        &self,
        target: ControlTarget,
        value: ControlValue,
    ) -> (ControlTarget, ControlValue) {
        let source = match &target {
            ControlTarget::Custom(name) => Some(TriggerSource::Custom(name.clone())),
            ControlTarget::GpioPin(pin) => Some(TriggerSource::GpioPin(*pin)),
            _ => None,
        };

        if let Some(src) = source {
            if let Some(route) = self.routes.get(&src) {
                // If the route expects a specific value, check it
                if let Some(expected) = &route.expected_value {
                    if *expected != value {
                        return (target, value);
                    }
                }

                // Route matches, substitute target
                return (route.target.clone(), value);
            }
        }

        // Default: return unchanged
        (target, value)
    }
}
