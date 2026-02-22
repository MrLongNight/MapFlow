# Lina StyleUI Journal

## 2024-05-23 – Initial Observation
**Learning:** The "Cyber Dark" theme is defined with specific colors (`DARK_GREY`, `CYAN_ACCENT`) and sharp corners (`CornerRadius::ZERO`), but `StyledPanel` in `widgets/panel.rs` uses rounded corners and hardcoded RGB values that deviate from the theme.
**Action:** specific widgets must be audited to ensure they reference `theme::colors` and `CornerRadius::ZERO` instead of hardcoded defaults.

## 2024-05-23 – LayerPanel Clutter
**Learning:** `LayerPanel` uses nested `ui.indent` which creates uneven spacing and wastes horizontal space. The "zebra striping" and selection highlight are implemented with ad-hoc logic in the loop, making it hard to maintain consistent row heights and alignments.
**Action:** Refactor list rendering to use a flat or controlled indentation approach with consistent row heights, and centralized styling for "selected" and "hovered" states.
