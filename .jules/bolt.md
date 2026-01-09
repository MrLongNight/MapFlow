## 2024-05-23 - Allocations in Hot Paths
**Learning:** The codebase frequently uses `.collect::<Vec<_>>()` inside render and UI loops (`main.rs`) to satisfy the borrow checker or for convenience, causing unnecessary per-frame allocations and data cloning (e.g. Strings).
**Action:** Replace `collect()` with direct iteration where ownership isn't strictly required, utilizing Rust's disjoint field borrowing capabilities to mutate UI state while iterating Core state.
## 2026-01-04 - Hot Path Allocation Removal
**Learning:** Rust's borrow checker is smart enough to allow simultaneous disjoint borrows (Partial Borrowing) even in complex loops. This enables removing intermediate `Vec` allocations (via `.collect()`) that were previously thought necessary to satisfy the borrow checker when modifying one field while iterating another.
**Action:** Always verify if `.collect::<Vec<_>>()` is truly needed for borrow checking or if it can be replaced by direct iteration.

## 2026-01-04 - Texture Registration Overhead
**Learning:** In `egui-wgpu` (and generally wgpu), registering a texture via `register_native_texture` is an expensive operation that creates a new BindGroup. Doing this every frame for every dynamic source (even if the underlying view pointer hasn't changed) is a significant performance anti-pattern.
**Action:** Always cache `egui::TextureId`s associated with `wgpu::TextureView`s. Use `Arc::ptr_eq` to cheaply verify if the view is identical to the cached one before re-registering.
