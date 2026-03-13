//!
//! Layer specific structs.
//!

use crate::layer::transform::Transform;
use crate::layer::types::{BlendMode, ResizeMode};
use glam::{Mat4, Vec2};
use serde::{Deserialize, Serialize};

/// A single layer in the composition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    /// Unique identifier for the layer
    pub id: u64,
    /// Display name of the layer
    pub name: String,
    /// Optional paint (media source) ID assigned to this layer
    pub paint_id: Option<u64>,
    /// List of mapping IDs associated with this layer
    pub mapping_ids: Vec<u64>,
    /// Blend mode for compositing
    pub blend_mode: BlendMode,
    /// Opacity/video fader (V) - 0.0 = transparent, 1.0 = opaque (Phase 1, Month 4)
    pub opacity: f32,
    /// Visibility state of the layer
    pub visible: bool,
    /// Solo mode (S) - isolate this layer (Phase 1, Month 4)
    pub solo: bool,
    /// Bypass mode (B) - skip layer in render pipeline (Phase 1, Month 4)
    pub bypass: bool,
    /// Lock state to prevent accidental changes
    pub locked: bool,
    /// Layer transform - position, scale, rotation, anchor (Phase 1, Month 4)
    pub transform: Transform,
    /// The effect chain for this layer.
    pub effect_chain: crate::effects::EffectChain,
    /// Legacy transform matrix (for backward compatibility)
    #[serde(skip)]
    pub legacy_transform: Mat4,

    /// Parent Layer ID (if part of a group)
    #[serde(default)]
    pub parent_id: Option<u64>,
    /// Whether this layer is a group
    #[serde(default)]
    pub is_group: bool,
    /// UI State: whether the group is collapsed
    #[serde(default)]
    pub collapsed: bool,
}

impl Layer {
    /// Create a new layer
    pub fn new(id: u64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            paint_id: None,
            mapping_ids: Vec::new(),
            blend_mode: BlendMode::default(),
            opacity: 1.0,
            visible: true,
            solo: false,
            bypass: false,
            locked: false,
            transform: Transform::default(),
            effect_chain: crate::effects::EffectChain::default(),
            legacy_transform: Mat4::IDENTITY,
            parent_id: None,
            is_group: false,
            collapsed: false,
        }
    }

    /// Set the paint for this layer
    pub fn with_paint(mut self, paint_id: u64) -> Self {
        self.paint_id = Some(paint_id);
        self
    }

    /// Set blend mode
    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    /// Set opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Add a mapping to this layer
    pub fn add_mapping(&mut self, mapping_id: u64) {
        if !self.mapping_ids.contains(&mapping_id) {
            self.mapping_ids.push(mapping_id);
        }
    }

    /// Remove a mapping from this layer
    pub fn remove_mapping(&mut self, mapping_id: u64) {
        self.mapping_ids.retain(|&id| id != mapping_id);
    }

    /// Check if layer should be rendered
    pub fn should_render(&self) -> bool {
        self.visible && !self.bypass && self.opacity > 0.0 && self.paint_id.is_some()
    }

    /// Rename the layer
    pub fn rename(&mut self, new_name: impl Into<String>) {
        self.name = new_name.into();
    }

    /// Toggle bypass mode
    pub fn toggle_bypass(&mut self) {
        self.bypass = !self.bypass;
    }

    /// Toggle solo mode
    pub fn toggle_solo(&mut self) {
        self.solo = !self.solo;
    }

    /// Set transform with resize mode
    pub fn set_transform_with_resize(
        &mut self,
        mode: ResizeMode,
        source_size: Vec2,
        target_size: Vec2,
    ) {
        self.transform
            .apply_resize_mode(mode, source_size, target_size);
    }

    /// Get transform matrix for rendering
    pub fn get_transform_matrix(&self, content_size: Vec2) -> Mat4 {
        self.transform.to_matrix(content_size)
    }
}

#[cfg(test)]
mod tests_guardian {
    use super::*;
    use crate::layer::types::BlendMode;

    #[test]
    fn test_layer_new_initialization() {
        let layer = Layer::new(42, "My Layer");
        assert_eq!(layer.id, 42);
        assert_eq!(layer.name, "My Layer");
        assert_eq!(layer.paint_id, None);
        assert!(layer.mapping_ids.is_empty());
        assert_eq!(layer.blend_mode, BlendMode::Normal);
        assert_eq!(layer.opacity, 1.0);
        assert!(layer.visible);
        assert!(!layer.solo);
        assert!(!layer.bypass);
        assert!(!layer.locked);
        assert_eq!(layer.parent_id, None);
        assert!(!layer.is_group);
    }

    #[test]
    fn test_layer_builder_methods() {
        let layer = Layer::new(1, "Test")
            .with_paint(100)
            .with_blend_mode(BlendMode::Add)
            .with_opacity(0.5);
        assert_eq!(layer.paint_id, Some(100));
        assert_eq!(layer.blend_mode, BlendMode::Add);
        assert_eq!(layer.opacity, 0.5);

        // Clamping logic
        let over = Layer::new(2, "O").with_opacity(2.0);
        assert_eq!(over.opacity, 1.0);
        let under = Layer::new(3, "U").with_opacity(-1.0);
        assert_eq!(under.opacity, 0.0);
    }

    #[test]
    fn test_layer_mapping_management() {
        let mut layer = Layer::new(1, "MappingTest");
        layer.add_mapping(10);
        layer.add_mapping(20);
        layer.add_mapping(10); // Duplicate should be ignored
        assert_eq!(layer.mapping_ids, vec![10, 20]);

        layer.remove_mapping(10);
        assert_eq!(layer.mapping_ids, vec![20]);

        layer.remove_mapping(99); // Removing non-existent
        assert_eq!(layer.mapping_ids, vec![20]);
    }

    #[test]
    fn test_layer_should_render_logic() {
        let mut layer = Layer::new(1, "RenderTest").with_paint(10);
        assert!(layer.should_render());

        layer.visible = false;
        assert!(!layer.should_render());
        layer.visible = true;

        layer.bypass = true;
        assert!(!layer.should_render());
        layer.bypass = false;

        layer.opacity = 0.0;
        assert!(!layer.should_render());
        layer.opacity = 1.0;

        layer.paint_id = None;
        assert!(!layer.should_render());
    }

    #[test]
    fn test_layer_state_toggles() {
        let mut layer = Layer::new(1, "ToggleTest");
        assert!(!layer.solo);
        layer.toggle_solo();
        assert!(layer.solo);

        assert!(!layer.bypass);
        layer.toggle_bypass();
        assert!(layer.bypass);
    }
}
