# Lina StyleUI Journal

## 2024-05-22 – [Visual Gap Analysis]
**Learning:** MapFlow's current UI is "flat dark" but lacks the "Cyber Dark" structure found in industry standards (Resolume, MadMapper).
- **Problem:** Visual Hierarchy is weak. Panels blend together. Lists are dense and unstyled. Active states are low-contrast.
- **Reference Standard:** Resolume/MadMapper use:
    - **Strong Borders:** Panels are clearly contained.
    - **Neon Accents:** Active states (play, selected) are high-contrast Cyan or Orange.
    - **Headers:** Content vs. Controls is strictly separated.
**Action:** Implement "Cyber Dark" theme:
1.  **Container Strategy:** Use `egui::Frame` with visible strokes/rounding for panels.
2.  **Accent Strategy:** Define a "Cyber Cyan" or "Neon Orange" for `Visuals.selection`.
3.  **Typography:** Ensure headers are distinct (e.g., Bold/Different Color) from data.

## 2024-05-22 – [Theme Definition]
**Learning:** `egui` default dark theme is functional but too "gray".
**Action:** Will look for `ctx.set_visuals` to inject:
- Background: Darker (almost black).
- Panel Background: Dark Gray.
- Stroke: Lighter Gray for definition.
- Accent: High saturation.

## 2024-05-23 – [Hierarchy via Color Depth]
**Learning:** To create hierarchy without adding layout complexity (margins/padding), color depth is effective.
- **Insight:** Separating `window_fill` (Background) and `panel_fill` (Foreground) creates a "floating panel" effect even with standard `egui` layouts.
- **Palette:**
    - Window: `(5, 5, 8)` (Almost Black/Navy)
    - Panel: `(18, 18, 24)` (Deep Navy)
    - Border: `(80, 80, 90)` (Blue-Grey)
**Action:** Applied these constants to `Theme::Resolume`. Future panels should respect `ui.visuals().panel_fill` to inherit this depth automatically.

## 2024-05-24 – [List & Table Patterns]
**Learning:** Using `ui.group` for list items creates excessive visual noise ("box-in-box").
- **Insight:** Clean lists use `egui::Frame` with subtle background variations (zebra striping) and no stroke for individual rows.
- **Pattern:**
    - **Selection:** `Visuals.selection.bg_fill.linear_multiply(0.2)` for row background.
    - **Striping:** `Visuals.faint_bg_color` for odd rows.
    - **Buttons:** Consolidate repeated widget logic into helpers (e.g., `icon_button`) to enforce consistent active/hover states.
**Action:** Refactored `LayerPanel` to use this pattern, removing nested groups and aligning controls horizontally.

## 2026-02-01 – [Node Visual Hierarchy]
**Learning:** Hardcoded colors in custom painting code (like `module_canvas`) drift from the central theme, causing visual inconsistency.
- **Insight:** Node editors are "Canvas" elements and should use the `Panel` color for bodies but require a distinct `Header` color to establish hierarchy within the node itself.
- **Pattern:**
    - **Node Body:** `colors::DARK_GREY` (matches panels).
    - **Node Header:** `colors::LIGHTER_GREY` (creates contrast vs body).
    - **Separator:** `colors::STROKE_GREY` (sharp definition).
**Action:** Refactored `ModuleCanvas` to use `crate::theme::colors` constants, enforcing the Cyber Dark palette on the node graph.

## 2026-02-16 – [Refactoring Mapping and Audio Panels]
**Learning:** `egui::Frame` with `corner_radius(0.0)` and zebra striping is essential for the Cyber Dark look.
- **Insight:** `MappingPanel` was using mixed UI paradigms. Refactoring it to use `render_panel_header` and consistent row layouts significantly improves readability.
- **Pattern:**
    - **Header:** `render_panel_header`
    - **List:** `egui::ScrollArea` + `egui::Frame` (zebra) + `ui.horizontal`
    - **Actions:** Right-aligned icon buttons (`delete_button`, `lock_button`, `solo_button`).
**Action:** Applied this pattern to `MappingPanel` and `AudioPanel`. Also added `lock_button` to `custom.rs` for reuse.

## 2024-05-24 – [Unified Widget Colors]
**Learning:** Hardcoded RGB values in widgets (like `AudioMeter` and overlays) create subtle visual noise and drift from the core theme.
- **Insight:** Even "utility" widgets like meters and overlays must strictly adhere to the `crate::theme::colors` palette to maintain the "Cyber Dark" immersion.
- **Pattern:**
    - **Utility Backgrounds:** `colors::DARKER_GREY` (for meter backgrounds, overlays).
    - **Utility Frames:** `colors::LIGHTER_GREY` (frames) + `colors::STROKE_GREY` (strokes).
    - **Geometry:** `CornerRadius::ZERO` (sharp corners) for all panels and overlays.
    - **Status Colors:** `colors::MINT_ACCENT` (Good/FPS), `colors::CYAN_ACCENT` (Info/Time), `colors::ERROR_COLOR` (Locked/Error).
**Action:** Removed hardcoded colors and rounded corners from `AudioMeter`, `StyledPanel`, `lock_button`, and `render_stats_overlay`. Replaced with theme constants and sharp corners.

## 2026-02-18 – [Strict Shape Geometry in Visualizers]
**Learning:** Some custom drawn UI elements like the `AudioPanel` visualizer were still using hardcoded float values (e.g., `2.0` or `1.0`) for corner radii, breaking the Cyber Dark theme's sharp aesthetic.
- **Insight:** Any `ui.painter().rect_filled` or `ui.painter().rect_stroke` must strictly use `egui::CornerRadius::ZERO` instead of literal float values to maintain consistency.
- **Action:** Updated `AudioPanel` to replace float rounding with `egui::CornerRadius::ZERO`. Added to guiding principles.

## 2026-03-01 – [Theme Visual Geometry Consistency]
**Learning:** Default style overrides for `egui::style::Widgets` in custom themes (like High Contrast, Synthwave, Dark, etc) still used rounded corners (e.g., `CornerRadius::same(2)`) which contradict the standard Cyber Dark theme requirements for MapFlow UI. Furthermore, custom input widgets (like `hold_to_action_button`) had hardcoded `CornerRadius::same(4)` or `CornerRadius::same(6)`.
- **Insight:** Global widget definitions and helper macros must follow MapFlow's unified visual structure. Rounded corners in some UI elements conflict with the rigid, sharp style intended for MapFlow.
- **Action:** Applied `CornerRadius::ZERO` globally across all visual definitions in `crate::core::theme` and custom widget utilities in `crate::widgets::custom`, strictly enforcing the Cyber Dark angularity for all interactive and structural components.

## 2024-05-24 – [Empty State Visibility]
**Learning:** Plain `ui.label("No data")` blends in with regular data points, creating visual confusion about whether data is missing or if "No data" is the actual value.
**Action:** Enforce empty/no data states to use `egui::RichText::new("...").weak().italics()` across all modules (e.g., "No matching nodes found.", "No MIDI devices"). This provides immediate visual differentiation.

## 2024-05-24 – [Empty State Consistency Update]
**Learning:** Several empty/no data states in various panels and inspectors lacked the italicization required to clearly differentiate them from standard data points.
**Action:** Applied `.weak().italics()` to these states across `layer.rs`, `source.rs`, `mapping_panel.rs`, and `effect_chain_panel.rs` to enforce consistency with the existing Cyber Dark theme guidelines.
