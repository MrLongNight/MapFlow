---
description: Pre-commit checks - run before every git commit to ensure code quality
---

# Pre-Commit Check Workflow

## Option 1: Using pre-commit Framework (Recommended)

// turbo
1. Install pre-commit (one-time setup):
   ```powershell
   pip install pre-commit
   pre-commit install
   ```

2. Pre-commit now runs automatically on every `git commit`
   - Trailing whitespace is fixed
   - Code is formatted with `cargo fmt`
   - Cargo.toml files are sorted

3. If hooks make changes, re-stage and commit again:
   ```powershell
   git add -A
   git commit -m "your message"
   ```

## Option 2: Using PowerShell Script

// turbo
1. Run the pre-commit check script:
   ```powershell
   .\scripts\pre-commit-check.ps1
   ```

2. If the script reports errors:
   - Fix all errors before committing
   - Re-run the script until all checks pass

## What gets checked:
- **Trailing whitespace**: Automatically removed
- **cargo fmt**: Formats all Rust code
- **cargo clippy**: Linter for warnings
- **Cargo.toml sorting**: Dependencies sorted alphabetically

## GitHub Integration
PRs are automatically fixed by **pre-commit.ci** if formatting issues are detected.
No manual intervention needed - fixes are committed directly to the PR branch.
