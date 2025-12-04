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

Es gibt **drei Möglichkeiten**, Jules zu aktivieren:

#### Option 1: Jules GitHub App (Empfohlen - Einfachste Lösung) ⭐

1. **Installiere die Jules GitHub App:**
   - Besuche: https://github.com/apps/jules
   - Klicke auf "Install" und wähle dein Repository aus
   - Erlaube Zugriff auf das VjMapper Repository

2. **Fertig!** Jules überwacht automatisch:
   - Issues mit dem Label `jules-task` oder `jules`
   - Erstellt automatisch PRs mit dem Label `jules-pr`
   - Keine weitere Konfiguration nötig

**Vorteile:**
- ✅ Keine API-Keys erforderlich
- ✅ Automatische Session-Erstellung bei neuen Issues
- ✅ Native GitHub-Integration
- ✅ Sicher und von Google verwaltet

#### Option 2: Jules API mit GitHub Actions (Automatisch via Workflow)

1. **API Key generieren:**
   - Besuche: https://jules.google.com
   - Melde dich an und verbinde deinen GitHub Account
   - Gehe zu Settings → API Keys
   - Generiere einen neuen API Key

2. **API Key als Repository Secret hinzufügen:**
   ```bash
   # Via GitHub UI:
   # Repository Settings → Secrets and variables → Actions → New repository secret
   # Name: JULES_API_KEY
   # Value: <dein-api-key>
   ```

3. **Workflow aktivieren:**
   - Der Workflow `.github/workflows/JULES-02_session-trigger.yml` ist bereits konfiguriert
   - Er triggert automatisch bei Issues mit `jules-task` Label
   - Er nutzt den JULES_API_KEY um Sessions zu erstellen

**Vorteile:**
- ✅ Volle Kontrolle über API-Calls
- ✅ Workflow-basierte Automatisierung
- ✅ Batch-Processing möglich
- ✅ Bereits implementiert in diesem Repository

#### Option 3: Manuelle Session-Erstellung (Für Testing/Debugging)

```bash
# Via Jules CLI:
jules remote new --repo . --prompt "Fix issue #123"

# Via cURL (REST API):
curl 'https://jules.googleapis.com/v1alpha/sessions' \
  -X POST \
  -H "Content-Type: application/json" \
  -H 'X-Goog-Api-Key: YOUR_API_KEY' \
  -d '{
    "prompt": "Implement feature from issue #123",
    "sourceContext": {
      "source": "sources/github/MrLongNight/VjMapper",
      "githubRepoContext": { "startingBranch": "main" }
    }
  }'
```

**Vorteile:**
- ✅ Direkte Kontrolle
- ✅ Gut für Testing
- ✅ Kein Workflow-Setup nötig

---

### 🎯 Empfohlene Konfiguration

**Für dieses Repository (VjMapper):**

**Phase 1 - Quick Start (5 Minuten):**
1. Installiere Jules GitHub App (Option 1)
2. Issues werden automatisch erkannt
3. Fertig! ✅

**Phase 2 - Erweiterte Automatisierung (optional):**
1. Zusätzlich API Key als Secret hinzufügen
2. Ermöglicht erweiterte Workflow-Features
3. Batch-Processing von Issues

**Aktueller Status:**
- ✅ Workflow `JULES-02_session-trigger.yml` ist implementiert
- ✅ Auto-Merge Workflow ist konfiguriert
- ⏳ JULES_API_KEY Secret fehlt (optional - nur für API-basierte Automatisierung)
- ⏳ Jules GitHub App muss installiert werden (empfohlen)

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
gh workflow run JULES-01_create-issues.yml
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

### 2. Automatische Jules Session-Erstellung

**Neu implementiert!** Der Workflow `JULES-02_session-trigger.yml` automatisiert die Session-Erstellung:

#### Automatische Trigger:

**Wenn ein Issue erstellt oder gelabelt wird:**
```
Issue mit jules-task Label erstellt/hinzugefügt
    ↓
Workflow: JULES-02_session-trigger.yml läuft automatisch
    ↓
Tracking-Kommentar wird zum Issue hinzugefügt
    ↓
Jules API Session wird erstellt (wenn JULES_API_KEY vorhanden)
    ↓
Jules beginnt mit der Arbeit
```

**Manuell für existierende Issues:**
```bash
# Einzelnes Issue triggern
gh workflow run JULES-02_session-trigger.yml -f issue_number=123

# ALLE offenen jules-task Issues triggern (Batch-Modus)
gh workflow run JULES-02_session-trigger.yml
```

#### Was der Workflow macht:

1. **Automatische Erkennung:**
   - Triggert bei neuem Issue mit `jules-task` Label
   - Triggert wenn `jules-task` Label zu existierendem Issue hinzugefügt wird
   - Kann manuell für beliebige Issues getriggert werden

2. **Tracking-Kommentar:**
   - Fügt Kommentar zum Issue hinzu mit Status
   - Informiert über nächste Schritte
   - Dokumentiert Session-ID (wenn API genutzt)

3. **API Integration (optional):**
   - Wenn `JULES_API_KEY` Secret konfiguriert ist:
     - Erstellt automatisch Jules API Session
     - Issue-Titel und Body werden als Prompt verwendet
     - Session-Link wird im Issue-Kommentar hinterlegt
   - Ohne API Key:
     - Workflow läuft trotzdem (Tracking-Kommentar)
     - Jules GitHub App übernimmt (wenn installiert)

4. **Batch-Processing:**
   - Workflow kann alle offenen jules-task Issues auf einmal verarbeiten
   - Nützlich bei Repository-Setup
   - Rate-Limiting berücksichtigt

#### Workflow-Dateien:

```
.github/workflows/
├── JULES-02_session-trigger.yml    # NEU: Triggert Jules Sessions
├── JULES-01_create-issues.yml      # Erstellt Issues aus ROADMAP
├── JULES-03_pr-automation.yml      # Auto-Merge für Jules PRs
└── ...
```

### 3. Jules Verarbeitung

Nach Session-Erstellung arbeitet Jules am Issue:

1. **Issue-Analyse:** Jules liest die Issue-Beschreibung und Acceptance Criteria
2. **Branch-Erstellung:** Erstellt Branch `jules/issue-<number>-<title>`
3. **Implementierung:** Schreibt Code gemäß den Anforderungen
4. **Testing:** Führt lokale Tests durch
5. **PR-Erstellung:** Öffnet PR mit:
   - Link zum originalen Issue (`Closes #<number>`)
   - Beschreibung der Änderungen
   - Test-Ergebnisse
   - `jules-pr` Label (automatisch)

### 4. Automatisches Testing

Nach PR-Erstellung laufen automatisch (via `CI-01_build-and-test.yml`):

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

### 5. Auto-Merge Logik

Der Auto-Merge (via `JULES-03_pr-automation.yml`) erfolgt, wenn:

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

### 6. Dokumentations-Update

Nach erfolgreichem Merge (via `DOCS-01_update-changelog.yml`):

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
