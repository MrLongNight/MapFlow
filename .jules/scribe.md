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

# Scribe Journal - 2026-02-23

## Documentation Structure Transition

I have completed the migration of documentation references from the legacy numbered folder structure (`01-GETTING-STARTED`, `02-USER-GUIDE`, etc.) to the new semantic directory structure (`docs/user/`, `docs/dev/`, `docs/project/`).

### Actions Taken
- **Updated Main Documentation**:
  - `README.md`: Updated links to point to semantic directories.
  - `CONTRIBUTING.md`: Updated links to build and architecture docs.
  - `crates/mapmap/README.md`: Updated links to user guide and architecture.
- **Updated User Documentation**:
  - `docs/user/getting-started/README.md`: Updated links to manual and architecture.
  - `docs/user/getting-started/QUICK-START.md`: Fixed link to User Manual.
  - `docs/user/getting-started/BUILD.md`: Fixed link to Architecture docs.
  - `docs/user/tutorials/01-HELLO-WORLD.md`: Fixed link to Build instructions.
- **Updated Developer/Project Documentation**:
  - `docs/dev/setup/DEVELOPMENT-SETUP.md`: Fixed link to Installation guide.
  - `docs/project/roadmap/PROJECT-PHASES.md`: Fixed link to Multi-PC architecture doc.
  - `docs/project/cicd/WORKFLOW_CONTROL.md`: Fixed link to Installation guide.

### Observations
- The codebase now consistently uses the semantic documentation structure.
- Legacy numbered folders are no longer referenced in active documentation files.
