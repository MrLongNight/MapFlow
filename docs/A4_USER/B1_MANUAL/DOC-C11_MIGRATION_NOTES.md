# Vorce V2 Migration Notes

## Renamed Elements

* **Repository URL:** The repository is now located at `https://github.com/Vorce-Studios/Vorce`.
* **Binaries:** The main executable is now `Vorce` (or `Vorce.exe` on Windows).
* **OSC Namespaces:** All OSC commands are now prefixed with `/vorce` instead of `/mapmap` or `/mapflow`. For example, `/mapflow/layer/1/opacity` is now `/vorce/layer/1/opacity`. (Legacy namespaces `/mapflow` and `/mapmap` are kept for compatibility but deprecated).
* **File Extensions:** Project files and autosaves use the `.vorce` extension instead of the previous formats. (`.mflow` and `.mapmap` are read-only compatibility aliases).
* **Environment Variables:** Environment variables starting with `MAPMAP_` or `MAPFLOW_` should be updated to `VORCE_`. For example, `MAPFLOW_SELF_HOSTED_RUN_VISUAL_AUTOMATION` is now `VORCE_SELF_HOSTED_RUN_VISUAL_AUTOMATION`.

## Deprecated Names

All references to "MapMap" and "MapFlow" (the legacy application names) have been replaced with "Vorce" in documentation, scripts, and code. Please update any personal scripts or external integrations that relied on the old naming convention.
