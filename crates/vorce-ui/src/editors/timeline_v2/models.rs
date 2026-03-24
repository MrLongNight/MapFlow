use vorce_core::module::ModuleId;
use serde::{Deserialize, Serialize};

/// Show orchestration mode for module arrangement.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ShowMode {
    /// Fully automatic module switching by timeline time.
    #[default]
    FullyAutomated,
    /// Timeline advances automatically, module switch is confirmed manually.
    SemiAutomated,
    /// Module switching is manual only (timeline acts as arrangement board).
    Manual,
    /// Hybrid logic combining time and triggers.
    Hybrid,
    /// Playback stops at markers, waiting for explicit trigger to continue.
    Trackline,
}

impl ShowMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::FullyAutomated => "Fully Auto",
            Self::SemiAutomated => "Semi Auto",
            Self::Manual => "Manual",
            Self::Hybrid => "Hybrid",
            Self::Trackline => "Trackline",
        }
    }
}

/// A scheduled module block on the show timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModuleArrangementItem {
    /// Unique ID for stable runtime selection.
    pub id: u64,
    /// Target module.
    pub module_id: ModuleId,
    /// Block start time in seconds.
    pub start_time: f32,
    /// Block duration in seconds.
    pub duration: f32,
    /// Whether this block is active in runtime.
    pub enabled: bool,
    /// Trigger that must be active to start this block (Hybrid Mode).
    pub start_trigger: Option<String>,
}

impl Default for ModuleArrangementItem {
    fn default() -> Self {
        Self {
            id: 0,
            module_id: 0,
            start_time: 0.0,
            duration: 8.0,
            enabled: true,
            start_trigger: None,
        }
    }
}

impl ModuleArrangementItem {
    pub fn end_time(&self) -> f32 {
        self.start_time + self.duration.max(0.1)
    }
}
