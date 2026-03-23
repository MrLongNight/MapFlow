# DOC-C15: Canonical Rename Matrix and Compatibility Policy

This document defines the single source of truth for the MapMap to MapFlow renaming initiative (Issue #1229). No large-scale rename should begin before these decisions are approved.

## 1. Canonical Rename Matrix

The following matrix standardizes all shipping and integration surfaces to the new "MapFlow" branding.

| Surface | Decision / New Value |
| :--- | :--- |
| **Product and Display Name** | MapFlow |
| **Repository Slug** | `MapFlow` |
| **Cargo Crate/Package Prefix** | `mapflow-` |
| **Binary/Executable Name** | `MapFlow` |
| **Bundle/Package Identifiers** | `org.mapflow.app` |
| **Environment Variable Prefix** | `MAPFLOW_` |
| **OSC Namespace** | `/mapflow/` |
| **Default Project File Extension** | `.mflow` |
| **MCP / External Resource IDs** | `mapflow-mcp` |
| **Release Artifact Prefix** | `MapFlow-` |

## 2. Compatibility Policy

For each externally visible identifier, the following compatibility policy applies:

| Surface | Policy | Details |
| :--- | :--- | :--- |
| **`.mapmap` files** | Deprecate with sunset window | Keep as a read-only alias. The app will load `.mapmap` files but all saves will default to `.mflow`. Support will be removed in the next major version. |
| **`.mflow` files** | Keep | The new standard default project file extension. |
| **`/mapmap/` OSC** | Deprecate with sunset window | OSC endpoints will temporarily listen to both `/mapmap/` and `/mapflow/`. Deprecation warnings will be logged for `/mapmap/` usage. |
| **`MAPFLOW_*` Env Vars** | Keep | Retain as the standard environment variable prefix. Legacy `MAPMAP_*` variables will be ignored (Hard break). |
| **Old repo URLs & badges** | Hard break with migration steps | GitHub automatically redirects old URLs, but all hardcoded links and badges in documentation must be updated immediately. |
| **Old binary/app names** | Hard break with migration steps | Releasing new binaries under the legacy name stops immediately. Installers will prompt to uninstall old versions if detected. |
| **Legacy `MapMap` references** | Hard break with migration steps | Batch search-and-replace across docs, comments, UI labels, and history logs. No legacy references should remain in the UI. |

## 3. Release and Versioning Recommendation

**Cutover Version:** `v1.0.0` (or the next immediate major/minor release boundary).
- **Pre-cutover:** Prepare all renaming tools, verify tests pass with the new names, and finalize this document.
- **Cutover release:** Implement the rename across all code, crates, and assets in a single, coordinated massive PR (or short-lived feature branch).
- **Post-cutover:** Monitor bug reports related to file loading or integrations (OSC, plugins) and enforce the compatibility policy via deprecation warnings.

## 4. Migration Note Outline

### For Contributors
- All new crates must use the `mapflow-` prefix.
- Use `MAPFLOW_` for environment variables.
- Run the provided renaming scripts before opening new PRs.
- Avoid using "MapMap" in any new documentation or code comments.

### For Users
- Your existing `.mapmap` project files are safe. Opening them in the new version will automatically upgrade them, and the next save will create a `.mflow` file.
- The application binary is now `MapFlow`. Please update any custom shortcuts or startup scripts.

### For Integrators (OSC, APIs)
- OSC commands must migrate to the `/mapflow/` namespace. The legacy `/mapmap/` namespace is deprecated and will be removed in a future update.
- If you rely on environment variables for automation, ensure they start with `MAPFLOW_`.
