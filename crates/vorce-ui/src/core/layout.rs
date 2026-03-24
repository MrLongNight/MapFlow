use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Represents one of the 5 predefined layout slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum UiSlot {
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

/// Unique identifiers for all available UI panels/modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum PanelId {
    MediaBrowser,
    Dashboard,
    MasterControls,
    AudioPanel,
    Inspector,
    Timeline,
    ModuleCanvas,
    NodeEditor,
    Preview,
    EffectChain,
}

/// Manages the distribution of panels into UI slots.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlotManager {
    /// Mapping of slot to list of panel IDs.
    pub assignments: HashMap<UiSlot, Vec<PanelId>>,
    /// Visibility of each panel.
    pub visibility: HashMap<PanelId, bool>,
}

impl Default for SlotManager {
    fn default() -> Self {
        let mut assignments = HashMap::new();

        // Default Configuration (Hybrid System Standard)
        assignments.insert(UiSlot::Top, vec![]); // Toolbar/Menu handled separately for now
        assignments.insert(UiSlot::Left, vec![PanelId::Preview, PanelId::MediaBrowser, PanelId::Dashboard, PanelId::AudioPanel]);
        assignments.insert(UiSlot::Right, vec![PanelId::Inspector, PanelId::EffectChain]);
        assignments.insert(UiSlot::Bottom, vec![PanelId::Timeline]);
        assignments.insert(UiSlot::Center, vec![PanelId::ModuleCanvas]);

        let mut visibility = HashMap::new();
        visibility.insert(PanelId::MediaBrowser, true);
        visibility.insert(PanelId::Dashboard, true);
        visibility.insert(PanelId::MasterControls, true);
        visibility.insert(PanelId::AudioPanel, false);
        visibility.insert(PanelId::Inspector, true);
        visibility.insert(PanelId::Timeline, true);
        visibility.insert(PanelId::ModuleCanvas, true);
        visibility.insert(PanelId::NodeEditor, false);
        visibility.insert(PanelId::Preview, true);
        visibility.insert(PanelId::EffectChain, true);

        Self {
            assignments,
            visibility,
        }
    }
}

impl SlotManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns a panel to a slot.
    pub fn assign_to_slot(&mut self, panel: PanelId, slot: UiSlot) {
        // Remove from any existing slot first
        for panels in self.assignments.values_mut() {
            panels.retain(|&p| p != panel);
        }

        if let Some(panels) = self.assignments.get_mut(&slot) {
            if !panels.contains(&panel) {
                panels.push(panel);
            }
        }
    }

    /// Returns the list of panels assigned to a slot.
    pub fn get_panels(&self, slot: UiSlot) -> Vec<PanelId> {
        self.assignments.get(&slot).cloned().unwrap_or_default()
    }

    /// Toggles visibility of a panel.
    pub fn set_visible(&mut self, panel: PanelId, visible: bool) {
        self.visibility.insert(panel, visible);
    }

    /// Returns true if a panel is visible.
    pub fn is_visible(&self, panel: PanelId) -> bool {
        *self.visibility.get(&panel).unwrap_or(&false)
    }
}
