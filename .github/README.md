# GitHub Actions & CI/CD - Dokumentationsübersicht

> **Zentrale Anlaufstelle für alle CI/CD und Jules Integration Dokumentation**

## 🎯 Quick Links

| Dokument | Zweck | Für wen? |
|----------|-------|----------|
| [SETUP_GUIDE.md](SETUP_GUIDE.md) | **5-Minuten Setup** | ⭐ Start hier |
| [WORKFLOW_CONTROL.md](WORKFLOW_CONTROL.md) | **Workflows steuern** | Alle Nutzer |
| [JULES_ISSUES_EXPLANATION.md](JULES_ISSUES_EXPLANATION.md) | **Jules Prozess** | Jules Nutzer |
| [JULES_INTEGRATION.md](JULES_INTEGRATION.md) | **Jules Config** | Admins |
| [workflows/README.md](workflows/README.md) | **Workflow Details** | Entwickler |
| [FEEDBACK_ADDRESSED.md](FEEDBACK_ADDRESSED.md) | **Changelog** | Info |

## 🚀 Schnellstart

### 1. Jules Issues erstellen (einmalig)
```bash
# Labels synchronisieren
gh label sync --file .github/labels.yml

# Issues erstellen
gh workflow run CI-03_create-issues.yml

# Status prüfen
gh run watch
```

### 2. CI/CD nutzen
```bash
# Standard (alle Plattformen + Tests)
gh workflow run "CI/CD"

# Schnell (nur Linux)
gh workflow run "CI/CD" -f skip_platforms=true

# Sehr schnell (Linux, keine Tests)
gh workflow run "CI/CD" -f skip_platforms=true -f skip_tests=true
```

### 3. Workflows kontrollieren
```bash
# Auto-Merge deaktivieren
# Editiere: .github/workflows/CI-05_pr-automation.yml
# Setze: AUTO_MERGE_ENABLED: false

# CodeQL für PRs deaktivieren
# Editiere: .github/workflows/CI-02_security-scan.yml
# Setze: SCAN_ON_PR_ENABLED: false
```

## 📚 Dokumentationsstruktur

```
.github/
├── README.md                          # ← Diese Datei (Übersicht)
│
├── SETUP_GUIDE.md                     # 5-Min Setup, Quick Start
├── WORKFLOW_CONTROL.md                # Workflows ein-/ausschalten
├── JULES_ISSUES_EXPLANATION.md        # Warum Jules Issues manuell
├── JULES_INTEGRATION.md               # Jules API Konfiguration
├── FEEDBACK_ADDRESSED.md              # Was wurde umgesetzt
│
├── workflows/
│   ├── README.md                      # Technische Workflow-Details
│   ├── CI-01_build-and-test.yml                 # CI/CD (6 Jobs)
│   ├── CI-02_security-scan.yml                     # Security Scan
│   ├── CI-03_create-issues.yml        # Issues erstellen
│   ├── CI-05_pr-automation.yml        # Auto-Merge
│   ├── CI-06_update-changelog.yml       # CHANGELOG
│   └── CI-ADMIN-01_sync-labels.yml                # Labels sync
│
├── ISSUE_TEMPLATE/
│   ├── development_task.yml           # Jules Tasks
│   ├── bug_report.yml                 # Bugs
│   └── feature_request.yml            # Features
│
├── labels.yml                         # Label Config
├── workflows.config.yml               # Workflow Config
└── pull_request_template.md           # PR Template
```

## 🎓 Für verschiedene Nutzergruppen

### 🆕 Erste Schritte
1. Lies: [SETUP_GUIDE.md](SETUP_GUIDE.md)
2. Erstelle Jules Issues: `gh workflow run CI-03_create-issues.yml`
3. Fertig! Jules kann loslegen

### 👨‍💻 Entwickler
1. Lies: [workflows/README.md](workflows/README.md)
2. Verstehe: [WORKFLOW_CONTROL.md](WORKFLOW_CONTROL.md)
3. Nutze: CI/CD Optionen für schnellere Entwicklung

### 🤖 Jules Nutzer
1. Lies: [JULES_ISSUES_EXPLANATION.md](JULES_ISSUES_EXPLANATION.md)
2. Verstehe: Warum Issues manuell erstellt werden
3. Konfiguriere: Jules API (siehe [JULES_INTEGRATION.md](JULES_INTEGRATION.md))

### 👔 Admins
1. Lies: [JULES_INTEGRATION.md](JULES_INTEGRATION.md)
2. Konfiguriere: Branch Protection Rules
3. Steuere: Workflows mit [WORKFLOW_CONTROL.md](WORKFLOW_CONTROL.md)

## ❓ Häufige Fragen

### Warum wurden Jules Issues nicht automatisch erstellt?
**Antwort:** Absichtlich! Manual dispatch (`workflow_dispatch`) gibt dir Kontrolle. Siehe [JULES_ISSUES_EXPLANATION.md](JULES_ISSUES_EXPLANATION.md)

### Warum so viele CI Checks?
**Antwort:** Multi-Platform Support (Linux/macOS/Windows) + Quality Gates + Security. Alle sinnvoll und reduzierbar. Siehe [WORKFLOW_CONTROL.md](WORKFLOW_CONTROL.md)

### Kann ich Workflows deaktivieren?
**Antwort:** Ja! Mehrere Methoden verfügbar. Siehe [WORKFLOW_CONTROL.md](WORKFLOW_CONTROL.md)

### Warum schlagen Checks fehl?
**Antwort:** Checks sind nicht fehlerhaft - sie laufen nur auf `main` oder bei PR zu `main`. Dieser Branch ist noch nicht gemerged. Nach Merge: alle Checks laufen.

### Wie erstelle ich Jules Issues?
**Antwort:** `gh workflow run CI-03_create-issues.yml` - Siehe [JULES_ISSUES_EXPLANATION.md](JULES_ISSUES_EXPLANATION.md)

## 🔧 Workflows im Überblick

### CI/CD (CI-01_build-and-test.yml)
**Wann:** Push/PR zu main, Manual  
**Dauer:** ~15 min (Standard), ~5 min (nur Linux)  
**Jobs:** 6 (Quality, 3×Build, Security, Gate)  
**Optionen:**
- `skip_platforms: true` - Nur Linux
- `skip_tests: true` - Keine Tests

### CodeQL Security Scan (CI-02_security-scan.yml)
**Wann:** Push/PR zu main, Wöchentlich, Manual  
**Dauer:** ~10 min  
**Jobs:** 1 (Security Analysis)  
**Kontrolle:** `SCAN_ON_PR_ENABLED: false` für PRs deaktivieren

### Jules Issues Creation (CI-03_create-issues.yml)
**Wann:** Manual only  
**Dauer:** <1 min  
**Jobs:** 1 (Erstellt 8 Issues)  
**Verwendung:** `gh workflow run CI-03_create-issues.yml`

### Jules Auto-Merge (CI-05_pr-automation.yml)
**Wann:** Bei Jules PRs automatisch  
**Dauer:** <1 min  
**Jobs:** 1 (Merged wenn Checks ✅)  
**Kontrolle:** `AUTO_MERGE_ENABLED: false` zum Deaktivieren

### Update Documentation (CI-06_update-changelog.yml)
**Wann:** Nach Merge in main  
**Dauer:** <1 min  
**Jobs:** 1 (CHANGELOG Update)  
**Kontrolle:** Nicht deaktivierbar (läuft selten)

### Sync Labels (CI-ADMIN-01_sync-labels.yml)
**Wann:** Bei Änderungen an labels.yml  
**Dauer:** <1 min  
**Jobs:** 1 (Label Sync)  
**Kontrolle:** Nicht nötig (läuft sehr selten)

## 📊 Ressourcen-Übersicht

### Workflow-Laufzeit (Standard)
- **CI/CD:** ~15 min (alle Plattformen)
- **CodeQL:** ~10 min
- **Andere:** <2 min kombiniert

**Total bei PR:** ~25 min

### Reduzierte Laufzeit (Optionen)
- **CI/CD:** ~5 min (nur Linux, mit Tests)
- **CI/CD:** ~3 min (nur Linux, ohne Tests)
- **CodeQL:** Deaktiviert für PRs

**Total bei PR:** ~5-8 min

## 🎯 Best Practices

### Für schnelle Entwicklung
```bash
# Lokal testen
cargo fmt && cargo clippy && cargo test

# CI nur auf Linux
gh workflow run "CI/CD" -f skip_platforms=true
```

### Für Production-Ready
```bash
# Alle Checks laufen lassen
# Nichts deaktivieren
# Full CI/CD Pipeline
```

### Für Jules Integration
```bash
# 1. Issues einmalig erstellen
gh workflow run CI-03_create-issues.yml

# 2. Jules konfigurieren
# 3. Auto-Merge aktiv lassen
# 4. PRs werden automatisch gemerged
```

## 🆘 Hilfe & Support

### Bei Problemen:
1. **Dokumentation prüfen** (siehe oben)
2. **Workflow Logs ansehen** (GitHub Actions Tab)
3. **Issue öffnen** mit Label `workflows`
4. **Kontakt:** @MrLongNight

### Debug-Commands:
```bash
# Workflow Status
gh run list --workflow="CI/CD"

# Specific Run
gh run view <run-id> --log

# Issues anzeigen
gh issue list --label "jules-task"

# PRs anzeigen
gh pr list --label "jules-pr"
```

## 📦 Templates & Konfiguration

### Issue Templates
- `development_task.yml` - Für Jules Development Tasks
- `bug_report.yml` - Für Bug Reports
- `feature_request.yml` - Für Feature Requests

### PR Template
- `pull_request_template.md` - Vollständiges PR Template

### Konfiguration
- `labels.yml` - 40+ Label Definitionen
- `workflows.config.yml` - Workflow Konfiguration (informativ)

## 🔗 Externe Links

- **GitHub Actions Docs:** https://docs.github.com/en/actions
- **Rust CI/CD Best Practices:** https://rust-lang.github.io/api-guidelines/
- **CodeQL Documentation:** https://codeql.github.com/docs/

## ✅ Status

| Komponente | Status | Version |
|------------|--------|---------|
| CI/CD Pipeline | ✅ Produktionsbereit | 1.0 |
| Jules Integration | ✅ Produktionsbereit | 1.0 |
| Dokumentation | ✅ Vollständig | 1.0 |
| Workflow Control | ✅ Implementiert | 1.0 |

---

**Letztes Update:** 2024-12-04  
**Maintainer:** VjMapper Team  
**Status:** ✅ Produktionsbereit

**Nächster Schritt:** Lies [SETUP_GUIDE.md](SETUP_GUIDE.md) für 5-Minuten Setup! 🚀
