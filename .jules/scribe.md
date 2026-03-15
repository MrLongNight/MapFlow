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

## 2026-03-11 - Fixed broken legacy references across documentation
**Observation:** Several documents (e.g. `CONTRIBUTING.md`, `DOC-C1_OVERVIEW.md`, `DOC-C10_PROJECT_PHASES.md` and `DOC-C6_DOCUMENTATION_AUDIT.md`) contained outdated file paths referring to legacy structures like `docs/project/general/CODE-OF-CONDUCT.md` or `docs/dev/architecture/MULTI-PC-FEASIBILITY.md`.
**Action:** Re-linked to their new A-series documentation paths (`docs/A3_PROJECT/B5_GOVERNANCE/DOC-C1_CODE_OF_CONDUCT.md`, `../DOC-A1_MODULE_TREE.md`, etc) to reflect the 3-level folder architecture migration. Checked off tasks in `DOC-C6_DOCUMENTATION_AUDIT.md`.

## 2026-03-12 - User Manual Updates and Cleanup
**Observation:** `docs/A4_USER/B1_MANUAL/DOC-C0_README.md` was missing a link to `DOC-C7_MIDI_CONTROL.md`, and `DOC-C7_MIDI_CONTROL.md` had "TODO" placeholders in public documentation which should be avoided according to Scribe's rules. Also, there were Git merge conflict markers left in `docs/A3_PROJECT/B3_OPERATIONS/DOC-C2_TECHNICAL_DEBT_AND_BUGS.md`.
**Action:**
1. Added the link to `DOC-C7_MIDI_CONTROL.md` in `DOC-C0_README.md`.
2. Cleaned up `DOC-C7_MIDI_CONTROL.md` by replacing "TODO" with "Einschränkungen" and rephrasing appropriately.
3. Cleaned up Git merge conflict markers in `DOC-C2_TECHNICAL_DEBT_AND_BUGS.md`.

## 2026-03-13 - Broken Links in Crate READMEs
**Erkenntnis:** The `crates/mapmap/README.md` was still pointing to the old documentation structure (`docs/user/getting-started/`, `docs/user/manual/`, `docs/dev/architecture/`).
**Aktion:** Updated the markdown links in `crates/mapmap/README.md` to point to the new semantic paths `docs/A4_USER/B1_MANUAL/DOC-C2_QUICKSTART.md`, `docs/A4_USER/B1_MANUAL/DOC-C0_README.md`, and `docs/A1_SYSTEM/B1_ARCHITECTURE/DOC-C1_OVERVIEW.md`.

## 2026-03-15 - Broken Links to ROADMAP.md
**Erkenntnis:** The root `ROADMAP.md` was removed/relocated, but multiple documentation files (like `README.md` and files in `docs/`) and the `scripts/dev-tools/check-links.py` script were still referencing it, leading to broken links.
**Aktion:** Updated all references to point to the new location `docs/project/roadmap/README.md`.
