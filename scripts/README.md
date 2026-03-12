# Scripts

This directory now stores the real script implementations in thematic subfolders.

## Layout

- `archive/`: archived scripts and log artifacts kept for review or fallback
- `build/`: build environment, dependency, runtime DLL, and shader tooling
- `codex-cli/`: Codex CLI-specific entry scripts
- `codex-web/`: Codex Web container setup and maintenance hooks
- `dev-tools/`: local quality checks and repository maintenance helpers
- `docs/`: documentation and icon maintenance helpers
- `gemini-cli/`: Gemini/Jules monitor helpers
- `jules/`: Jules-specific setup and helper scripts
- `mapflow/`: packaging, local runtime, and MapFlow utility scripts

## Naming

- New script names use lowercase kebab-case.
- File extensions stay platform-appropriate: `.sh`, `.ps1`, `.bat`, `.py`.

## Compatibility

The `scripts/` root is now intentionally clean.
Archived former top-level entrypoints live under `scripts/archive/entrypoints/`.
Use the subfolder paths for active documentation and automation.

## Prepare Pre-Commit

- `scripts/jules/prepare-pre-commit.sh`
- `scripts/codex-web/prepare-pre-commit.sh`
- `scripts/codex-cli/prepare-pre-commit.ps1`
- `scripts/gemini-cli/prepare-pre-commit.ps1`

## Review Candidates

The following scripts have been moved into `scripts/archive/review/` because their continued need is unclear:

- `scripts/archive/review/build/make-tarball.sh`
- `scripts/archive/review/docs/update-contributors.sh`
- `scripts/archive/review/docs/update-doc-links.sh`
- `scripts/archive/review/docs/update-osc.sh`
- `scripts/archive/review/jules/send-to-jules.ps1`
