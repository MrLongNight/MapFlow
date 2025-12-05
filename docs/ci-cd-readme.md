# CI/CD & Jules Integration - Quick Reference

> **Vereinfachte CI/CD Pipeline mit Google Jules API Integration**

## 🎯 Was ist implementiert?

Eine vollständige, produktionsbereite CI/CD Pipeline mit automatisierter Entwicklung durch Jules AI:

- ✅ **Multi-Platform CI/CD** - Automatisches Bauen und Testen auf Linux, macOS, Windows
- ✅ **Security Scanning** - CodeQL und Dependency Audits
- ✅ **Jules Integration** - Automatisierte Issue-Bearbeitung und PR-Merging
- ✅ **Auto-Documentation** - Changelog wird automatisch aktualisiert
- ✅ **Quality Gates** - Formatierung, Linting, Tests müssen bestehen

## 🚀 Quick Start (5 Minuten)

```bash
# 1. Labels erstellen
gh label sync --file .github/labels.yml

# 2. Jules aktivieren (wähle eine Option):

# Option A: Jules GitHub App (Empfohlen - Einfachste Lösung)
open https://github.com/apps/jules
# → Installiere die App für dein Repository
# → Fertig! Jules überwacht automatisch jules-task Issues

# Option B: Jules API + GitHub Actions
# → Siehe detaillierte Anleitung: .github/JULES_API_SETUP.md
# → Generiere API-Key bei https://jules.google.com
# → Füge als Secret hinzu: gh secret set JULES_API_KEY

# 3. Alle Jules Development Issues erstellen
gh workflow run CI-03_create-issues.yml

# 4. Status prüfen
gh run watch

# 5. Issues anzeigen
gh issue list --label "jules-task"
```

**Das war's!** Jules Sessions werden jetzt automatisch erstellt, PRs werden automatisch gemerged.

**Neu:** Der Workflow `CI-04_session-trigger.yml` triggert automatisch Jules API Sessions wenn Issues mit `jules-task` Label erstellt/gelabelt werden. 🎉

## 📁 Datei-Struktur

```
.github/
├── workflows/
│   ├── CI-01_build-and-test.yml              # Haupt-CI/CD Pipeline
│   ├── CI-02_security-scan.yml               # Security Scanning
│   ├── CI-03_create-issues.yml               # Jules Issues erstellen (einmalig)
│   ├── CI-04_session-trigger.yml             # Jules API Session Trigger
│   ├── CI-05_pr-automation.yml               # ✨ Auto-Merge mit Fehlerbehandlung
│   ├── CI-06_update-changelog.yml            # Changelog Updates
│   ├── CI-07_post-merge-automation.yml       # 🆕 Post-Merge (Issue close, Roadmap, Next)
│   ├── CI-08_monitor-jules-session.yml       # 🆕 Jules Session Monitoring & PR Creation
│   ├── CI-ADMIN-01_sync-labels.yml           # Label Synchronisierung
│   └── README.md                             # Workflow Dokumentation
├── ISSUE_TEMPLATE/
│   ├── development_task.yml        # Template für Jules Tasks
│   ├── bug_report.yml              # Bug Reports
│   └── feature_request.yml         # Feature Requests
├── labels.yml                       # Label Konfiguration
├── pull_request_template.md        # PR Template
├── JULES_INTEGRATION.md            # Detaillierte Jules Doku
└── SETUP_GUIDE.md                  # Setup-Anleitung
```

## 🔄 Workflow-Ablauf

### Vollautomatischer Jules Workflow:

```
1. Issue mit jules-task Label erstellt/gelabelt
    ↓
2. CI-04_session-trigger.yml triggert automatisch (oder manuell)
    ↓
3. Jules API Session wird erstellt (wenn Key vorhanden)
    ↓
4. CI-08_monitor-jules-session.yml überwacht Session (alle 5 Min.)
    ↓
5. Jules bearbeitet Issue & erstellt Branch
    ↓
6. CI-08 erkennt fertige Session & erstellt PR mit jules-pr Label
    ↓
7. CI/CD Pipeline (CI-01_build-and-test.yml) läuft automatisch
    ↓
8a. Bei SUCCESS: CI-05_pr-automation.yml merged PR automatisch
    ↓
8b. Bei FEHLER: CI-05 erstellt @jules Kommentar mit Fehlerdetails
    ↓
9. Bei SUCCESS: CI-07_post-merge-automation.yml:
    - Schließt Issue automatisch
    - Aktualisiert ROADMAP.md
    - Triggert CI-04 für nächstes jules-task Issue
    ↓
10. CI-06_update-changelog.yml: CHANGELOG.md wird aktualisiert
    ↓
11. Zyklus wiederholt sich automatisch für nächstes Issue
```

**✨ Vollständig automatisiert!** Der komplette Workflow läuft ohne manuelle Eingriffe:
- Automatische Session-Erstellung bei neuen jules-task Issues
- Kontinuierliche Überwachung laufender Sessions
- Automatische PR-Erstellung bei Session-Completion
- Intelligente Fehlerbehandlung mit @jules Benachrichtigungen
- Auto-Merge bei erfolgreichen Checks
- Automatische Roadmap-Updates und Issue-Schließung
- Selbst-triggernde Fortsetzung mit nächstem Issue

### CI/CD Pipeline (bei jedem PR):

```
Push/PR zu main
    ↓
Code Quality (fmt, clippy)
    ↓
Build & Test (Linux, macOS, Windows)
    ↓
Security Audit
    ↓
Success Gate
    ↓
Bereit zum Merge
```

## 🎬 Workflows im Detail

### 1. CI/CD Pipeline (`CI-01_build-and-test.yml`)
- **Trigger:** Push/PR zu main
- **Was:** Baut und testet auf allen Plattformen
- **Dauer:** ~10-15 Minuten

### 2. Jules Issues Creation (`CI-03_create-issues.yml`)
- **Trigger:** Manuell (einmalig)
- **Was:** Erstellt 8 vordefinierte Development Issues
- **Dauer:** ~1 Minute

### 3. Jules Session Trigger (`CI-04_session-trigger.yml`)
- **Trigger:** Automatisch bei Issues mit `jules-task` Label oder manuell
- **Was:** Erstellt Jules Sessions für Issues
- **Features:**
  - Automatische Erkennung neuer jules-task Issues
  - Wählt ältestes offenes Issue bei manueller Ausführung
  - Tracking-Kommentare im Issue
  - API-Integration (wenn JULES_API_KEY vorhanden)
- **Dauer:** Sekunden

### 4. Jules Auto-Merge (`CI-05_pr-automation.yml`) ✨ Enhanced
- **Trigger:** Bei Jules PRs automatisch, nach CI-Completion
- **Was:** Merged PRs wenn alle Checks bestehen, mit Fehlerbehandlung
- **Features:**
  - Wartet auf alle Checks
  - Bei Erfolg: Auto-Merge mit Squash
  - Bei Fehler: Erstellt detaillierten @jules Kommentar
  - Erkennt Merge-Konflikte
  - Intelligente Fehleranalyse
- **Dauer:** Sekunden

### 5. Documentation Update (`CI-06_update-changelog.yml`)
- **Trigger:** Bei Merge in main
- **Was:** Updates CHANGELOG.md
- **Dauer:** Sekunden

### 6. Security Scan (`CI-02_security-scan.yml`)
- **Trigger:** Push/PR + wöchentlich
- **Was:** CodeQL Security Analysis
- **Dauer:** ~5-10 Minuten

### 7. Post-Merge Automation (`CI-07_post-merge-automation.yml`) 🆕
- **Trigger:** Automatisch nach Jules PR Merge
- **Was:** Schließt Issue, updated Roadmap, triggert nächste Session
- **Features:**
  - Automatisches Issue-Schließen
  - ROADMAP.md Update mit Completion-Status
  - Triggert CI-04 für nächstes jules-task Issue
  - Erstellt Erfolgs-Kommentare
- **Dauer:** Sekunden

### 8. Monitor Jules Session (`CI-08_monitor-jules-session.yml`) 🆕
- **Trigger:** Scheduled (alle 5 Minuten) oder manuell
- **Was:** Überwacht aktive Jules Sessions und erstellt PRs
- **Features:**
  - Findet aktive Sessions aus Issue-Kommentaren
  - Pollt Jules API für Session-Status
  - Erstellt automatisch PR bei Completion
  - Benachrichtigt bei Fehler oder Completion
  - Fügt jules-pr Label hinzu
- **Dauer:** Sekunden

## 📊 Monitoring

### Dashboard Commands

```bash
# Issues
gh issue list --label "jules-task"              # Alle Jules Tasks
gh issue list --label "priority: critical"      # Kritische Issues

# PRs
gh pr list --label "jules-pr"                   # Alle Jules PRs
gh pr view <number> --json statusCheckRollup    # PR Status

# Workflows
gh run list --workflow="CI/CD Pipeline"         # CI Runs
gh run watch                                     # Aktuellen Run beobachten
```

### Status Badges

Füge diese zu README.md hinzu:

```markdown
![CI/CD](https://github.com/MrLongNight/VjMapper/actions/workflows/CI-01_build-and-test.yml/badge.svg)
![Security](https://github.com/MrLongNight/VjMapper/actions/workflows/CI-02_security-scan.yml/badge.svg)
```

## 🔐 Sicherheit

### Was ist abgesichert?

- ✅ **Minimal Permissions** - Workflows haben nur benötigte Rechte
- ✅ **No Command Injection** - Alle Inputs sind escaped
- ✅ **Safe Auto-Merge** - Nur bei bestandenen Checks
- ✅ **Dependency Scanning** - Cargo audit läuft regelmäßig
- ✅ **CodeQL Analysis** - Wöchentliche Security Scans

### Security Best Practices

1. **Nie Secrets committen**
2. **Branch Protection aktiviert**
3. **Required Checks konfiguriert**
4. **Auto-Merge nur für Jules PRs**
5. **Regelmäßige Reviews von merged PRs**

## 🛠️ Troubleshooting

### Problem: CI schlägt fehl

```bash
# Lokal reproduzieren
cargo fmt --check
cargo clippy
cargo test

# Logs prüfen
gh run view <run-id> --log
```

### Problem: Jules Session wird nicht automatisch erstellt

**Checklist:**
- [ ] Issue hat `jules-task` Label?
- [ ] Workflow `CI-04_session-trigger.yml` existiert?
- [ ] JULES_API_KEY konfiguriert?

```bash
# Debug
# Check ob Workflow getriggert wurde
gh run list --workflow="Jules Session Trigger" --limit 5

# Check Workflow-Logs
gh run view --log

# Check Issue-Kommentare
gh issue view <issue-number> --comments

# Manuel triggern
gh workflow run CI-04_session-trigger.yml -f issue_number=<issue-number>
```

**Lösungen:**
1. **Kein Workflow-Run:**
   - Issue braucht `jules-task` Label
   - Workflow-Datei muss in main branch sein

2. **Workflow läuft, aber keine Session:**
   - Konfiguriere JULES_API_KEY Secret
   - Siehe: `.github/JULES_API_SETUP.md`

3. **API-Key fehlt:**
   ```bash
   # API-Key hinzufügen
   gh secret set JULES_API_KEY
   # Key von https://jules.google.com (Settings → API-Keys)
   ```

### Problem: Jules Session läuft, aber kein PR wird erstellt

**Checklist:**
- [ ] CI-08 (Monitor) läuft alle 5 Minuten?
- [ ] Session ist wirklich abgeschlossen?
- [ ] Jules hat einen Branch erstellt?

```bash
# Check Monitoring Workflow
gh run list --workflow="Monitor Jules Session" --limit 5

# Check Logs
gh run view --log

# Manuel triggern
gh workflow run CI-08_monitor-jules-session.yml
```

**Lösungen:**
1. **Monitoring läuft nicht:**
   - Stelle sicher dass CI-08 in main branch ist
   - GitHub Actions müssen aktiviert sein

2. **Session noch nicht fertig:**
   - Warten - CI-08 prüft alle 5 Minuten
   - Jules Sessions können mehrere Minuten dauern

3. **Kein Branch gefunden:**
   - PR muss manuell erstellt werden
   - Check Jules Session UI für Branch-Name

### Problem: Auto-Merge funktioniert nicht

**Checklist:**
- [ ] PR hat `jules-pr` Label?
- [ ] Alle Checks sind grün?
- [ ] Keine Merge Konflikte?
- [ ] Kein Draft?
- [ ] CI-05 wurde getriggert?

```bash
# Debug
gh pr view <number> --json mergeable,statusCheckRollup

# Check Auto-Merge Workflow
gh run list --workflow="PR Auto-Merge" --limit 5

# Check für @jules Kommentare (bei Fehler)
gh pr view <number> --comments
```

**Bei Fehler:**
- CI-05 erstellt automatisch einen @jules Kommentar mit Details
- Jules kann PR direkt updaten
- Checks laufen automatisch erneut

### Problem: Build-Dependencies fehlen

**Linux:**
```bash
sudo apt-get install -y \
  pkg-config libfontconfig1-dev libfreetype6-dev \
  libasound2-dev libxcb1-dev libavcodec-dev \
  libavformat-dev libavutil-dev libswscale-dev
```

**macOS:**
```bash
brew install ffmpeg pkg-config
```

## 📚 Dokumentation

- **[Implementation Summary](.github/IMPLEMENTATION_SUMMARY.md)** - 🆕 Vollständige Implementierungs-Übersicht
- **[Complete Automation](.github/JULES_AUTOMATION_COMPLETE.md)** - 🆕 Detaillierter Workflow mit Diagrammen
- **[Quick Reference](.github/WORKFLOW_QUICKREF.md)** - 🆕 Schnellreferenz & Commands
- **[Workflows README](.github/workflows/README.md)** - Workflow Details
- **[Jules API Setup](.github/JULES_API_SETUP.md)** - Detaillierte Jules Setup-Anleitung
- **[Setup Guide](.github/SETUP_GUIDE.md)** - Schritt-für-Schritt Anleitung
- **[Jules Integration](.github/JULES_INTEGRATION.md)** - Jules Konfiguration & Workflows
- **[Issue Templates](.github/ISSUE_TEMPLATE/)** - Templates für Issues

## 💡 Tipps & Tricks

### Für Entwickler

1. **Lokale Pre-Commit Checks:**
   ```bash
   cargo fmt && cargo clippy && cargo test
   ```

2. **Watch Mode während Entwicklung:**
   ```bash
   cargo watch -x check -x test
   ```

3. **Schneller Build:**
   ```bash
   cargo build --release --jobs=$(nproc)
   ```

### Für Jules

1. **Immer PR Template verwenden**
2. **Related Issue verlinken:** `Closes #123`
3. **Tests lokal ausführen vor PR**
4. **Clear commit messages**

### Für Projekt-Manager

1. **Weekly Review:**
   ```bash
   gh issue list --label "jules-task" --state closed --limit 10
   ```

2. **Progress Tracking:**
   ```bash
   gh issue list --label "jules-task" --json title,state | jq
   ```

3. **CI Health:**
   ```bash
   gh run list --workflow="CI/CD Pipeline" --limit 10
   ```

## 🎯 Erfolgs-Metriken

### Aktuelle Ziele

- ✅ **CI Success Rate:** >95%
- ✅ **Auto-Merge Rate:** >90% (für Jules PRs)
- ✅ **Average Merge Time:** <30 Minuten
- ✅ **Security Alerts:** 0 critical
- ✅ **Test Coverage:** >80%

### Tracking

```bash
# CI Success Rate
gh run list --workflow="CI/CD Pipeline" --limit 20 \
  | grep -c "completed" | xargs -I {} echo "Total: {}"

# Auto-Merged PRs
gh pr list --state closed --label "jules-pr" --limit 20 \
  | grep -c "Merged" | xargs -I {} echo "Auto-merged: {}"
```

## 🚦 Status Indicators

| Component | Status | Notes |
|-----------|--------|-------|
| CI/CD Pipeline | ✅ Ready | Multi-platform builds |
| Security Scan | ✅ Ready | CodeQL + audit |
| Jules Integration | ✅ Ready | Auto-merge configured |
| Documentation | ✅ Ready | Auto-updates |
| Label System | ✅ Ready | Synced |

## 🔗 Links

- **Repository:** https://github.com/MrLongNight/VjMapper
- **Actions:** https://github.com/MrLongNight/VjMapper/actions
- **Issues:** https://github.com/MrLongNight/VjMapper/issues?q=label%3Ajules-task
- **PRs:** https://github.com/MrLongNight/VjMapper/pulls?q=label%3Ajules-pr

## 🆘 Hilfe

1. **Dokumentation lesen** (siehe oben)
2. **Workflow Logs prüfen**
3. **Issue öffnen** mit Label `workflows`
4. **Kontakt:** @MrLongNight

---

**Version:** 1.0  
**Status:** ✅ Produktionsbereit  
**Letztes Update:** 2024-12-04

**Nächster Schritt:** `gh workflow run CI-03_create-issues.yml` 🚀
