# GitHub Actions Workflows

This directory contains automated workflows for the MapFlow project, implementing a comprehensive CI/CD pipeline with Jules AI integration.

## 🤖 Workflows Overview

### 1. CI/CD Pipeline (`CI-01_build-and-test.yml`) ⚡ OPTIMIZED

**Purpose:** Comprehensive continuous integration and deployment pipeline

**Triggers:**

- Push to `main` branch (with path filters)
- Pull requests to `main` (with path filters)
- Manual dispatch

> [!TIP]
> **Optimization:** Path filters + parallel jobs reduce runtime by ~30-50%

**Path Filters (only runs when these change):**

- `crates/**`, `Cargo.toml`, `Cargo.lock`, `scripts/**`, `deny.toml`

**Jobs (PARALLEL execution after pre-checks):**

- **Pre-Checks:** Auto-formatting and Clippy fixes
- **Code Quality:** Linting and dependency checks (parallel)
- **Build & Test:** Linux builds with audio (parallel)
- **Windows Build:** Optional - only on `main` or with `test-windows` label
- **Security Audit:** cargo-audit and dependency review (parallel)
- **Success Gate:** Ensures all checks pass before merge

### 2. CodeQL Security Scan (`CI-02_security-scan.yml`) ⚡ OPTIMIZED

**Purpose:** Automated security vulnerability detection

**Triggers:**

- Push to `main` branch (with path filters)
- Weekly schedule (Mondays at 00:00 UTC)
- Manual dispatch

> [!TIP]
> **Optimization:** Removed PR trigger - CodeQL is slow and not needed for every PR.
> Fast security checks (cargo-audit) are in CI-01 for PRs.

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

### 5. Jules PR Auto-Merge (`CI-05_pr-automation.yml`) ✨ Enhanced

**Purpose:** Automatically merge Jules PRs when all checks pass, with intelligent error handling

**Triggers:**

- Pull request events (opened, synchronize, reopened, labeled)
- Check suite completion
- Workflow run completion (CI-01)
- Manual dispatch

**Features:**

- **Intelligent Check Monitoring:** Waits for all checks to complete
- **Success Path:** Auto-merges when all checks pass
- **Error Path:** Creates detailed @jules comments with failure information
- **Merge Conflict Detection:** Notifies about conflicts
- **Failed Check Details:** Includes check names, summaries, and links
- **Retry Support:** Jules can update PR, checks re-run automatically

**Auto-Merge Conditions:**

1. ✅ PR labeled with `jules-pr` or body contains "Created by Jules"
2. ✅ All CI checks pass (except auto-merge workflow itself)
3. ✅ No merge conflicts
4. ✅ No review requested changes
5. ✅ Not a draft PR

**Error Handling:**

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
- **Label Management:** Adds jules-pr label automatically
- **Timeout:** 30 minutes (reduced from 360)

**Workflow:**

1. Triggered by CI-04, branch push, or manual dispatch
2. Find all open jules-task issues
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

## 🔐 Permissions Required

The workflows require the following GitHub permissions:

- `contents: write` - For committing documentation updates
- `issues: write` - For creating and managing issues
- `pull-requests: write` - For managing PRs
- `security-events: write` - For CodeQL findings
- `checks: read` - For reading check status

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
   - Automatically triggered when issue gets jules-task label
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
   - Adds jules-pr label
   - Links to issue and session
   - Notifies on issue

**🧪 Phase 3: Automated Testing**
5. **CI/CD Pipeline (CI-01):**

- Triggered automatically on PR
- Code quality checks (format, lint)
- Multi-platform builds
- Security scanning

**✅ Phase 4: Merge Decision**
6. **Success Path (CI-05):**

- All checks pass → automatic merge
- Success comment added
- Triggers post-merge automation

7. **Error Path (CI-05):**
   - Checks fail → detailed @jules comment
   - Lists all failed checks with summaries
   - Jules can update PR
   - Checks re-run automatically

**📝 Phase 5: Post-Merge Actions**
8. **Documentation Updates (CI-06 & CI-07):**

- ROADMAP.md marked as completed
- Changelog entry added
- Issue automatically closed
- Success comments added

9. **Continuous Automation (CI-07):**
   - Triggers CI-04 for next oldest jules-task issue
   - Cycle repeats automatically
   - Fully self-sustaining workflow

**🎯 Result:** Fully automated development pipeline from issue creation to merge, with intelligent error handling and continuous operation.

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
# Trigger CI/CD pipeline
gh workflow run "CI/CD Pipeline"

# Create issues from roadmap (dry run)
gh workflow run auto-create-issues.yml -f dry_run=true

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

### Troubleshooting

**Issue: CI fails with dependency errors**

- Check system dependencies in `CI-01_build-and-test.yml`
- Verify FFmpeg installation
- Check package availability on runner OS

**Issue: Auto-merge not working**

- Verify PR has `jules-pr` label
- Check all required checks pass
- Ensure no merge conflicts
- Review branch protection rules

**Issue: Issues not created from ROADMAP**

- Verify ROADMAP.md format
- Check workflow permissions
- Run with dry_run=true first
- Check logs for parsing errors

## 📝 Best Practices

1. **Always test workflows with dry-run or manual dispatch first**
2. **Monitor workflow runs regularly**
3. **Keep ROADMAP.md format consistent**
4. **Use proper labels for automation**
5. **Review auto-merged PRs periodically**
6. **Update documentation when workflows change**
7. **Set up notifications for workflow failures**

## 🔗 Related Documentation

- [Issue Templates](../ISSUE_TEMPLATE/)
- [Pull Request Template](../pull_request_template.md)
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

**Trigger:**

- Push zu `main` Branch
- **Pull Requests zu `main` Branch** ✅ (automatisch)
- Manual Dispatch

**Jobs bei Pull Requests:**

1. **Quality Gate** (Format & Lint)
   - Formatierung prüfen (`cargo fmt`)
   - Clippy Linting (`cargo clippy`)
   - Dependency Sortierung (`cargo sort`)

2. **Security Scan**
   - Vulnerability Scan (`cargo audit`)
   - Dependency Review (`cargo deny`)

3. **Build & Test (Linux)**
   - Release Build mit CI-Features
   - Integration Tests (`cargo nextest`)

4. **Build & Test (Windows)** - Optional
   - Nur bei `test-windows` Label oder auf `main`
   - Audio-only Build

5. **Validation Success** - Final Gate
   - Sammelt alle Check-Results
   - Benachrichtigt bei Fehlern mit detaillierter Anleitung

**Status:** ✅ Alle Checks laufen automatisch bei jedem Pull Request

> [!IMPORTANT]
> **Troubleshooting:** Falls die Checks im GitHub UI nicht als "Expected" erscheinen:
>
> 1. Stelle sicher, dass die Workflows im Repository aktiviert sind (Settings → Actions → General → "Allow all actions")
> 2. Markiere die Jobs in den Branch Protection Rules als "required" (Settings → Branches → main → "Require status checks to pass before merging")
> 3. Empfohlene required checks:
>    - `Quality Gate (Format & Lint)`
>    - `Security Scan`
>    - `Build & Test (Linux)`
>    - `Validation Success`

### CICD-DevFlow_Job02_AutoMerge.yml

**Trigger:**

- Pull Request Events (labeled, synchronize, opened, reopened)
- Check Suite Completion
- Workflow Run Completion

**Funktion:**

- Wartet auf erfolgreichen Abschluss aller Checks
- Merged automatisch bei grünen Checks
- Erstellt hilfreiche Fehler-Comments bei fehlgeschlagenen Checks

---

## 🔧 Branch Protection Rules Konfiguration

Um die PR-Checks als "required" zu markieren, folge diesen Schritten:

1. Gehe zu **Settings** → **Branches** → **main**
2. Aktiviere "Require status checks to pass before merging"
3. Wähle folgende Checks als required aus:
   - `Quality Gate (Format & Lint)`
   - `Security Scan`
   - `Build & Test (Linux)`
   - `Validation Success`
4. Optional: Aktiviere "Require branches to be up to date before merging"

Die Checks werden dann als "Expected" im PR angezeigt und müssen vor dem Merge grün sein.

---

📋 New PR-Check Flow:
┌─────────────────────────────────────────────────────────┐
│                    PR erstellt                          │
└────────────────────┬────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
         ▼                       ▼
┌─────────────────┐    ┌─────────────────┐
│ pre-commit.ci   │    │ GitHub Actions  │
│                 │    │ Validation      │
│ • cargo fmt     │    │                 │
│ • trailing ws   │    │ • Build & Test  │
│ • YAML/TOML     │    │ • Security      │
│ • Markdown      │    │ • Clippy        │
│                 │    │                 │
│ ⚡ ~30s         │    │ ⏰ ~10min       │
│ ✅ Auto-Push   │    │ ❌ Report only │
└────────┬────────┘    └────────┬────────┘
         │                      │
         └──────────┬───────────┘
                    │
                    ▼
         ┌──────────────────┐
         │ Copilot Review   │
         │ ~1-2 min         │
         └────────┬─────────┘
                  │
                  ▼
         ┌──────────────────┐
         │ Auto-Merge       │
         │ • Check Status   │
         │ • Merge if OK    │
         │ • Or @jules      │
         └──────────────────┘

✅ Zusammenfassung der Dateipfade:
Datei Pfad Grund
.markdownlint.json Root Wird von markdownlint-cli im Root gesucht
.secrets.baseline Root Wird von detect-secrets im Root gesucht
.pre-commit-config.yaml Root Standard für pre-commit
copilot-instructions.md .github/ GitHub-spezifische Config
Workflows .github/workflows/ GitHub Actions Standard

✅ Vollständige Commit-Reihenfolge:

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
**Last Updated:** 2026-02-09 (PR-Check Konfiguration validiert)
**Maintained By:** MapFlow Team
