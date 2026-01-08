## 2024-05-23 - Allocations in Hot Paths
**Learning:** The codebase frequently uses `.collect::<Vec<_>>()` inside render and UI loops (`main.rs`) to satisfy the borrow checker or for convenience, causing unnecessary per-frame allocations and data cloning (e.g. Strings).
**Action:** Replace `collect()` with direct iteration where ownership isn't strictly required, utilizing Rust's disjoint field borrowing capabilities to mutate UI state while iterating Core state.
## 2026-01-04 - Hot Path Allocation Removal
**Learning:** Rust's borrow checker is smart enough to allow simultaneous disjoint borrows (Partial Borrowing) even in complex loops. This enables removing intermediate `Vec` allocations (via `.collect()`) that were previously thought necessary to satisfy the borrow checker when modifying one field while iterating another.
**Action:** Always verify if `.collect::<Vec<_>>()` is truly needed for borrow checking or if it can be replaced by direct iteration.
