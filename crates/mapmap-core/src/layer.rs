//! Layer system for compositing multiple video sources
//!
//! Layers provide a hierarchical structure for organizing and compositing
//! multiple media sources with different blend modes and transforms.

use glam::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Blend mode for compositing layers
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BlendMode {
    /// Normal alpha blending (default)
    #[default]
    Normal,
    /// Add colors (lighten)
    Add,
    /// Subtract colors (darken)
    Subtract,
    /// Multiply colors (darken)
    Multiply,
    /// Screen colors (lighten)
    Screen,
    /// Overlay (combination of multiply and screen)
    Overlay,
    /// Soft light
    SoftLight,
    /// Hard light
    HardLight,
    /// Lighten only (max)
    Lighten,
    /// Darken only (min)
    Darken,
    /// Color dodge
    ColorDodge,
    /// Color burn
    ColorBurn,
    /// Difference
    Difference,
    /// Exclusion
    Exclusion,
}

impl BlendMode {
    /// Get shader function name for this blend mode
    pub fn shader_function(&self) -> &'static str {
        match self {
            BlendMode::Normal => "blend_normal",
            BlendMode::Add => "blend_add",
            BlendMode::Subtract => "blend_subtract",
            BlendMode::Multiply => "blend_multiply",
            BlendMode::Screen => "blend_screen",
            BlendMode::Overlay => "blend_overlay",
            BlendMode::SoftLight => "blend_soft_light",
            BlendMode::HardLight => "blend_hard_light",
            BlendMode::Lighten => "blend_lighten",
            BlendMode::Darken => "blend_darken",
            BlendMode::ColorDodge => "blend_color_dodge",
            BlendMode::ColorBurn => "blend_color_burn",
            BlendMode::Difference => "blend_difference",
            BlendMode::Exclusion => "blend_exclusion",
        }
    }

    /// List all available blend modes
    pub fn all() -> &'static [BlendMode] {
        &[
            BlendMode::Normal,
            BlendMode::Add,
            BlendMode::Subtract,
            BlendMode::Multiply,
            BlendMode::Screen,
            BlendMode::Overlay,
            BlendMode::SoftLight,
            BlendMode::HardLight,
            BlendMode::Lighten,
            BlendMode::Darken,
            BlendMode::ColorDodge,
            BlendMode::ColorBurn,
            BlendMode::Difference,
            BlendMode::Exclusion,
        ]
    }
}

/// Resize mode for automatic content fitting (Phase 1, Month 6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResizeMode {
    /// Fill - Scale to cover entire composition, crop excess
    Fill,
    /// Fit - Scale to fit within composition, letterbox/pillarbox
    #[default]
    Fit,
    /// Stretch - Non-uniform scale to fill composition exactly
    Stretch,
    /// Original - 1:1 pixel mapping, no scaling
    Original,
}

impl ResizeMode {
    /// Calculate transform matrix for this resize mode
    /// Returns scale and translation to apply
    pub fn calculate_transform(&self, source_size: Vec2, target_size: Vec2) -> (Vec2, Vec2) {
        // Prevent division by zero if source is empty
        if source_size.x.abs() < f32::EPSILON || source_size.y.abs() < f32::EPSILON {
            return (Vec2::ZERO, Vec2::ZERO);
        }

        // Prevent weird behavior if target is empty
        if target_size.x.abs() < f32::EPSILON || target_size.y.abs() < f32::EPSILON {
            return (Vec2::ZERO, Vec2::ZERO);
        }

        match self {
            ResizeMode::Fill => {
                // Scale to cover (largest dimension fills, crop other)
                let scale_x = target_size.x / source_size.x;
                let scale_y = target_size.y / source_size.y;
                let scale = scale_x.max(scale_y);
                (Vec2::splat(scale), Vec2::ZERO)
            }
            ResizeMode::Fit => {
                // Scale to fit (smallest dimension fills, letterbox other)
                let scale_x = target_size.x / source_size.x;
                let scale_y = target_size.y / source_size.y;
                let scale = scale_x.min(scale_y);
                (Vec2::splat(scale), Vec2::ZERO)
            }
            ResizeMode::Stretch => {
                // Non-uniform scale to fill exactly
                let scale = target_size / source_size;
                (scale, Vec2::ZERO)
            }
            ResizeMode::Original => {
                // No scaling, 1:1 pixel mapping
                (Vec2::ONE, Vec2::ZERO)
            }
        }
    }
}

/// Transform properties for layers (Phase 1, Month 4)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    /// Position offset in pixels (X, Y)
    pub position: Vec2,
    /// Scale factor (Width, Height) - 1.0 = 100%
    pub scale: Vec2,
    /// Rotation in radians (X, Y, Z) - Euler angles
    pub rotation: Vec3,
    /// Anchor point for transform origin (0-1 normalized, 0.5 = center)
    pub anchor: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: Vec3::ZERO,
            anchor: Vec2::splat(0.5), // Center by default
        }
    }
}

impl Transform {
    /// Create a new identity transform
    pub fn identity() -> Self {
        Self::default()
    }

    /// Create transform with position
    pub fn with_position(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create transform with scale
    pub fn with_scale(scale: Vec2) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Create transform with uniform scale
    pub fn with_uniform_scale(scale: f32) -> Self {
        Self {
            scale: Vec2::splat(scale),
            ..Default::default()
        }
    }

    /// Create transform with rotation (in radians)
    pub fn with_rotation(rotation: Vec3) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    /// Set Z rotation (most common for 2D)
    pub fn with_rotation_z(angle: f32) -> Self {
        Self {
            rotation: Vec3::new(0.0, 0.0, angle),
            ..Default::default()
        }
    }

    /// Calculate 4x4 transformation matrix
    /// Order: Translate → Rotate → Scale (TRS)
    pub fn to_matrix(&self, content_size: Vec2) -> Mat4 {
        // Calculate pivot point (origin for rotation/scale) based on anchor
        // For 0..1 meshes, (0,0) is Top-Left.
        // Pivot needs to be absolute offset from Top-Left.
        let pivot = content_size * self.anchor;

        // Build transformation matrix
        // 1. Translate pivot to origin
        let translate_pivot_to_origin = Mat4::from_translation(Vec3::new(-pivot.x, -pivot.y, 0.0));

        // 2. Scale
        let scale = Mat4::from_scale(Vec3::new(self.scale.x, self.scale.y, 1.0));

        // 3. Rotate (Euler XYZ order)
        let rotation = Mat4::from_euler(
            glam::EulerRot::XYZ,
            self.rotation.x,
            self.rotation.y,
            self.rotation.z,
        );

        // 4. Translate back to pivot + apply final position
        let translate_final = Mat4::from_translation(Vec3::new(
            pivot.x + self.position.x,
            pivot.y + self.position.y,
            0.0,
        ));

        // Combine: Final Translation → Rotation → Scale → Pivot Translation
        translate_final * rotation * scale * translate_pivot_to_origin
    }

    /// Apply resize mode to this transform
    pub fn apply_resize_mode(&mut self, mode: ResizeMode, source_size: Vec2, target_size: Vec2) {
        let (scale, position) = mode.calculate_transform(source_size, target_size);
        self.scale = scale;
        self.position = position;
    }
}

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
    pub effect_chain: super::effects::EffectChain,
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
            effect_chain: super::effects::EffectChain::default(),
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

/// Composition metadata and master controls (Phase 1, Month 5)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Composition {
    /// Composition name
    pub name: String,
    /// Optional description
    pub description: String,
    /// Master opacity (M) - global opacity multiplier (Phase 1, Month 4)
    pub master_opacity: f32,
    /// Master speed (S) - global speed multiplier (Phase 1, Month 5)
    pub master_speed: f32,
    /// Composition size in pixels (width, height)
    pub size: (u32, u32),
    /// Frame rate (FPS) for playback
    pub frame_rate: f32,
}

impl Default for Composition {
    fn default() -> Self {
        Self {
            name: "Untitled Composition".to_string(),
            description: String::new(),
            master_opacity: 1.0,
            master_speed: 1.0,
            size: (1920, 1080),
            frame_rate: 60.0,
        }
    }
}

impl Composition {
    /// Create a new composition
    pub fn new(name: impl Into<String>, size: (u32, u32), frame_rate: f32) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            master_opacity: 1.0,
            master_speed: 1.0,
            size,
            frame_rate,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set master opacity (clamped 0.0-1.0)
    pub fn set_master_opacity(&mut self, opacity: f32) {
        self.master_opacity = opacity.clamp(0.0, 1.0);
    }

    /// Set master speed (clamped 0.1-10.0)
    pub fn set_master_speed(&mut self, speed: f32) {
        self.master_speed = speed.clamp(0.1, 10.0);
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_mode_shader_function() {
        assert_eq!(BlendMode::Normal.shader_function(), "blend_normal");
        assert_eq!(BlendMode::Multiply.shader_function(), "blend_multiply");
        assert_eq!(BlendMode::Screen.shader_function(), "blend_screen");
    }

    #[test]
    fn test_layer_creation() {
        let layer = Layer::new(1, "Test Layer")
            .with_paint(100)
            .with_blend_mode(BlendMode::Multiply)
            .with_opacity(0.5);

        assert_eq!(layer.id, 1);
        assert_eq!(layer.name, "Test Layer");
        assert_eq!(layer.paint_id, Some(100));
        assert_eq!(layer.blend_mode, BlendMode::Multiply);
        assert_eq!(layer.opacity, 0.5);
        assert!(layer.visible);
    }

    #[test]
    fn test_layer_should_render() {
        let mut layer = Layer::new(1, "Test");

        // Not visible without paint
        assert!(!layer.should_render());

        layer.paint_id = Some(100);
        assert!(layer.should_render());

        layer.visible = false;
        assert!(!layer.should_render());

        layer.visible = true;
        layer.opacity = 0.0;
        assert!(!layer.should_render());
    }

    #[test]
    fn test_layer_manager_basic() {
        let mut manager = LayerManager::new();

        let id1 = manager.create_layer("Layer 1");
        let id2 = manager.create_layer("Layer 2");

        assert_eq!(manager.len(), 2);
        assert!(manager.get_layer(id1).is_some());
        assert!(manager.get_layer(id2).is_some());
    }

    #[test]
    fn test_layer_manager_remove() {
        let mut manager = LayerManager::new();

        let id = manager.create_layer("Test Layer");
        assert_eq!(manager.len(), 1);

        let removed = manager.remove_layer(id);
        assert!(removed.is_some());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_layer_manager_reorder() {
        let mut manager = LayerManager::new();

        let id1 = manager.create_layer("Layer 1");
        let id2 = manager.create_layer("Layer 2");
        let id3 = manager.create_layer("Layer 3");

        // Initially: [1, 2, 3]
        assert_eq!(manager.layers()[0].id, id1);
        assert_eq!(manager.layers()[2].id, id3);

        // Move layer 1 up: [2, 1, 3]
        manager.move_layer_up(id1);
        assert_eq!(manager.layers()[0].id, id2);
        assert_eq!(manager.layers()[1].id, id1);

        // Move layer 3 down: [3, 2, 1]
        manager.move_layer_down(id3);
        manager.move_layer_down(id3);
        assert_eq!(manager.layers()[0].id, id3);
    }

    #[test]
    fn test_layer_manager_visible_layers() {
        let mut manager = LayerManager::new();

        let id1 = manager.create_layer("Layer 1");
        let id2 = manager.create_layer("Layer 2");
        let id3 = manager.create_layer("Layer 3");

        // Set paint IDs so they can render
        manager.get_layer_mut(id1).unwrap().paint_id = Some(100);
        manager.get_layer_mut(id2).unwrap().paint_id = Some(101);
        manager.get_layer_mut(id3).unwrap().paint_id = Some(102);

        // All visible
        assert_eq!(manager.visible_layers().count(), 3);

        // Hide one layer
        manager.get_layer_mut(id2).unwrap().visible = false;
        assert_eq!(manager.visible_layers().count(), 2);

        // Solo one layer
        manager.get_layer_mut(id1).unwrap().solo = true;
        assert_eq!(manager.visible_layers().count(), 1);
        assert_eq!(manager.visible_layers().next().unwrap().id, id1);
    }

    #[test]
    fn test_layer_manager_duplicate() {
        let mut manager = LayerManager::new();

        let id1 = manager.create_layer("Original");
        manager.get_layer_mut(id1).unwrap().paint_id = Some(100);
        manager.get_layer_mut(id1).unwrap().opacity = 0.7;

        let id2 = manager.duplicate_layer(id1).unwrap();

        assert_eq!(manager.len(), 2);
        let dup = manager.get_layer(id2).unwrap();
        assert!(dup.name.contains("copy"));
        assert_eq!(dup.paint_id, Some(100));
        assert_eq!(dup.opacity, 0.7);
    }

    #[test]
    fn test_layer_mappings() {
        let mut layer = Layer::new(1, "Test");

        layer.add_mapping(10);
        layer.add_mapping(20);
        assert_eq!(layer.mapping_ids.len(), 2);

        // Adding duplicate should not increase count
        layer.add_mapping(10);
        assert_eq!(layer.mapping_ids.len(), 2);

        layer.remove_mapping(10);
        assert_eq!(layer.mapping_ids.len(), 1);
        assert_eq!(layer.mapping_ids[0], 20);
    }

    #[test]
    fn test_layer_manager_move_layer_to() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer("Layer 1");
        let id2 = manager.create_layer("Layer 2");
        let id3 = manager.create_layer("Layer 3");

        // Initial state: [1, 2, 3]
        assert_eq!(manager.layers()[0].id, id1);
        assert_eq!(manager.layers()[1].id, id2);
        assert_eq!(manager.layers()[2].id, id3);

        // Move Layer 1 to the end (index 2)
        // Expected: [2, 3, 1]
        assert!(manager.move_layer_to(id1, 2));
        assert_eq!(manager.layers()[0].id, id2);
        assert_eq!(manager.layers()[1].id, id3);
        assert_eq!(manager.layers()[2].id, id1);

        // Move Layer 3 to the beginning (index 0)
        // Expected: [3, 2, 1]
        assert!(manager.move_layer_to(id3, 0));
        assert_eq!(manager.layers()[0].id, id3);
        assert_eq!(manager.layers()[1].id, id2);
        assert_eq!(manager.layers()[2].id, id1);

        // Try moving to invalid index
        assert!(!manager.move_layer_to(id1, 100));
        // State should remain [3, 2, 1]
        assert_eq!(manager.layers()[2].id, id1);
    }

    #[test]
    fn test_layer_manager_rename() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer("Old Name");

        assert!(manager.rename_layer(id, "New Name"));
        assert_eq!(manager.get_layer(id).unwrap().name, "New Name");

        assert!(!manager.rename_layer(999, "Ghost"));
    }

    #[test]
    fn test_resize_mode_calculations() {
        let source = Vec2::new(100.0, 50.0); // 2:1 aspect ratio
        let target = Vec2::new(200.0, 200.0); // 1:1 aspect ratio

        // 1. FILL: Should scale to cover.
        // Scale X = 200/100 = 2.0
        // Scale Y = 200/50 = 4.0
        // Fill takes max(2.0, 4.0) = 4.0
        let (scale, pos) = ResizeMode::Fill.calculate_transform(source, target);
        assert_eq!(scale, Vec2::splat(4.0));
        assert_eq!(pos, Vec2::ZERO);

        // 2. FIT: Should scale to fit inside.
        // Scale X = 2.0, Scale Y = 4.0
        // Fit takes min(2.0, 4.0) = 2.0
        let (scale, pos) = ResizeMode::Fit.calculate_transform(source, target);
        assert_eq!(scale, Vec2::splat(2.0));
        assert_eq!(pos, Vec2::ZERO);

        // 3. STRETCH: Should scale distinct X and Y.
        let (scale, pos) = ResizeMode::Stretch.calculate_transform(source, target);
        assert_eq!(scale, Vec2::new(2.0, 4.0));
        assert_eq!(pos, Vec2::ZERO);

        // 4. ORIGINAL: 1:1
        let (scale, pos) = ResizeMode::Original.calculate_transform(source, target);
        assert_eq!(scale, Vec2::ONE);
        assert_eq!(pos, Vec2::ZERO);
    }

    #[test]
    fn test_transform_matrix_calculation() {
        // Identity
        let transform = Transform::identity();
        let size = Vec2::new(100.0, 100.0);
        let matrix = transform.to_matrix(size);
        assert_eq!(matrix, Mat4::IDENTITY);

        // Translation
        let transform = Transform::with_position(Vec2::new(10.0, 20.0));
        let matrix = transform.to_matrix(size);
        let expected = Mat4::from_translation(Vec3::new(10.0, 20.0, 0.0));
        assert_eq!(matrix, expected);

        // Rotation (90 deg around Z)
        // Note: Anchor is default (0.5, 0.5) = (50, 50)
        let transform = Transform::with_rotation_z(std::f32::consts::FRAC_PI_2);
        let matrix = transform.to_matrix(size);

        // Check a specific point: Right Edge (100, 50)
        // 1. To anchor (relative): (50, 0)
        // 2. Rotate 90 deg: (0, 50)
        // 3. Back from anchor: (50, 100)
        let point = Vec3::new(100.0, 50.0, 0.0);
        let transformed = matrix.transform_point3(point);

        assert!(
            (transformed.x - 50.0).abs() < 0.001,
            "Expected X=50.0, got {}",
            transformed.x
        );
        assert!(
            (transformed.y - 100.0).abs() < 0.001,
            "Expected Y=100.0, got {}",
            transformed.y
        );
    }

    #[test]
    fn test_transform_matrix_custom_anchor() {
        let size = Vec2::new(100.0, 100.0);

        // Test Rotation around Top-Left (0,0)
        let mut transform = Transform::with_rotation_z(std::f32::consts::FRAC_PI_2);
        transform.anchor = Vec2::new(0.0, 0.0); // Pivot at Top-Left

        let matrix = transform.to_matrix(size);

        // Point (100, 0) - Top Right
        // Should rotate 90 deg around (0,0) -> (0, 100)
        let point = Vec3::new(100.0, 0.0, 0.0);
        let transformed = matrix.transform_point3(point);

        assert!(
            transformed.x.abs() < 0.001,
            "Expected X=0.0, got {}",
            transformed.x
        );
        assert!(
            (transformed.y - 100.0).abs() < 0.001,
            "Expected Y=100.0, got {}",
            transformed.y
        );

        // Test Rotation around Bottom-Right (1,1)
        transform.anchor = Vec2::new(1.0, 1.0); // Pivot at (100, 100)
        let matrix = transform.to_matrix(size);

        // Point (0, 0) - Top Left
        // Relative to pivot (100, 100) is (-100, -100)
        // Rotated 90 deg around pivot:
        // x' = -y = -(-100) = 100
        // y' = x = -100
        // Absolute: Pivot + relative = (100+100, 100-100) = (200, 0)
        let point = Vec3::new(0.0, 0.0, 0.0);
        let transformed = matrix.transform_point3(point);

        assert!(
            (transformed.x - 200.0).abs() < 0.001,
            "Expected X=200.0, got {}",
            transformed.x
        );
        assert!(
            transformed.y.abs() < 0.001,
            "Expected Y=0.0, got {}",
            transformed.y
        );
    }

    #[test]
    fn test_layer_effective_opacity() {
        let mut manager = LayerManager::new();
        manager.composition.set_master_opacity(0.5);

        let id = manager.create_layer("Test");
        if let Some(layer) = manager.get_layer_mut(id) {
            layer.opacity = 0.5;
        }

        // Must drop mutable borrow before calling get_effective_opacity
        let layer = manager.get_layer(id).unwrap();
        // Effective = 0.5 * 0.5 = 0.25
        assert_eq!(manager.get_effective_opacity(layer), 0.25);

        manager.composition.set_master_opacity(1.0);
        let layer = manager.get_layer(id).unwrap();
        assert_eq!(manager.get_effective_opacity(layer), 0.5);

        manager.composition.set_master_opacity(0.0);
        let layer = manager.get_layer(id).unwrap();
        assert_eq!(manager.get_effective_opacity(layer), 0.0);
    }

    #[test]
    fn test_composition_limits() {
        let mut comp = Composition::default();

        comp.set_master_opacity(1.5);
        assert_eq!(comp.master_opacity, 1.0);

        comp.set_master_opacity(-0.5);
        assert_eq!(comp.master_opacity, 0.0);

        comp.set_master_speed(20.0);
        assert_eq!(comp.master_speed, 10.0);

        comp.set_master_speed(0.0);
        assert_eq!(comp.master_speed, 0.1);
    }

    #[test]
    fn test_layer_eject_all() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer("L1");
        let id2 = manager.create_layer("L2");

        manager.get_layer_mut(id1).unwrap().paint_id = Some(1);
        manager.get_layer_mut(id2).unwrap().paint_id = Some(2);

        manager.eject_all();

        assert_eq!(manager.get_layer(id1).unwrap().paint_id, None);
        assert_eq!(manager.get_layer(id2).unwrap().paint_id, None);
    }

    #[test]
    fn test_layer_groups_and_hierarchy() {
        let mut manager = LayerManager::new();
        let group_id = manager.create_group("Group");
        let layer_id = manager.create_layer("Child");

        assert!(manager.get_layer(group_id).unwrap().is_group);

        // Reparent
        manager.reparent_layer(layer_id, Some(group_id));
        assert_eq!(
            manager.get_layer(layer_id).unwrap().parent_id,
            Some(group_id)
        );

        // Check is_descendant
        assert!(manager.is_descendant(layer_id, group_id));
        assert!(!manager.is_descendant(group_id, layer_id));
    }

    #[test]
    fn test_cycle_prevention() {
        let mut manager = LayerManager::new();
        let group1 = manager.create_group("G1");
        let group2 = manager.create_group("G2");
        let group3 = manager.create_group("G3");

        // G1 -> G2 -> G3
        manager.reparent_layer(group2, Some(group1));
        manager.reparent_layer(group3, Some(group2));

        assert!(manager.is_descendant(group3, group1));

        // Try to make G1 a child of G3 (Cycle)
        manager.reparent_layer(group1, Some(group3));

        // Should be ignored
        assert_eq!(manager.get_layer(group1).unwrap().parent_id, None);
    }

    #[test]
    fn test_hierarchy_opacity() {
        let mut manager = LayerManager::new();
        manager.composition.set_master_opacity(1.0);

        let group_id = manager.create_group("Group");
        let layer_id = manager.create_layer("Layer");

        manager.get_layer_mut(group_id).unwrap().opacity = 0.5;
        manager.get_layer_mut(layer_id).unwrap().opacity = 0.5;

        manager.reparent_layer(layer_id, Some(group_id));

        let layer = manager.get_layer(layer_id).unwrap();
        // 0.5 * 0.5 * 1.0 = 0.25
        assert_eq!(manager.get_effective_opacity(layer), 0.25);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_blend_mode_integrity() {
        for mode in BlendMode::all() {
            let func = mode.shader_function();
            assert!(
                !func.is_empty(),
                "BlendMode {:?} has empty shader function",
                mode
            );
            assert!(
                func.starts_with("blend_"),
                "BlendMode {:?} shader function '{}' should start with 'blend_'",
                mode,
                func
            );
        }
    }

    #[test]
    fn test_resize_mode_advanced() {
        // Landscape Source (200x100)
        let src_landscape = Vec2::new(200.0, 100.0);
        // Portrait Target (100x200)
        let tgt_portrait = Vec2::new(100.0, 200.0);

        // 1. FILL: Must cover the target completely.
        // Scale X: 100/200 = 0.5
        // Scale Y: 200/100 = 2.0
        // Max(0.5, 2.0) = 2.0
        // Result size: 400x200 (covers 100x200)
        let (scale, pos) = ResizeMode::Fill.calculate_transform(src_landscape, tgt_portrait);
        assert_eq!(scale, Vec2::splat(2.0));
        assert_eq!(pos, Vec2::ZERO);

        // 2. FIT: Must fit inside the target.
        // Min(0.5, 2.0) = 0.5
        // Result size: 100x50 (fits in 100x200)
        let (scale, pos) = ResizeMode::Fit.calculate_transform(src_landscape, tgt_portrait);
        assert_eq!(scale, Vec2::splat(0.5));
        assert_eq!(pos, Vec2::ZERO);

        // Portrait Source (100x200)
        let src_portrait = Vec2::new(100.0, 200.0);
        // Landscape Target (200x100)
        let tgt_landscape = Vec2::new(200.0, 100.0);

        // 3. FILL
        // Scale X: 200/100 = 2.0
        // Scale Y: 100/200 = 0.5
        // Max = 2.0
        let (scale, _) = ResizeMode::Fill.calculate_transform(src_portrait, tgt_landscape);
        assert_eq!(scale, Vec2::splat(2.0));

        // 4. FIT
        // Min = 0.5
        let (scale, _) = ResizeMode::Fit.calculate_transform(src_portrait, tgt_landscape);
        assert_eq!(scale, Vec2::splat(0.5));
    }

    #[test]
    fn test_layer_removal_orphans_children() {
        let mut manager = LayerManager::new();
        let group_id = manager.create_group("Parent Group");
        let child_id = manager.create_layer("Child Layer");

        // Establish relationship
        manager.reparent_layer(child_id, Some(group_id));
        assert_eq!(
            manager.get_layer(child_id).unwrap().parent_id,
            Some(group_id)
        );

        // Remove Parent
        manager.remove_layer(group_id);

        // Assert Child is orphaned (parent_id is None)
        assert_eq!(manager.get_layer(child_id).unwrap().parent_id, None);
    }

    #[test]
    fn test_deep_hierarchy_cycle_detection() {
        let mut manager = LayerManager::new();
        // Create 5 layers: A -> B -> C -> D -> E
        let id_a = manager.create_group("A");
        let id_b = manager.create_group("B");
        let id_c = manager.create_group("C");
        let id_d = manager.create_group("D");
        let id_e = manager.create_group("E");

        manager.reparent_layer(id_b, Some(id_a));
        manager.reparent_layer(id_c, Some(id_b));
        manager.reparent_layer(id_d, Some(id_c));
        manager.reparent_layer(id_e, Some(id_d));

        // Verify E is descendant of A
        assert!(manager.is_descendant(id_e, id_a));

        // Attempt to make A a child of E (Cycle)
        manager.reparent_layer(id_a, Some(id_e));

        // Should be rejected, A's parent should remain None
        assert_eq!(manager.get_layer(id_a).unwrap().parent_id, None);
    }

    #[test]
    fn test_reparent_to_self() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer("Selfie");

        // Try to parent to self
        manager.reparent_layer(id, Some(id));

        assert_eq!(manager.get_layer(id).unwrap().parent_id, None);
    }

    #[test]
    fn test_layer_opacity_clamping() {
        let layer = Layer::new(1, "Clamped")
            .with_opacity(1.5)
            .with_opacity(-0.5);

        // Should be clamped to 0.0 (last operation was -0.5)
        assert_eq!(layer.opacity, 0.0);

        let layer = Layer::new(2, "Clamped High").with_opacity(1.5);
        assert_eq!(layer.opacity, 1.0);
    }

    #[test]
    fn test_resize_mode_zero_size() {
        // Source is zero
        let source_zero = Vec2::ZERO;
        let target = Vec2::new(100.0, 100.0);

        // Should return finite values (handle division by zero)
        let (scale, pos) = ResizeMode::Fill.calculate_transform(source_zero, target);
        assert!(scale.is_finite(), "Scale should be finite with zero source");
        assert!(
            pos.is_finite(),
            "Position should be finite with zero source"
        );
        assert_eq!(scale, Vec2::ZERO);

        let (scale, _) = ResizeMode::Fit.calculate_transform(source_zero, target);
        assert!(scale.is_finite());
        assert_eq!(scale, Vec2::ZERO);

        // Target is zero
        let source = Vec2::new(100.0, 100.0);
        let target_zero = Vec2::ZERO;

        let (scale, _) = ResizeMode::Fill.calculate_transform(source, target_zero);
        assert!(scale.is_finite());
        assert_eq!(scale, Vec2::ZERO);
    }
}