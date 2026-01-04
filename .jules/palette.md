# Palette's Journal - Critical UX/A11y Learnings

*No entries yet.*

## 2025-10-24 - [Visibility Toggle Tooltips]
**Learning:** Icon-only buttons (like the eye icon for visibility) in compact panels need explicit tooltips, especially when state is conveyed only by icon change.
**Action:** Always verify "icon-only" widgets have a corresponding  call with a localized string.

## 2025-10-24 - [Visibility Toggle Tooltips]
**Learning:** Icon-only buttons (like the eye icon for visibility) in compact panels need explicit tooltips, especially when state is conveyed only by icon change.
**Action:** Always verify "icon-only" widgets have a corresponding `.on_hover_text()` call with a localized string.
## 2025-10-24 - [Empty States]
**Learning:** Large interactive areas like canvases can feel broken when empty. Adding a simple centered instruction text dramatically improves approachability for new users.
**Action:** Identify 'container' widgets and always add an is_empty check with a helpful call-to-action.
