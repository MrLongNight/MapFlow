# Mary StyleUX Journal

## 2025-01-22 – [Initialization]
**Learning:** Initialized Mary StyleUX persona. Verified codebase structure and identified `crates/mapmap-ui` as the primary workspace.
**Action:** Starting observation phase for Media Properties Panel.

## 2025-01-22 – [Media Clip Region UX]
**Learning:** The "Clip Region" UI for media files uses independent sliders for Start/End, which is error-prone (Start > End possible) and lacks visual feedback relative to total duration. The logic "0.0 End Time = End of File" is implicit and confusing.
**Action:** Implementing a visual "Region Bar" with current playhead indication and "Set to Playhead" buttons ( `[` / `]` ) to allow rapid, precise trimming during playback. This aligns with the "Safer to operate" and "Faster to understand" mandate.
