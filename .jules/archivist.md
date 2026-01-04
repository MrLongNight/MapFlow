# Archivist Journal

## Session: 2025-12-30 (Jules - Archivist)

### Initial Scan Findings
The root directory contained several files violating the project structure:
- `SECURITY.md` (Should be in `.github/`)
- `knowledge.md` (Should be in `.jules/`)
- Scripts (`jules-setup.sh`, `run_mapflow.bat`) in root.
- Temporary logs (`check_*.txt`, `core_error.txt`, `test_results.txt`).
- `VjMapper.code-workspace` (Should be in `.vscode/`).
- `VERSION.txt` (Redundant with Cargo.toml).

### Actions Taken
- Moved misplaced markdown files to appropriate directories.
- Moved scripts to `scripts/`.
- Moved workspace file to `.vscode/`.
- Deleted temporary log files.
- Archived `VERSION.txt` and `.mapmap_autosave` to `.temp-archive/`.

### Rules Enforced
- Root directory kept clean (only `Cargo.toml`, `README.md`, etc.).
- Temp files deleted or archived.
