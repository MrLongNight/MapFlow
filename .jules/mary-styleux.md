## 2024-05-23 – Live Mode Implementation

**Learning:** Live performance environments require explicit safety mechanisms. Accidental drag-and-drop or deletion of nodes during a show can be catastrophic. Standard "Edit" interfaces are too dangerous for "Play" scenarios.
**Action:** Implemented a "Live Mode" (Canvas Lock) that disables structural changes (drag, delete, connect) while keeping parameter inspection active. This separation of "Composition" vs "Performance" intent is crucial for mapping software.

**Learning:** Slider constraints must be enforced at the input level. Visual widgets that rely on `start < end` logic can break if the underlying sliders allow setting `start > end`.
**Action:** Always wrap `ui.add(Slider)` with clamping logic if the value has dependencies on other values (like a time range). Implemented "proxy values" for sliders to handle special cases like "0.0 means End of File" intuitively.
