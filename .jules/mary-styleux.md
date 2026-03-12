## 2026-03-10 - Safely gating destructive UI actions
**Learning:** Destructive actions like Clear or Remove in permanent views must use the hold-to-confirm pattern to prevent live-performance accidents. Transient menus can keep normal buttons.
**Action:** Replaced standard button clicks for Clear and Remove actions in timeline, OSC, and Paint panels with hold-to-confirm variants using WARN_COLOR.

## 2024-05-20 - Standardize empty state styling
**Learning:** Empty or 'no data' states (e.g., 'No media loaded', 'No mask loaded', 'No mappings created yet') must always be styled using `egui::RichText::new("...").weak().italics()` to visually differentiate them from regular data points.
**Action:** Replaced missing `.italics()` for several `RichText` usages across MapFlow.
