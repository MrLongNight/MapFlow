## 2026-03-10 - Safely gating destructive UI actions
**Learning:** Destructive actions like Clear or Remove in permanent views must use the hold-to-confirm pattern to prevent live-performance accidents. Transient menus can keep normal buttons.
**Action:** Replaced standard button clicks for Clear and Remove actions in timeline, OSC, and Paint panels with hold-to-confirm variants using WARN_COLOR.
