---
description: Pre-commit checks - run before every git commit to ensure code quality
---

# Pre-Commit Check Workflow

Before EVERY `git commit`, you MUST run the pre-commit check script:

// turbo
1. Run the pre-commit check script:
   ```powershell
   .\scripts\pre-commit-check.ps1
   ```

2. If the script reports errors:
   - Fix all errors before committing
   - Re-run the script until all checks pass

3. If cargo fmt made changes:
   - The changed files are automatically staged
   - Include them in your commit

4. Only after all checks pass, proceed with:
   ```powershell
   git add <files>
   git commit -m "your message"
   git push origin main
   ```

## What the script checks:
- **cargo fmt**: Formats all Rust code
- **Format changes**: Detects if fmt modified files
- **cargo check**: Verifies compilation
- **cargo clippy**: Runs linter for warnings
