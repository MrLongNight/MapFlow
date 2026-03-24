# DOC-C15: Canonical Rename Matrix and Compatibility Policy

This document defines the single source of truth for the canonical renaming to "Vorce" and the compatibility policy for legacy surfaces.

## 1. Canonical Rename Matrix

The following matrix standardizes all shipping and integration surfaces to the final "Vorce" branding.

| Surface | Canonical Target Value |
| :--- | :--- |
| **Product and Display Name** | Vorce |
| **Repository Slug** | `Vorce` |
| **Cargo Crate/Package Prefix** | `vorce` |
| **Binary/Executable Name** | `Vorce` |
| **Bundle/Package Identifiers** | `org.vorce.app` |
| **Environment Variable Prefix** | `VORCE_` |
| **Project Sync Prefix** | `VORCE_PROJECT_*` |
| **OSC Namespace** | `/vorce/` |
| **Default Project File Extension** | `.vorce` |
| **MCP / External Resource IDs** | `vorce-*` |
| **Release Artifact Prefix** | `Vorce-*` |
| **Managed Issue Block Heading / Marker Prefix** | `## Vorce Project Manager` / `vorce-*` |

## 2. Compatibility Policy

For each externally visible identifier, the following strict compatibility policy applies:

| Surface | Policy | Details |
| :--- | :--- | :--- |
| **`.vorce` files** | Canonical | The new canonical standard default project file extension. All saves default to `.vorce`. |
| **`.mapmap` and `.mflow` files** | Legacy migration surface | Kept as a read-only alias. The app will load `.mapmap` and `.mflow` files to allow migration, but all output/saves will be `.vorce`. |
| **`/vorce/` OSC** | Canonical | The new standard canonical OSC namespace. |
| **`/mapmap/` and `/mapflow/` OSC** | Explicit compatibility alias | OSC endpoints may temporarily listen to both `/mapmap/` and `/mapflow/` strictly as explicit compatibility aliases to support existing integrations, but will issue deprecation warnings. |
| **`VORCE_` and `VORCE_PROJECT_*` Env Vars** | Canonical | These are the canonical environment prefixes. |
| **Old repo URLs & badges** | Hard break with migration steps | GitHub automatically redirects old URLs, but all hardcoded links and badges in documentation must be updated immediately to `Vorce`. |
| **Old binary/app names** | Hard break with migration steps | Releasing new binaries under legacy names (`MapMap` or `MapFlow`) stops immediately. Installers will prompt to uninstall old versions if detected. |
| **Legacy `MapMap` and `MapFlow` references** | Restricted | `MapMap` and `MapFlow` references may remain **only** inside migration notes or documented compatibility exceptions. |

## 3. Test and PR-Check Handling

- **Atomic Updates:** Every downstream rename PR must update all affected tests, fixtures, examples, and CI references within the **same PR**.
- **No Broken State:** The rollout of the rename must **not** rely on temporarily broken PR checks. All CI and tests must pass at every stage.

## 4. Migration Note Outline

### For Contributors
- All new crates must use the `vorce` prefix.
- Use `VORCE_` and `VORCE_PROJECT_*` for environment variables.
- Run the provided renaming scripts before opening new PRs.
- Avoid using "MapMap" or "MapFlow" in any new documentation or code comments outside of explicit compatibility contexts.

### For Users
- Your existing `.mapmap` and `.mflow` project files are safe. Opening them in the new version will automatically upgrade them, and the next save will create a `.vorce` file.
- The application binary is now `Vorce`. Please update any custom shortcuts or startup scripts.

### For Integrators (OSC, APIs)
- OSC commands must migrate to the `/vorce/` namespace. The legacy `/mapmap/` and `/mapflow/` namespaces are strictly kept as explicit compatibility aliases and will issue deprecation warnings.
- If you rely on environment variables for automation, ensure they start with `VORCE_`.
