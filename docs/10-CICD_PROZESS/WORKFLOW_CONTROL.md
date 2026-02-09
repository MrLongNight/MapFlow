# Workflow Control Guide

> **Anleitung zur Steuerung der CI/CD Workflows**

## 🎮 Workflows Ein-/Ausschalten

### Methode 1: Umgebungsvariablen in Workflow-Dateien

Die einfachste Methode ist, die `env` Variablen direkt in den Workflow-Dateien zu ändern:

#### CI/CD Workflow (CI-01_build-and-test.yml)
```yaml
# Keine globale Deaktivierung - verwende stattdessen manual dispatch mit Optionen
```

**Manueller Run mit Optionen:**
```bash
# Nur Linux bauen (überspringt macOS/Windows)
gh workflow run "CI/CD" -f skip_platforms=true

# Tests überspringen (schnellerer Build)
gh workflow run "CI/CD" -f skip_tests=true

# Beides kombinieren
gh workflow run "CI/CD" -f skip_platforms=true -f skip_tests=true
```

#### Jules Auto-Merge (CI-05_pr-automation.yml)
```yaml
env:
  AUTO_MERGE_ENABLED: true  # Auf 'false' setzen um zu deaktivieren
```

**Deaktivieren:**
1. Öffne `.github/workflows/CI-05_pr-automation.yml`
2. Ändere `AUTO_MERGE_ENABLED: true` zu `AUTO_MERGE_ENABLED: false`
3. Commit und push

#### CodeQL Security Scan (CI-02_security-scan.yml)
```yaml
env:
  SCAN_ON_PR_ENABLED: true  # Auf 'false' setzen um PR-Scans zu deaktivieren
```

**Note:** Wöchentliche Scans laufen weiter, nur PR-Scans werden deaktiviert.

### Methode 2: GitHub Actions UI

Workflows können auch über die GitHub UI deaktiviert werden:

1. Gehe zu **Actions** Tab
2. Wähle den Workflow aus der linken Sidebar
3. Klicke auf **"..."** (drei Punkte) oben rechts
4. Wähle **"Disable workflow"**

**Vorteil:** Workflow läuft gar nicht mehr
**Nachteil:** Muss manuell wieder aktiviert werden

### Methode 3: Branch Protection Rules anpassen

Wenn du bestimmte Checks nicht mehr benötigst:

1. **Settings** → **Branches** → **Branch protection rules**
2. Wähle die Regel für `main`
3. Unter "Require status checks to pass before merging"
4. Entferne nicht benötigte Checks

## 📋 Welche Checks gibt es und warum?

### 1. CI/CD Workflow (CI-01_build-and-test.yml)

**Checks:**
- **Code Quality (Format & Lint)** - 1 Job
  - `cargo fmt --check` - Prüft Code-Formatierung
  - `cargo clippy` - Prüft Code-Qualität und häufige Fehler

- **Build & Test** - 3 Jobs (Linux, macOS, Windows)
  - Debug Build
  - Release Build
  - Tests ausführen
  - Doc Tests
  - Dokumentation generieren

- **Security Audit** - 1 Job
  - `cargo audit` - Prüft Dependencies auf Sicherheitslücken

- **CI Success Gate** - 1 Job
  - Fasst alle Checks zusammen

**Total:** 6 Jobs

**Warum so viele?**
- **3 Plattformen:** Rust-Projekt muss auf allen Plattformen laufen
- **Quality Gates:** Verhindert, dass schlechter Code gemerged wird
- **Security:** Wichtig für Produktion

**Reduzierung möglich:**
```bash
# Nur Linux bauen (von 6 auf 4 Jobs)
gh workflow run "CI/CD" -f skip_platforms=true

# Tests überspringen (schneller, aber weniger sicher)
gh workflow run "CI/CD" -f skip_tests=true
```

### 2. CodeQL Security Scan (CI-02_security-scan.yml)

**Checks:**
- **Analyze Code** - 1 Job
  - Deep Security Analysis von Rust Code
  - Findet potenzielle Sicherheitslücken

**Warum?**
- Professionelle Security-Analyse
- Findet Bugs die normale Tests nicht finden
- Best Practice für Open Source

**Reduzierung:**
- Läuft nur bei Push zu `main` und PRs
- Kann über `SCAN_ON_PR_ENABLED: false` für PRs deaktiviert werden
- Wöchentlicher Scan bleibt aktiv (wichtig!)

### 3. Jules PR Auto-Merge (CI-05_pr-automation.yml)

**Checks:**
- **Auto-Merge Jules PR** - 1 Job
  - Merged automatisch Jules PRs wenn alle Checks bestehen

**Warum?**
- Automatisierung des Merge-Prozesses
- Nur für Jules PRs aktiv
- Normale PRs nicht betroffen

**Kontrolle:**
- Über `AUTO_MERGE_ENABLED` variable
- Läuft nur wenn PR `jules-pr` Label hat

### 4. Update Documentation (CI-06_update-changelog.yml)

**Checks:**
- **Update Changelog** - 1 Job
  - Aktualisiert CHANGELOG.md nach Merge

**Warum?**
- Dokumentation aktuell halten
- Läuft nur nach erfolgreichem Merge
- Minimal und schnell

### 5. Sync Labels (CI-ADMIN-01_sync-labels.yml)

**Checks:**
- **Sync Repository Labels** - 1 Job
  - Synchronisiert Labels aus `.github/labels.yml`

**Warum?**
- Läuft nur bei Änderungen an `labels.yml`
- Sehr selten aktiv
- Hält Label-System konsistent

## 🔧 Empfohlene Konfiguration

### Für Entwicklung (Schnell)
```bash
# CI/CD nur auf Linux
gh workflow run "CI/CD" -f skip_platforms=true

# Optional: Tests überspringen
gh workflow run "CI/CD" -f skip_tests=true
```

### Für Testing (Balance)
```bash
# Standard: Alle Plattformen, alle Tests
gh workflow run "CI/CD"
```

### Für Production (Maximal)
```bash
# Standard + alle Checks aktiv
# Nichts ändern, alles läuft automatisch
```

## 🚨 Warum Checks fehlschlagen könnten

### Häufige Ursachen:

1. **Build-Dependencies fehlen**
   - Linux: FFmpeg, fontconfig, freetype
   - macOS: Meist vorhanden
   - Windows: FFmpeg kann fehlen

2. **Code-Formatting**
   - `cargo fmt` nicht ausgeführt
   - Lösung: `cargo fmt` lokal ausführen

3. **Clippy Warnings**
   - Code-Qualität-Issues
   - Lösung: `cargo clippy --fix`

4. **Tests schlagen fehl**
   - Bugs im Code
   - Lösung: Tests lokal ausführen und fixen

5. **Security Vulnerabilities**
   - Veraltete Dependencies
   - Lösung: `cargo update` und prüfen

### Debug-Tipps:

```bash
# Lokal alle Checks ausführen (wie CI)
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
cargo test --verbose --all-features
cargo audit

# Wenn alles lokal funktioniert, funktioniert es auch in CI
```

## 📊 Minimalkonfiguration

Wenn du wirklich nur das Nötigste willst:

### Nur Code Quality Checks behalten:

1. **Deaktiviere** in `.github/workflows/CI-02_security-scan.yml`:
   ```yaml
   env:
     SCAN_ON_PR_ENABLED: false
   ```

2. **Deaktiviere** in `.github/workflows/CI-05_pr-automation.yml`:
   ```yaml
   env:
     AUTO_MERGE_ENABLED: false
   ```

3. **Bei CI/CD Runs** immer mit Optionen:
   ```bash
   gh workflow run "CI/CD" -f skip_platforms=true -f skip_tests=true
   ```

**Result:** Nur noch Format + Clippy + Linux Build (ca. 5 Minuten statt 15)

## 🎯 Zusammenfassung

| Workflow | Jobs | Dauer | Kann deaktiviert werden? | Empfehlung |
|----------|------|-------|-------------------------|------------|
| CI/CD | 6 | ~15 min | Teilweise (Optionen) | Behalten, evtl. Optionen nutzen |
| CodeQL | 1 | ~10 min | Ja (für PRs) | Behalten für `main` |
| Jules Auto-Merge | 1 | <1 min | Ja | Behalten wenn Jules aktiv |
| Update Docs | 1 | <1 min | Nein (läuft nur nach merge) | Behalten |
| Sync Labels | 1 | <1 min | Nein (läuft selten) | Behalten |

**Empfohlene Aktion:**
- ✅ Alle Workflows behalten
- ✅ Bei Bedarf CI/CD mit Optionen ausführen (`skip_platforms`, `skip_tests`)
- ✅ CodeQL für PRs deaktivieren wenn zu langsam (`SCAN_ON_PR_ENABLED: false`)
- ✅ Lokale Checks ausführen vor Push (`cargo fmt && cargo clippy && cargo test`)

## 🆘 Support

Bei Fragen oder Problemen:
1. Siehe [SETUP_GUIDE.md](.github/SETUP_GUIDE.md)
2. Siehe [workflows/README.md](.github/workflows/README.md)
3. Issue öffnen mit Label `workflows`

---

**Erstellt:** 2024-12-04
**Version:** 1.0
