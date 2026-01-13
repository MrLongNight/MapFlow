# Lina StyleUI Journal

## 2026-01-24 – Toolbar Hierarchy with Egui Frames
**Learning:** `egui::Frame` is essential for creating visual hierarchy in complex toolbars. By wrapping distinct sections (Context, Actions, View) in a Frame with `inner_margin` and `fill(ui.visuals().panel_fill)`, we create a "toolbar" feel that separates it from the canvas content without needing custom widget rendering.
**Action:** Use `egui::Frame` + `ui.horizontal` as the standard pattern for all panel headers and toolbars.

## 2026-01-24 – Egui 0.27 Slider Constraints
**Learning:** `egui::Slider` in v0.27 does not support a fluent `.width()` builder method. Layout sizing for sliders must be handled by the parent container (e.g., `ui.add_sized` or `ui.with_layout`) or by accepting default sizing behavior.
**Action:** Avoid `.width()` on Sliders; rely on layout containers or `ui.add_sized` if precise width is critical.
