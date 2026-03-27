# Lina StyleUI Journal

## 2024-03-27 - Centralized Fallback & Empty State Labels
**Learning:** Found multiple places in `vorce-ui` panels where plain `.weak().italics()` or `ui.label(...)` configurations were used inline for fallback text and empty states (like "No parameters", "No presets", "No signal", etc). This leads to visual inconsistency.
**Action:** Replaced inline `ui.label` calls for fallback text with the standard globally accessible helper `crate::widgets::custom::render_info_label`. Always use this shared helper to maintain a consistent visual language across all panels for missing states.
