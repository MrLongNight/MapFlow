use serde::{Deserialize, Serialize};
use crate::module::types::socket::ModuleSocket;
use crate::module::types::socket::ModuleSocketType;
use rand::RngExt;

/// Target parameter for a trigger input
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TriggerTarget {
    /// No target (default)
    #[default]
    None,
    /// Opacity value (0.0 to 1.0).
    Opacity,
    /// Brightness factor.
    Brightness,
    /// Contrast factor.
    Contrast,
    /// Saturation adjustment.
    Saturation,
    /// Hue shift in degrees.
    HueShift,
    /// Enumeration variant.
    ScaleX,
    /// Enumeration variant.
    ScaleY,
    /// Rotation angle.
    Rotation,
    /// Enumeration variant.
    OffsetX,
    /// Enumeration variant.
    OffsetY,
    /// Enumeration variant.
    FlipH,
    /// Enumeration variant.
    FlipV,
    /// Specific Effect Parameter (by name)
    Param(String),
}

/// Mapping mode for trigger value transformation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TriggerMappingMode {
    /// Direct mapping
    #[default]
    Direct,
    /// Fixed value when triggered
    Fixed,
    /// Random value in [min, max] range when triggered
    RandomInRange,
    /// Smoothed with attack/release
    Smoothed {
        /// Attack time in seconds
        attack: f32,
        /// Release time in seconds
        release: f32,
    },
}

/// Configuration for how a trigger input maps to a target parameter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerConfig {
    /// Target parameter to control
    pub target: TriggerTarget,
    /// Mapping mode
    pub mode: TriggerMappingMode,
    /// Minimum output value
    pub min_value: f32,
    /// Maximum output value
    pub max_value: f32,
    /// Invert the trigger value (1 - value)
    pub invert: bool,
    /// Threshold for Fixed mode
    pub threshold: f32,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            target: TriggerTarget::None,
            mode: TriggerMappingMode::Direct,
            min_value: 0.0,
            max_value: 1.0,
            invert: false,
            threshold: 0.5,
        }
    }
}

impl TriggerConfig {
    /// Associated function.
    pub fn for_target(target: TriggerTarget) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }

    /// Method implementation.
    pub fn apply(&self, raw_value: f32) -> f32 {
        let value = if self.invert {
            1.0 - raw_value
        } else {
            raw_value
        };

        match &self.mode {
            TriggerMappingMode::Direct => {
                self.min_value + (self.max_value - self.min_value) * value
            }
            TriggerMappingMode::Fixed => {
                if value > self.threshold {
                    self.max_value
                } else {
                    self.min_value
                }
            }
            TriggerMappingMode::RandomInRange => {
                if value > 0.0 {
                    let mut rng = rand::rng();
                    rng.random_range(self.min_value..=self.max_value)
                } else {
                    self.min_value
                }
            }
            TriggerMappingMode::Smoothed { .. } => {
                self.min_value + (self.max_value - self.min_value) * value
            }
        }
    }
}

/// Types of logic triggers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    /// Enumeration variant.
    AudioFFT {
        /// Component property or field.
        band: AudioBand,
        /// Component property or field.
        threshold: f32,
        /// Component property or field.
        output_config: AudioTriggerOutputConfig,
    },
    /// Enumeration variant.
    Random {
        /// Component property or field.
        min_interval_ms: u32,
        /// Component property or field.
        max_interval_ms: u32,
        /// Component property or field.
        probability: f32,
    },
    /// Enumeration variant.
    Fixed {
        /// Component property or field.
        interval_ms: u32,
        /// Component property or field.
        offset_ms: u32,
    },
    /// Enumeration variant.
    Midi {
        /// Component property or field.
        device: String,
        /// The MIDI channel (0-15) associated with this message.
        channel: u8,
        /// The MIDI note number (0-127).
        note: u8,
    },
    /// Enumeration variant.
    Osc {
        /// Component property or field.
        address: String,
    },
    /// Enumeration variant.
    Shortcut {
        /// Component property or field.
        key_code: String,
        /// Component property or field.
        modifiers: u8,
    },
    /// Enumeration variant.
    Beat,
}

/// Audio frequency bands for FFT trigger
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioBand {
    /// Enumeration variant.
    SubBass,
    /// Enumeration variant.
    Bass,
    /// Enumeration variant.
    LowMid,
    /// Enumeration variant.
    Mid,
    /// Enumeration variant.
    HighMid,
    /// Enumeration variant.
    UpperMid,
    /// Enumeration variant.
    Presence,
    /// Enumeration variant.
    Brilliance,
    /// Enumeration variant.
    Air,
    /// Enumeration variant.
    Peak,
    /// Enumeration variant.
    BPM,
}

/// Configuration for AudioFFT trigger outputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioTriggerOutputConfig {
    /// Component property or field.
    pub frequency_bands: bool,
    /// Component property or field.
    pub volume_outputs: bool,
    /// Component property or field.
    pub beat_output: bool,
    /// Component property or field.
    pub bpm_output: bool,
    #[serde(default)]
    /// Component property or field.
    pub inverted_outputs: std::collections::HashSet<String>,
}

impl Default for AudioTriggerOutputConfig {
    fn default() -> Self {
        Self {
            frequency_bands: false,
            volume_outputs: false,
            beat_output: true,
            bpm_output: false,
            inverted_outputs: std::collections::HashSet::new(),
        }
    }
}

impl AudioTriggerOutputConfig {
    /// Method implementation.
    pub fn generate_outputs(&self) -> Vec<ModuleSocket> {
        let mut outputs = Vec::new();

        if self.frequency_bands {
            let bands = [
                "SubBass Out",
                "Bass Out",
                "LowMid Out",
                "Mid Out",
                "HighMid Out",
                "UpperMid Out",
                "Presence Out",
                "Brilliance Out",
                "Air Out",
            ];
            for b in bands {
                outputs.push(ModuleSocket {
                    name: b.to_string(),
                    socket_type: ModuleSocketType::Trigger,
                });
            }
        }

        if self.volume_outputs {
            outputs.push(ModuleSocket {
                name: "RMS Volume".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
            outputs.push(ModuleSocket {
                name: "Peak Volume".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        if self.beat_output {
            outputs.push(ModuleSocket {
                name: "Beat Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        if self.bpm_output {
            outputs.push(ModuleSocket {
                name: "BPM Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        if outputs.is_empty() {
            outputs.push(ModuleSocket {
                name: "Beat Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        outputs
    }
}
