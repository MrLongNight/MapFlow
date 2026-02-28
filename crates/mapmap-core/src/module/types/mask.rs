//!
//! Mask types.
//!

use serde::{Deserialize, Serialize};

/// Defines the geometry or texture used to mask a layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaskType {
    /// Mask using an external image file.
    File {
        /// File path to the mask asset.
        path: String,
    },
    /// Mask using a procedural geometric shape.
    Shape(MaskShape),
    /// Mask using a linear or radial gradient.
    Gradient {
        /// Orientation of the gradient in degrees.
        angle: f32,
        /// Edge smoothness of the gradient transition.
        softness: f32,
    },
}

/// Available procedural shapes for masks.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MaskShape {
    /// Circular mask.
    Circle,
    /// Rectangular mask.
    Rectangle,
    /// Triangular mask.
    Triangle,
    /// Star-shaped mask.
    Star,
    /// Elliptical mask.
    Ellipse,
}
