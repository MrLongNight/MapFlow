Master issue for the global rebranding of the project from current `MapFlow` and legacy `MapMap` surfaces to the finalized target identity `Vorce`. This supersedes the StageGraph-specific attempt in #1179.

## Status
Planning. Canonical target fixed to `Vorce`; execution remains blocked until #1229 is explicitly approved.

## Why this issue exists
The repository currently mixes several identity layers:
- current user-facing brand: `MapFlow`
- legacy/workspace slug: `mapmap`
- legacy UI strings: `MapMap`
- external compatibility surfaces: `/mapmap/`, `/mapflow/`, `.mapmap`, `.mflow`, `MAPFLOW_*`, `MAPFLOW_PROJECT_*`, `info.mapmapteam.*`

A safe rebranding must treat those surfaces separately instead of performing a blind search/replace.

## Canonical Rename Matrix
| Surface | Current | Target | Notes |
| --- | --- | --- | --- |
| Product / display name | `MapFlow` | `Vorce` | user-facing name |
| Historical legacy name | `MapMap` | migration-only | docs and compatibility notes only |
| Repo slug | `MapFlow` | `Vorce` | GitHub repository |
| Cargo crate prefix | `mapmap` | `vorce` | package/import prefix |
| Binary / executable | mixed `MapFlow` / `mapmap` | `Vorce` | platform-specific packaging |
| Bundle ID | `info.mapmapteam.MapFlow` | `org.vorce.app` | macOS and package metadata |
| Env var prefix | `MAPFLOW_` | `VORCE_` | runtime and CI/CD |
| Project sync vars | `MAPFLOW_PROJECT_*` | `VORCE_PROJECT_*` | issue/project automation |
| OSC namespace | `/mapmap/`, `/mapflow/` | `/vorce/` | external API |
| Project extension | `.mapmap`, `.mflow` | `.vorce` | default save format |
| MCP / external server IDs | mixed `MapFlow` / `mapmap` | `vorce-*` | explicit compatibility policy required |
| Release artifact prefix | `MapFlow-*` | `Vorce-*` | zips, tarballs, installers |
| Issue tracking block | `## MapFlow Project Manager` | `## Vorce Project Manager` | managed issue-body section |

Source of truth for this matrix: #1229 and `docs/A3_PROJECT/B1_PLANNING/DOC-C15_RENAME_MATRIX_AND_COMPAT_POLICY.md`.

## Subissues
- [ ] #1229 `__SI-01_MAI-001_CANONICAL_RENAME_MATRIX_AND_COMPAT_POLICY_V2`
- [ ] #1204 `__SI-02_MAI-001_CRATES_WORKSPACE_AND_PACKAGE_RENAME_V2`
- [ ] #1205 `__SI-03_MAI-001_SCRIPTS_ENV_AND_RUNTIME_PATHS_V2`
- [ ] #1206 `__SI-04_MAI-001_PROTOCOLS_API_AND_EXTERNAL_IDS_V2`
- [ ] #1207 `__SI-05_MAI-001_PROJECT_FORMAT_LOGGING_AND_COMPAT_V2`
- [ ] #1215 `__SI-06_MAI-001_UI_BRANDING_ASSETS_AND_WINDOW_TITLES_V2`
- [ ] #1230 `__SI-07_MAI-001_REPO_RELEASE_AND_PACKAGE_METADATA_V2`
- [ ] #1214 `__SI-08_MAI-001_WORKFLOWS_DOCS_AND_MIGRATION_NOTES_V2`

## Acceptance Criteria
- [ ] the canonical rename matrix is approved in #1229
- [ ] every breaking surface has an explicit policy: alias, deprecation window, or hard cut
- [ ] workspace/package names, scripts, UI, protocols, file formats, release artifacts, issue tracking, and docs are migrated consistently
- [ ] supported compatibility surfaces are tested or intentionally retired with migration guidance
- [ ] release workflows emit correctly branded artifacts on Windows, macOS, and Linux
- [ ] GitHub repo renaming is planned and executed with redirect, badge, and integration checks
- [ ] no supported user-facing or shipping surface still shows mixed `MapFlow` / `MapMap` branding except documented legacy compatibility notices
- [ ] rollout notes cover repo rename, badge/link updates, issue/project automation, and migration guidance for contributors and integrators
- [ ] no unresolved placeholder such as `[NEW_NAME]`, `[NEW_SLUG]`, or `[NEW_EXT]` remains in the active rebranding plan

## Non-Goals
- unrelated refactors or behavior changes outside branding and migration scope
- removing legacy compatibility aliases before #1229 decides their lifecycle
- repeating the abandoned StageGraph-specific rename strategy from #1179

## Notes
- This work is expected to touch code, packaging, CI/CD, release artifacts, GitHub repository metadata, issue automation, and documentation.
- The historical StageGraph attempt remains closed in #1179 and is not the source of truth for the final rename.
