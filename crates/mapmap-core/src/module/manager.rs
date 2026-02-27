//! Module Manager - Manages multiple scenes (modules)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::module::types::{ModuleId, MapFlowModule, ModulePartId, PartType, ModulePlaybackMode, SharedMediaState, ModulePartType, OutputType};
use crate::module::config::default_color_palette;

/// Manages multiple modules (Scenes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManager {
    pub modules: HashMap<ModuleId, MapFlowModule>,
    pub next_module_id: ModuleId,
    pub next_part_id: ModulePartId,
    #[serde(skip, default = "default_color_palette")]
    pub color_palette: Vec<[f32; 4]>,
    pub next_color_index: usize,
    /// Shared media registry
    #[serde(default)]
    pub shared_media: SharedMediaState,
    /// Incrementing counter tracking graph structural changes
    #[serde(skip)]
    pub graph_revision: u64,
}

impl PartialEq for ModuleManager {
    fn eq(&self, other: &Self) -> bool {
        self.modules == other.modules
            && self.next_module_id == other.next_module_id
            && self.next_part_id == other.next_part_id
            && self.next_color_index == other.next_color_index
            && self.shared_media == other.shared_media
    }
}

impl ModuleManager {
    /// Create a new module manager
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            next_module_id: 1,
            next_part_id: 1,
            color_palette: default_color_palette(),
            next_color_index: 0,
            shared_media: SharedMediaState::new(),
            graph_revision: 1,
        }
    }

    /// Mark the graph as dirty by incrementing revision
    pub fn mark_dirty(&mut self) {
        self.graph_revision = self.graph_revision.wrapping_add(1);
    }

    /// Add a part to a specific module
    pub fn add_part_to_module(
        &mut self,
        module_id: ModuleId,
        part_type: PartType,
        position: (f32, f32),
    ) -> Option<ModulePartId> {
        self.mark_dirty();
        self.modules
            .get_mut(&module_id)
            .map(|module| module.add_part(part_type, position))
    }

    /// Get the next available unique name for a module
    pub fn get_next_available_name(&self, base_name: &str) -> String {
        let mut i = 1;
        loop {
            let name = format!("{} {}", base_name, i);
            if !self.modules.values().any(|m| m.name == name) {
                return name;
            }
            i += 1;
        }
    }

    /// Create a new module
    pub fn create_module(&mut self, mut name: String) -> ModuleId {
        self.mark_dirty();
        // Enforce uniqueness to prevent duplicate names
        if self.modules.values().any(|m| m.name == name) {
            name = self.get_next_available_name(&name);
        }

        let id = self.next_module_id;
        self.next_module_id += 1;

        let color = self.color_palette[self.next_color_index % self.color_palette.len()];
        self.next_color_index += 1;

        let module = MapFlowModule {
            id,
            name,
            color,
            parts: Vec::new(),
            connections: Vec::new(),
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        self.modules.insert(id, module);
        id
    }

    /// Delete a module
    pub fn delete_module(&mut self, id: ModuleId) {
        self.mark_dirty();
        self.modules.remove(&id);
    }

    /// List all modules
    pub fn list_modules(&self) -> Vec<&MapFlowModule> {
        self.modules.values().collect()
    }

    /// Set module color
    pub fn set_module_color(&mut self, id: ModuleId, color: [f32; 4]) {
        if let Some(module) = self.modules.get_mut(&id) {
            module.color = color;
        }
    }

    /// Get module by ID (mutable)
    pub fn get_module_mut(&mut self, id: ModuleId) -> Option<&mut MapFlowModule> {
        self.mark_dirty();
        self.modules.get_mut(&id)
    }

    /// Get a module by ID (immutable)
    pub fn get_module(&self, id: ModuleId) -> Option<&MapFlowModule> {
        self.modules.get(&id)
    }

    /// Get all modules as a slice-like iterator
    pub fn modules(&self) -> Vec<&MapFlowModule> {
        self.modules.values().collect()
    }

    /// Get all modules mutably
    pub fn modules_mut(&mut self) -> Vec<&mut MapFlowModule> {
        self.modules.values_mut().collect()
    }

    /// Generate a new part ID
    pub fn next_part_id(&mut self) -> ModulePartId {
        let id = self.next_part_id;
        self.next_part_id += 1;
        id
    }

    /// Duplicate a module
    pub fn duplicate_module(&mut self, module_id: ModuleId) -> Option<ModuleId> {
        let module = self.modules.get(&module_id)?;
        let mut new_module = module.clone();
        let new_id = self.next_module_id;
        self.next_module_id += 1;

        new_module.id = new_id;
        new_module.name = self.get_next_available_name(&format!("{} (Copy)", module.name));

        self.modules.insert(new_id, new_module);
        Some(new_id)
    }

    /// Rename a module
    pub fn rename_module(&mut self, module_id: ModuleId, new_name: String) -> bool {
        // Check uniqueness
        if self
            .modules
            .values()
            .any(|m| m.name == new_name && m.id != module_id)
        {
            return false;
        }

        if let Some(module) = self.modules.get_mut(&module_id) {
            module.name = new_name;
            true
        } else {
            false
        }
    }
}

impl Default for ModuleManager {
    fn default() -> Self {
        Self::new()
    }
}
