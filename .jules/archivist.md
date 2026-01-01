# Archivist's Journal

## 2025-01-20: Initial Cleanup

### Actions Taken
- **Cleaned Root Directory:** Removed non-standard files to adhere to `MAPFLOW PROJEKTSTRUKTUR`.
- **Moved `knowledge.md`:** to `.jules/knowledge.md` (Jules-specific documentation).
- **Moved `SECURITY.md`:** to `.github/SECURITY.md` (Standard GitHub location).
- **Archived `VERSION.txt`:** to `.temp-archive/` (Redundant with Cargo.toml, invalid file type for root).
- **Archived `.mapmap_autosave`:** to `.temp-archive/` (Runtime artifact).

### Observations
- `AGENTS.md` remains in root as a critical exception for agent context, despite not being explicitly in the allowed list of the prompt's `MAPFLOW PROJEKTSTRUKTUR`.
- `.mapmap_autosave` suggests runtime files are leaking into the repo root. `.gitignore` has `.*.autosave` but it may not be catching this specific filename format. Monitor for recurrence.
