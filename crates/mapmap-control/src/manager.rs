//! Unified control system manager
//!
//! This module provides a unified interface for managing all control systems
//! (MIDI, OSC, DMX, Web API, Cue system, and keyboard shortcuts).
//!
//! Refactored to remove legacy learn modes and use simplified mapping.

use crate::error::{ControlError, Result};
use crate::shortcuts::{Action, Key, KeyBindings, Modifiers};
use crate::target::{ControlTarget, ControlValue};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

#[cfg(feature = "midi")]
use crate::midi::MidiInputHandler;

use crate::cue::CueList;
use crate::dmx::{ArtNetSender, SacnSender};

#[cfg(feature = "osc")]
use crate::osc::{OscClient, OscMapping, OscServer};

/// Unified control system manager
pub struct ControlManager {
    #[cfg(feature = "midi")]
    pub midi_input: Option<MidiInputHandler>,

    #[cfg(feature = "osc")]
    pub osc_server: Option<OscServer>,
    #[cfg(feature = "osc")]
    pub osc_clients: Vec<OscClient>,
    #[cfg(feature = "osc")]
    pub osc_mapping: OscMapping,

    pub artnet_sender: Option<ArtNetSender>,
    pub sacn_sender: Option<SacnSender>,

    pub cue_list: CueList,
    pub key_bindings: KeyBindings,

    /// Event callback for control changes
    #[allow(clippy::type_complexity)]
    control_callback: Option<Arc<Mutex<dyn FnMut(ControlTarget, ControlValue) + Send>>>,
}

impl ControlManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "midi")]
            midi_input: None,

            #[cfg(feature = "osc")]
            osc_server: None,
            #[cfg(feature = "osc")]
            osc_clients: Vec::new(),
            #[cfg(feature = "osc")]
            osc_mapping: OscMapping::new(),

            artnet_sender: None,
            sacn_sender: None,

            cue_list: CueList::new(),
            key_bindings: KeyBindings::new(),

            control_callback: None,
        }
    }

    /// Set control change callback
    pub fn set_control_callback<F>(&mut self, callback: F)
    where
        F: FnMut(ControlTarget, ControlValue) + Send + 'static,
    {
        self.control_callback = Some(Arc::new(Mutex::new(callback)));
    }

    /// Initialize MIDI input (Robust)
    #[cfg(feature = "midi")]
    pub fn init_midi_input(&mut self) -> Result<()> {
        info!("Initializing MIDI input");
        match MidiInputHandler::new() {
            Ok(handler) => {
                self.midi_input = Some(handler);
                Ok(())
            }
            Err(e) => {
                warn!("MIDI input initialization failed: {}", e);
                // We return Ok even if MIDI fails, so the app can continue
                // Or we return Err and app handles it.
                // The prompt says "Fehlerrobustheit ... bspw. keine Panics".
                // Returning Err is fine if app handles it. But here we can just log warn.
                // However, the function returns Result, so caller expects it.
                // I will return Err but ensuring no panic.
                Err(e)
            }
        }
    }

    /// Initialize OSC server
    #[cfg(feature = "osc")]
    pub fn init_osc_server(&mut self, port: u16) -> Result<()> {
        info!("Initializing OSC server on port {}", port);
        match OscServer::new(port) {
            Ok(server) => {
                self.osc_server = Some(server);
                Ok(())
            }
            Err(e) => {
                warn!("OSC server initialization failed: {}", e);
                Err(e)
            }
        }
    }

    /// Add an OSC client for feedback.
    #[cfg(feature = "osc")]
    pub fn add_osc_client(&mut self, addr: &str) -> Result<()> {
        info!("Adding OSC client to {}", addr);
        let client = OscClient::new(addr)?;
        self.osc_clients.push(client);
        Ok(())
    }

    /// Remove an OSC client.
    #[cfg(feature = "osc")]
    pub fn remove_osc_client(&mut self, addr: &str) {
        self.osc_clients.retain(|c| c.destination_str() != addr);
    }

    /// Initialize Art-Net sender
    pub fn init_artnet(&mut self, universe: u16, target: &str) -> Result<()> {
        info!(
            "Initializing Art-Net sender for universe {} to {}",
            universe, target
        );
        match ArtNetSender::new(universe, target) {
            Ok(sender) => {
                self.artnet_sender = Some(sender);
                Ok(())
            }
            Err(e) => {
                warn!("Art-Net sender initialization failed: {}", e);
                Err(e)
            }
        }
    }

    /// Initialize sACN sender
    pub fn init_sacn(&mut self, universe: u16, source_name: &str) -> Result<()> {
        info!(
            "Initializing sACN sender for universe {} with source {}",
            universe, source_name
        );
        match SacnSender::new(universe, source_name) {
            Ok(sender) => {
                self.sacn_sender = Some(sender);
                Ok(())
            }
            Err(e) => {
                warn!("sACN sender initialization failed: {}", e);
                Err(e)
            }
        }
    }

    /// Update all control systems (call every frame)
    pub fn update(&mut self) {
        // Process MIDI messages
        #[cfg(feature = "midi")]
        self.process_midi_messages();

        // Process OSC messages
        #[cfg(feature = "osc")]
        self.process_osc_messages();

        // Update cue system
        self.cue_list.update();
    }

    /// Process MIDI messages
    #[cfg(feature = "midi")]
    fn process_midi_messages(&mut self) {
        // Collect messages to process to avoid borrow checker issues
        let mut controls_to_apply = Vec::new();

        if let Some(midi_input) = &self.midi_input {
            while let Some(message) = midi_input.poll_message() {
                // Learn mode removed. Directly map.

                // Get mapping and collect control values
                if let Some(mapping) = midi_input.get_mapping() {
                    if let Some((target, value)) = mapping.get_control_value(&message) {
                        controls_to_apply.push((target, value));
                    }
                }
            }
        }

        // Apply collected controls
        for (target, value) in controls_to_apply {
            self.apply_control(target, value);
        }
    }

    /// Process OSC messages
    #[cfg(feature = "osc")]
    fn process_osc_messages(&mut self) {
        let mut controls_to_apply = Vec::new();

        if let Some(osc_server) = &mut self.osc_server {
            while let Some(packet) = osc_server.poll_packet() {
                // Learn mode removed.

                // Try to map and apply the control
                if let rosc::OscPacket::Message(msg) = packet {
                    if let Some(target) = self.osc_mapping.get(&msg.addr) {
                        let value_result = match target {
                            ControlTarget::LayerPosition(_) => {
                                crate::osc::types::osc_to_vec2(&msg.args)
                            }
                            _ => crate::osc::types::osc_to_control_value(&msg.args),
                        };

                        if let Ok(value) = value_result {
                            controls_to_apply.push((target.clone(), value));
                        }
                    }
                }
            }
        }

        for (target, value) in controls_to_apply {
            self.apply_control(target, value);
        }
    }

    /// Validate control value for security issues (e.g. path traversal)
    fn validate_security(&self, target: &ControlTarget, value: &ControlValue) -> Result<()> {
        if let ControlValue::String(s) = value {
            // Check for path traversal attempts while avoiding false positives for text (e.g., "Loading...")
            // We block:
            // - Exact match ".."
            // - Start with "../" or "..\"
            // - Contains "/../" or "\..\" or mixed separators
            // - End with "/.." or "\.."
            if s == ".."
                || s.starts_with("../")
                || s.starts_with("..\\")
                || s.contains("/../")
                || s.contains("\\..\\")
                || s.contains("/..\\")
                || s.contains("\\../")
                || s.ends_with("/..")
                || s.ends_with("\\..")
            {
                // Get name for error message
                let name = match target {
                    ControlTarget::PaintParameter(_, name) => name.clone(),
                    ControlTarget::EffectParameter(_, name) => name.clone(),
                    ControlTarget::Custom(name) => name.clone(),
                    _ => target.name(),
                };

                return Err(ControlError::InvalidParameter(format!(
                    "Security violation: Path traversal detected in value for {}",
                    name
                )));
            }
        }
        Ok(())
    }

    /// Apply a control change
    pub fn apply_control(&mut self, target: ControlTarget, value: ControlValue) {
        // SECURITY: Validate potential file paths to prevent traversal
        if let Err(e) = self.validate_security(&target, &value) {
            warn!("Security violation in apply_control: {}", e);
            return;
        }

        info!("Control change: {:?} = {:?}", target, value);

        // Call the control callback if set
        if let Some(callback) = &self.control_callback {
            if let Ok(mut cb) = callback.lock() {
                cb(target.clone(), value.clone());
            }
        }

        // Send OSC feedback to all clients
        #[cfg(feature = "osc")]
        for client in &mut self.osc_clients {
            if let Err(e) = client.send_update(&target, &value) {
                warn!(
                    "Failed to send OSC feedback to {}: {}",
                    client.destination_str(),
                    e
                );
            }
        }
    }

    /// Execute an action
    pub fn execute_action(&mut self, action: Action) {
        info!("Executing action: {:?}", action);

        match action {
            Action::NextCue => {
                let _ = self.cue_list.next();
            }
            Action::PrevCue => {
                let _ = self.cue_list.prev();
            }
            Action::GotoCue(id) => {
                let _ = self.cue_list.goto_cue(id, None);
            }
            _ => {
                // Other actions would be handled by the application
                info!("Action requires application handling: {:?}", action);
            }
        }
    }

    /// Handle keyboard input
    pub fn handle_key_press(&mut self, key: Key, modifiers: &Modifiers) {
        if let Some(action) = self.key_bindings.find_action(key, modifiers) {
            self.execute_action(action);
        }
    }

    /// Send DMX data via Art-Net
    pub fn send_artnet(&mut self, channels: &[u8; 512], target: &str) -> Result<()> {
        if let Some(sender) = &mut self.artnet_sender {
            sender.send_dmx(channels, target)?;
        } else {
            // No error if not initialized, just ignore? Or error?
            // User requested robustness. If Art-Net not init, maybe we shouldn't fail hard.
            // But returning Err allows caller to know.
            return Err(ControlError::DmxError(
                "Art-Net not initialized".to_string(),
            ));
        }
        Ok(())
    }

    /// Send DMX data via sACN
    pub fn send_sacn(&mut self, channels: &[u8; 512]) -> Result<()> {
        if let Some(sender) = &mut self.sacn_sender {
            sender.send_dmx(channels)?;
        } else {
            return Err(ControlError::DmxError("sACN not initialized".to_string()));
        }
        Ok(())
    }

    /// Get a list of all possible control targets.
    // This is a placeholder and should be populated from the main application state.
    pub fn get_all_control_targets(&self) -> Vec<ControlTarget> {
        vec![
            ControlTarget::LayerOpacity(1),
            ControlTarget::LayerVisibility(1),
            ControlTarget::MasterOpacity,
            ControlTarget::MasterBlackout,
        ]
    }

    /// Get a mutable reference to the cue list
    pub fn cue_list_mut(&mut self) -> &mut CueList {
        &mut self.cue_list
    }
}

impl Default for ControlManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_manager() {
        let manager = ControlManager::new();
        #[cfg(feature = "osc")]
        {
            assert!(manager.osc_server.is_none());
            assert!(manager.osc_clients.is_empty());
        }
    }

    #[test]
    fn test_key_bindings() {
        let mut manager = ControlManager::new();

        // Default bindings should include Space -> TogglePlayPause
        manager.handle_key_press(Key::Space, &Modifiers::new());

        // This should work without panicking
    }

    #[test]
    fn test_control_callback() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let mut manager = ControlManager::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        manager.set_control_callback(move |_target, _value| {
            called_clone.store(true, Ordering::SeqCst);
        });

        manager.apply_control(ControlTarget::LayerOpacity(0), ControlValue::Float(0.5));

        assert!(called.load(Ordering::SeqCst));
    }
    #[test]
    fn test_cue_execution() {
        let mut manager = ControlManager::new();

        // Add some dummy cues to test navigation
        manager.cue_list.add_cue(
            crate::cue::Cue::new(1, "Cue 1".to_string())
                .with_fade_duration(std::time::Duration::from_millis(0)),
        );
        manager.cue_list.add_cue(
            crate::cue::Cue::new(2, "Cue 2".to_string())
                .with_fade_duration(std::time::Duration::from_millis(0)),
        );

        // Test Goto 1
        manager.execute_action(Action::GotoCue(1));
        manager.update();
        assert_eq!(manager.cue_list.current_cue(), Some(1));

        // Test Next
        manager.execute_action(Action::NextCue);
        manager.update();
        assert_eq!(manager.cue_list.current_cue(), Some(2));

        // Test Prev
        manager.execute_action(Action::PrevCue);
        manager.update();
        assert_eq!(manager.cue_list.current_cue(), Some(1));
    }

    #[test]
    fn test_security_validation() {
        let mut manager = ControlManager::new();
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        manager.set_control_callback(move |_target, _value| {
            called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        // 1. Safe value should pass
        manager.apply_control(
            ControlTarget::EffectParameter(0, "file_path".to_string()),
            ControlValue::String("safe_file.txt".to_string()),
        );
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
        called.store(false, std::sync::atomic::Ordering::SeqCst);

        // 2. Unsafe value should be blocked (contains "..")
        manager.apply_control(
            ControlTarget::EffectParameter(0, "file_path".to_string()),
            ControlValue::String("../secret.txt".to_string()),
        );
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));

        // 3. Unsafe value with different casing in parameter name
        manager.apply_control(
            ControlTarget::EffectParameter(0, "MySourceFile".to_string()),
            ControlValue::String("../secret.txt".to_string()),
        );
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));

        // 4. ".." in non-sensitive parameter should pass (debatable, but current logic allows it)
        manager.apply_control(
            ControlTarget::Custom("MyLabel".to_string()),
            ControlValue::String("Dots..are..okay..here".to_string()),
        );
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
        called.store(false, std::sync::atomic::Ordering::SeqCst);

        // 5. "Loading..." should pass for a "source" parameter (ellipses are valid text)
        manager.apply_control(
            ControlTarget::Custom("TextSource".to_string()),
            ControlValue::String("Loading...".to_string()),
        );
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
        called.store(false, std::sync::atomic::Ordering::SeqCst);

        // 6. Unsafe value in a parameter that doesn't sound like a file path
        // previously allowed, now should be blocked
        manager.apply_control(
            ControlTarget::Custom("UserDescription".to_string()),
            ControlValue::String("../../../etc/passwd".to_string()),
        );
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
    }
}




