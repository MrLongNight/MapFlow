//!
//! Module part types.
//!

use crate::module::types::hue::HueNodeType;
use crate::module::types::layer::LayerType;
use crate::module::types::mask::MaskType;
use crate::module::types::module::ModulePartId;
use crate::module::types::modulizer::ModulizerType;
use crate::module::types::node_link::{LinkMode, NodeLinkData};
use crate::module::types::output::OutputType;
use crate::module::types::socket::{ModuleSocket, ModuleSocketType};
use crate::module::types::source::SourceType;
use crate::module::types::trigger::{TriggerConfig, TriggerType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the visual graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModulePart {
    /// Unique identifier for this entity.
    pub id: ModulePartId,
    /// Type and configuration data
    pub part_type: ModulePartType,
    /// 2D Position on canvas
    pub position: (f32, f32),
    /// Custom size (width, height)
    #[serde(default)]
    pub size: Option<(f32, f32)>,
    /// Link system configuration
    #[serde(default)]
    pub link_data: NodeLinkData,
    /// Input sockets
    pub inputs: Vec<ModuleSocket>,
    /// Output sockets
    pub outputs: Vec<ModuleSocket>,
    /// Trigger target configuration (Input Socket Index -> Target Parameter)
    #[serde(default)]
    pub trigger_targets: HashMap<String, TriggerConfig>,
}

impl ModulePart {
    /// Calculate the current sockets based on configuration
    pub fn compute_sockets(&self) -> (Vec<ModuleSocket>, Vec<ModuleSocket>) {
        let (mut inputs, mut outputs) = self.part_type.get_default_sockets();

        if self.link_data.mode == LinkMode::Master {
            outputs.push(ModuleSocket::output(
                "link_out",
                "Link Out",
                ModuleSocketType::Link,
            ));
        }

        if self.link_data.mode == LinkMode::Slave {
            inputs.push(ModuleSocket::input(
                "link_in",
                "Link In",
                ModuleSocketType::Link,
            ));
        }

        if self.link_data.trigger_input_enabled {
            inputs.push(
                ModuleSocket::input_mappable(
                    "trigger_vis_in",
                    "Trigger In (Vis)",
                    ModuleSocketType::Event,
                )
                .multi_input(),
            );
        }

        (inputs, outputs)
    }
}

/// Comprehensive enum of all node types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulePartType {
    /// Event-based trigger node.
    Trigger(TriggerType),
    /// A node that provides visual or audio content.
    Source(SourceType),
    /// A node used for cropping or shaping content.
    Mask(MaskType),
    /// A node that processes or modifies content (e.g., Effects).
    Modulizer(ModulizerType),
    /// A compositing layer within a scene.
    Layer(LayerType),
    /// Geometry definition for mapping.
    Mesh(crate::module::types::mesh::MeshType),
    /// Hue shift in degrees.
    Hue(HueNodeType),
    /// Enumeration variant.
    Output(OutputType),
}

impl ModulePartType {
    /// Method implementation.
    pub fn get_default_sockets(&self) -> (Vec<ModuleSocket>, Vec<ModuleSocket>) {
        match self {
            ModulePartType::Trigger(trigger_type) => {
                let outputs = match trigger_type {
                    TriggerType::AudioFFT { output_config, .. } => output_config.generate_outputs(),
                    _ => vec![ModuleSocket::standard_trigger_out(
                        "trigger_out",
                        "Trigger Out",
                    )],
                };
                (vec![], outputs)
            }
            ModulePartType::Mask(_) => (
                vec![
                    ModuleSocket::standard_media_in("media_in", "Media In").primary(),
                    ModuleSocket::standard_media_in("mask_in", "Mask In"),
                ],
                vec![ModuleSocket::standard_media_out("media_out", "Media Out").primary()],
            ),
            ModulePartType::Modulizer(_) => (
                vec![
                    ModuleSocket::standard_media_in("media_in", "Media In").primary(),
                    ModuleSocket::standard_trigger_in("trigger_in", "Trigger In"),
                ],
                vec![ModuleSocket::standard_media_out("media_out", "Media Out").primary()],
            ),
            ModulePartType::Layer(_) => (
                vec![
                    ModuleSocket::standard_media_in("media_in", "Input").primary(),
                    ModuleSocket::standard_trigger_in("trigger_in", "Trigger"),
                ],
                vec![ModuleSocket::standard_layer_out("layer_out", "Output").primary()],
            ),
            ModulePartType::Source(SourceType::BevyAtmosphere { .. })
            | ModulePartType::Source(SourceType::BevyHexGrid { .. })
            | ModulePartType::Source(SourceType::Bevy3DShape { .. })
            | ModulePartType::Source(SourceType::BevyCamera { .. }) => (
                vec![ModuleSocket::standard_trigger_in(
                    "trigger_in",
                    "Trigger In",
                )],
                vec![ModuleSocket::standard_media_out("media_out", "Media Out").primary()],
            ),
            ModulePartType::Source(SourceType::BevyParticles { .. }) => (
                vec![ModuleSocket::standard_trigger_in(
                    "spawn_trigger",
                    "Spawn Trigger",
                )],
                vec![ModuleSocket::standard_media_out("media_out", "Media Out").primary()],
            ),
            ModulePartType::Source(_) => (
                vec![ModuleSocket::standard_trigger_in(
                    "trigger_in",
                    "Trigger In",
                )],
                vec![ModuleSocket::standard_media_out("media_out", "Media Out").primary()],
            ),
            ModulePartType::Output(out) => match out {
                OutputType::Hue { .. } => (
                    vec![
                        ModuleSocket::standard_layer_in("layer_in", "Layer In").primary(),
                        ModuleSocket::standard_trigger_in("trigger_in", "Trigger In"),
                    ],
                    vec![],
                ),
                _ => (
                    vec![ModuleSocket::standard_layer_in("layer_in", "Layer In").primary()],
                    vec![],
                ),
            },
            ModulePartType::Mesh(_) => (
                vec![
                    ModuleSocket::standard_media_in("vertex_in", "Vertex In").primary(),
                    ModuleSocket::standard_trigger_in("control_in", "Control In"),
                ],
                vec![ModuleSocket::standard_media_out("geometry_out", "Geometry Out").primary()],
            ),
            ModulePartType::Hue(_) => (
                vec![
                    ModuleSocket::standard_trigger_in("brightness_in", "Brightness"),
                    ModuleSocket::standard_media_in("color_in", "Color (RGB)"),
                    ModuleSocket::standard_trigger_in("strobe_in", "Strobe"),
                ],
                vec![],
            ),
        }
    }
}

/// Simplified part type for UI creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartType {
    /// Event-based trigger node.
    Trigger,
    /// A node that provides visual or audio content.
    Source,
    /// GPU-accelerated 3D particle system.
    BevyParticles,
    /// Standard 3D geometric primitive (Cube, Sphere, etc.).
    Bevy3DShape,
    /// A node used for cropping or shaping content.
    Mask,
    /// Enumeration variant.
    Modulator,
    /// Geometry definition for mapping.
    Mesh,
    /// A compositing layer within a scene.
    Layer,
    /// Hue shift in degrees.
    Hue,
    /// Enumeration variant.
    Output,
}
