# Sequential Jules Issue Processing

## 🎯 Overview

The Jules CI/CD integration now enforces **sequential processing** of issues to ensure quality and prevent conflicts. Only one issue is processed at a time, and the next issue is automatically picked up after the current work is completed.

## 🔄 How It Works

### 1. Issue Detection

When an issue is labeled with `jules-task`, the CI-04 workflow is triggered.

### 2. Busy Check

Before starting work, CI-04 checks if there are any open Jules PRs:

```yaml
# Check for existing open Jules PRs
- If open Jules PR(s) exist: ⏸️ Queue the issue
- If no open Jules PRs: ✅ Process the issue
```

### 3. Queueing

If Jules is busy (has an open PR), the issue is queued:

- A comment is added to the issue explaining the queue status
- The issue shows which PR is currently being worked on
- No duplicate session is created

### 4. Sequential Processing

After a Jules PR is merged:

1. CI-07 (Post-Merge Automation) runs
2. Related issue is closed
3. ROADMAP.md is updated
4. **CI-04 is automatically triggered** for the next queued issue

### 5. Next Issue Selection

CI-04 automatically selects the **oldest open jules-task** issue:

- Issues are sorted by creation date
- The oldest issue is processed first
- FIFO (First In, First Out) queue principle

## 📋 Example Flow

```
Issue #1 labeled with "jules-task"
    ↓
CI-04: No open Jules PRs → Start working on #1
    ↓
Jules creates PR #10 for Issue #1
    ↓
Issue #2 labeled with "jules-task"
    ↓
CI-04: Found open PR #10 → Queue Issue #2
    ↓
Comment added to Issue #2: "Jules is currently busy..."
    ↓
PR #10 passes all checks and is merged
    ↓
CI-07: Close Issue #1, Update ROADMAP, Trigger CI-04
    ↓
CI-04: No open Jules PRs → Start working on #2
    ↓
Jules creates PR #11 for Issue #2
    ↓
... (cycle continues)
```

## 🎨 User Experience

### When an Issue is Queued

Users will see a comment on their issue:

```markdown
⏸️ **Jules is currently busy**

Jules is already working on another issue. This issue is queued and will be 
processed after the current work is completed.

**Open Jules PR(s):** #10 - Multi-Window Rendering Implementation

**What happens next:**
1. The current PR will be reviewed and merged
2. Once merged, this issue will be automatically picked up
3. You will be notified when Jules starts working on this issue

_Sequential processing ensures quality and prevents conflicts._
```

### When Work Starts

When Jules starts working on a queued issue:

```markdown
🤖 **Jules Session Triggered** — Jules session created. 🔗 [Session URL]
```

## 🔧 Technical Details

### Concurrency Control

The workflow uses multiple layers of concurrency control:

1. **Repository-wide lock** (concurrency group):
   ```yaml
   concurrency:
     group: ci-04-jules-session-${{ github.repository }}
     cancel-in-progress: false
   ```
   Prevents parallel runs of CI-04 workflow.

2. **PR existence check** (new):
   - Before processing, checks for open Jules PRs
   - Blocks processing if any open PR exists
   - Adds informative comment to queued issue

3. **Auto-trigger after merge**:
   - CI-07 automatically triggers CI-04 after merge
   - Ensures next issue is picked up immediately

### Key Checks

CI-04 performs these checks in order:

1. ✅ Issue has `jules-task` label
2. ✅ Issue is open (not closed)
3. ✅ **No open Jules PRs exist** ← NEW
4. ✅ No existing Jules session for this issue
5. ✅ JULES_API_KEY is configured

### Workflow Modifications

**CI-04_session-trigger.yml:**
- Added `check-jules-pr` step
- All processing steps now check `has_open_pr == 'false'`
- Added queue notification step
- Enhanced logging with PR status

**CI-07_post-merge-automation.yml:**
- Already triggers CI-04 after merge (no changes needed)
- Ensures automatic processing of next queued issue

## 🚀 Benefits

### 1. Quality Assurance
- One issue at a time = focused, quality work
- No parallel sessions = no conflicts
- Complete testing before moving to next issue

### 2. Resource Management
- Prevents API rate limiting
- Efficient use of CI/CD resources
- Predictable workflow execution

### 3. Conflict Prevention
- No concurrent changes to same files
- Sequential merges = clean history
- Easier to debug issues

### 4. Transparency
- Clear queue status for users
- Visible order of processing
- Predictable completion timeline

## 📊 Monitoring

### Check Queue Status

List all queued issues:
```bash
gh issue list --label "jules-task" --state open
```

### Check Active Work

List open Jules PRs:
```bash
gh pr list --label "jules-pr" --state open
```

### View Workflow Runs

```bash
gh run list --workflow="CI-04: Session Trigger"
```

## 🔍 Troubleshooting

### Issue Not Being Picked Up

**Problem:** Issue has `jules-task` label but nothing happens.

**Check:**
1. Is there an open Jules PR? If yes, issue is queued.
2. Check workflow runs: `gh run list --workflow="CI-04"`
3. Look for comments on the issue indicating queue status

**Solution:**
- Wait for current PR to merge
- Or manually trigger: `gh workflow run CI-04_session-trigger.yml`

### Multiple Issues Triggered

**Problem:** Multiple issues seem to be processed at once.

**Impossible:** The new system prevents this.
- Only one issue can be processed at a time
- If you see multiple PRs, they were created before this update

**Verification:**
```bash
# Check PR creation dates
gh pr list --label "jules-pr" --json number,createdAt,title
```

### Queue Not Moving

**Problem:** PR merged but next issue not picked up.

**Check:**
1. Was CI-07 executed? `gh run list --workflow="CI-07"`
2. Did CI-07 trigger CI-04? Check workflow logs
3. Are there more `jules-task` issues? `gh issue list --label "jules-task"`

**Solution:**
```bash
# Manually trigger next issue
gh workflow run CI-04_session-trigger.yml
```

## 🎓 Best Practices

### For Users

1. **Label issues when ready:** Only add `jules-task` when issue is ready
2. **Be patient:** If queued, wait for current work to complete
3. **Check queue:** Use issue list to see position in queue

### For Maintainers

1. **Monitor queue depth:** Keep reasonable number of queued issues
2. **Review queue order:** Oldest issues are processed first
3. **Prioritize manually:** Close/reopen issues to change queue order

## 📝 Version History

**v1.0 - 2024-12-10:**
- Initial implementation of sequential processing
- Added PR existence check in CI-04
- Enhanced queue notifications
- Integrated with existing CI-07 auto-trigger

---

**Last Updated:** 2024-12-10  
**Author:** GitHub Copilot  
**Related Workflows:** CI-04, CI-07
