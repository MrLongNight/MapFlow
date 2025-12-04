# GitHub Actions Workflows

This directory contains automated workflows for the VjMapper project, implementing a comprehensive CI/CD pipeline with Jules AI integration.

## 🤖 Workflows Overview

### 1. CI/CD Pipeline (`CI-01_build-and-test.yml`)

**Purpose:** Comprehensive continuous integration and deployment pipeline

**Triggers:**
- Push to `main` branch
- Pull requests to `main`
- Manual dispatch

**Jobs:**
- **Code Quality:** Formatting (`cargo fmt`) and linting (`cargo clippy`)
- **Build & Test:** Multi-platform builds (Linux, macOS, Windows)
- **Security Audit:** Dependency vulnerability scanning
- **Success Gate:** Ensures all checks pass before merge

**Features:**
- Caches Cargo dependencies for faster builds
- Parallel execution on multiple platforms
- Generates release artifacts
- Comprehensive test execution

### 2. CodeQL Security Scan (`CI-02_security-scan.yml`)

**Purpose:** Automated security vulnerability detection

**Triggers:**
- Push to `main` branch
- Pull requests to `main`
- Weekly schedule (Mondays at 00:00 UTC)
- Manual dispatch

**Features:**
- Deep security analysis of Rust code
- Security and quality queries
- Automatic issue creation for findings
- Integration with GitHub Security tab

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
- **Option 1 (Recommended):** Install Jules GitHub App at https://github.com/apps/jules
  - No API key needed
  - Works automatically with all `jules-task` issues
- **Option 2:** Add `JULES_API_KEY` as repository secret for GitHub Action-based automation
  - Get API key from https://jules.google.com (Settings)
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

### 5. Jules PR Auto-Merge (`CI-05_pr-automation.yml`)

**Purpose:** Automatically merge Jules PRs when all checks pass

**Triggers:**
- Pull request events (opened, synchronize, labeled)
- Check suite completion

**Features:**
- Simple and reliable auto-merge logic
- Checks all CI tests pass
- Prevents merge on conflicts or requested changes
- Automatically closes related issues
- No complex validation - just works!

**Auto-Merge Conditions:**
1. ✅ PR labeled with `jules-pr` or body contains "Created by Jules"
2. ✅ All CI checks pass (except auto-merge workflow itself)
3. ✅ No merge conflicts
4. ✅ No review requested changes
5. ✅ Not a draft PR

### 6. Update Documentation (`CI-06_update-changelog.yml`)

**Purpose:** Keep CHANGELOG.md up to date automatically

**Triggers:**
- Pull request closed/merged

**Features:**
- Simple changelog updates
- Adds entry for each merged PR
- Commits changes automatically
- No complex parsing or updates - just adds changelog entries!

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

### Jules Workflow

1. **Issue Creation:**
   - Manual creation via issue templates
   - Automatic creation from ROADMAP.md
   - Issues labeled with `jules-task`

2. **Jules Processing:**
   - Jules monitors `jules-task` issues
   - Creates branch and implements changes
   - Opens PR with proper labels and references

3. **Automated Testing:**
   - CI/CD pipeline runs automatically
   - Code quality checks execute
   - Security scanning performed

4. **Auto-Merge:**
   - If all checks pass → automatic merge
   - If checks fail → comment posted, manual review required
   - Related issue automatically closed

5. **Documentation Update:**
   - ROADMAP.md updated
   - Changelog entry added
   - Progress tracked

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

**Last Updated:** 2024-12-04  
**Maintained By:** VjMapper Team
