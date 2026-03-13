//!
//! Layer manager handling collections.
//!

use crate::layer::composition::Composition;
use crate::layer::layer_struct::Layer;
use serde::{Deserialize, Serialize};

/// Layer manager for organizing and rendering layers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayerManager {
    /// List of layers managed by this manager
    layers: Vec<Layer>,
    /// Next available layer ID
    next_id: u64,
    /// Composition metadata and master controls
    pub composition: Composition,
}

impl LayerManager {
    /// Create a new layer manager
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            next_id: 1,
            composition: Composition::default(),
        }
    }

    /// Create layer manager with custom composition
    pub fn with_composition(composition: Composition) -> Self {
        Self {
            layers: Vec::new(),
            next_id: 1,
            composition,
        }
    }

    /// Add a new layer
    pub fn add_layer(&mut self, mut layer: Layer) -> u64 {
        if layer.id == 0 {
            layer.id = self.next_id;
            self.next_id += 1;
        }
        let id = layer.id;
        self.layers.push(layer);
        id
    }

    /// Create and add a new layer
    pub fn create_layer(&mut self, name: impl Into<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let layer = Layer::new(id, name);
        self.layers.push(layer);
        id
    }

    /// Create and add a new layer group
    pub fn create_group(&mut self, name: impl Into<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut layer = Layer::new(id, name);
        layer.is_group = true;
        self.layers.push(layer);
        id
    }

    /// Remove a layer by ID.
    ///
    /// If the layer is a group, children will be orphaned (parent_id set to None).
    pub fn remove_layer(&mut self, id: u64) -> Option<Layer> {
        // Orphan children first
        for layer in &mut self.layers {
            if layer.parent_id == Some(id) {
                layer.parent_id = None;
            }
        }

        if let Some(index) = self.layers.iter().position(|l| l.id == id) {
            Some(self.layers.remove(index))
        } else {
            None
        }
    }

    /// Reparent a layer to a new parent group
    pub fn reparent_layer(&mut self, layer_id: u64, new_parent_id: Option<u64>) {
        // Validation: Prevent cycles
        if let Some(pid) = new_parent_id {
            if pid == layer_id {
                return; // Cannot parent to self
            }
            if self.is_descendant(pid, layer_id) {
                return; // Cannot parent to a descendant (cycle)
            }
        }

        if let Some(layer) = self.get_layer_mut(layer_id) {
            layer.parent_id = new_parent_id;
        }
    }

    /// Check if `layer_a` is a descendant of `layer_b` (b -> ... -> a)
    pub fn is_descendant(&self, layer_a: u64, layer_b: u64) -> bool {
        let mut current_id = layer_a;
        // Simple cycle detection limit to avoid infinite loops if state is already bad
        let mut depth = 0;
        while let Some(layer) = self.get_layer(current_id) {
            if depth > 100 {
                return false;
            }
            depth += 1;

            if let Some(pid) = layer.parent_id {
                if pid == layer_b {
                    return true;
                }
                current_id = pid;
            } else {
                return false;
            }
        }
        false
    }

    /// Get a layer by ID
    pub fn get_layer(&self, id: u64) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Get a mutable layer by ID
    pub fn get_layer_mut(&mut self, id: u64) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Get all layers
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Get all visible layers in render order
    ///
    /// ⚡ Bolt: Returns an iterator to avoid allocation per frame.
    pub fn visible_layers(&self) -> impl Iterator<Item = &Layer> {
        // Check if any layer is solo'd
        let has_solo = self.layers.iter().any(|l| l.solo);

        self.layers.iter().filter(move |layer| {
            if has_solo {
                // Only render solo layers when any layer is solo'd
                layer.solo && layer.should_render()
            } else {
                layer.should_render()
            }
        })
    }

    /// Move layer up in stack (higher z-order)
    pub fn move_layer_up(&mut self, id: u64) -> bool {
        if let Some(index) = self.layers.iter().position(|l| l.id == id) {
            if index < self.layers.len() - 1 {
                self.layers.swap(index, index + 1);
                return true;
            }
        }
        false
    }

    /// Move layer down in stack (lower z-order)
    pub fn move_layer_down(&mut self, id: u64) -> bool {
        if let Some(index) = self.layers.iter().position(|l| l.id == id) {
            if index > 0 {
                self.layers.swap(index, index - 1);
                return true;
            }
        }
        false
    }

    /// Move layer to specific index
    pub fn move_layer_to(&mut self, id: u64, new_index: usize) -> bool {
        if let Some(old_index) = self.layers.iter().position(|l| l.id == id) {
            if new_index < self.layers.len() {
                let layer = self.layers.remove(old_index);
                self.layers.insert(new_index, layer);
                return true;
            }
        }
        false
    }

    /// Get number of layers
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Check if manager is empty
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Clear all layers
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Duplicate a layer
    pub fn duplicate_layer(&mut self, id: u64) -> Option<u64> {
        if let Some(layer) = self.get_layer(id).cloned() {
            let new_id = self.next_id;
            self.next_id += 1;
            let mut new_layer = layer;
            new_layer.id = new_id;
            new_layer.name = format!("{} (copy)", new_layer.name);
            self.layers.push(new_layer);
            Some(new_id)
        } else {
            None
        }
    }

    /// Rename a layer (Phase 1, Month 4)
    pub fn rename_layer(&mut self, id: u64, new_name: impl Into<String>) -> bool {
        if let Some(layer) = self.get_layer_mut(id) {
            layer.rename(new_name);
            true
        } else {
            false
        }
    }

    /// Swap two layers by ID
    pub fn swap_layers(&mut self, id1: u64, id2: u64) -> bool {
        let pos1 = self.layers.iter().position(|l| l.id == id1);
        let pos2 = self.layers.iter().position(|l| l.id == id2);

        if let (Some(p1), Some(p2)) = (pos1, pos2) {
            self.layers.swap(p1, p2);
            true
        } else {
            false
        }
    }

    /// Eject all content (X) - remove paint from all layers (Phase 1, Month 4)
    pub fn eject_all(&mut self) {
        for layer in &mut self.layers {
            layer.paint_id = None;
        }
    }

    /// Get effective opacity for a layer (layer opacity × parent opacity × master opacity)
    pub fn get_effective_opacity(&self, layer: &Layer) -> f32 {
        let mut opacity = layer.opacity;
        let mut current_id = layer.parent_id;
        let mut depth = 0;

        while let Some(pid) = current_id {
            if depth > 100 {
                break;
            } // Safety break
            depth += 1;

            if let Some(parent) = self.get_layer(pid) {
                opacity *= parent.opacity;
                current_id = parent.parent_id;
            } else {
                break;
            }
        }

        if self.composition.master_blackout {
            return 0.0;
        }

        opacity * self.composition.master_opacity
    }

    /// Get effective speed (layer speed × master speed)
    /// Note: Individual layer speed not yet implemented, returns master speed
    pub fn get_effective_speed(&self) -> f32 {
        self.composition.master_speed
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_with_composition() {
        let comp = Composition {
            master_opacity: 0.5,
            ..Default::default()
        };
        let manager = LayerManager::with_composition(comp.clone());
        assert_eq!(manager.composition.master_opacity, 0.5);
    }

    #[test]
    fn test_create_group() {
        let mut manager = LayerManager::new();
        let id = manager.create_group("Group 1");
        let group = manager.get_layer(id).unwrap();
        assert!(group.is_group);
    }

    #[test]
    fn test_remove_nonexistent_layer() {
        let mut manager = LayerManager::new();
        assert!(manager.remove_layer(999).is_none());
    }

    #[test]
    fn test_swap_nonexistent_layers() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer("Layer 1");

        assert!(!manager.swap_layers(id1, 999));
        assert!(!manager.swap_layers(999, id1));
        assert!(!manager.swap_layers(888, 999));
    }

    #[test]
    fn test_move_layer_to_out_of_bounds() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer("Layer 1");

        assert!(!manager.move_layer_to(id, 5));
        assert!(!manager.move_layer_to(999, 0));
    }

    #[test]
    fn test_move_layer_up_down_extremes() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer("Layer 1");
        let id2 = manager.create_layer("Layer 2");

        // Layer 2 is at index 1 (top of stack). Moving up should fail.
        assert!(!manager.move_layer_up(id2));

        // Layer 1 is at index 0 (bottom of stack). Moving down should fail.
        assert!(!manager.move_layer_down(id1));

        // Non-existent layer
        assert!(!manager.move_layer_up(999));
        assert!(!manager.move_layer_down(999));
    }

    #[test]
    fn test_duplicate_nonexistent_layer() {
        let mut manager = LayerManager::new();
        assert!(manager.duplicate_layer(999).is_none());
    }

    #[test]
    fn test_rename_nonexistent_layer() {
        let mut manager = LayerManager::new();
        assert!(!manager.rename_layer(999, "New Name"));
    }

    #[test]
    fn test_reparent_to_self() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer("Layer 1");
        manager.reparent_layer(id, Some(id));
        assert_eq!(manager.get_layer(id).unwrap().parent_id, None);
    }

    #[test]
    fn test_get_effective_speed() {
        let mut manager = LayerManager::new();
        manager.composition.master_speed = 2.5;
        assert_eq!(manager.get_effective_speed(), 2.5);
    }

    #[test]
    fn test_manager_remove_group_orphans_children() {
        let mut manager = LayerManager::new();
        let group_id = manager.create_group("Group");
        let child_id = manager.create_layer("Child");
        manager.reparent_layer(child_id, Some(group_id));

        assert_eq!(
            manager.get_layer(child_id).unwrap().parent_id,
            Some(group_id)
        );
        manager.remove_layer(group_id);

        // Child should still exist but be orphaned
        assert!(manager.get_layer(child_id).is_some());
        assert_eq!(manager.get_layer(child_id).unwrap().parent_id, None);
    }

    #[test]
    fn test_manager_reparent_prevents_cycles() {
        let mut manager = LayerManager::new();
        let parent_id = manager.create_group("Parent");
        let child_id = manager.create_group("Child");
        let grand_child_id = manager.create_layer("GrandChild");

        manager.reparent_layer(child_id, Some(parent_id));
        manager.reparent_layer(grand_child_id, Some(child_id));

        // Attempt to parent Parent to GrandChild (cycle)
        manager.reparent_layer(parent_id, Some(grand_child_id));

        // Should have failed
        assert_eq!(manager.get_layer(parent_id).unwrap().parent_id, None);
    }

    #[test]
    fn test_manager_effective_opacity_hierarchy() {
        let mut manager = LayerManager::new();
        manager.composition.master_opacity = 0.5;

        let parent_id = manager.create_group("Parent");
        let child_id = manager.create_layer("Child");

        manager.get_layer_mut(parent_id).unwrap().opacity = 0.8;
        manager.get_layer_mut(child_id).unwrap().opacity = 0.5;
        manager.reparent_layer(child_id, Some(parent_id));

        let child_layer = manager.get_layer(child_id).unwrap();
        // 0.5 (child) * 0.8 (parent) * 0.5 (master) = 0.2
        assert!((manager.get_effective_opacity(child_layer) - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_manager_effective_opacity_master_blackout() {
        let mut manager = LayerManager::new();
        let layer_id = manager.create_layer("Layer");

        let layer = manager.get_layer(layer_id).unwrap();
        assert_eq!(manager.get_effective_opacity(layer), 1.0);

        manager.composition.master_blackout = true;
        let layer_after = manager.get_layer(layer_id).unwrap();
        assert_eq!(manager.get_effective_opacity(layer_after), 0.0);
    }

    #[test]
    fn test_manager_visible_layers_solo_logic() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer("Layer 1");
        let id2 = manager.create_layer("Layer 2");
        let id3 = manager.create_layer("Layer 3");

        manager.get_layer_mut(id1).unwrap().paint_id = Some(1);
        manager.get_layer_mut(id2).unwrap().paint_id = Some(2);
        manager.get_layer_mut(id3).unwrap().paint_id = Some(3);

        assert_eq!(manager.visible_layers().count(), 3);

        manager.get_layer_mut(id2).unwrap().solo = true;

        let visible: Vec<_> = manager.visible_layers().collect();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, id2);
    }
}
