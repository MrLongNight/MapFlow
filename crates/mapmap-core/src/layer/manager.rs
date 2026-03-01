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
