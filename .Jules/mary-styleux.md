# Mary StyleUX Journal

## 2024-05-21 - Effect Node UX Refactor
**Learning:** Live performance requires distinct visual hierarchy for "Modifier" nodes (Effects), not just Sources. Default generic UI makes it hard to distinguish between "configuring a setting" and "recovering from a mistake".
**Action:** Applied the "Live Header Pattern" to Effect nodes.
**Pattern:**
1.  **Header:** Big Icon + Name.
2.  **Safety:** Top-right "Reset" button (always visible) to clear parameters.
3.  **Grouping:** Separated "Type Selection" from "Parameters".

## Established Patterns
*   **Live Performance Header:** Large timecode/title + Transport controls.
*   **Safe Reset:** Distinct button to restore defaults.
*   **Async UI Action:** Use `UIAction` enum to bridge `egui` and `tokio`.
