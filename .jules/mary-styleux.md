# Mary StyleUX Journal

## 2024-05-24 – Safe Destructive Actions
**Learning:** Immediate "click-to-delete" buttons on nodes are dangerous in live performance contexts. Users may accidentally delete a critical node while trying to select or move it.
**Action:** Implemented a "Hold-to-Confirm" pattern (0.6s hold) for node deletion.
- **Visuals:** Added a circular progress indicator filling the delete button.
- **Interaction:** Requires holding mouse button OR focusing and holding Space/Enter.
- **Accessibility:** Ensure custom interactive rects support focus and keyboard events if they replace standard buttons. Replaced duplicated layout logic with a helper method to ensure hit-testing and rendering stay in sync.

## 2024-05-25 – Live Transport Safety
**Learning:** Standard small transport buttons are prone to miss-clicks in low-light/stress environments. Immediate speed changes (sliders) are hard to reset precisely.
**Action:** Redesigned Media Inspector Transport.
- **Visuals:** Grouped "Transport" (Play/Pause/Stop) separately from "Options" (Loop/Reverse). Increased button hit targets (40px height).
- **Safety:** Applied "Hold-to-Confirm" pattern to the Stop button (inline implementation) to prevent silence. Added "Reset Speed" button.
- **Feedback:** Added hover-timestamp tooltip to timeline for predictable seeking.
