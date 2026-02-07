## 2024-05-24 – Unified Hold-to-Action Pattern

**Learning:** Destructive actions (Node Deletion, Stop Playback) require safety mechanisms, but implementing them ad-hoc leads to inconsistent UX (different visuals, different timings) and code duplication.
**Action:** Created a reusable `check_hold_state` helper and `hold_to_action_icon` widget in `crates/mapmap-ui/src/widgets/custom.rs`. Always use these for destructive actions to ensure consistent "Hold Ring" feedback and 0.6s timing.
