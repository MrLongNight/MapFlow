# 🚀 Setup Guide: Jules Integration

> **Schnellstart-Anleitung für die Jules CI/CD Integration**

## ✅ Was wurde implementiert?

1. **CI/CD Pipeline** - Automatisches Testen und Bauen auf allen Plattformen
2. **Jules Issue Creation** - Alle Development Tasks als Issues vordefiniert
3. **Auto-Merge System** - Jules PRs werden automatisch gemerged
4. **Dokumentations-Updates** - Changelog wird automatisch aktualisiert

## 🎯 Setup in 5 Schritten

### Schritt 1: Labels erstellen

```bash
# Labels aus Konfiguration synchronisieren
gh label sync --file .github/labels.yml
```

**Oder manuell** die wichtigsten Labels erstellen:
- `jules-task` (für Issues die Jules bearbeiten soll)
- `jules-pr` (für PRs von Jules)
- `priority: critical`, `priority: high`, `priority: medium`

### Schritt 2: Jules Issues erstellen

```bash
# Alle 8 Development Issues auf einmal erstellen
gh workflow run JULES-01_create-issues.yml

# Status prüfen
gh run watch
```

Das erstellt automatisch:
- ✅ Multi-Window Rendering (Critical)
- ✅ Frame Synchronization (Critical)
- ✅ Build System Fix (High)
- ✅ Still Image Support (High)
- ✅ Animated Format Support (Medium)
- ✅ ProRes Codec Support (Medium)
- ✅ Advanced Geometric Correction (Medium)
- ✅ Output Configuration Persistence (Medium)

### Schritt 3: Branch Protection konfigurieren (Optional)

Für `main` Branch:

1. **Settings** → **Branches** → **Add rule**
2. Branch name pattern: `main`
3. Aktivieren:
   - ✅ Require status checks to pass before merging
     - ✅ CI/CD Pipeline
     - ✅ Code Quality
   - ✅ Require branches to be up to date before merging
4. Optional (für manuelle Reviews):
   - ☐ Require a pull request before merging
   - ☐ Require approvals: 1

### Schritt 4: Jules API konfigurieren

**Jules sollte konfiguriert werden um:**

1. **Issues überwachen:**
   - Repository: `MrLongNight/VjMapper`
   - Label: `jules-task`
   - Check Interval: 5 Minuten

2. **PRs erstellen mit:**
   - Label: `jules-pr`
   - Branch Prefix: `jules/`
   - PR Template verwenden
   - Related Issue verlinken: `Closes #<issue_number>`

3. **Best Practices:**
   - Lokale Tests vor PR ausführen
   - `cargo fmt` und `cargo clippy` ausführen
   - Clear commit messages

### Schritt 5: Testen

**Test mit einem einfachen Issue:**

1. Issue manuell erstellen oder eines der auto-generierten verwenden
2. Jules verarbeiten lassen
3. PR beobachten in Actions Tab
4. Auto-Merge validieren

## 🔍 Überwachung

### Issues anzeigen
```bash
# Alle Jules Tasks
gh issue list --label "jules-task"

# Nach Priority filtern
gh issue list --label "jules-task" --label "priority: critical"
```

### PRs überwachen
```bash
# Alle Jules PRs
gh pr list --label "jules-pr"

# PR Status checken
gh pr view <pr-number> --json statusCheckRollup,mergeable
```

### Workflows überwachen
```bash
# CI/CD Pipeline Status
gh run list --workflow="CI/CD Pipeline"

# Specific run anzeigen
gh run view <run-id> --log
```

## 🎬 Workflow-Ablauf

```
1. Issue mit jules-task Label existiert
         ↓
2. Jules liest Issue und Acceptance Criteria
         ↓
3. Jules implementiert Lösung in Branch jules/issue-123
         ↓
4. Jules erstellt PR mit jules-pr Label
         ↓
5. CI/CD Pipeline läuft automatisch
         ↓
6. Auto-Merge Workflow prüft Status
         ↓
7. Wenn alle Checks ✅ → Auto-Merge
         ↓
8. Issue wird automatisch geschlossen
         ↓
9. CHANGELOG.md wird aktualisiert
```

## 📁 Wichtige Dateien

| Datei | Zweck |
|-------|-------|
| `.github/workflows/CI-01_build-and-test.yml` | Hauptsächlicher CI/CD Pipeline |
| `.github/workflows/JULES-01_create-issues.yml` | Erstellt alle Jules Issues |
| `.github/workflows/JULES-03_pr-automation.yml` | Auto-Merge für Jules PRs |
| `.github/workflows/DOCS-01_update-changelog.yml` | Changelog Updates |
| `.github/workflows/CI-02_security-scan.yml` | Security Scanning |
| `.github/ISSUE_TEMPLATE/development_task.yml` | Template für neue Tasks |
| `.github/labels.yml` | Label Konfiguration |
| `.github/JULES_INTEGRATION.md` | Detaillierte Jules Dokumentation |

## 🔧 Troubleshooting

### Problem: Labels existieren nicht

```bash
# Labels sync ausführen
gh label sync --file .github/labels.yml
```

### Problem: CI schlägt fehl

```bash
# Lokal reproduzieren
cargo fmt --check
cargo clippy --all-targets
cargo test --verbose

# Build testen
cargo build --release
```

### Problem: Auto-Merge funktioniert nicht

**Checkliste:**
- [ ] PR hat `jules-pr` Label?
- [ ] Alle CI Checks bestanden?
- [ ] Keine Merge Konflikte?
- [ ] Keine "Changes Requested" Reviews?
- [ ] PR ist kein Draft?

**Debug:**
```bash
# Check PR Status
gh pr view <pr-number> --json mergeable,mergeStateStatus

# Check Workflow Logs
gh run list --workflow="Jules PR Auto-Merge"
gh run view <run-id> --log
```

### Problem: Issues wurden doppelt erstellt

**Lösung:** Der Workflow prüft bereits existierende Issues. Doppelte werden übersprungen.

Manuell aufräumen:
```bash
# Doppelte Issues finden und schließen
gh issue list --label "jules-task" --state all
gh issue close <issue-number>
```

## 💡 Tipps & Best Practices

### Für Issue-Erstellung

✅ **DO:**
- Klare Acceptance Criteria definieren
- Technische Details und Dateipfade angeben
- Priority Labels setzen
- Related Documentation verlinken

❌ **DON'T:**
- Vage Beschreibungen
- Zu große Tasks (besser aufteilen)
- Fehlende Test-Requirements

### Für Jules PRs

✅ **DO:**
- PR Template komplett ausfüllen
- Alle Tests lokal ausführen
- Related Issue verlinken mit "Closes #123"
- Clear commit messages

❌ **DON'T:**
- Draft PRs (werden nicht auto-merged)
- PRs ohne Tests
- Merge Konflikte

### Monitoring

**Täglich:**
- Jules PR Status checken
- CI/CD Failures überprüfen

**Wöchentlich:**
- Offene Issues reviewen
- Merged PRs stichprobenartig prüfen
- ROADMAP.md manuell updaten

## 🆘 Hilfe benötigt?

1. **Workflow Logs prüfen:** GitHub Actions Tab → Select Workflow → View Logs
2. **Dokumentation lesen:** `.github/workflows/README.md`
3. **Issue öffnen:** Mit Label `workflows` oder `automation`
4. **Kontakt:** @MrLongNight für dringende Probleme

## 📚 Weitere Dokumentation

- [Workflow README](.github/workflows/README.md) - Detaillierte Workflow-Dokumentation
- [Jules Integration](.github/JULES_INTEGRATION.md) - Umfassende Jules Anleitung
- [Issue Templates](.github/ISSUE_TEMPLATE/) - Templates für Issues
- [PR Template](.github/pull_request_template.md) - Template für PRs

## ✨ Das war's!

Nach diesen 5 Schritten ist die Jules Integration einsatzbereit:

```bash
# 1. Labels erstellen
gh label sync --file .github/labels.yml

# 2. Issues erstellen
gh workflow run JULES-01_create-issues.yml

# 3. Jules konfigurieren (siehe Schritt 4)

# 4. Ersten Test-PR von Jules abwarten

# 5. Workflow beobachten und freuen! 🎉
```

---

**Version:** 1.0  
**Letztes Update:** 2024-12-04  
**Status:** Produktionsbereit ✅
