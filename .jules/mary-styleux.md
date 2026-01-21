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
