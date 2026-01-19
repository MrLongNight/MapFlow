# Mary StyleUX Journal

## 2024-05-23 – [Initial Observation]
**Learning:** MapFlow uses `egui` immediate mode GUI. The "Node Properties" panel is implemented in `module_canvas.rs` within `render_properties_popup`. The current implementation for Media File properties uses small emoji buttons for transport control, which are hard to hit and lack active state feedback.
**Action:** Replace emoji-only buttons with `selectable_label` or larger buttons that indicate state. Use explicit `RichText` for better legibility.
