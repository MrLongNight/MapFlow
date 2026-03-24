//! Mapping - Connects Paint to Output Mesh
//!
//! A Mapping represents the connection between a media source (Paint) and
//! its output geometry (Mesh), including transformation and rendering properties.

use crate::{Mesh, PaintId};
use serde::{Deserialize, Serialize};

/// Unique identifier for a Mapping
pub type MappingId = u64;

/// Mapping - connects a Paint to an output Mesh
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mapping {
    /// Unique identifier for the mapping
    pub id: MappingId,
    /// Display name of the mapping
    pub name: String,

    /// The paint (media source) to map
    pub paint_id: PaintId,

    /// Output mesh (warping geometry)
    pub mesh: Mesh,

    /// Visibility
    pub visible: bool,

    /// Solo mode (only show this mapping)
    pub solo: bool,

    /// Locked (prevent editing)
    pub locked: bool,

    /// Opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: f32,

    /// Depth (Z-order for layering)
    pub depth: f32,
}

impl Mapping {
    /// Create a new mapping
    pub fn new(id: MappingId, name: impl Into<String>, paint_id: PaintId, mesh: Mesh) -> Self {
        Self {
            id,
            name: name.into(),
            paint_id,
            mesh,
            visible: true,
            solo: false,
            locked: false,
            opacity: 1.0,
            depth: 0.0,
        }
    }

    /// Create a quad mapping
    pub fn quad(id: MappingId, name: impl Into<String>, paint_id: PaintId) -> Self {
        Self::new(id, name, paint_id, Mesh::quad())
    }

    /// Create a triangle mapping
    pub fn triangle(id: MappingId, name: impl Into<String>, paint_id: PaintId) -> Self {
        Self::new(id, name, paint_id, Mesh::triangle())
    }

    /// Is this mapping renderable?
    pub fn is_renderable(&self) -> bool {
        self.visible && !self.solo && self.opacity > 0.0
    }
}

/// Manages all mappings in the project
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MappingManager {
    /// List of mappings
    mappings: Vec<Mapping>,
    /// Next available mapping ID
    next_id: MappingId,
}

impl MappingManager {
    /// Create a new mapping manager
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a mapping
    pub fn add_mapping(&mut self, mut mapping: Mapping) -> MappingId {
        if mapping.id == 0 {
            mapping.id = self.next_id;
            self.next_id += 1;
        }
        let id = mapping.id;
        self.mappings.push(mapping);
        id
    }

    /// Remove a mapping
    pub fn remove_mapping(&mut self, id: MappingId) -> Option<Mapping> {
        self.mappings
            .iter()
            .position(|m| m.id == id)
            .map(|index| self.mappings.remove(index))
    }

    /// Get a mapping by ID
    pub fn get_mapping(&self, id: MappingId) -> Option<&Mapping> {
        self.mappings.iter().find(|m| m.id == id)
    }

    /// Get a mutable mapping by ID
    pub fn get_mapping_mut(&mut self, id: MappingId) -> Option<&mut Mapping> {
        self.mappings.iter_mut().find(|m| m.id == id)
    }

    /// Get all mappings
    pub fn mappings(&self) -> &[Mapping] {
        &self.mappings
    }

    /// Get all mappings (mutable)
    pub fn mappings_mut(&mut self) -> &mut [Mapping] {
        &mut self.mappings
    }

    /// Get visible mappings (sorted by depth)
    pub fn visible_mappings(&self) -> Vec<&Mapping> {
        let mut mappings: Vec<&Mapping> =
            self.mappings.iter().filter(|m| m.is_renderable()).collect();

        // Sort by depth (back to front)
        mappings.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap());

        mappings
    }

    /// Get mappings for a specific paint
    pub fn mappings_for_paint(&self, paint_id: PaintId) -> Vec<&Mapping> {
        self.mappings
            .iter()
            .filter(|m| m.paint_id == paint_id)
            .collect()
    }

    /// Check if any mapping is in solo mode
    pub fn has_solo(&self) -> bool {
        self.mappings.iter().any(|m| m.solo)
    }

    /// Get solo mappings only
    pub fn solo_mappings(&self) -> Vec<&Mapping> {
        self.mappings
            .iter()
            .filter(|m| m.solo && m.visible)
            .collect()
    }

    /// Move mapping up in Z-order
    pub fn move_up(&mut self, id: MappingId) -> bool {
        if let Some(mapping) = self.get_mapping_mut(id) {
            mapping.depth += 1.0;
            true
        } else {
            false
        }
    }

    /// Move mapping down in Z-order
    pub fn move_down(&mut self, id: MappingId) -> bool {
        if let Some(mapping) = self.get_mapping_mut(id) {
            mapping.depth -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_creation() {
        let mapping = Mapping::quad(1, "Test Mapping", 10);
        assert_eq!(mapping.id, 1);
        assert_eq!(mapping.paint_id, 10);
        assert!(mapping.visible);
        assert_eq!(mapping.opacity, 1.0);
    }

    #[test]
    fn test_mapping_manager() {
        let mut manager = MappingManager::new();

        let mapping1 = Mapping::quad(0, "Mapping 1", 10);
        let id1 = manager.add_mapping(mapping1);

        let mapping2 = Mapping::quad(0, "Mapping 2", 20);
        let id2 = manager.add_mapping(mapping2);

        assert_ne!(id1, id2);
        assert_eq!(manager.mappings().len(), 2);

        manager.remove_mapping(id1);
        assert_eq!(manager.mappings().len(), 1);
    }

    #[test]
    fn test_visible_mappings() {
        let mut manager = MappingManager::new();

        let mut mapping1 = Mapping::quad(0, "Visible", 10);
        mapping1.depth = 1.0;
        manager.add_mapping(mapping1);

        let mut mapping2 = Mapping::quad(0, "Hidden", 20);
        mapping2.visible = false;
        manager.add_mapping(mapping2);

        let mut mapping3 = Mapping::quad(0, "Back", 30);
        mapping3.depth = 0.0;
        manager.add_mapping(mapping3);

        let visible = manager.visible_mappings();
        assert_eq!(visible.len(), 2);
        // Should be sorted by depth (0.0, then 1.0)
        assert_eq!(visible[0].name, "Back");
        assert_eq!(visible[1].name, "Visible");
    }

    #[test]
    fn test_solo_mode() {
        let mut manager = MappingManager::new();

        let mapping1 = Mapping::quad(0, "Normal", 10);
        manager.add_mapping(mapping1);

        let mut mapping2 = Mapping::quad(0, "Solo", 20);
        mapping2.solo = true;
        manager.add_mapping(mapping2);

        assert!(manager.has_solo());

        let solo = manager.solo_mappings();
        assert_eq!(solo.len(), 1);
        assert_eq!(solo[0].name, "Solo");
    }

    #[test]
    fn test_z_order() {
        let mut manager = MappingManager::new();

        let mapping = Mapping::quad(0, "Test", 10);
        let id = manager.add_mapping(mapping);

        manager.move_up(id);
        assert_eq!(manager.get_mapping(id).unwrap().depth, 1.0);

        manager.move_down(id);
        manager.move_down(id);
        assert_eq!(manager.get_mapping(id).unwrap().depth, -1.0);
    }
}

#[cfg(test)]
mod tests_guardian {
    use super::*;

    #[test]
    fn test_mapping_manager_depth_sorting_stability() {
        let mut manager = MappingManager::new();

        // Add mappings with mixed depths and insertion order
        let m1 = Mapping {
            id: 1,
            depth: 2.0,
            opacity: 1.0,
            ..Mapping::quad(1, "Top", 0)
        };
        let m2 = Mapping {
            id: 2,
            depth: 0.0,
            opacity: 1.0,
            ..Mapping::quad(2, "Bottom", 0)
        };
        let m3 = Mapping {
            id: 3,
            depth: 1.0,
            opacity: 1.0,
            ..Mapping::quad(3, "Middle", 0)
        };
        // Same depth as m3
        let m4 = Mapping {
            id: 4,
            depth: 1.0,
            opacity: 1.0,
            ..Mapping::quad(4, "Middle 2", 0)
        };

        manager.add_mapping(m3.clone()); // Middle first
        manager.add_mapping(m1.clone()); // Top
        manager.add_mapping(m2.clone()); // Bottom
        manager.add_mapping(m4.clone()); // Middle 2

        let visible = manager.visible_mappings();

        assert_eq!(visible.len(), 4);

        // Expected order: Bottom (0.0), Middle (1.0), Middle 2 (1.0), Top (2.0)
        // Note: Sort is stable or unstable? `sort_by` in Rust std is stable.
        // So m3 should come before m4 because it was inserted first?
        // Wait, `sort_by` is STABLE.
        // Insertion order: m3, m1, m2, m4.
        // Sorted by depth:
        // m2 (0.0)
        // m3 (1.0)
        // m4 (1.0)
        // m1 (2.0)

        assert_eq!(visible[0].id, 2);
        assert_eq!(visible[1].id, 3);
        assert_eq!(visible[2].id, 4);
        assert_eq!(visible[3].id, 1);
    }

    #[test]
    fn test_mapping_manager_move_z_limits() {
        let mut manager = MappingManager::new();
        let id = manager.add_mapping(Mapping::quad(1, "Test", 0));

        // Initial depth 0.0

        // Move up -> 1.0
        manager.move_up(id);
        assert_eq!(manager.get_mapping(id).unwrap().depth, 1.0);

        // Move down -> 0.0
        manager.move_down(id);
        assert_eq!(manager.get_mapping(id).unwrap().depth, 0.0);

        // Move down -> -1.0
        manager.move_down(id);
        assert_eq!(manager.get_mapping(id).unwrap().depth, -1.0);

        // Is there a limit? Currently no explicit limit in code.
    }
}
