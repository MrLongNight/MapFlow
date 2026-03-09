# Philips Hue Integration Plan

## Status
*   **Current Phase**: MVP (Phase 1 & Partial Phase 2)
*   **Implemented**:
    *   `hue` module in `mapmap-control` (Discovery, DTLS Client, Effect Engine structures).
    *   `OutputType::Hue` in `mapmap-core`.
    *   UI in `ModuleCanvas` (Discovery, Manual IP, Spatial Editor).

## Roadmap

### Phase 1: Foundation (Current PR)
*   [x] Port `HueFlow` core logic to `mapmap-control`.
*   [x] Add `OutputType::Hue` to `mapmap-core` data model.
*   [x] Implement basic UI for adding Hue nodes and editing properties.
*   [x] Implement Bridge Discovery (mDNS/N-UPnP) via `tokio` async tasks.
*   [x] Implement 2D Spatial Editor for lamp positioning.

### Phase 2: Connection & Streaming (Next Steps)
*   [ ] **Pairing Flow**: Implement the async pairing logic (pushlink button) in the UI.
*   [ ] **Configuration Persistence**: Save/Load paired bridge credentials safely.
*   [ ] **Runtime Integration**: Hook `EntertainmentEngine` into the main application render loop.
    *   The engine needs to receive `AudioSpectrum` data from the audio analyzer.
    *   It needs to send DTLS packets to the bridge.
*   [ ] **Area Fetching**: Fetch "Entertainment Areas" from the bridge API and populate the dropdown.

### Phase 3: Spatial Mapping & Effects
*   [ ] **Spatial Mapping**: Map the 2D "Virtual Room" positions to the video texture.
    *   Sample colors from the video frame at the normalized (X,Y) positions of the lamps.
    *   Send these colors to the `EntertainmentEngine`.
*   [ ] **Effect Implementation**: Connect the `LightEffect` traits to the Node parameters.
    *   Allow users to select effects (Pulse, Strobe, Flow) via the Modulizer or Node properties.

### Phase 4: Automation & Refinement
*   [ ] **Timeline Integration**: Allow automating Hue parameters (Brightness, Mode) via the Timeline.
*   [ ] **Global Settings**: Move Bridge Setup to Global Settings (if desired) or keep per-module.
*   [ ] **Performance Optimization**: Ensure DTLS streaming doesn't block the main UI thread.

## Architecture
*   **Control**: `crates/mapmap-control/src/hue/` handles all protocol logic.
*   **State**: `HueConfig` in `mapmap-core` stores the topology.
*   **UI**: `mapmap-ui` renders the setup and spatial editor.
