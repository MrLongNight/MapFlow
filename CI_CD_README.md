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
# → Generiere API Key bei https://jules.google.com
# → Füge als Secret hinzu: gh secret set JULES_API_KEY

# 3. Alle Jules Development Issues erstellen
gh workflow run create-jules-issues.yml

# 4. Status prüfen
gh run watch

# 5. Issues anzeigen
gh issue list --label "jules-task"
```

**Das war's!** Jules Sessions werden jetzt automatisch erstellt, PRs werden automatisch gemerged.

**Neu:** Der Workflow `jules-session-trigger.yml` triggert automatisch Jules API Sessions wenn Issues mit `jules-task` Label erstellt/gelabelt werden. 🎉

## 📁 Datei-Struktur

```
.github/
├── workflows/
│   ├── Build_Rust.yml              # Haupt-CI/CD Pipeline
│   ├── codeql.yml                  # Security Scanning
│   ├── create-jules-issues.yml     # Jules Issues erstellen (einmalig)
│   ├── jules-session-trigger.yml   # NEU: Triggert Jules API Sessions
│   ├── jules-pr-automation.yml     # Auto-Merge für Jules PRs
│   ├── update-documentation.yml    # Changelog Updates
│   ├── sync-labels.yml             # Label Synchronisierung
│   └── README.md                   # Workflow Dokumentation
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
Issue mit jules-task Label erstellt/gelabelt
    ↓
jules-session-trigger.yml triggert automatisch
    ↓
Jules API Session wird erstellt (wenn Key vorhanden)
    ↓
Jules bearbeitet Issue
    ↓
Jules erstellt PR mit jules-pr Label
    ↓
CI/CD Pipeline (Build_Rust.yml) läuft automatisch
    ↓
jules-pr-automation.yml: Auto-Merge wenn alle Checks ✅
    ↓
Issue wird automatisch geschlossen
    ↓
update-documentation.yml: CHANGELOG.md wird aktualisiert
```

**Neu:** Der Workflow ist jetzt vollständig automatisiert! Sobald ein Issue das `jules-task` Label erhält, wird automatisch eine Jules Session getriggert.

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

### 1. CI/CD Pipeline (`Build_Rust.yml`)
- **Trigger:** Push/PR zu main
- **Was:** Baut und testet auf allen Plattformen
- **Dauer:** ~10-15 Minuten

### 2. Jules Issues Creation (`create-jules-issues.yml`)
- **Trigger:** Manuell (einmalig)
- **Was:** Erstellt 8 vordefinierte Development Issues
- **Dauer:** ~1 Minute

### 3. Jules Session Trigger (`jules-session-trigger.yml`) 🆕
- **Trigger:** Automatisch bei Issues mit `jules-task` Label oder manuell
- **Was:** Erstellt Jules API Sessions für Issues
- **Features:**
  - Automatische Erkennung neuer jules-task Issues
  - Tracking-Kommentare im Issue
  - API-Integration (wenn JULES_API_KEY vorhanden)
  - Batch-Processing aller offenen Issues
- **Dauer:** Sekunden

### 4. Jules Auto-Merge (`jules-pr-automation.yml`)
- **Trigger:** Bei Jules PRs automatisch
- **Was:** Merged PRs wenn alle Checks bestehen
- **Dauer:** Sekunden

### 5. Documentation Update (`update-documentation.yml`)
- **Trigger:** Bei Merge in main
- **Was:** Updates CHANGELOG.md
- **Dauer:** Sekunden

### 6. Security Scan (`codeql.yml`)
- **Trigger:** Push/PR + wöchentlich
- **Was:** CodeQL Security Analysis
- **Dauer:** ~5-10 Minuten

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
![CI/CD](https://github.com/MrLongNight/VjMapper/actions/workflows/Build_Rust.yml/badge.svg)
![Security](https://github.com/MrLongNight/VjMapper/actions/workflows/codeql.yml/badge.svg)
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
- [ ] Workflow `jules-session-trigger.yml` existiert?
- [ ] Jules GitHub App installiert ODER JULES_API_KEY konfiguriert?

```bash
# Debug
# Check ob Workflow getriggert wurde
gh run list --workflow="Jules Session Trigger" --limit 5

# Check Workflow-Logs
gh run view --log

# Check Issue-Kommentare
gh issue view <issue-number> --comments

# Manuel triggern
gh workflow run jules-session-trigger.yml -f issue_number=<issue-number>
```

**Lösungen:**
1. **Kein Workflow-Run:**
   - Issue braucht `jules-task` Label
   - Workflow-Datei muss in main branch sein

2. **Workflow läuft, aber keine Session:**
   - Option A: Installiere Jules GitHub App (empfohlen)
   - Option B: Konfiguriere JULES_API_KEY Secret
   - Siehe: `.github/JULES_API_SETUP.md`

3. **API Key fehlt:**
   ```bash
   # API Key hinzufügen
   gh secret set JULES_API_KEY
   # Key von https://jules.google.com (Settings → API Keys)
   ```

### Problem: Auto-Merge funktioniert nicht

**Checklist:**
- [ ] PR hat `jules-pr` Label?
- [ ] Alle Checks sind grün?
- [ ] Keine Merge Konflikte?
- [ ] Kein Draft?

```bash
# Debug
gh pr view <number> --json mergeable,statusCheckRollup
```

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

- **[Jules API Setup](.github/JULES_API_SETUP.md)** - 🆕 Detaillierte Jules Setup-Anleitung
- **[Setup Guide](.github/SETUP_GUIDE.md)** - Schritt-für-Schritt Anleitung
- **[Jules Integration](.github/JULES_INTEGRATION.md)** - Jules Konfiguration & Workflows
- **[Workflows README](.github/workflows/README.md)** - Workflow Details
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

**Nächster Schritt:** `gh workflow run create-jules-issues.yml` 🚀
