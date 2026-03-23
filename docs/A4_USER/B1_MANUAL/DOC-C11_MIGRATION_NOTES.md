# MapFlow V2 Migration Notes

## Renamed Elements

*   **Repository URL:** The repository is now located at `https://github.com/MrLongNight/MapFlow`.
*   **Binaries:** The main executable is now `MapFlow` (or `MapFlow.exe` on Windows).
*   **OSC Namespaces:** All OSC commands are now prefixed with `/mapflow` instead of `/mapmap`. For example, `/mapmap/layer/1/opacity` is now `/mapflow/layer/1/opacity`.
*   **File Extensions:** Project files and autosaves use the `.mflow` extension instead of the previous format.
*   **Environment Variables:** Environment variables starting with `MAPMAP_` should be updated to `MAPFLOW_`. For example, `MAPMAP_SELF_HOSTED_RUN_VISUAL_AUTOMATION` is now `MAPFLOW_SELF_HOSTED_RUN_VISUAL_AUTOMATION`.

## Deprecated Names

All references to "MapMap" (the legacy application name) have been replaced with "MapFlow" in documentation, scripts, and code. Please update any personal scripts or external integrations that relied on the old naming convention.
