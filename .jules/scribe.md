# Scribe Journal - 2026-02-15

## Documentation Fixes & Updates

I have addressed several broken links and outdated status indicators in the documentation.

### Actions Taken
- **Broken Links Fixed**:
  - `README.md`: Pointed "Installation" link to `docs/01-GETTING-STARTED/INSTALLATION.md` (was `docs/08-TECHNICAL/SETUP_GUIDE.md`).
  - `docs/10-CICD_PROZESS/WORKFLOW_CONTROL.md`: Pointed "Support" link to `docs/01-GETTING-STARTED/INSTALLATION.md` (was `.github/SETUP_GUIDE.md`).
- **Roadmap Update**:
  - Updated "Render Pipeline & Module Logic" status in `ROADMAP.md` to reflect that the main application entry point is implemented and stabilization is in progress.
- **Changelog Update**:
  - Added missing entry for 2026-02-06 ("Safe Reset Clip" feature).

### Observations
- The `docs/` structure is comprehensive but has some legacy references to files that have been moved or renamed.
- `ROADMAP.md` status fields need regular manual verification against the Changelog.

## 2026-03-05 - Folder Structure Migration Fixes
**Observation:** Some files still had legacy `0[1-9]-...` references to the documentation directory (e.g. `docs/01-GETTING-STARTED/INSTALLATION.md`) instead of the new semantic folder structure (like `docs/user/getting-started/INSTALLATION.md`).
**Action:** Fixed `crates/mapmap/README.md`, `docs/user/getting-started/BUILD.md`, `docs/project/cicd/WORKFLOW_CONTROL.md`, `docs/project/cicd/WORKFLOW_QUICKREF.md`, and `docs/project/audits/DOCUMENTATION_AUDIT.md` to point to the correct semantic documentation paths.

## 2026-03-10 - Dokumentations-Architektur Migration
**Erkenntnis:** Die Dokumentationsstruktur in `docs/` wurde von der bisherigen `user/`, `dev/`, `project/` Struktur auf eine 3-Ebenen-Struktur (`A1_SYSTEM`, `A2_DEVELOPMENT`, `A3_PROJECT`, `A4_USER`) umgestellt.
**Aktion:** Haupt-README.md und docs/README.md angepasst, um diese neue Struktur als Einstiegspunkte korrekt zu reflektieren. Alte `New_README.md` gelöscht.
