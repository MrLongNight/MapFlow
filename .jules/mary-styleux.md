# Mary StyleUX Journal

## 2024-05-24 – Safe Destructive Actions
**Learning:** Immediate "click-to-delete" buttons on nodes are dangerous in live performance contexts. Users may accidentally delete a critical node while trying to select or move it.
**Action:** Implemented a "Hold-to-Confirm" pattern (0.6s hold) for node deletion.
- **Visuals:** Added a circular progress indicator filling the delete button.
- **Interaction:** Requires holding mouse button OR focusing and holding Space/Enter.
- **Accessibility:** Ensure custom interactive rects support focus and keyboard events if they replace standard buttons. Replaced duplicated layout logic with a helper method to ensure hit-testing and rendering stay in sync.

## 2024-05-27 – Multi-Node Interactions & Layout Hygiene
**Learning:** Users expect "Shift+Click" selections to act as a cohesive group. Currently, dragging a selection only moves the specific node under the cursor, breaking the user's mental model of "Selection". Additionally, the lack of grid snapping leads to messy, hard-to-read graphs that degrade "at-a-glance" comprehension during live performance.
**Action:** Implemented Multi-Node Dragging and "Magnetic Grid Snapping" (20px).
- **Group Drag:** Moving one selected node moves all selected nodes, maintaining relative positions.
- **Collision:** Group movement checks collisions against non-selected nodes only.
- **Snapping:** Default 20px grid snap. Hold 'Alt' to bypass (Precision Mode).

## 2024-06-03 – Widget Safety & Accessibility
**Learning:** Custom interactive widgets (like sliders) must manually implement accessibility features that `egui` standard widgets provide out-of-the-box (keyboard navigation, `WidgetInfo`). Also, "Hold-to-trigger" buttons require careful state management to prevent auto-repeating triggers that can cause accidental destructive actions.
**Action:** When creating custom widgets:
1.  Implement `response.widget_info(...)`.
2.  Handle `has_focus()` and keyboard inputs (Arrows, Shift).
3.  For hold interactions, use a persistent "triggered" flag to enforce "trigger once per press".

## 2024-06-05 – Custom Button Accessibility Pattern
**Learning:** Custom-drawn buttons (like icon buttons) often lack visual focus indicators and accessibility metadata, making them unusable for keyboard and screen reader users. `egui::Response` provides `widget_info` which must be explicitly populated for custom widgets.
**Action:** established a standard pattern for custom buttons:
1.  **Focus:** Always draw a focus ring (e.g., `ui.visuals().selection.stroke`) when `response.has_focus()`.
2.  **Metadata:** Always call `response.widget_info` with a labeled `WidgetType::Button`. Use descriptive labels (e.g., `format!("{:?}", icon)`), not generic ones.
3.  **Interaction:** Ensure `Sense::click()` is used and verify keyboard activation support.

## 2026-02-17 – Universal Safety for Deletion
**Learning:** Destructive actions in context menus (e.g., "Delete Node", "Delete Connection") were still immediate-click, posing a risk during live performance. Consistency is key: if node deletion is safe on the canvas, it must be safe in the menu.
**Action:** Enforced "Hold-to-Confirm" on all delete actions.
- **Implementation:** Refactored `delete_button` widget to use `hold_to_action_button`.
- **Context Menus:** Replaced standard buttons with `hold_to_action_button` in `ModuleCanvas` menus.
- **Visuals:** Used standard trash icon "🗑" and `colors::ERROR_COLOR` (Red) to signal danger.

## 2026-02-18 – Safe Reset & Keyboard Productivity
**Learning:** Users need a way to experiment freely with effects, which requires a quick "Reset" to a known good state (defaults). Immediate reset buttons are too dangerous during live performance.
**Action:** Implemented a standard "Safe Reset" pattern:
- **Effect Level:** "Hold-to-Confirm" button (0.6s) in the effect header.
- **Parameter Level:** Right-click context menu "Reset to Default" for individual controls.
- **Power User:** Added keyboard shortcuts (Alt+Arrow Up/Down) to drag handles for quick reordering without mouse drag, improving accessibility and speed.

## 2026-02-19 – Eradicating Live-Performance Hazards
**Learning:** Destructive actions ("Reset to Default", "Delete Module", "Remove Paint", "Eject All Layers") scattered across various panels remained as immediate-click buttons, creating severe live-performance hazards when performing UI actions in a hurry.
**Action:** Pushed the "Hold-to-Confirm" pattern globally to all immediate resetting or deleting panel actions.
- **Implementation:** Mass-replaced `ui.button("Reset to Default").clicked()` and similar calls (`btn-reset-defaults`, `menu-delete`, `btn-eject-all`) with `hold_to_action_button` across effect chain panels, context menus, sidebar, and edge blend panels.
- **Consistency:** Ensured the same `WARN_COLOR` for reset actions and `ERROR_COLOR` for destructive actions are consistently used with the required `0.6s` delay to enforce user intent.