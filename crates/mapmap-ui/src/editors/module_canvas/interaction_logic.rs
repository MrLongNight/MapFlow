use mapmap_core::module::ModulePartId;

use super::controller;
use super::state::ModuleCanvas;
use super::types::MediaPlaybackCommand;

impl ModuleCanvas {
    pub fn take_playback_commands(&mut self) -> Vec<(ModulePartId, MediaPlaybackCommand)> {
        std::mem::take(&mut self.pending_playback_commands)
    }

    pub fn get_selected_part_id(&self) -> Option<ModulePartId> {
        self.selected_parts.last().copied()
    }

    pub fn set_active_module(&mut self, module_id: Option<u64>) {
        self.active_module_id = module_id;
        // Also clear selection when switching modules
        self.selected_parts.clear();
        self.dragging_part = None;
        self.creating_connection = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn active_module_id(&self) -> Option<u64> {
        self.active_module_id
    }

    pub fn set_audio_data(&mut self, data: mapmap_core::audio_reactive::AudioTriggerData) {
        self.audio_trigger_data = data;
    }

    pub fn get_audio_trigger_data(&self) -> Option<&mapmap_core::audio_reactive::AudioTriggerData> {
        Some(&self.audio_trigger_data)
    }

    pub fn get_rms_volume(&self) -> f32 {
        self.audio_trigger_data.rms_volume
    }

    pub fn is_beat_detected(&self) -> bool {
        self.audio_trigger_data.beat_detected
    }

    #[cfg(feature = "midi")]
    pub fn process_midi_message(&mut self, message: mapmap_control::midi::MidiMessage) {
        controller::process_midi_message(self, message);
    }

    #[cfg(not(feature = "midi"))]
    pub fn process_midi_message(&mut self, message: ()) {
        controller::process_midi_message(self, message);
    }
}
