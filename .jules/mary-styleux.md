# Mary StyleUX Journal

## 2024-05-23 – [Initial Observation]
**Learning:** MapFlow uses `egui` immediate mode GUI. The "Node Properties" panel is implemented in `module_canvas.rs` within `render_properties_popup`. The current implementation for Media File properties uses small emoji buttons for transport control, which are hard to hit and lack active state feedback.
**Action:** Replace emoji-only buttons with `selectable_label` or larger buttons that indicate state. Use explicit `RichText` for better legibility.

## 2024-05-23 – [Live Performance Header Pattern]
**Learning:** Users in live performance environments need "at a glance" status for critical time-based nodes (like Media Players). Standard small labels and split controls increase cognitive load and risk of error under stress.
**Action:** Implemented a reusable "Live Performance Header" pattern consisting of:
1. Large Monospace Timecode (Size 22+, High Contrast).
2. Consolidated Transport Bar with large (60x40px minimum) hit targets.
3. Color-coded active states (Green=Play, Yellow=Pause).
This pattern should be applied to other time-based nodes (e.g., Timeline, Sequencer) in the future.

## 2024-05-23 – [Safe Reset Pattern]
**Learning:** Destructive actions like "Reset" were previously placed adjacent to interactive controls or at the bottom of forms, risking accidental clicks and poor visibility.
**Action:** Implemented a standard pattern for property sections:
1. Place "Reset" buttons in the top-right of the section header or content area.
2. Use distinct, smaller visual styling (e.g., "↺ Reset" or icon-only) compared to primary actions.
3. Ensure tooltips (`on_hover_text`) clearly describe the scope of the reset (e.g., "Reset Transform defaults").

## 2024-05-23 – [Async UI Action Pattern]
**Learning:** Blocking file dialogs (`rfd::FileDialog::pick_file`) in the immediate mode UI thread freeze the entire application rendering, which is unacceptable for live performance software.
**Action:** Implemented an "Async UI Action Pattern":
1. Define a variant in `UIAction` (e.g., `PickMediaFile`).
2. Dispatch this action from the UI (instead of calling blocking code).
3. Handle the action in the main event loop by spawning a `tokio` task.
4. Send the result back to the main thread via an internal channel (e.g., `McpAction` or dedicated channel).

## 2026-01-28 – [Docked Inspector Preference]
**Learning:** Floating property popups (`egui::Window`) in a node graph environment obscure the context (the graph connections) and are unpredictable in positioning/focus management, making them unsafe for live tweaking.
**Action:** Migrated "Node Properties" from a popup to a dedicated, docked `InspectorPanel` on the right side. This ensures the node graph remains visible while editing, and provides a stable, predictable target for mouse interaction. Future node-based editors (e.g., Shader Graph) should follow this pattern.
