# Jules AI Integration Guide

> **Hinweis:** Diese Anleitung erklärt die Integration der Google Jules API für automatisierte Entwicklung und Pull-Request-Management.

## 📋 Überblick

Die Jules-Integration ermöglicht es, Entwicklungsaufgaben automatisch von einem KI-Agenten bearbeiten zu lassen. Der komplette Workflow umfasst:

1. **Issue-Generierung** aus ROADMAP.md
2. **Jules-Verarbeitung** der Issues
3. **Automatisches Testing** der PRs
4. **Auto-Merge** bei erfolgreichen Tests
5. **Dokumentations-Updates** nach dem Merge

## 🚀 Setup-Anleitung

### Schritt 1: GitHub Labels Konfigurieren

```bash
# Labels aus der Konfigurationsdatei synchronisieren
gh label sync --file .github/labels.yml
```

Die wichtigsten Labels für Jules:
- `jules-task`: Markiert Issues, die Jules bearbeiten kann
- `jules-pr`: Markiert PRs von Jules
- `priority: critical/high/medium/low`: Priorisierung

### Schritt 2: Branch Protection Rules

Empfohlene Branch-Protection-Einstellungen für `main`:

1. **Require status checks to pass:**
   - ✅ CI/CD Pipeline
   - ✅ Code Quality
   - ✅ Security Audit

2. **Require review (optional):**
   - Wenn manuelle Reviews gewünscht sind
   - Kann für Jules-PRs deaktiviert werden

3. **Require branches to be up to date:**
   - ✅ Aktiviert für saubere Merges

4. **Allow force pushes:** ❌ Deaktiviert
5. **Allow deletions:** ❌ Deaktiviert

### Schritt 3: Jules API Konfiguration

**Wichtig:** Die Jules API-Konfiguration ist nicht öffentlich und sollte separat bereitgestellt werden.

#### Erforderliche Konfiguration für Jules:

```yaml
# jules-config.yml (Beispiel - nicht öffentlich!)
repository:
  owner: "MrLongNight"
  repo: "VjMapper"
  
issue_monitoring:
  labels: ["jules-task"]
  check_interval: 300  # 5 Minuten
  
pull_request:
  label: "jules-pr"
  branch_prefix: "jules/"
  auto_label: true
  
template:
  use_pr_template: true
  add_metadata: true
  
behavior:
  max_concurrent_tasks: 3
  timeout_minutes: 120
  auto_close_on_failure: false
```

### Schritt 4: Workflow Permissions

Stelle sicher, dass die GitHub Actions die richtigen Permissions haben:

```yaml
permissions:
  contents: write      # Für Commits und Documentation Updates
  issues: write        # Für Issue-Management
  pull-requests: write # Für PR-Management
  checks: read         # Für Status-Checks
  security-events: write # Für Security-Scans
```

## 🔄 Workflow-Beschreibung

### 1. Issue-Erstellung

**Einmalig alle Issues erstellen:**
```bash
# Alle Jules Development Issues auf einmal erstellen
gh workflow run create-jules-issues.yml
```

Dieser Workflow erstellt automatisch alle 8 Haupt-Development-Tasks basierend auf ROADMAP.md:
- Multi-Window Rendering (Critical)
- Frame Synchronization (Critical)  
- Build System Fix (High)
- Still Image Support (High)
- Animated Format Support (Medium)
- ProRes Codec Support (Medium)
- Advanced Geometric Correction (Medium)
- Output Configuration Persistence (Medium)

**Zusätzliche Issues manuell erstellen:**
- Nutze die Issue-Templates in `.github/ISSUE_TEMPLATE/`
- Label `jules-task` hinzufügen
- Acceptance Criteria klar definieren

### 2. Jules Verarbeitung

Jules überwacht Issues mit dem `jules-task` Label:

1. **Issue-Analyse:** Jules liest die Issue-Beschreibung und Acceptance Criteria
2. **Branch-Erstellung:** Erstellt Branch `jules/issue-<number>-<title>`
3. **Implementierung:** Schreibt Code gemäß den Anforderungen
4. **Testing:** Führt lokale Tests durch
5. **PR-Erstellung:** Öffnet PR mit:
   - Link zum originalen Issue
   - Beschreibung der Änderungen
   - Test-Ergebnisse
   - `jules-pr` Label

### 3. Automatisches Testing

Nach PR-Erstellung laufen automatisch:

- **Code Quality Checks:**
  - `cargo fmt --check` (Formatierung)
  - `cargo clippy` (Linting)

- **Build & Test:**
  - Multi-Platform Builds (Linux, macOS, Windows)
  - Unit Tests (`cargo test`)
  - Doc Tests
  - Integration Tests

- **Security Scans:**
  - CodeQL Analysis
  - Dependency Audit (`cargo audit`)

### 4. Auto-Merge Logik

Der Auto-Merge erfolgt, wenn:

```
✅ Alle CI-Checks bestanden
✅ Keine Merge-Konflikte
✅ Keine "Changes Requested" Reviews
✅ PR ist nicht als Draft markiert
✅ PR hat `jules-pr` Label oder "Created by Jules" im Body
```

**Ablauf:**
1. Validierung der Bedingungen
2. Squash-Merge in `main`
3. Automatisches Schließen des related Issues
4. Commit-Message: "Auto-merge Jules PR #<number>: <title>"

### 5. Dokumentations-Update

Nach erfolgreichem Merge:

- **CHANGELOG.md:** Fügt automatisch Changelog-Entry hinzu
- **ROADMAP.md:** Wird manuell aktualisiert (Tasks als completed markieren)
- **Related Issue:** Wird automatisch geschlossen

## 📝 Best Practices

### Für Issue-Erstellung

1. **Klare Beschreibungen:**
   ```markdown
   Implementiere Multi-Window-Rendering für synchronized outputs.
   
   Acceptance Criteria:
   - [ ] Window-per-output Architektur implementiert
   - [ ] Frame-Synchronisation funktioniert
   - [ ] Tests für 2+ Displays
   ```

2. **Technische Details bereitstellen:**
   - Relevante Dateipfade
   - Zu verwendende APIs
   - Architektur-Hinweise

3. **Priority setzen:**
   - Critical: Blockiert andere Arbeit
   - High: Wichtig für Milestone
   - Medium: Standard-Priority
   - Low: Nice-to-have

### Für Jules PRs

1. **Jules sollte:**
   - PR-Template verwenden
   - Alle Tests lokal ausführen vor PR
   - Clear commit messages schreiben
   - Related Issue verlinken

2. **Review-Prozess:**
   - Auto-merge nur bei 100% erfolgreichen Tests
   - Bei Fehlern: Manuelles Review erforderlich
   - Comments von Reviewern werden berücksichtigt

3. **Monitoring:**
   - Regelmäßig merged PRs überprüfen
   - Qualität der Jules-Implementierungen bewerten
   - Feedback in Issue-Templates einarbeiten

## 🔍 Monitoring & Debugging

### Status Überprüfen

```bash
# Aktuelle Workflow-Runs anzeigen
gh run list --workflow="CI/CD Pipeline"

# Spezifischen Run ansehen
gh run view <run-id> --log

# Jules PRs finden
gh pr list --label "jules-pr"

# Jules Tasks finden
gh issue list --label "jules-task"
```

### Häufige Probleme

#### Problem: Auto-Merge funktioniert nicht

**Diagnose:**
```bash
# PR-Status prüfen
gh pr view <pr-number> --json mergeable,mergeStateStatus,statusCheckRollup

# Workflow-Logs prüfen
gh run view --log
```

**Lösungen:**
- Merge-Konflikte auflösen
- Fehlgeschlagene Checks reparieren
- Branch-Protection-Rules überprüfen
- `jules-pr` Label vorhanden?

#### Problem: Issues werden nicht erstellt

**Diagnose:**
```bash
# Workflow manuell mit dry-run ausführen
gh workflow run auto-create-issues.yml -f dry_run=true

# Logs überprüfen
gh run view --log
```

**Lösungen:**
- ROADMAP.md Format überprüfen
- Permissions überprüfen
- Bereits existierende Issues prüfen

#### Problem: CI schlägt fehl

**Diagnose:**
```bash
# Build lokal reproduzieren
cargo build --verbose
cargo clippy --all-targets
cargo test --verbose
```

**Lösungen:**
- Dependencies aktualisieren
- System-Dependencies installieren
- FFmpeg-Installation überprüfen

## 🔐 Sicherheit

### Wichtige Sicherheitsmaßnahmen

1. **Keine Secrets im Code:**
   - Verwende GitHub Secrets für API-Keys
   - Jules-Konfiguration nicht committen
   - Sensitive Daten in `.gitignore`

2. **Code Review:**
   - Auch auto-merged PRs stichprobenartig prüfen
   - Security-Scans immer aktiviert
   - Dependency-Audits regelmäßig ausführen

3. **Branch Protection:**
   - Force-Push deaktiviert
   - Required status checks
   - Signed commits (optional aber empfohlen)

### Security Workflow

- **CodeQL:** Läuft wöchentlich + bei jedem PR
- **Cargo Audit:** Prüft Dependencies auf Vulnerabilities
- **Manual Review:** Bei sicherheitskritischen Änderungen

## 📊 Metriken & Reporting

### KPIs für Jules Integration

- **Success Rate:** Prozent der erfolgreich gemerged Jules PRs
- **Time to Merge:** Durchschnittliche Zeit von Issue bis Merge
- **Quality:** Anzahl der Bugs/Regressions nach Jules PRs
- **Coverage:** Anzahl der von Jules bearbeiteten vs. totalen Issues

### Monitoring Dashboard

```bash
# Statistiken anzeigen
gh api repos/MrLongNight/VjMapper/issues \
  --jq '[.[] | select(.labels[].name == "jules-task")] | length'

gh api repos/MrLongNight/VjMapper/pulls \
  --jq '[.[] | select(.labels[].name == "jules-pr")] | length'
```

## 🔧 Wartung

### Regelmäßige Aufgaben

**Täglich:**
- Jules PR Status überprüfen
- Fehlgeschlagene Workflows prüfen

**Wöchentlich:**
- Auto-generierte Issues reviewen
- ROADMAP.md Fortschritt überprüfen
- Merge-Queue überprüfen

**Monatlich:**
- Jules-Performance analysieren
- Workflow-Optimierungen implementieren
- Issue-Templates aktualisieren

### Updates

Bei Änderungen am System:

1. **Workflow-Updates:**
   - In Feature-Branch testen
   - Manual dispatch verwenden
   - Logs sorgfältig prüfen

2. **Jules-Config-Updates:**
   - Mit Dry-Run testen
   - Schrittweise ausrollen
   - Rollback-Plan haben

## 📚 Weitere Ressourcen

- [Workflow README](.github/workflows/README.md)
- [Issue Templates](.github/ISSUE_TEMPLATE/)
- [PR Template](.github/pull_request_template.md)
- [ROADMAP.md](../ROADMAP.md)
- [GitHub Actions Docs](https://docs.github.com/en/actions)

## 🆘 Support

Bei Problemen:

1. **Workflow-Logs prüfen:** GitHub Actions Tab
2. **Issue öffnen:** Mit `workflows` oder `automation` Label
3. **Kontakt:** @MrLongNight für kritische Probleme

---

**Letztes Update:** 2024-12-04  
**Version:** 1.0  
**Status:** Produktionsbereit
