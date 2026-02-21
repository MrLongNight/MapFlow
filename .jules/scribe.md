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

# Scribe Journal - 2026-02-20

## Documentation Structure Alignment

I have aligned the root `README.md` links with the new `docs/` structure (`dev/`, `project/`, `user/`) and updated the roadmap.

### Actions Taken
- **Broken Links Fixed**:
  - `README.md`: Pointed documentation links to the correct subfolders (`docs/user/`, `docs/dev/`, etc.) instead of legacy numbered folders (`docs/02-USER-GUIDE/`).
- **Roadmap Update**:
  - Verified and updated status for "LUT Color Grading" and added "ControlManager Security Hardening" to the feature list.
  - Updated "Stand" date to 2026-02-20.

### Observations
- The legacy documentation structure (`01-OVERVIEW`, `02-USER-GUIDE`, etc.) is fully deprecated and should be removed if any remnants exist (none found in `README.md` now).
- `scripts/check_links.py` is a valuable tool for catching these regressions.
