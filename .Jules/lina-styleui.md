## 2025-02-14 - Empty states should use visually muted and italicized styling
**Learning:** In egui, plain `ui.label("No data")` looks like an active value. For empty states, a visual distinction is needed to reduce clutter and guide the eye away from "missing" elements.
**Action:** Use `ui.label(egui::RichText::new("...").weak().italics())` for messages like "No media loaded", "No configurable parameters", etc.
