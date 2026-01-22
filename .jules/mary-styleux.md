# Mary StyleUX Journal

## 2024-10-25 – Media Properties Panel Safety & Alignment
**Learning:** The current "Media File Properties" panel places "Reset" buttons (destructive actions) immediately below slider groups. In a live performance context, this creates a high risk of accidental resets when trying to adjust the bottom-most slider. Additionally, slider labels are currently placed to the right or inconsistently, increasing cognitive load when scanning values.

**Action:**
1. Move "Reset" actions to the section header (top-right), clearly separated from the controls.
2. Use `egui::Grid` to enforce a strict "Label | Control" layout for all property groups (Color, Transform, Appearance).
3. Ensure consistent spacing and alignment to reduce visual noise.
