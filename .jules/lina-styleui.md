# Lina StyleUI Journal

## 2024-05-22 – [Visual Gap Analysis]
**Learning:** MapFlow's current UI is "flat dark" but lacks the "Cyber Dark" structure found in industry standards (Resolume, MadMapper).
- **Problem:** Visual Hierarchy is weak. Panels blend together. Lists are dense and unstyled. Active states are low-contrast.
- **Reference Standard:** Resolume/MadMapper use:
    - **Strong Borders:** Panels are clearly contained.
    - **Neon Accents:** Active states (play, selected) are high-contrast Cyan or Orange.
    - **Headers:** Content vs. Controls is strictly separated.
**Action:** Implement "Cyber Dark" theme:
1.  **Container Strategy:** Use `egui::Frame` with visible strokes/rounding for panels.
2.  **Accent Strategy:** Define a "Cyber Cyan" or "Neon Orange" for `Visuals.selection`.
3.  **Typography:** Ensure headers are distinct (e.g., Bold/Different Color) from data.

## 2024-05-22 – [Theme Definition]
**Learning:** `egui` default dark theme is functional but too "gray".
**Action:** Will look for `ctx.set_visuals` to inject:
- Background: Darker (almost black).
- Panel Background: Dark Gray.
- Stroke: Lighter Gray for definition.
- Accent: High saturation.
