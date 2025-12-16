# GitHub Actions & CI/CD - Dokumentationsübersicht

> **Zentrale Anlaufstelle für alle CI/CD und Jules Integration Dokumentation**

## 🎮 Master-Switch für Automation

Die gesamte Jules-Automatisierung kann **mit einer einzigen Variable** aktiviert/deaktiviert werden:

```yaml
# In jedem Workflow unter 'env:':
JULES_AUTOMATION_ENABLED: true   # Auf 'false' setzen zum Deaktivieren
```

**Workflows mit Master-Switch:**
- `CI-04_session-trigger.yml` - Jules Session starten
- `CI-05_pr-automation.yml` - Auto-Merge
- `CI-07_post-merge-automation.yml` - Post-Merge Aktionen
- `CI-08_monitor-jules-session.yml` - Session Monitoring

## 🎯 Quick Links

| Dokument | Zweck | Für wen? |
|----------|-------|----------|
| [SETUP_GUIDE.md](SETUP_GUIDE.md) | **5-Minuten Setup** | ⭐ Start hier |
| [WORKFLOW_CONTROL.md](WORKFLOW_CONTROL.md) | **Workflows steuern** | Alle Nutzer |
| [WORKFLOW_QUICKREF.md](WORKFLOW_QUICKREF.md) | **Schnellreferenz** | Alle Nutzer |
| [JULES_INTEGRATION.md](JULES_INTEGRATION.md) | **Jules Config** | Admins |
| [workflows/README.md](workflows/README.md) | **Workflow Details** | Entwickler |
| [automation-config.yml](automation-config.yml) | **Zentrale Konfiguration** | Admins |

## 🚀 Schnellstart

### 1. Automatische Issue-Verarbeitung
Alle Issues (Bug Reports, Feature Requests, Development Tasks) werden **automatisch** von Jules bearbeitet:

1. User erstellt Issue via Template → bekommt automatisch `jules-task` Label
2. CI-04 triggert Jules Session
3. CI-08 überwacht Session (1-Min Intervall, nur wenn aktiv)
4. Bei Fertigstellung: PR wird erstellt
5. CI-01 führt Tests durch
6. CI-05 merged automatisch (wenn alle ✅)
7. CI-07 schließt Issue und startet nächste Session

### 2. CI/CD nutzen
```bash
# Standard (alle Plattformen + Tests)
gh workflow run "CI-01:Build&Test"

# Schnell (nur Linux)
gh workflow run "CI-01:Build&Test" -f skip_platforms=true

# Sehr schnell (Linux, keine Tests)
gh workflow run "CI-01:Build&Test" -f skip_platforms=true -f skip_tests=true
```

### 3. Automation steuern
```bash
# ALLES deaktivieren: In CI-04, CI-05, CI-07, CI-08:
# Setze: JULES_AUTOMATION_ENABLED: false

# Nur Auto-Merge deaktivieren:
# In CI-05: Setze AUTO_MERGE_ENABLED: false

# CodeQL für PRs deaktivieren:
# In CI-02: Setze SCAN_ON_PR_ENABLED: false
```

## 📚 Dokumentationsstruktur

```
.github/
├── README_CICD.md                     # ← Diese Datei (Übersicht)
├── automation-config.yml              # Zentrale Konfiguration (informativ)
│
├── SETUP_GUIDE.md                     # 5-Min Setup, Quick Start
├── WORKFLOW_CONTROL.md                # Workflows ein-/ausschalten
├── WORKFLOW_QUICKREF.md               # Schnellreferenz
├── JULES_ISSUES_EXPLANATION.md        # Warum Jules Issues manuell
├── JULES_INTEGRATION.md               # Jules API Konfiguration
│
├── workflows/
│   ├── README.md                      # Technische Workflow-Details
│   ├── CI-01_build-and-test.yml       # CI/CD (Build & Test)
│   ├── CI-02_security-scan.yml        # Security Scan (CodeQL)
│   ├── CI-04_session-trigger.yml      # Jules Session starten
│   ├── CI-05_pr-automation.yml        # Auto-Merge
│   ├── CI-06_update-changelog.yml     # CHANGELOG Update
│   ├── CI-07_post-merge-automation.yml # Post-Merge Tasks
│   ├── CI-08_monitor-jules-session.yml # Session Monitoring (on-demand)
│   ├── CI-09B_create-release.yml      # Release erstellen
│   └── CI-ADMIN-01_sync-labels.yml    # Labels sync
│
├── ISSUE_TEMPLATE/
│   ├── development_task.yml           # Jules Tasks (auto: jules-task)
│   ├── bug_report.yml                 # Bugs (auto: jules-task)
│   └── feature_request.yml            # Features (auto: jules-task)
│
├── labels.yml                         # Label Config
└── PULL_REQUEST_TEMPLATE.md           # PR Template
```

## 🔄 Vollautomatischer Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│              VOLLAUTOMATISCHER WORKFLOW (v2.0)                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  👤 User erstellt Issue (Bug/Feature/Task)                      │
│           ↓ (automatisches jules-task Label)                    │
│  🤖 CI-04: Jules Session wird erstellt                          │
│           ↓ (triggert CI-08)                                    │
│  ⏱️  CI-08: Polling alle 60 Sekunden (nur während Session aktiv)│
│           ↓ (bei Fertigstellung)                                │
│  📝 PR wird automatisch erstellt                                 │
│           ↓                                                      │
│  🧪 CI-01: Build & Test auf allen Plattformen                   │
│           ↓                                                      │
│  🔒 CI-02: Security Scan                                        │
│           ↓                                                      │
│  ✅ CI-05: Auto-Merge wenn alle Checks grün                     │
│           ↓                                                      │
│  📋 CI-06: CHANGELOG Update                                      │
│           ↓                                                      │
│  🏁 CI-07: Issue schließen + ROADMAP + Next Session triggern    │
│           ↓                                                      │
│  🔄 Nächstes jules-task Issue wird verarbeitet                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 🎓 Für verschiedene Nutzergruppen

### 🆕 Erste Schritte
1. Lies: [SETUP_GUIDE.md](SETUP_GUIDE.md)
2. Erstelle ein Issue via Template (Bug/Feature/Task)
3. Fertig! Jules bearbeitet es automatisch

### 👔 Admins (Automation kontrollieren)
1. **Alles deaktivieren**: `JULES_AUTOMATION_ENABLED: false` in CI-04, CI-05, CI-07, CI-08
2. **Nur Auto-Merge**: `AUTO_MERGE_ENABLED: false` in CI-05
3. Lies: [automation-config.yml](automation-config.yml) für Details

### 👨‍💻 Entwickler
1. Lies: [workflows/README.md](workflows/README.md)
2. Verstehe: [WORKFLOW_CONTROL.md](WORKFLOW_CONTROL.md)
3. Nutze: CI/CD Optionen für schnellere Entwicklung

## ❓ Häufige Fragen

### Wie deaktiviere ich die komplette Automation?
**Antwort:** Setze `JULES_AUTOMATION_ENABLED: false` in den Workflow-Dateien CI-04, CI-05, CI-07, CI-08.

### Warum so viele CI Checks?
**Antwort:** Multi-Platform Support (Linux/macOS/Windows) + Quality Gates + Security. Alle sinnvoll und reduzierbar. Siehe [WORKFLOW_CONTROL.md](WORKFLOW_CONTROL.md)

### Warum läuft CI-08 nicht alle 5 Minuten?
**Antwort:** CI-08 wurde optimiert - es läuft nur on-demand, wenn CI-04 eine Session erstellt, und pollt dann alle 60 Sekunden.

### Werden alle Issues automatisch bearbeitet?
**Antwort:** Ja! Bug Reports, Feature Requests und Development Tasks bekommen automatisch das `jules-task` Label und werden von Jules verarbeitet.

## 📊 Workflows im Überblick

| Workflow | Trigger | Dauer | Master-Switch |
|----------|---------|-------|---------------|
| CI-01 Build & Test | Push/PR | ~10-15 min | Nein |
| CI-02 Security Scan | Push/PR/Weekly | ~5-10 min | Nein |
| CI-04 Session Trigger | Issue labeled | Sekunden | ✅ Ja |
| CI-05 PR Auto-Merge | PR events/Checks | Sekunden | ✅ Ja |
| CI-06 Changelog | PR merged | Sekunden | Nein |
| CI-07 Post-Merge | PR merged | Sekunden | ✅ Ja |
| CI-08 Monitoring | On-demand | 1min/check | ✅ Ja |
| CI-09B Release | Tag/Manual | ~5-10 min | Nein |
| CI-ADMIN-01 Labels | Manual | Sekunden | Nein |

## ✅ Status

| Komponente | Status | Version |
|------------|--------|---------|
| CI/CD Pipeline | ✅ Produktionsbereit | 2.0 |
| Jules Integration | ✅ Vollautomatisch | 2.0 |
| Issue Templates | ✅ Auto-Labels | 2.0 |
| Master-Switch | ✅ Implementiert | 2.0 |

---

**Letztes Update:** 2024-12-16  
**Maintainer:** VjMapper Team  
**Status:** ✅ Produktionsbereit

**Nächster Schritt:** Lies [SETUP_GUIDE.md](SETUP_GUIDE.md) für 5-Minuten Setup! 🚀
