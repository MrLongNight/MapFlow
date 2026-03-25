## 2024-05-18 – Standardize info labels and empty states
**Learning:** Hardcoded, inline `egui::RichText::new(...).weak().italics()` calls lead to spaghetti dependencies when panels try to reuse an inspector's internal style, and create inconsistencies if not properly centralized.
**Action:** Always use the globally accessible `crate::widgets::custom::render_info_label` and `crate::widgets::custom::render_missing_preview_banner` for empty states across panels to maintain a clean UI architecture and consistent styling.

## 2024-05-24 – Enhance visual feedback and accessibility for hold-to-action buttons
**Learning:** During stress or live-performance scenarios, missing visual confirmation when a "hold-to-confirm" action triggers makes the interface unpredictable. Also, using default generic text or `Debug` format enum strings inside `.widget_info` negatively affects screen reader usability.
**Action:** Ensure hold-to-confirm actions render a distinct 1-frame visual "flash" (like a full-width background fill or thicker border) when triggered (`progress >= 1.0`). Always map `hover_text` to the accessibility label via `WidgetInfo::labeled` instead of raw icon text to provide meaningful context for assistive technologies.

## 2026-03-25 - Prevent discoverability regressions in generic accessiblity fallbacks
**Learning:** In custom widgets, eliminating generic accessibility fallbacks (e.g., for `hold_to_action_button` or `hold_to_action_icon`) can cause severe discoverability regressions if the caller provides an empty `hover_text` string, leading to blank or unhelpful tooltips and ARIA labels.
**Action:** Dynamically incorporate the available action `text` or instructions into a fallback string (e.g., `format!("{} (Hold to confirm)", hover_text)` or `format!("Hold to confirm {}...", text)`) instead of leaving it blank or using generic "Hold Action Icon" text, ensuring actions remain discoverable and meaningful for screen readers.
