# CI/CD Workflow Quick Reference

> **Schnellreferenz für alle CI/CD Workflows**

## 🎯 Workflow-Übersicht

| Workflow | Trigger | Zweck | Dauer |
|----------|---------|-------|-------|
| **CICD-DevFlow: Job01 Validation** | Push/PR | Build, Test, Clippy, Format, Security | ~10-15 min |
| **CICD-DevFlow: Job02 AutoMerge** | PR Events | Automatisches Mergen nach erfolgreichen Checks | Sekunden |
| **CICD-MainFlow: Job01 Changelog** | Push main | Aktualisiert CHANGELOG.md & schließt Issues | Sekunden |
| **CICD-MainFlow: Job02 Backup** | Schedule | GFS-Backup (Tag/Woche/Monat) | ~2 min |
| **CICD-IssueFlow: Job01 Trigger** | Label/Issue | Startet Jules Session für Issues | Sekunden |
| **gemini-dispatch** | Comment/PR | Router für KI-Befehle (@gemini-cli) | Sekunden |
| **gemini-review** | Call/PR | KI-basiertes Pull Request Review | ~2-5 min |
| **gemini-triage** | Issue open | Automatisches Labeling von Issues | Sekunden |

## 🚀 Häufige Kommandos

### Issue Management

```bash
# Issue mit jules-task Label erstellen
gh issue create --label "jules-task" --title "Title" --body "Description"

# Alle Jules Tasks anzeigen
gh issue list --label "jules-task"

# Offene Jules Tasks
gh issue list --label "jules-task" --state open

# Status eines Issues
gh issue view <issue-number>
```

### Session Management

```bash
# Session für Issue triggern
gh workflow run CI-04_session-trigger.yml -f issue_number=<N>

# Ältestes Issue automatisch wählen
gh workflow run CI-04_session-trigger.yml

# Session Status prüfen (via Monitoring)
gh workflow run CI-08_monitor-jules-session.yml

# Session Monitoring History
gh run list --workflow="Monitor Jules Session" --limit 10
```

### PR Management

```bash
# Jules PRs anzeigen
gh pr list --label "jules-pr"

# PR Status prüfen
gh pr view <pr-number>
gh pr view <pr-number> --json mergeable,statusCheckRollup

# PR Checks anzeigen
gh pr checks <pr-number>

# PR Comments (inkl. @jules Notifications)
gh pr view <pr-number> --comments
```

### Workflow Management

```bash
# Alle laufenden Workflows
gh run list --limit 10

# Spezifischer Workflow
gh run list --workflow="PR Auto-Merge"
gh run list --workflow="Post-Merge Automation"

# Workflow Logs
gh run view <run-id> --log

# Live-Monitoring
gh run watch
```

## 🔄 Typische Szenarien

### Szenario 1: Neues Jules Task starten

```bash
# 1. Issue erstellen
gh issue create \
  --label "jules-task,priority: high" \
  --title "Implement feature X" \
  --body "Description of the task..."

# 2. Session wird automatisch getriggert
# Alternativ manuell:
gh workflow run CI-04_session-trigger.yml

# 3. Monitoring prüfen
gh run watch
```

### Szenario 2: Batch von Tasks verarbeiten

```bash
# 1. Alle Development Issues erstellen
gh workflow run CI-03_create-issues.yml

# 2. Erste Session starten
gh workflow run CI-04_session-trigger.yml

# 3. System läuft automatisch weiter
# (Nach jedem Merge wird nächstes Issue automatisch gestartet)
```

### Szenario 3: Fehlerhafte Session debuggen

```bash
# 1. Issue und Session-ID identifizieren
gh issue view <issue-number> --comments

# 2. Monitoring Logs prüfen
gh run list --workflow="Monitor Jules Session"
gh run view <run-id> --log

# 3. Manual re-trigger wenn nötig
gh workflow run CI-08_monitor-jules-session.yml
```

### Szenario 4: Failed PR untersuchen

```bash
# 1. PR Status prüfen
gh pr view <pr-number> --json statusCheckRollup

# 2. Failed Checks identifizieren
gh pr checks <pr-number>

# 3. @jules Kommentar prüfen
gh pr view <pr-number> --comments | grep "@jules"

# 4. Build Logs ansehen
gh run list --workflow="Build & Test"
gh run view <run-id> --log
```

### Szenario 5: ROADMAP Status prüfen

```bash
# 1. Letzte Roadmap Changes
git log --oneline -10 ROADMAP.md

# 2. Current status
cat ROADMAP.md | grep -A 20 "## Open Items"

# 3. Recently completed
cat ROADMAP.md | grep -A 10 "Recently Completed"
```

## ⚠️ Troubleshooting

### Problem-Lösung Matrix

| Problem | Symptom | Lösung |
|---------|---------|--------|
| Session nicht erstellt | Kein Kommentar im Issue | Check JULES_API_KEY, trigger manuell |
| PR nicht erstellt | Session fertig, kein PR | Check CI-08 Logs, trigger manuell |
| Auto-Merge failed | PR offen trotz grünen Checks | Check PR mergeable status |
| Checks schlagen fehl | @jules Kommentar | Jules updated PR automatisch |
| Merge Conflicts | Auto-Merge failed Kommentar | @jules behebt Konflikte |
| Roadmap nicht updated | Nach Merge fehlt Update | Check CI-07 Logs |
| Nächste Session nicht gestartet | Kein neues jules-task Issue | Manuell triggern oder Issues erstellen |

### Debug Commands

```bash
# Full System Status
echo "=== Open Jules Tasks ==="
gh issue list --label "jules-task" --state open

echo "=== Active Jules PRs ==="
gh pr list --label "jules-pr"

echo "=== Recent Workflow Runs ==="
gh run list --limit 5

echo "=== Failed Runs (last 24h) ==="
gh run list --status failure --created $(date -d '1 day ago' +%Y-%m-%d)

# Detailed PR Analysis
gh pr view <pr-number> --json \
  title,number,state,mergeable,statusCheckRollup,labels,body

# Workflow Run Details
gh api repos/{owner}/{repo}/actions/runs/<run-id> | jq .
```

## 📋 Checklists

### Setup Checklist

- [ ] JULES_API_KEY Secret konfiguriert
- [ ] Labels synchronisiert (`gh label sync`)
- [ ] Alle Workflows in main branch
- [ ] GitHub Actions aktiviert
- [ ] Branch Protection Rules konfiguriert
- [ ] Test Issue erstellt und durchlaufen

### Before Manual Intervention

- [ ] Check Workflow Logs
- [ ] Review Issue Comments
- [ ] Verify PR Status
- [ ] Check API Key validity
- [ ] Confirm no pending runs

### After Major Changes

- [ ] Test mit Single Issue
- [ ] Monitor Complete Cycle
- [ ] Check Error Handling
- [ ] Verify Documentation Updates
- [ ] Update ROADMAP if needed

## 🎯 Performance Tips

### Optimierung

```bash
# Cache Status prüfen
gh api repos/{owner}/{repo}/actions/cache/usage

# Lange laufende Workflows identifizieren
gh run list --json workflowName,conclusion,createdAt,updatedAt \
  | jq '.[] | select(.conclusion == "success") |
    {workflow: .workflowName, duration: (.updatedAt | fromdate) - (.createdAt | fromdate)}'

# Failed Runs analysieren
gh run list --status failure --limit 20 --json workflowName,conclusion \
  | jq 'group_by(.workflowName) | map({workflow: .[0].workflowName, count: length})'
```

### Best Practices

1. **Issue-Beschreibungen:** Klar und konkret
2. **Labels:** Immer verwenden für besseres Tracking
3. **Monitoring:** Regelmäßig Logs prüfen
4. **Documentation:** ROADMAP up-to-date halten
5. **Error Handling:** @jules Kommentare beachten

## 📊 Monitoring Dashboard

### Daily Check

```bash
#!/bin/bash
# daily-check.sh

echo "=== Daily CI/CD Status ==="
echo ""

echo "📋 Open Jules Tasks:"
gh issue list --label "jules-task" --state open | wc -l

echo "🔄 Active Jules PRs:"
gh pr list --label "jules-pr" | wc -l

echo "✅ Merged Today:"
gh pr list --label "jules-pr" --state merged \
  --search "merged:>=$(date +%Y-%m-%d)" | wc -l

echo "❌ Failed Runs (last 24h):"
gh run list --status failure --created $(date -d '1 day ago' +%Y-%m-%d) | wc -l

echo ""
echo "Last 5 Runs:"
gh run list --limit 5
```

### Weekly Report

```bash
#!/bin/bash
# weekly-report.sh

START_DATE=$(date -d '7 days ago' +%Y-%m-%d)

echo "=== Weekly CI/CD Report ==="
echo "Period: $START_DATE to $(date +%Y-%m-%d)"
echo ""

echo "📊 Issues Processed:"
gh issue list --label "jules-task" --state closed \
  --search "closed:>=$START_DATE" | wc -l

echo "✅ PRs Merged:"
gh pr list --label "jules-pr" --state merged \
  --search "merged:>=$START_DATE" | wc -l

echo "🎯 Success Rate:"
TOTAL=$(gh run list --created $START_DATE | wc -l)
SUCCESS=$(gh run list --created $START_DATE --status success | wc -l)
echo "$((SUCCESS * 100 / TOTAL))%"
```

## 🔗 Links

- **CI/CD README:** [CI_CD_README.md](README_CICD.md)
- **Workflow Details:** [workflows/README.md](../../.github/workflows/README.md)
- **Roadmap:** [ROADMAP.md](../../ROADMAP.md)

---

**Version:** 1.0
**Last Updated:** 2024-12-04

**💡 Tipp:** Dieses Dokument als Bookmark speichern für schnellen Zugriff!
