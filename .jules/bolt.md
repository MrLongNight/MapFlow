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

## 2026-06-15 - Queue Submission Batching
**Learning:** Submitting command buffers to the `wgpu` queue inside a loop (e.g. for generating N previews) causes significant driver overhead due to repeated synchronization and validation.
**Action:** Batch multiple render passes into a single `CommandEncoder` and submit once at the end of the loop. Use `begin_frame` (if available) to reset resource caches before the batch starts to ensure optimal buffer reuse.

## 2026-06-21 - Immediate Mode Geometry Checks
**Learning:** In immediate mode UIs (`egui`), complex geometry checks (like iterative Bezier hit testing) running per-frame for every object (N connections) creates a linear CPU bottleneck. A simple AABB broad-phase check dramatically reduces the work for the vast majority of non-interacted objects.
**Action:** Always implement a cheap broad-phase check (AABB, bounding circle) before performing expensive detailed hit testing in render loops.

## 2026-10-24 - Deep Cloning in UI Loops
**Learning:** `module_sidebar.rs` was performing `modules.into_iter().cloned().collect()` inside the `show()` method (called every frame). This deep-cloned the entire module graph (nodes, connections, strings) 60 times a second, creating massive unnecessary allocation traffic.
**Action:** When iterating collections for UI display, always prefer iterating references (`&T`) directly. If a closure requires ownership of a field (like `id: u64`), capture just that field, not the whole struct.

## 2026-05-28 - HashMap Allocations in Loops
**Learning:** Updating a `HashMap` inside a loop using `insert` with a cloned key (`String`) every frame causes massive unnecessary allocations if the key already exists.
**Action:** Always check `get_mut` first to update in place without allocating a new key. Only `insert` (and clone the key) if the entry is missing.

## 2026-05-31 - Hoist RNG Initialization
**Learning:** `rand::rng()` initialization inside a hot loop (e.g., particle spawning) causes redundant thread-local storage lookups for every iteration, adding overhead in performance-critical systems.
**Action:** Hoist `rand::rng()` initialization outside the loop to retrieve the thread-local handle once per frame/system update.
