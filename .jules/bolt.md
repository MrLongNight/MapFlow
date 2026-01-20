## 2024-05-23 - Allocations in Hot Paths
**Learning:** The codebase frequently uses `.collect::<Vec<_>>()` inside render and UI loops (`main.rs`) to satisfy the borrow checker or for convenience, causing unnecessary per-frame allocations and data cloning (e.g. Strings).
**Action:** Replace `collect()` with direct iteration where ownership isn't strictly required, utilizing Rust's disjoint field borrowing capabilities to mutate UI state while iterating Core state.

## 2026-01-04 - Texture Registration Overhead
**Learning:** In `egui-wgpu` (and generally wgpu), registering a texture via `register_native_texture` is an expensive operation that creates a new BindGroup. Doing this every frame for every dynamic source (even if the underlying view pointer hasn't changed) is a significant performance anti-pattern.
**Action:** Always cache `egui::TextureId`s associated with `wgpu::TextureView`s. Use `Arc::ptr_eq` to cheaply verify if the view is identical to the cached one before re-registering.

## 2026-01-04 - Hot Path Allocation Removal (mem::take)
**Learning:** Deep cloning large state vectors (like `RenderOps`) just to satisfy borrow checker rules for a method call is a major performance waste. `std::mem::take` allows temporarily moving the data out (leaving a default/empty instance), using it, and then restoring it, avoiding allocation completely.
**Action:** Before cloning a struct field to pass it to a method on `self`, check if the field can be temporarily `take`n and restored.

## 2026-01-26 - O(N) Shifting in Rolling Windows
**Learning:** The FPS calculation logic used `Vec::remove(0)` to maintain a rolling window of 60 samples. `Vec::remove(0)` shifts all remaining elements, making it O(N). While negligible for N=60, it represents "unnecessary work" in a hot path.
**Action:** Replace `Vec` with `VecDeque` for rolling windows. `pop_front()` is O(1), aligning with the "Speed is a feature" philosophy.

## 2026-05-21 - Iterating VecDeque Windows
**Learning:** `VecDeque` does not support slice methods like `.windows()` directly because its memory is not guaranteed to be contiguous. Calling `make_contiguous` moves memory, which defeats the purpose of O(1) operations.
**Action:** For simple sliding windows on `VecDeque` (like calculating deltas), use `iter().zip(iter().skip(1))` instead of converting to a slice or `Vec`.

## 2026-05-26 - Transient Index Caching
**Learning:** For graph traversal in a hot path (`ModuleEvaluator::evaluate`), re-allocating `HashMap`s and `Vec`s every frame to build acceleration indices is a performance bottleneck.
**Action:** Use persistent fields (`part_id_to_idx`, `conn_incoming_indices`) in the evaluator struct and clear/repopulate them using `clear()` (preserving capacity) instead of dropping them. This converts per-frame allocation overhead to amortized zero allocation.
