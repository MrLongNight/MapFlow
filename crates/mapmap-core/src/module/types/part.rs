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
    pub trigger_targets: HashMap<usize, TriggerConfig>,
}

impl ModulePart {
    /// Calculate the current sockets based on configuration
    pub fn compute_sockets(&self) -> (Vec<ModuleSocket>, Vec<ModuleSocket>) {
        let (mut inputs, mut outputs) = self.part_type.get_default_sockets();

        if self.link_data.mode == LinkMode::Master {
            outputs.push(ModuleSocket {
                name: "Link Out".to_string(),
                socket_type: ModuleSocketType::Link,
            });
        }

        if self.link_data.mode == LinkMode::Slave {
            inputs.push(ModuleSocket {
                name: "Link In".to_string(),
                socket_type: ModuleSocketType::Link,
            });
        }

        if self.link_data.trigger_input_enabled {
            inputs.push(ModuleSocket {
                name: "Trigger In (Vis)".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
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
                    _ => vec![ModuleSocket {
                        name: "Trigger Out".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    }],
                };
                (vec![], outputs)
            }
            ModulePartType::Mask(_) => (
                vec![
                    ModuleSocket {
                        name: "Media In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Mask In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                ],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Modulizer(_) => (
                vec![
                    ModuleSocket {
                        name: "Media In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Trigger In".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Layer(_) => (
                vec![
                    ModuleSocket {
                        name: "Input".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Trigger".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![ModuleSocket {
                    name: "Output".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
            ),
            ModulePartType::Source(SourceType::BevyAtmosphere { .. })
            | ModulePartType::Source(SourceType::BevyHexGrid { .. })
            | ModulePartType::Source(SourceType::Bevy3DShape { .. })
            | ModulePartType::Source(SourceType::BevyCamera { .. }) => (
                vec![ModuleSocket {
                    name: "Trigger In".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Source(SourceType::BevyParticles { .. }) => (
                vec![ModuleSocket {
                    name: "Spawn Trigger".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Source(_) => (
                vec![ModuleSocket {
                    name: "Trigger In".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Output(out) => match out {
                OutputType::Hue { .. } => (
                    vec![
                        ModuleSocket {
                            name: "Layer In".to_string(),
                            socket_type: ModuleSocketType::Layer,
                        },
                        ModuleSocket {
                            name: "Trigger In".to_string(),
                            socket_type: ModuleSocketType::Trigger,
                        },
                    ],
                    vec![],
                ),
                _ => (
                    vec![ModuleSocket {
                        name: "Layer In".to_string(),
                        socket_type: ModuleSocketType::Layer,
                    }],
                    vec![],
                ),
            },
            ModulePartType::Mesh(_) => (
                vec![
                    ModuleSocket {
                        name: "Vertex In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Control In".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![ModuleSocket {
                    name: "Geometry Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Hue(_) => (
                vec![
                    ModuleSocket {
                        name: "Brightness".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                    ModuleSocket {
                        name: "Color (RGB)".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Strobe".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
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
