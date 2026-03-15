# GitHub Actions Workflows

This directory contains automated workflows for the MapFlow project, implementing a comprehensive CI/CD pipeline with Jules AI integration.

## 🤖 Workflows Overview

> [!IMPORTANT]
> **Current CI/CD Strategy (updated 2026-03-15):**
>
> The workflow setup has been unified to prevent pull requests from getting stuck because of inconsistent check sets.
>
> - **Pull requests are the only merge-relevant validation path**
> - **Automatic `push` validation on `main` was removed from the PR-gate workflows**
> - **`Build & Test (Windows)` now runs on every pull request**
> - **CodeQL remains available, but is no longer part of the required merge-blocking checks**
> - **Auto-merge is centrally controlled by `CICD-DevFlow: Job02 Auto-Merge`**
>
> This reduces GitHub Actions minute consumption while also making the PR gate deterministic.

### 1. CI/CD Pipeline (`CI-01_build-and-test.yml`) ⚡ HISTORICAL CONTEXT

**Purpose:** Comprehensive continuous integration and deployment pipeline

**Status:** Historical documentation entry from an earlier pipeline generation.

**Note:**  
This section is intentionally preserved for repository history and migration context.  
The actively maintained PR-gate workflows are now the `CICD-DevFlow_*` workflows documented further below.

**Historical Triggers:**

- Push to `main` branch (with path filters)
- Pull requests to `main` (with path filters)
- Manual dispatch

> [!TIP]
> **Optimization:** Path filters + parallel jobs reduce runtime by ~30-50%

**Historical Path Filters (only runs when these change):**

- `crates/**`, `Cargo.toml`, `Cargo.lock`, `scripts/**`, `deny.toml`

**Historical Jobs (PARALLEL execution after pre-checks):**

- **Pre-Checks:** Auto-formatting and Clippy fixes
- **Code Quality:** Linting and dependency checks (parallel)
- **Build & Test:** Linux builds with audio (parallel)
- **Windows Build:** Optional - only on `main` or with `test-windows` label
- **Security Audit:** cargo-audit and dependency review (parallel)
- **Success Gate:** Ensures all checks pass before merge

### 2. CodeQL Security Scan (`CI-02_security-scan.yml`) ⚡ UPDATED

**Purpose:** Automated security vulnerability detection

**Current Triggers:**

- Weekly schedule
- Manual dispatch

**Current role:**

- Provides a slower, deeper security analysis using CodeQL
- Is **not** part of the required PR merge gate
- Is intended for scheduled and on-demand security review

> [!TIP]
> **Optimization:** CodeQL was removed from the required PR-gate path because it is comparatively slow and expensive.
> Fast PR security checks remain in the validation workflow.

**Historical Trigger Context:**

- Push to `main` branch
- Weekly schedule
- Manual dispatch

### 3. Create Jules Development Issues (`CI-03_create-issues.yml`)

**Purpose:** Create all Jules development issues at once

**Triggers:**

- Manual dispatch only (run once to create all issues)

**Features:**

- Creates all development tasks from ROADMAP.md as GitHub issues
- Properly labeled with `jules-task` for Jules to process
- Includes priority labels and phase information
- Prevents duplicate creation
- Simple one-time setup

**Usage:**

```bash
# Create all Jules issues (run once)
gh workflow run CI-03_create-issues.yml
```

**Note:** This should be run ONCE to create all initial issues. Issues are pre-defined in the workflow, not parsed from ROADMAP.md (simpler and more reliable).

### 4. Jules Session Trigger (`CI-04_session-trigger.yml`) 🆕

**Purpose:** Automatically trigger Jules sessions when issues are created or labeled

**Triggers:**

- Issue opened with `jules-task` label
- `jules-task` label added to existing issue
- Manual dispatch (single issue)

**Features:**

- **Automatic Detection:** Monitors all issues with `jules-task` label
- **Official Jules Action:** Uses `google-labs-code/jules-action@v1` for reliable session creation
- **Tracking Comments:** Adds status comments to issues
- **Flexible Setup:** Works with or without API key (supports Jules GitHub App)

**Usage:**

```bash
# Automatically triggered when issue gets jules-task label

# Or manually trigger for specific issue:
gh workflow run CI-04_session-trigger.yml -f issue_number=123
```

**Configuration:**

- **Option 1 (Recommended):** Install Jules GitHub App at <https://github.com/apps/jules>
  - No API key needed
  - Works automatically with all `jules-task` issues
- **Option 2:** Add `JULES_API_KEY` as repository secret for GitHub Action-based automation
  - Get API key from <https://jules.google.com> (Settings)
  - Workflow uses official `google-labs-code/jules-action`
- **Fallback:** Manual session creation via jules.google.com

**What it does:**

1. Detects issues with `jules-task` label
2. Extracts issue details (number, title, body)
3. Adds tracking comment to issue
4. If `JULES_API_KEY` is configured:
   - Uses `google-labs-code/jules-action@v1` to create Jules session
   - Passes issue content as prompt
   - Jules creates branch, implements changes, opens PR
5. Updates issue with success/failure status
6. If no API key:
   - Still adds tracking comment
   - Jules GitHub App takes over (if installed)

### 5. Jules PR Auto-Merge (`CI-05_pr-automation.yml`) ✨ Historical Reference

**Purpose:** Automatically merge Jules PRs when all checks pass, with intelligent error handling

**Status:** Historical reference entry kept for documentation continuity.  
The actively used auto-merge logic now lives in `CICD-DevFlow_Job02_AutoMerge.yml`.

**Historical Triggers:**

- Pull request events (opened, synchronize, reopened, labeled)
- Check suite completion
- Workflow run completion (CI-01)
- Manual dispatch

**Historical Features:**

- **Intelligent Check Monitoring:** Waits for all checks to complete
- **Success Path:** Auto-merges when all checks pass
- **Error Path:** Creates detailed @jules comments with failure information
- **Merge Conflict Detection:** Notifies about conflicts
- **Failed Check Details:** Includes check names, summaries, and links
- **Retry Support:** Jules can update PR, checks re-run automatically

**Historical Auto-Merge Conditions:**

1. ✅ PR labeled with `jules-pr` or body contains "Created by Jules"
2. ✅ All CI checks pass (except auto-merge workflow itself)
3. ✅ No merge conflicts
4. ✅ No review requested changes
5. ✅ Not a draft PR

**Historical Error Handling:**

- Detects failed checks and collects details
- Creates @jules comment with:
  - List of failed checks
  - Error summaries
  - Links to detailed logs
- Allows Jules to fix and re-submit

### 6. Update Documentation (`CI-06_update-changelog.yml`)

**Purpose:** Keep CHANGELOG.md up to date automatically

**Triggers:**

- Pull request closed/merged

**Features:**

- Simple changelog updates
- Adds entry for each merged PR
- Commits changes automatically
- No complex parsing or updates - just adds changelog entries!

### 7. Post-Merge Automation (`CI-07_post-merge-automation.yml`) 🆕

**Purpose:** Complete post-merge tasks: close issue, update ROADMAP, trigger next session

**Triggers:**

- Pull request closed/merged (Jules PRs only)
- Manual dispatch

**Features:**

- **Issue Management:** Automatically closes related issue
- **ROADMAP Updates:** Marks tasks as completed
- **Continuous Automation:** Triggers CI-04 for next jules-task issue
- **Progress Tracking:** Adds completion comments

**Workflow:**

1. Extract issue number from PR body
2. Close related issue with success comment
3. Update ROADMAP.md:
   - Mark task as completed
   - Add PR reference
   - Commit changes
4. Trigger CI-04 for next oldest jules-task issue

### 8. Monitor Jules Session (`CI-08_monitor-jules-session.yml`) ⚡ OPTIMIZED

**Purpose:** Event-based monitoring of Jules sessions (no continuous polling)

**Triggers:**

- `workflow_call` from CI-04 (when session starts)
- `push` to `jules-*` branches (when Jules creates PR branch)
- Manual `workflow_dispatch`

> [!TIP]
> **Optimization:** Changed from continuous 6-hour polling to event-based triggers.
> **Savings:** ~2000+ minutes/week

**Features:**

- **Single-Run Check:** Runs once per trigger, no polling loop
- **Active Session Detection:** Finds sessions from issue comments
- **Automatic PR Creation:** Creates PR when session completes
- **Label Management:** Adds `jules-pr` label automatically
- **Timeout:** 30 minutes (reduced from 360)

**Workflow:**

1. Triggered by CI-04, branch push, or manual dispatch
2. Find all open `jules-task` issues
3. Check Jules API for each session (single check)
4. When session completes:
   - Create PR with proper labels
   - Add success comment to issue
5. Exit (no re-trigger)

## 🏷️ Labels Used

The automation system uses the following labels:

- `jules-task`: Issues that can be processed by Jules
- `jules-pr`: Pull requests created by Jules
- `priority: critical`: Critical priority tasks
- `priority: high`: High priority tasks
- `priority: medium`: Medium priority tasks
- `enhancement`: New features or improvements
- `bug`: Bug reports
- `feature-request`: Feature requests
- `documentation`: Documentation updates
- `test-windows`: Forces or documents Windows-relevant validation scope in historical workflow context

## 🔐 Permissions Required

The workflows require the following GitHub permissions:

- `contents: write` - For committing documentation updates
- `issues: write` - For creating and managing issues
- `pull-requests: write` - For managing PRs
- `security-events: write` - For CodeQL findings
- `checks: read` - For reading check status
- `statuses: read` - For reading external commit status integrations like `pre-commit.ci`

## 🚀 Jules Integration Setup

### Prerequisites

1. **GitHub Token:** The workflows use `GITHUB_TOKEN` which is automatically provided by GitHub Actions
2. **Jules API Configuration:** Configure Jules to:
   - Monitor issues with `jules-task` label
   - Create PRs with `jules-pr` label or "Created by Jules" in body
   - Follow the PR template format

### Complete Jules Automation Workflow

**📋 Phase 1: Issue Creation & Session Start**

1. **Issue Creation:**
   - Manual creation via issue templates
   - Batch creation via CI-03
   - Issues labeled with `jules-task`

2. **Session Trigger (CI-04):**
   - Automatically triggered when issue gets `jules-task` label
   - Or manually triggered for oldest open issue
   - Creates Jules session via API
   - Adds tracking comment to issue

**🔄 Phase 2: Session Monitoring**

3. **Continuous Monitoring (CI-08):**

- Runs every 5 minutes
- Polls Jules API for session status
- Detects when sessions complete or fail

4. **PR Creation (CI-08):**
   - Automatically creates PR when session completes
   - Adds `jules-pr` label
   - Links to issue and session
   - Notifies on issue

**🧪 Phase 3: Automated Testing**

5. **Current PR Gate (`CICD-DevFlow_Job01_Validation.yml`):**

- Triggered automatically on pull requests
- Code quality checks (format, lint)
- Security scan via `cargo-deny`
- Linux build and tests
- Windows build and tests
- Final success gate

**✅ Phase 4: Merge Decision**

6. **Current Success Path (`CICD-DevFlow_Job02_AutoMerge.yml`):**

- Waits for the required PR checks
- Evaluates mergeability and branch state
- Merges automatically when all required checks pass
- Adds/update managed PR comments

7. **Current Blocking Conditions:**

- Required PR checks are missing, pending, or failed
- Merge conflicts exist
- PR is behind `main`
- PR is draft or closed

**📝 Phase 5: Post-Merge Actions**

8. **Documentation Updates and Follow-up Automation:**

- CHANGELOG updates remain available in dedicated workflows
- Additional project automation can run separately from the merge gate
- Post-merge self-hosted validation can be enabled independently

**🎯 Result:** A more deterministic PR validation and merge pipeline, while preserving the broader Jules automation ecosystem.

### Configuration

To enable Jules integration:

1. Ensure Jules has access to the repository
2. Configure Jules to use the `development_task.yml` issue template
3. Set Jules to label PRs with `jules-pr`
4. Configure branch protection rules (recommended):
   - Require status checks to pass
   - Require review from code owners (optional)
   - Require branches to be up to date

## 📊 Monitoring

### Check Workflow Status

```bash
# List workflow runs
gh run list

# View specific run
gh run view <run-id>

# Watch a run in real-time
gh run watch <run-id>
```

### Trigger Workflows Manually

```bash
# Trigger validation workflow
gh workflow run "CICD-DevFlow: Job01 Validation"

# Trigger CodeQL security analysis manually
gh workflow run "Security Analysis"

# Trigger auto-merge re-evaluation manually
gh workflow run "CICD-DevFlow: Job02 Auto-Merge"

# Trigger documentation update
gh workflow run CI-06_update-changelog.yml
```

## 🛠️ Maintenance

### Adding New Workflows

1. Create `.yml` file in `.github/workflows/`
2. Define triggers and jobs
3. Test with manual dispatch first
4. Update this README

### Modifying Existing Workflows

1. Test changes in a feature branch
2. Use workflow dispatch for testing
3. Monitor logs carefully
4. Update documentation
5. Keep **workflow triggers**, **Branch Protection Rules**, and **auto-merge expected checks** aligned

### Troubleshooting

**Issue: CI fails with dependency errors**

- Check system dependencies in the active validation workflow
- Verify FFmpeg installation
- Check package availability on runner OS

**Issue: Auto-merge not working**

- Verify all required PR checks passed on the latest PR head commit
- Ensure no merge conflicts exist
- Review branch protection rules
- Confirm the auto-merge workflow expects the same checks as branch protection

**Issue: Pull requests seem to hang waiting for checks**

- Check whether Branch Protection requires outdated check names
- Ensure CodeQL is not configured as a required PR check
- Verify the required set matches the current `CICD-DevFlow` workflow names
- Confirm `pre-commit.ci - pr` finished successfully

**Issue: Issues not created from ROADMAP**

- Verify ROADMAP.md format
- Check workflow permissions
- Run with dry run or manual trigger first
- Check logs for parsing errors

## 📝 Best Practices

1. **Always test workflows with dry-run or manual dispatch first**
2. **Monitor workflow runs regularly**
3. **Keep ROADMAP.md format consistent**
4. **Use proper labels for automation**
5. **Review auto-merged PRs periodically**
6. **Update documentation when workflows change**
7. **Set up notifications for workflow failures**
8. **Treat PR validation as the single source of truth for merge decisions**

## 🔗 Related Documentation

- [Issue Templates](../ISSUE_TEMPLATE/)
- [ROADMAP.md](../../ROADMAP.md)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)

## 📞 Support

For issues with workflows:

1. Check workflow logs in Actions tab
2. Review this documentation
3. Open an issue with `workflows` label
4. Contact @MrLongNight for critical issues

---

## 🔄 Aktuelle CICD-DevFlow Workflows

### CICD-DevFlow_Job01_Validation.yml

**Aktuelle Trigger:**

- **Pull Requests zu `main` Branch** ✅ (automatisch)
- Manual Dispatch
- Workflow Call

> [!IMPORTANT]
> Automatische `push`-Validierung auf `main` wurde aus diesem Workflow entfernt, um GitHub-Action-Minuten zu sparen und doppelte Check-Sätze zu vermeiden.

**Aktuelle Jobs bei Pull Requests:**

1. **Quality Gate** (Format & Lint)
   - Formatierung prüfen (`cargo fmt`)
   - Clippy Linting (`cargo clippy`)

2. **Security Scan**
   - Advisory Scan über `cargo-deny`

3. **Build & Test (Linux)**
   - Workspace-Build mit CI-Features
   - Workspace-Tests mit CI-Features

4. **Build & Test (Windows)**
   - Läuft jetzt **immer** auf Pull Requests
   - Release Build mit Audio-Feature-Set

5. **Validation Success** - Final Gate
   - Sammelt alle Check-Results
   - Schlägt fehl, wenn einer der Pflichtjobs fehlschlägt

**Status:** ✅ Alle merge-relevanten Pflichtchecks laufen automatisch bei jedem Pull Request.

> [!IMPORTANT]
> **Troubleshooting:** Falls die Checks im GitHub UI nicht als "Expected" erscheinen:
>
> 1. Stelle sicher, dass die Workflows im Repository aktiviert sind (Settings → Actions → General → "Allow all actions")
> 2. Markiere die Jobs in den Branch Protection Rules als "required" (Settings → Branches → main → "Require status checks to pass before merging")
> 3. Empfohlene required checks:
>    - `pre-commit.ci - pr`
>    - `Quality Gate (Format & Lint)`
>    - `Security Scan`
>    - `Build & Test (Linux)`
>    - `Build & Test (Windows)`
>    - `Validation Success`

### CICD-DevFlow_Job02_AutoMerge.yml

**Aktuelle Trigger:**

- Pull Request Events (`opened`, `reopened`, `synchronize`, `labeled`, `unlabeled`, `ready_for_review`)
- Workflow Run Completion von `CICD-DevFlow: Job01 Validation`
- Manual Dispatch

**Funktion:**

- Wartet auf erfolgreichen Abschluss aller Pflicht-Checks
- Bewertet Mergebarkeit und Branch-Status
- Fordert bei `behind` ein Branch-Update an
- Merged automatisch bei grünen Checks
- Erstellt hilfreiche PR-Kommentare bei blockierenden Zuständen

**Aktueller Pflichtsatz für Auto-Merge:**

- `pre-commit.ci - pr`
- `Quality Gate (Format & Lint)`
- `Security Scan`
- `Build & Test (Linux)`
- `Build & Test (Windows)`
- `Validation Success`

> [!IMPORTANT]
> `Analyze (rust)` aus CodeQL ist **kein** Auto-Merge-Pflichtcheck mehr.

### CICD-DevFlow_Job00_EnableAutoMerge.yml

**Status:** Deaktiviert.

**Grund:**  
MapFlow verwendet bewusst nur noch einen zentralen Auto-Merge-Mechanismus über `CICD-DevFlow_Job02_AutoMerge.yml`, damit keine widersprüchlichen Zustände zwischen GitHub Auto-Merge und eigenem Workflow entstehen.

### CI-02_security-scan.yml

**Aktuelle Trigger:**

- Weekly Schedule
- Manual Dispatch

**Funktion:**

- Führt CodeQL-Sicherheitsanalyse aus
- Dient der tieferen Sicherheitsüberwachung
- Ist nicht merge-blockierend für Pull Requests

---

## 🔧 Branch Protection Rules Konfiguration

Um die PR-Checks als "required" zu markieren, folge diesen Schritten:

1. Gehe zu **Settings** → **Branches** → **main**
2. Aktiviere "Require status checks to pass before merging"
3. Wähle folgende Checks als required aus:
   - `pre-commit.ci - pr`
   - `Quality Gate (Format & Lint)`
   - `Security Scan`
   - `Build & Test (Linux)`
   - `Build & Test (Windows)`
   - `Validation Success`
4. **Nicht** als required markieren:
   - `Analyze (rust)`
   - Release-Workflows
   - Backup-Workflows
   - Post-Merge Self-Hosted Workflows
   - Changelog-/Cleanup-Workflows
5. Optional: Aktiviere "Require branches to be up to date before merging"

Die Checks werden dann als "Expected" im PR angezeigt und müssen vor dem Merge grün sein.

---

📋 Current PR-Check Flow:
┌─────────────────────────────────────────────────────────┐
│                    PR erstellt                          │
└────────────────────┬────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
         ▼                       ▼
┌─────────────────┐    ┌─────────────────────────────────┐
│ pre-commit.ci   │    │ GitHub Actions Validation       │
│                 │    │                                 │
│ • cargo fmt     │    │ • Quality Gate                  │
│ • trailing ws   │    │ • Security Scan                 │
│ • YAML/TOML     │    │ • Build & Test (Linux)          │
│ • Markdown      │    │ • Build & Test (Windows)        │
│                 │    │ • Validation Success            │
│ ⚡ ~30s         │    │                                 │
│ ✅ Auto-Push   │    │ ✅ Merge-relevant gate           │
└────────┬────────┘    └───────────────┬────��────────────┘
         │                             │
         └─────────────┬───────────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │ Copilot Review       │
            │ ~1-2 min             │
            └──────────┬───────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │ Auto-Merge           │
            │ • Check Status       │
            │ • Merge if OK        │
            │ • Or comment reason  │
            └──────────────────────┘

✅ Zusammenfassung der Dateipfade:
Datei | Pfad | Grund
--- | --- | ---
`.markdownlint.json` | Root | Wird von `markdownlint-cli` im Root gesucht
`.secrets.baseline` | Root | Wird von `detect-secrets` im Root gesucht
`.pre-commit-config.yaml` | Root | Standard für `pre-commit`
`copilot-instructions.md` | `.github/` | GitHub-spezifische Config
Workflows | `.github/workflows/` | GitHub Actions Standard

✅ Historische Commit-Reihenfolge für die PR-Check-Einführung:

```bash
# Schritt 1: Root-Config-Dateien
git add .markdownlint.json
git add .secrets.baseline
git commit -m "config: add markdownlint and secrets baseline"

# Schritt 2: Pre-Commit erweitern
git add .pre-commit-config.yaml
git commit -m "ci: enhance pre-commit with Rust, markdown, and security checks"

# Schritt 3: Copilot Instructions
git add .github/copilot-instructions.md
git commit -m "docs: add Copilot review instructions"

# Schritt 4: Workflows
git add .github/workflows/CICD-DevFlow_Job01_Validation.yml
git add .github/workflows/CICD-DevFlow_Job02_AutoMerge.yml
git commit -m "ci: implement validation and auto-merge with Jules feedback"

# Push alles
git push
```

**Last Updated:** 2026-03-15 (PR-Gate vereinheitlicht, Windows immer aktiv, CodeQL aus Required Checks entfernt)  
**Maintained By:** MapFlow Team
