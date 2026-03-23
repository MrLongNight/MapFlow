---
design_depth: quick
task_complexity: medium
topic: video-pipeline-wiring
date: 2026-03-21
---

# Design-Dokument: Video Pipeline Wiring (FramePipeline)

## 1. Problem Statement
The `FramePipeline` in `mapflow-media` is fully implemented for robust media decoding and uploading but is not yet wired into the main application loop. The current application relies on older legacy rendering paths or direct player updates that do not take full advantage of the `FramePipeline` background scheduling. This is identified as a critical root cause for video blackscreens.

## 2. Approach
- **Integration**: Update `MediaPlayerHandle` in `crates/mapflow/src/orchestration/media.rs` to instantiate and use a `FramePipeline`.
- **Upload Function**: Provide an `upload_fn` to the pipeline that uses the `TexturePool` to upload CPU frames directly to the GPU queue.
- **Cleanup**: Mark the old thread-based polling logic inside `create_player_handle` as obsolete and replace it entirely with the `FramePipeline` lifecycle.

## 3. Risk Assessment
- **Low Risk**: The `FramePipeline` is already heavily tested in isolation. Connecting it to the `TexturePool` is a straightforward data bridging task.
- **Performance Impact**: Positive. Moving decoding and uploading to dedicated pipeline threads will unblock the main render loop.
