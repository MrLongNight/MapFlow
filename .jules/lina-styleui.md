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

## 2026-03-01 – [Panel Standardization]
**Learning:** Implemented "Cyber Dark" panel containers using `cyber_panel_frame` and `render_panel_header`.
- **Insight:** `egui::Window` can use `.frame(...)` to override the content frame, applying sharp corners and dark background.
- **Pattern:** For recursive trees (like Layer Panel), pass a `&mut usize` row index to implement global zebra striping across nested levels.
**Action:** Applied this pattern to Layer, Inspector, and Output panels.
