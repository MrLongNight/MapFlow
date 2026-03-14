## 2024-03-14 - Empty States Should use Subtle Buttons

**Learning:** Empty states in properties panels (like "No media loaded") were incorrectly using large primary buttons for file selection which draws too much attention compared to populated states.
**Action:** Use standard `ui.button("Select...")` with standard sizing inside empty states to maintain visual hierarchy and consistency with the populated state's folder icon buttons.
