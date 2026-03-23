# Module Sockets and Connectors

This document specifies the standards for node sockets in MapFlow to prevent drift and ensure schema consistency.

## Socket Types (ModuleSocketType)

The following signal variants exist in the current system:

*   **Trigger**: Represents an event-based signal (automation, beat, etc.).
*   **Media**: Represents a flow of visual data (Textures, Meshes, Mask geometries).
*   **Layer**: Represents a full compositing layer stream including its metadata.
*   **Effect**: Represents an effect application path.
*   **Output**: Represents a final output sink.
*   **Link**: Used for master/slave linking configuration.

## Standard Socket Builders

To prevent drift and ensure nodes always present standard and reliable socket IDs, you must use the following standard builders located in `ModuleSocket` (`crates/mapflow-core/src/module/types/socket.rs`) instead of constructing sockets manually via `ModuleSocket::input()`:

*   `standard_media_in()` -> ID: `"media_in"`
*   `standard_media_out()` -> ID: `"media_out"`
*   `standard_layer_in()` -> ID: `"layer_in"`
*   `standard_layer_out()` -> ID: `"layer_out"`
*   `standard_trigger_in()` -> ID: `"trigger_in"`
*   `standard_trigger_out()` -> ID: `"trigger_out"`

Using these ensures that ID nomenclature is consistent across the entire node ecosystem.

## Intentional Exceptions

While standard socket builders should be used wherever possible, there are specific intentional exceptions for legacy compatibility or clarity:

*   **BevyParticles**: Uses a custom `"spawn_trigger"` ID (Label: "Spawn Trigger") instead of the standard `"trigger_in"`. This explicitly indicates that the trigger causes a discrete spawn action rather than modulating continuous parameters.
*   **Mesh**: Uses legacy `"vertex_in"` and `"geometry_out"` IDs instead of standard media sockets, and `"control_in"` instead of `"trigger_in"`, to preserve compatibility with existing project graphs that map geometry signals through these specific IDs.
*   **Link / AudioFFT / Hue**: Specialized nodes with dynamic or hardware-specific outputs will continue to use domain-specific socket names.
