# MapFlow UI (`mapmap-ui`)

The user interface for MapFlow, built with `egui`. This crate handles all visual components, panels, and user interaction.

## Features

- **Module Canvas**: Node-based visual programming interface for routing signals and media.
- **Panels**: Specialized panels for layers, mapping, media, and output configuration.
- **Theme**: "Cyber Dark" theme optimized for low-light VJ environments.
- **Localization**: Internationalization support via Fluent.

## Architecture

The UI is driven by a `AppUI` state struct and renders to a `wgpu` backend via `egui-wgpu`.

```rust
use mapmap_ui::AppUI;

// In your main loop
ui_state.render(ctx, &state);
```
