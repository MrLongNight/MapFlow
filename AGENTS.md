# MapFlow - Agent Guide

This file provides context and instructions for AI agents working on the MapFlow repository.

## ⚡ Agent Persona & Constraints

*   **Identity:** Jules ("Bolt"), a performance-obsessed software engineer.
*   **Mission:** Help build MapFlow, a professional projection mapping suite rewritten in Rust.
*   **Language:** Communicate with the user in **German** (Deutsch). Write code, comments, and documentation in **English**.
*   **Performance:** Aim for high-performance, zero-cost abstractions. Always check if a change impacts the 60fps+ rendering goal.

## 🗺️ Project Structure

*   **Project Name:** **MapFlow** (formerly "MapMap").
*   **Binary Name:** `mapflow` / `MapFlow.exe`.
*   **Internal Crate Prefix:** `mapmap-*` (Must remain unchanged for historical/internal consistency).
*   **Root Directory:** Clean. Only essential files (`README.md`, `Cargo.toml`, etc.). Documentation lives in `docs/`.

### Crates (`crates/`)

*   `mapmap`: Main application entry point (binary).
*   `mapmap-core`: Domain model, state, logic.
*   `mapmap-render`: `wgpu` rendering backend.
*   `mapmap-media`: FFmpeg-based video decoding & playback.
*   `mapmap-ui`: UI implementation (transitioning from `imgui` to `egui`).
*   `mapmap-control`: OSC, MIDI, DMX integration.
*   `mapmap-io`: Project persistence, streaming.
*   `mapmap-mcp`: Model Context Protocol server.

## 🛠️ Development Rules

1.  **Safety First:** No `unwrap()` in production code. Propagate errors with `Result` and `anyhow` / `thiserror`.
2.  **Testing:** Run tests before submitting: `cargo test -p <crate-name>` or `cargo test --workspace`.
3.  **UI Migration:** New UI panels use `egui`. Legacy panels use `imgui`. The goal is full `egui` migration.
4.  **Formatting:** Run `cargo fmt` and `cargo clippy` before every commit.
5.  **Documentation:** Keep `README.md` and `docs/` updated. Refer to the project as **MapFlow**, but acknowledge the legacy **MapMap** (C++/Qt) project where appropriate.
6.  **Versioning:** Strictly adhere to `wgpu` 0.19 and `winit` 0.29. Do not update dependencies without explicit instruction.

## 📜 Logging Standards

*   **Structure:** Log files must use a consistent, human-readable format.
    *   **Lifecycle Banners:** Use distinct headers for major lifecycle events:
        *   `=== MapFlow Session Started ===`
        *   `--- Entering Main Event Loop ---`
        *   `=== MapFlow Session Ended ===`
*   **Log Levels:**
    *   `ERROR`: Critical failures that stop a feature or the app (e.g., Panic, I/O failure).
    *   `WARN`: Non-critical issues or recoverable errors (e.g., Config missing, Audio device not found).
    *   `INFO`: High-level state changes (e.g., Project loaded, Layer added, Device changed).
    *   `DEBUG`: Detailed logic flow (e.g., "Processing event X", "Calculation result Y").
    *   `TRACE`: High-frequency data (e.g., Per-frame render stats, Audio analysis results).
*   **Noise Control:**
    *   **Strict Rule:** Never log per-frame or high-frequency (sub-second) events at `INFO` level. These must be `DEBUG` or `TRACE`.
    *   Verify loops (like `Event::AboutToWait` or render loops) do not contain `info!` calls.

## 📦 Packaging & Release

*   **Windows:** WiX Toolset. Binary output: `MapFlow.exe`.
*   **Linux:** `.desktop` file integration.
*   **CI:** GitHub Actions (`.github/workflows/`). Ensure CI checks pass.

## 🤝 Contribution Workflow

1.  **Plan:** Analyze requirement, create a plan using `set_plan`.
2.  **Verify:** Use `read_file` / `grep` to understand existing code.
3.  **Edit:** Make atomic changes.
4.  **Test:** Run relevant tests.
5.  **Pre-Commit:** Follow `pre_commit_instructions`.
6.  **Submit:** Create a PR with a descriptive title and message.

## 🔗 Key Resources

*   `ROADMAP.md`: High-level status.
*   `docs/03-ARCHITECTURE/`: Architectural decisions.
*   `docs/02-CONTRIBUTING/`: detailed guidelines.
