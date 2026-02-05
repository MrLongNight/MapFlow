# Task: Implement GPU Settings & Performance Monitoring

## Objective

Provide the user with control over GPU selection (Multi-GPU support) and visibility into system performance metrics (CPU/GPU usage, VRAM).

## Implementation Steps

### 1. GPU Selection Backend (`crates/mapmap-render/src/backend.rs`)

- Expand `Backend::new` or create a factory to enumerate adapters.
- Use `wgpu::Instance::enumerate_adapters(wgpu::Backends::all())`.
- Filter for discrete GPUs vs integrated.
- Allow selecting a specific adapter by index or name.

### 2. Settings UI (`crates/mapmap-ui/src/settings_panel.rs`)

- Add a "Graphics" or "Performance" tab.
- **Adapter Dropdown**: List available adapters. Store selection in `UserConfig`.
- **Restart Requirement**: Note that changing GPU requires app restart.

### 3. Performance Metrics

- **CPU Usage**: Already partially implemented in `App::update` (placeholder?). Ensure `sysinfo` crate is used efficiently (update every 1s, not every frame).
- **GPU Usage**: `wgpu` does not strictly provide "Load %".
  - *Workaround*: Display VRAM usage if available (via `wgpu` allocation statistics extensions if enabled).
  - Or rely on `sysinfo` if it supports GPU components (limited support).
  - Allow user to set "Target FPS" and "VSync" mode (Auto/On/Off) in settings.

### 4. configuration Persistence (`crates/mapmap-ui/src/config.rs`)

- Add fields: `preferred_gpu: String`, `vsync_mode: String`.
- Load these on startup in `main.rs` before initializing `Backend`.

## Constraints

- Windows-primary.
- VSync toggling might require recreating the `Surface` configuration.
