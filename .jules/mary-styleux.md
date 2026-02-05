# Mary StyleUX Journal

## 2024-05-24 – Safe Destructive Actions

**Learning:** Immediate "click-to-delete" buttons on nodes are dangerous in live performance contexts. Users may accidentally delete a critical node while trying to select or move it.
**Action:** Implemented a "Hold-to-Confirm" pattern (0.6s hold) for node deletion.

- **Visuals:** Added a circular progress indicator filling the delete button.
- **Interaction:** Requires holding mouse button OR focusing and holding Space/Enter.
- **Accessibility:** Ensure custom interactive rects support focus and keyboard events if they replace standard buttons. Replaced duplicated layout logic with a helper method to ensure hit-testing and rendering stay in sync.

## 2024-05-24 – Asset Management Safety

**Learning:** Users managing complex projects often need to verify asset files without modifying the project state. The "Select File" dialog is insufficient for this (as it implies changing the file).
**Action:** Implemented "Reveal in File Explorer" pattern.

- **Visuals:** Added a consistent "↗" button next to file path inputs.
- **Interaction:** Opens the containing folder and highlights the file (Windows) or opens parent directory (Linux/macOS).
- **Safety:** Prevents accidental file changes by providing a read-only verification method. Validated cross-platform command spawning to ensure UI responsiveness.

## 2024-05-24 – CI Configuration & Empty Modules

**Learning:** `cargo-deny`'s `unlicensed` key is deprecated and causes CI failures. Empty Rust module files (`.rs` with 0 bytes or just newlines) cause `cargo fmt` diff failures in CI pipelines.
**Action:** Removed deprecated `unlicensed = "allow"` from `deny.toml`. Ensured all placeholder modules contain at least a comment (e.g., `// This module is currently empty.`) to satisfy `rustfmt`. Verified `cargo fmt --all -- --check` locally before submission.
