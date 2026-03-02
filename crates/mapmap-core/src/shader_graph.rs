//! Shader Graph System - Node-based Visual Shader Programming
//!
//! Phase 3: Effects Pipeline
//! Provides a flexible node-based system for creating custom shaders and effects

use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a shader graph node
pub type NodeId = u64;

/// Unique identifier for a shader graph
pub type GraphId = u64;

/// Data type for shader graph connections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// Single floating point value
    Float,
    /// 2D vector
    Vec2,
    /// 3D vector
    Vec3,
    /// 4D vector
    Vec4,
    /// RGB(A) color
    Color,
    /// 2D Texture
    Texture,
    /// Texture Sampler
    Sampler,
}

impl DataType {
    /// Get WGSL type string
    pub fn wgsl_type(&self) -> &'static str {
        match self {
            DataType::Float => "f32",
            DataType::Vec2 => "vec2<f32>",
            DataType::Vec3 => "vec3<f32>",
            DataType::Vec4 => "vec4<f32>",
            DataType::Color => "vec4<f32>",
            DataType::Texture => "texture_2d<f32>",
            DataType::Sampler => "sampler",
        }
    }
}

/// Input socket on a shader graph node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputSocket {
    /// Name of the socket
    pub name: String,
    /// Expected data type
    pub data_type: DataType,
    /// Default value if nothing is connected (for numeric inputs)
    pub default_value: Option<Vec4>,
    /// Connected output source: (node_id, output_socket_name)
    pub connected_output: Option<(NodeId, String)>,
}

/// Output socket on a shader graph node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputSocket {
    /// Name of the socket
    pub name: String,
    /// Output data type
    pub data_type: DataType,
}

/// Shader graph node type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    // Input nodes
    /// Texture input node
    TextureInput,
    /// Time input node (seconds)
    TimeInput,
    /// UV Coordinate input
    UVInput,
    /// Parameter input (float, vec, color)
    ParameterInput,
    /// Audio analysis input
    AudioInput,

    // Math nodes
    /// Addition
    Add,
    /// Subtraction
    Subtract,
    /// Multiplication
    Multiply,
    /// Division
    Divide,
    /// Power function
    Power,
    /// Sine function
    Sin,
    /// Cosine function
    Cos,
    /// Clamp value to range
    Clamp,
    /// Linear interpolation (Mix)
    Mix,
    /// Smoothstep interpolation
    Smoothstep,

    // Color nodes
    /// Color ramp / Gradient
    ColorRamp,
    /// HSV to RGB conversion
    HSVToRGB,
    /// RGB to HSV conversion
    RGBToHSV,
    /// Desaturate color
    Desaturate,
    /// Adjust brightness
    Brightness,
    /// Adjust contrast
    Contrast,

    // Texture operations
    /// Sample a texture
    TextureSample,
    /// Sample a texture with LOD
    TextureSampleLod,
    /// Combine multiple textures
    TextureCombine,
    /// Transform UV coordinates
    UVTransform,
    /// Distort UV coordinates
    UVDistort,

    // Effects
    /// Gaussian Blur
    Blur,
    /// Glow / Bloom effect
    Glow,
    /// Chromatic Aberration
    ChromaticAberration,
    /// Kaleidoscope effect
    Kaleidoscope,
    /// Pixel Sort / Glitch
    PixelSort,
    /// Edge Detection
    EdgeDetect,
    /// Displacement Mapping
    Displacement,

    // Utility
    /// Split vector/color into components
    Split,
    /// Combine components into vector/color
    Combine,
    /// Remap value range
    Remap,

    // Output
    /// Final Output Node
    Output,
}

impl NodeType {
    /// Get the display name for this node type
    pub fn display_name(&self) -> &'static str {
        match self {
            NodeType::TextureInput => "Texture Input",
            NodeType::TimeInput => "Time",
            NodeType::UVInput => "UV Coordinates",
            NodeType::ParameterInput => "Parameter",
            NodeType::AudioInput => "Audio Input",
            NodeType::Add => "Add",
            NodeType::Subtract => "Subtract",
            NodeType::Multiply => "Multiply",
            NodeType::Divide => "Divide",
            NodeType::Power => "Power",
            NodeType::Sin => "Sine",
            NodeType::Cos => "Cosine",
            NodeType::Clamp => "Clamp",
            NodeType::Mix => "Mix",
            NodeType::Smoothstep => "Smoothstep",
            NodeType::ColorRamp => "Color Ramp",
            NodeType::HSVToRGB => "HSV to RGB",
            NodeType::RGBToHSV => "RGB to HSV",
            NodeType::Desaturate => "Desaturate",
            NodeType::Brightness => "Brightness",
            NodeType::Contrast => "Contrast",
            NodeType::TextureSample => "Texture Sample",
            NodeType::TextureSampleLod => "Texture Sample LOD",
            NodeType::TextureCombine => "Texture Combine",
            NodeType::UVTransform => "UV Transform",
            NodeType::UVDistort => "UV Distort",
            NodeType::Blur => "Blur",
            NodeType::Glow => "Glow",
            NodeType::ChromaticAberration => "Chromatic Aberration",
            NodeType::Kaleidoscope => "Kaleidoscope",
            NodeType::PixelSort => "Pixel Sort",
            NodeType::EdgeDetect => "Edge Detect",
            NodeType::Displacement => "Displacement",
            NodeType::Split => "Split Channels",
            NodeType::Combine => "Combine Channels",
            NodeType::Remap => "Remap Range",
            NodeType::Output => "Output",
        }
    }

    /// Get the category for node palette
    pub fn category(&self) -> &'static str {
        match self {
            NodeType::TextureInput
            | NodeType::TimeInput
            | NodeType::UVInput
            | NodeType::ParameterInput
            | NodeType::AudioInput => "Input",

            NodeType::Add
            | NodeType::Subtract
            | NodeType::Multiply
            | NodeType::Divide
            | NodeType::Power
            | NodeType::Sin
            | NodeType::Cos
            | NodeType::Clamp
            | NodeType::Mix
            | NodeType::Smoothstep => "Math",

            NodeType::ColorRamp
            | NodeType::HSVToRGB
            | NodeType::RGBToHSV
            | NodeType::Desaturate
            | NodeType::Brightness
            | NodeType::Contrast => "Color",

            NodeType::TextureSample
            | NodeType::TextureSampleLod
            | NodeType::TextureCombine
            | NodeType::UVTransform
            | NodeType::UVDistort => "Texture",

            NodeType::Blur
            | NodeType::Glow
            | NodeType::ChromaticAberration
            | NodeType::Kaleidoscope
            | NodeType::PixelSort
            | NodeType::EdgeDetect
            | NodeType::Displacement => "Effects",

            NodeType::Split | NodeType::Combine | NodeType::Remap => "Utility",

            NodeType::Output => "Output",
        }
    }
}

/// Parameter value for animatable properties
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParameterValue {
    /// Single float value
    Float(f32),
    /// 2D Vector
    Vec2([f32; 2]),
    /// 3D Vector
    Vec3([f32; 3]),
    /// 4D Vector
    Vec4([f32; 4]),
    /// Color value (RGBA)
    Color([f32; 4]),
    /// String value (e.g. file paths)
    String(String),
}

/// Shader graph node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShaderNode {
    /// Unique Node ID
    pub id: NodeId,
    /// Type of node
    pub node_type: NodeType,
    /// Visual position in editor (x, y)
    pub position: (f32, f32),
    /// Input sockets
    pub inputs: Vec<InputSocket>,
    /// Output sockets
    pub outputs: Vec<OutputSocket>,
    /// Internal parameters (not exposed as sockets)
    pub parameters: HashMap<String, ParameterValue>,
}

impl ShaderNode {
    /// Create a new node of the specified type
    pub fn new(id: NodeId, node_type: NodeType) -> Self {
        let (inputs, outputs) = Self::create_sockets(&node_type);
        let parameters = Self::create_default_parameters(&node_type);

        Self {
            id,
            node_type,
            position: (0.0, 0.0),
            inputs,
            outputs,
            parameters,
        }
    }

    /// Create input and output sockets for a node type
    fn create_sockets(node_type: &NodeType) -> (Vec<InputSocket>, Vec<OutputSocket>) {
        let inputs = match node_type {
            NodeType::TextureInput => vec![],
            NodeType::TimeInput => vec![],
            NodeType::UVInput => vec![],

            NodeType::Add | NodeType::Subtract | NodeType::Multiply | NodeType::Divide => vec![
                InputSocket {
                    name: "A".to_string(),
                    data_type: DataType::Float,
                    default_value: Some(Vec4::new(0.0, 0.0, 0.0, 0.0)),
                    connected_output: None,
                },
                InputSocket {
                    name: "B".to_string(),
                    data_type: DataType::Float,
                    default_value: Some(Vec4::new(1.0, 0.0, 0.0, 0.0)),
                    connected_output: None,
                },
            ],

            NodeType::TextureSample => vec![
                InputSocket {
                    name: "Texture".to_string(),
                    data_type: DataType::Texture,
                    default_value: None,
                    connected_output: None,
                },
                InputSocket {
                    name: "UV".to_string(),
                    data_type: DataType::Vec2,
                    default_value: None,
                    connected_output: None,
                },
            ],

            NodeType::Output => vec![InputSocket {
                name: "Color".to_string(),
                data_type: DataType::Color,
                default_value: Some(Vec4::new(0.0, 0.0, 0.0, 1.0)),
                connected_output: None,
            }],

            _ => vec![], // TODO: Implement for all node types
        };

        let outputs = match node_type {
            NodeType::TextureInput => vec![OutputSocket {
                name: "Texture".to_string(),
                data_type: DataType::Texture,
            }],

            NodeType::TimeInput => vec![OutputSocket {
                name: "Time".to_string(),
                data_type: DataType::Float,
            }],

            NodeType::UVInput => vec![OutputSocket {
                name: "UV".to_string(),
                data_type: DataType::Vec2,
            }],

            NodeType::Add | NodeType::Subtract | NodeType::Multiply | NodeType::Divide => {
                vec![OutputSocket {
                    name: "Result".to_string(),
                    data_type: DataType::Float,
                }]
            }

            NodeType::TextureSample => vec![
                OutputSocket {
                    name: "Color".to_string(),
                    data_type: DataType::Color,
                },
                OutputSocket {
                    name: "Alpha".to_string(),
                    data_type: DataType::Float,
                },
            ],

            NodeType::Output => vec![],

            _ => vec![], // TODO: Implement for all node types
        };

        (inputs, outputs)
    }

    /// Create default parameters for a node type
    fn create_default_parameters(node_type: &NodeType) -> HashMap<String, ParameterValue> {
        let mut params = HashMap::new();

        match node_type {
            NodeType::Blur => {
                params.insert("radius".to_string(), ParameterValue::Float(1.0));
                params.insert("quality".to_string(), ParameterValue::Float(8.0));
            }
            NodeType::Brightness => {
                params.insert("amount".to_string(), ParameterValue::Float(0.0));
            }
            NodeType::Contrast => {
                params.insert("amount".to_string(), ParameterValue::Float(1.0));
            }
            _ => {}
        }

        params
    }
}

/// Shader graph - collection of connected nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShaderGraph {
    /// Unique Graph ID
    pub id: GraphId,
    /// Graph Name
    pub name: String,
    /// Collection of nodes mapped by ID
    pub nodes: HashMap<NodeId, ShaderNode>,
    next_node_id: NodeId,
}

impl ShaderGraph {
    /// Create a new empty shader graph
    pub fn new(id: GraphId, name: String) -> Self {
        Self {
            id,
            name,
            nodes: HashMap::new(),
            next_node_id: 1,
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node_type: NodeType) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;

        let node = ShaderNode::new(id, node_type);
        self.nodes.insert(id, node);

        id
    }

    /// Remove a node from the graph
    pub fn remove_node(&mut self, node_id: NodeId) -> Option<ShaderNode> {
        // TODO: Disconnect all connections to/from this node
        self.nodes.remove(&node_id)
    }

    /// Connect two nodes
    pub fn connect(
        &mut self,
        source_node: NodeId,
        source_output: &str,
        dest_node: NodeId,
        dest_input: &str,
    ) -> Result<(), String> {
        // Verify source node and output exist
        let source = self
            .nodes
            .get(&source_node)
            .ok_or_else(|| format!("Source node {} not found", source_node))?;

        if !source.outputs.iter().any(|o| o.name == source_output) {
            return Err(format!(
                "Output '{}' not found on source node",
                source_output
            ));
        }

        // Update destination node input
        let dest = self
            .nodes
            .get_mut(&dest_node)
            .ok_or_else(|| format!("Destination node {} not found", dest_node))?;

        let input = dest
            .inputs
            .iter_mut()
            .find(|i| i.name == dest_input)
            .ok_or_else(|| format!("Input '{}' not found on destination node", dest_input))?;

        input.connected_output = Some((source_node, source_output.to_string()));

        Ok(())
    }

    /// Disconnect an input
    pub fn disconnect(&mut self, node_id: NodeId, input_name: &str) -> Result<(), String> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or_else(|| format!("Node {} not found", node_id))?;

        let input = node
            .inputs
            .iter_mut()
            .find(|i| i.name == input_name)
            .ok_or_else(|| format!("Input '{}' not found", input_name))?;

        input.connected_output = None;

        Ok(())
    }

    /// Get the output node (should be exactly one)
    pub fn output_node(&self) -> Option<&ShaderNode> {
        self.nodes
            .values()
            .find(|n| n.node_type == NodeType::Output)
    }

    /// Validate the graph (check for cycles, disconnected nodes, etc.)
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check for output node
        if self.output_node().is_none() {
            errors.push("No output node found".to_string());
        }

        // TODO: Check for cycles, type mismatches, etc.

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_shader_graph() {
        let mut graph = ShaderGraph::new(1, "Test Graph".to_string());

        let _uv_node = graph.add_node(NodeType::UVInput);
        let _texture_node = graph.add_node(NodeType::TextureInput);
        let _sample_node = graph.add_node(NodeType::TextureSample);
        let _output_node = graph.add_node(NodeType::Output);

        assert_eq!(graph.nodes.len(), 4);
    }

    #[test]
    fn test_connect_nodes() {
        let mut graph = ShaderGraph::new(1, "Test Graph".to_string());

        let uv_node = graph.add_node(NodeType::UVInput);
        let sample_node = graph.add_node(NodeType::TextureSample);

        let result = graph.connect(uv_node, "UV", sample_node, "UV");
        assert!(result.is_ok());
    }

    #[test]
    fn test_node_types() {
        assert_eq!(NodeType::TextureSample.display_name(), "Texture Sample");
        assert_eq!(NodeType::Blur.category(), "Effects");
    }
}
