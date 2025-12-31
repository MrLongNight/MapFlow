## 2024-05-23 - Allocations in Hot Paths
**Learning:** The codebase frequently uses `.collect::<Vec<_>>()` inside render and UI loops (`main.rs`) to satisfy the borrow checker or for convenience, causing unnecessary per-frame allocations and data cloning (e.g. Strings).
**Action:** Replace `collect()` with direct iteration where ownership isn't strictly required, utilizing Rust's disjoint field borrowing capabilities to mutate UI state while iterating Core state.
