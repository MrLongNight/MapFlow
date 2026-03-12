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

## 2026-11-20 - Dynamic Mesh Buffer Reuse
**Learning:** For dynamic meshes updated every frame (like particles), creating new `Vec`s and calling `mesh.insert_attribute` drops the old buffer, forcing a new allocation every frame. This causes significant allocator churn.
**Action:** Use `mesh.remove_attribute(...)` to take ownership of the existing buffer, clear it (keeping capacity), refill it, and re-insert it. This results in zero steady-state allocations for dynamic mesh updates.

## 2026-10-25 - Static Helpers for Disjoint Borrows
**Learning:** `ModuleEvaluator` methods often need to mutate cached results while reading configuration state. Passing `&self` to helper methods prevents this due to borrowing rules, leading to unnecessary cloning (e.g. `triggers_to_eval`).
**Action:** Convert helper methods to associated static functions (`fn helper(arg1, arg2)`) that take only the necessary fields, allowing the caller to split `self` borrows and avoid cloning.

## 2026-03-05 - Redundant Map Re-computation in Hot Paths
**Learning:** In `ModuleEvaluator::trace_chain`, a `HashMap` mapping part IDs to indices was being rebuilt from scratch for every layer rendered, despite the same map being built and cached in the parent `evaluate` method. This O(N) operation per layer caused significant overhead in complex scenes.
**Action:** Always check if a required lookup structure (like an ID-to-index map) is already available in the parent context before building a local one. If so, reuse it to avoid redundant allocations and iterations.
## 2026-03-02 - Prevent Vec allocation in UI Hot Path
**Learning:** In UI loops run every frame, allocating an intermediate `Vec` using `.collect()` just to pass a filtered slice to rendering functions creates continuous heap churn. In , filtering items created a new  60 times a second.
**Action:** Use dynamic trait objects (`&mut dyn Iterator`) to represent variable iterator types (like `FilterMap` vs `Values`), avoiding  and avoiding `collect()` entirely when passing data to UI render functions.

## 2026-06-12 - Prevent Vec allocation in UI Hot Path
**Learning:** In UI loops run every frame, allocating an intermediate `Vec` using `.collect()` just to pass a filtered slice to rendering functions creates continuous heap churn. In `crates/mapmap/src/media_manager_ui.rs`, filtering items created a new `Vec` 60 times a second.
**Action:** Use dynamic trait objects (`&mut dyn Iterator`) to represent variable iterator types (like `FilterMap` vs `Values`), avoiding `Box` and avoiding `collect()` entirely when passing data to UI render functions.

## 2024-03-07 - Avoid O(N) copies of CPU FrameData
**Learning:** `FrameData::Cpu(Vec<u8>)` resulted in deep copies of potentially large image/video data every time a frame was cloned (e.g. when passed to rendering pipeline or orchestration). This caused significant performance overhead and unnecessary memory allocation.
**Action:** Use `Arc<Vec<u8>>` instead for CPU frame data so that clones only increment the reference count, making it an O(1) operation.

## 2026-03-08 - O(N) FrameData Allocations in Video Pipelines
**Learning:** `Vec<u8>` for `FrameData::Cpu` in `mapmap-io` resulted in large O(N) memory allocations (deep copying raw video frames) during video playback or transfer.
**Action:** `FrameData::Cpu` now wraps pixel buffers in an `Arc<Vec<u8>>`. Reusing the `Arc` pointer entirely bypasses the O(N) duplication, enabling zero-copy frame dispatch across multi-threaded renderer contexts.

## 2024-03-10 - Integer Math for YUV to RGB Conversion
**Learning:** Floating-point operations in hot loops, like `yuv_to_rgb` processing every pixel in video frames, can be a major performance bottleneck.
**Action:** When performing color space conversions or other pixel-level math, always prefer integer math approximations (e.g., multiplying by a constant and dividing by a power of two) over float operations to significantly reduce processing time.

## 2024-05-23 - Avoid String Cloning in TimelineModule Iterators
**Learning:** Structs collected into `Vec` inside UI hot loops (like `TimelineModule` in `mapmap/src/app/ui_layout.rs`) that own `String` fields cause massive per-frame allocation overhead.
**Action:** Change UI presentation structs to borrow strings (`&'a str`) instead of owning them, reducing `clone()` allocations in rendering loops to zero.
