//! Module Manager - Manages multiple scenes (modules)

use crate::module::config::default_color_palette;
use crate::module::types::{
    ModuleId, ModulePartId, ModulePlaybackMode, ModuleRepairReport, PartType, SharedMediaState,
    VorceModule,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manages multiple modules (Scenes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManager {
    /// The collection of all modules, indexed by ID.
    pub modules: HashMap<ModuleId, VorceModule>,
    /// The next available module ID.
    pub next_module_id: ModuleId,
    /// The next available part ID across all modules.
    pub next_part_id: ModulePartId,
    /// Predefined colors for new modules.
    #[serde(skip, default = "default_color_palette")]
    /// Predefined list of colors available for UI elements.
    pub color_palette: Vec<[f32; 4]>,
    /// Index to cycle through the color palette.
    pub next_color_index: usize,
    /// Shared media registry
    #[serde(default)]
    /// Global registry of media assets shared across the project.
    pub shared_media: SharedMediaState,
    /// Incrementing counter tracking graph structural changes
    #[serde(skip)]
    /// Incremental counter tracking changes to the graph structure.
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

        let module = VorceModule {
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
    pub fn list_modules(&self) -> Vec<&VorceModule> {
        self.modules.values().collect()
    }

    /// Set module color
    pub fn set_module_color(&mut self, id: ModuleId, color: [f32; 4]) {
        if let Some(module) = self.modules.get_mut(&id) {
            module.color = color;
        }
    }

    /// Get module by ID (mutable)
    pub fn get_module_mut(&mut self, id: ModuleId) -> Option<&mut VorceModule> {
        self.modules.get_mut(&id)
    }

    /// Get a module by ID (immutable)
    pub fn get_module(&self, id: ModuleId) -> Option<&VorceModule> {
        self.modules.get(&id)
    }

    /// Get all modules as a slice-like iterator
    pub fn modules(&self) -> Vec<&VorceModule> {
        self.modules.values().collect()
    }

    /// Get all modules mutably
    pub fn modules_mut(&mut self) -> Vec<&mut VorceModule> {
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

    /// Remove a module
    pub fn remove_module(&mut self, module_id: ModuleId) -> Option<VorceModule> {
        self.modules.remove(&module_id)
    }

    /// Repair a single module in place and mark the graph dirty if anything changed.
    pub fn repair_module(&mut self, module_id: ModuleId) -> Option<ModuleRepairReport> {
        let report = self.modules.get_mut(&module_id)?.repair_graph();
        if report.changed() {
            tracing::warn!(
                "Module {} repaired: refreshed_parts={}, removed_connections={}, removed_trigger_targets={}, normalized_parts={}",
                module_id,
                report.refreshed_parts,
                report.removed_connections,
                report.removed_trigger_targets,
                report.normalized_parts
            );
            self.mark_dirty();
        }
        Some(report)
    }

    /// Repair multiple modules and return reports for changed graphs.
    pub fn repair_modules<I>(&mut self, module_ids: I) -> Vec<(ModuleId, ModuleRepairReport)>
    where
        I: IntoIterator<Item = ModuleId>,
    {
        let mut reports = Vec::new();
        for module_id in module_ids {
            if let Some(report) = self.repair_module(module_id) {
                if report.changed() {
                    reports.push((module_id, report));
                }
            }
        }
        reports
    }
}

impl Default for ModuleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid_initialization_defaults() {
        let manager = ModuleManager::new();
        assert_eq!(manager.modules.len(), 0);
        assert_eq!(manager.next_module_id, 1);
        assert_eq!(manager.next_part_id, 1);
        assert_eq!(manager.next_color_index, 0);
        assert_eq!(manager.graph_revision, 1);
    }

    #[test]
    fn test_create_module_valid_name_creates_module() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("Test Module".to_string());
        assert_eq!(id, 1);
        assert_eq!(manager.modules.len(), 1);
        assert_eq!(manager.modules.get(&id).unwrap().name, "Test Module");
        assert_eq!(manager.graph_revision, 2);
    }

    #[test]
    fn test_create_module_duplicate_name_renames() {
        let mut manager = ModuleManager::new();
        let id1 = manager.create_module("Test Module".to_string());
        let id2 = manager.create_module("Test Module".to_string());
        assert_ne!(id1, id2);
        assert_eq!(manager.modules.get(&id1).unwrap().name, "Test Module");
        assert_eq!(manager.modules.get(&id2).unwrap().name, "Test Module 1");
    }

    #[test]
    fn test_delete_module_valid_id_removes_module() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("Test Module".to_string());
        manager.delete_module(id);
        assert_eq!(manager.modules.len(), 0);
    }

    #[test]
    fn test_rename_module_valid_name_updates() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("Old Name".to_string());
        assert!(manager.rename_module(id, "New Name".to_string()));
        assert_eq!(manager.modules.get(&id).unwrap().name, "New Name");
    }

    #[test]
    fn test_rename_module_duplicate_name_fails() {
        let mut manager = ModuleManager::new();
        let _id1 = manager.create_module("Name 1".to_string());
        let id2 = manager.create_module("Name 2".to_string());
        assert!(!manager.rename_module(id2, "Name 1".to_string()));
        assert_eq!(manager.modules.get(&id2).unwrap().name, "Name 2");
    }

    #[test]
    fn test_duplicate_module_valid_id_duplicates() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("Test Module".to_string());
        let duplicate_id = manager.duplicate_module(id).unwrap();
        assert_eq!(manager.modules.len(), 2);
        assert_eq!(
            manager.modules.get(&duplicate_id).unwrap().name,
            "Test Module (Copy) 1"
        );
    }

    #[test]
    fn test_rename_module_to_same_name_succeeds() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("Test".to_string());
        assert!(manager.rename_module(id, "Test".to_string()));
        assert_eq!(manager.get_module(id).unwrap().name, "Test");
    }

    #[test]
    fn test_set_module_color() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("Test".to_string());
        manager.set_module_color(id, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(manager.get_module(id).unwrap().color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_get_module_mut_and_get_module() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("Test".to_string());
        let revision_before = manager.graph_revision;

        {
            let module_mut = manager.get_module_mut(id).unwrap();
            module_mut.name = "Mutated".to_string();
        }

        let module = manager.get_module(id).unwrap();
        assert_eq!(module.name, "Mutated");
        assert_eq!(manager.graph_revision, revision_before);
        assert!(manager.get_module(999).is_none());
        assert!(manager.get_module_mut(999).is_none());
    }

    #[test]
    fn test_next_part_id_increments() {
        let mut manager = ModuleManager::new();
        assert_eq!(manager.next_part_id, 1);
        let id1 = manager.next_part_id();
        assert_eq!(id1, 1);
        assert_eq!(manager.next_part_id, 2);
    }

    #[test]
    fn test_modules_and_modules_mut_iterators() {
        let mut manager = ModuleManager::new();
        let _id1 = manager.create_module("A".to_string());
        let _id2 = manager.create_module("B".to_string());

        let modules = manager.modules();
        assert_eq!(modules.len(), 2);

        let mut_modules = manager.modules_mut();
        assert_eq!(mut_modules.len(), 2);
    }

    #[test]
    fn test_remove_module_returns_module() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("A".to_string());
        let module = manager.remove_module(id);
        assert!(module.is_some());
        assert_eq!(module.unwrap().id, id);
        assert!(manager.remove_module(id).is_none());
    }

    #[test]
    fn test_get_next_available_name_with_multiple_collisions() {
        let mut manager = ModuleManager::new();
        manager.create_module("Base".to_string());
        manager.create_module("Base 1".to_string());
        manager.create_module("Base 2".to_string());
        assert_eq!(manager.get_next_available_name("Base"), "Base 3");
    }

    #[test]
    fn test_duplicate_module_nonexistent_returns_none() {
        let mut manager = ModuleManager::new();
        assert!(manager.duplicate_module(999).is_none());
    }

    #[test]
    fn test_add_part_to_module() {
        let mut manager = ModuleManager::new();
        let id = manager.create_module("A".to_string());
        let part_id =
            manager.add_part_to_module(id, crate::module::types::PartType::Trigger, (0.0, 0.0));
        assert!(part_id.is_some());
        assert!(manager
            .add_part_to_module(999, crate::module::types::PartType::Trigger, (0.0, 0.0))
            .is_none());
    }

    #[test]
    fn test_list_modules() {
        let mut manager = ModuleManager::new();
        manager.create_module("A".to_string());
        assert_eq!(manager.list_modules().len(), 1);
    }
}
