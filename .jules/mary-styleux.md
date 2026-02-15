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
