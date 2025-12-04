# Jules API Setup Guide

> **Schritt-für-Schritt Anleitung zur Konfiguration der Jules API Integration**

## 📋 Übersicht

Es gibt **drei Optionen** um Jules für dein Repository zu aktivieren. Wähle die beste für deinen Use Case:

| Option | Schwierigkeit | Automatisierung | Empfohlen für |
|--------|---------------|-----------------|---------------|
| **1. Jules GitHub App** | ⭐ Sehr einfach | Vollautomatisch | Die meisten Projekte |
| **2. Jules API + Workflow** | ⭐⭐ Mittel | Vollautomatisch | Erweiterte Kontrolle |
| **3. Manuelle Sessions** | ⭐ Einfach | Manuell | Testing/Debugging |

## 🚀 Option 1: Jules GitHub App (Empfohlen)

### Warum diese Option?
- ✅ **Keine API-Keys erforderlich**
- ✅ **Keine Secrets konfigurieren**
- ✅ **Native GitHub-Integration**
- ✅ **Automatische Updates**
- ✅ **5 Minuten Setup**

### Setup-Schritte:

#### 1. Jules GitHub App installieren

```bash
# Öffne Installation Page
open https://github.com/apps/jules
# Oder: https://github.com/apps/jules
```

**In der GitHub UI:**
1. Klicke auf "Install" (grüner Button)
2. Wähle ob "All repositories" oder "Only select repositories"
3. Für "Only select repositories": Wähle `MrLongNight/VjMapper`
4. Klicke "Install"

#### 2. Permissions akzeptieren

Jules benötigt folgende Permissions:
- ✅ **Issues:** Read & Write (um Issues zu lesen und PRs zu verlinken)
- ✅ **Pull Requests:** Read & Write (um PRs zu erstellen)
- ✅ **Contents:** Read & Write (um Code zu ändern)
- ✅ **Workflows:** Read (um CI Status zu prüfen)

#### 3. Fertig! 🎉

Jules überwacht jetzt automatisch:
- Issues mit dem Label `jules-task` oder `jules`
- Erstellt automatisch Sessions
- Öffnet PRs mit dem Label `jules-pr`

### Testen:

```bash
# 1. Erstelle ein Test-Issue
gh issue create \
  --title "Test Jules Integration" \
  --body "Test if Jules picks up this issue automatically." \
  --label "jules-task"

# 2. Warte ein paar Sekunden

# 3. Check ob Jules kommentiert hat
gh issue view <issue-number> --comments

# 4. Check Jules Dashboard
open https://jules.google.com
```

---

## 🔧 Option 2: Jules API + GitHub Actions Workflow

### Warum diese Option?
- ✅ **Volle API-Kontrolle**
- ✅ **Workflow-basierte Automatisierung**
- ✅ **Batch-Processing möglich**
- ✅ **Custom Trigger-Logik**

### Setup-Schritte:

#### 1. Jules Account erstellen

1. Besuche: https://jules.google.com
2. Klicke "Sign in with Google"
3. Autorisiere mit deinem Google Account
4. Verbinde deinen GitHub Account

#### 2. GitHub Repository verbinden

In der Jules Web-UI:
1. Gehe zu "Settings" oder "Repositories"
2. Klicke "Connect Repository"
3. Wähle `MrLongNight/VjMapper`
4. Erlaube Zugriff

#### 3. API Key generieren

In der Jules Web-UI:
1. Gehe zu "Settings" → "API Keys"
2. Klicke "Generate new API key"
3. Gib einen Namen ein: "VjMapper GitHub Actions"
4. Kopiere den API Key (wird nur einmal angezeigt!)

**⚠️ Wichtig:** Speichere den API Key sicher, er wird nur einmal angezeigt!

#### 4. API Key als GitHub Secret hinzufügen

**Via GitHub UI:**
1. Gehe zu: `https://github.com/MrLongNight/VjMapper/settings/secrets/actions`
2. Klicke "New repository secret"
3. Name: `JULES_API_KEY`
4. Value: `<dein-api-key>`
5. Klicke "Add secret"

**Via GitHub CLI:**
```bash
# API Key aus Zwischenablage (macOS/Linux)
gh secret set JULES_API_KEY --body "$(pbpaste)"

# Oder manuell eingeben
gh secret set JULES_API_KEY
# Paste den API Key und drücke Enter, dann Ctrl+D
```

**Verifizieren:**
```bash
# Secret sollte in der Liste sein (Wert wird nicht angezeigt)
gh secret list
# Output: JULES_API_KEY  Updated YYYY-MM-DD
```

#### 5. Workflow ist bereits konfiguriert! ✅

Der Workflow `.github/workflows/CI-04_session-trigger.yml` ist bereits in diesem Repository vorhanden und aktiv.

**Was automatisch passiert:**
- Issue wird mit `jules-task` Label erstellt/gelabelt → Workflow triggert
- Workflow erstellt Jules API Session automatisch
- Jules beginnt mit der Arbeit

#### 6. Testen

```bash
# 1. Erstelle ein Test-Issue
gh issue create \
  --title "Test Jules API Integration" \
  --body "Test automatic session creation via API." \
  --label "jules-task"

# 2. Check Workflow-Logs
gh run list --workflow="Jules Session Trigger"
gh run watch

# 3. Check Issue für Session-Kommentar
gh issue view <issue-number> --comments

# 4. Check Jules Dashboard
open https://jules.google.com
```

---

## 🧪 Option 3: Manuelle Session-Erstellung

### Warum diese Option?
- ✅ **Gut für Testing**
- ✅ **Volle manuelle Kontrolle**
- ✅ **Keine Workflow-Konfiguration nötig**

### Setup-Schritte:

#### Via Jules Web-UI (Einfachste Methode)

1. Besuche: https://jules.google.com
2. Klicke "New Session"
3. Wähle Repository: `MrLongNight/VjMapper`
4. Gib Prompt ein (z.B. Issue-Titel und Beschreibung)
5. Klicke "Start Session"

#### Via Jules CLI

```bash
# 1. Jules CLI installieren
npm install -g @google-labs/jules-cli
# Oder: curl -fsSL https://jules.google.com/install.sh | bash

# 2. Login
jules login

# 3. Session erstellen
jules remote new \
  --repo MrLongNight/VjMapper \
  --prompt "Fix issue #123: Implement multi-window rendering"

# 4. Session überwachen
jules remote status
```

#### Via cURL (REST API)

⚠️ **Sicherheitshinweis:** API Keys sollten niemals direkt in der Shell oder Scripts hardcoded werden. Verwende sichere Methoden wie Environment-Variablen aus Credential Manager.

```bash
# ⚠️ NICHT EMPFOHLEN: API Key direkt in Shell
# export JULES_API_KEY="your-api-key-here"  # Landet in Shell History!

# ✅ BESSER: API Key aus sicherem Speicher laden
# macOS: security find-generic-password -s jules-api-key -w
# Linux: secret-tool lookup service jules api-key
# Oder: Aus Password Manager (1Password, LastPass, etc.)

# Session erstellen (Annahme: JULES_API_KEY ist sicher gesetzt)
curl 'https://jules.googleapis.com/v1alpha/sessions' \
  -X POST \
  -H "Content-Type: application/json" \
  -H "X-Goog-Api-Key: $JULES_API_KEY" \
  -d '{
    "prompt": "Implement feature from issue #123",
    "sourceContext": {
      "source": "sources/github/MrLongNight/VjMapper",
      "githubRepoContext": {
        "startingBranch": "main"
      }
    }
  }'
```

---

## 🔍 Verifizierung & Troubleshooting

### Check 1: Ist Jules aktiv?

```bash
# Check für Jules-PRs
gh pr list --label "jules-pr"

# Check für Jules-Kommentare in Issues
gh issue list --label "jules-task" --limit 5
gh issue view <issue-number> --comments | grep -i jules
```

### Check 2: Workflow läuft?

```bash
# Liste der letzten Workflow-Runs
gh run list --workflow="Jules Session Trigger" --limit 10

# Logs vom letzten Run
gh run view --log

# Aktiven Run beobachten
gh run watch
```

### Check 3: API Key funktioniert?

```bash
# Test API Key (wenn manuell konfiguriert)
# WICHTIG: Für Sicherheit, API Key aus sicherem Speicher laden
# Option 1: Von GitHub Secret (lokal nicht direkt verfügbar)
# Option 2: Verwende Umgebungsvariable aus sicherem Storage

# Test mit API Key aus Environment (bereits gesetzt)
curl 'https://jules.googleapis.com/v1alpha/sources/github/MrLongNight/VjMapper' \
  -H "X-Goog-Api-Key: $JULES_API_KEY"

# Sollte 200 OK zurückgeben mit Repository-Info

# ⚠️ SICHERHEITSHINWEIS: 
# - Niemals API Keys direkt in Shell-Befehlen verwenden
# - Nicht in Shell History speichern (export HISTCONTROL=ignorespace)
# - Verwende GitHub Secrets für Workflows
# - Für lokale Tests: Verwende Credential Manager oder .netrc
```

### Häufige Probleme:

#### Problem: "JULES_API_KEY secret is not configured"

**Lösung:**
```bash
# Secret hinzufügen (siehe Option 2, Schritt 4)
gh secret set JULES_API_KEY

# Verifizieren
gh secret list | grep JULES_API_KEY
```

#### Problem: "Jules does not pick up issues"

**Mögliche Ursachen:**
1. **Kein `jules-task` Label:**
   ```bash
   gh issue edit <issue-number> --add-label "jules-task"
   ```

2. **Jules App nicht installiert UND kein API Key:**
   - Wähle Option 1 oder Option 2 (siehe oben)

3. **Issue ist geschlossen:**
   - Jules arbeitet nur an offenen Issues

#### Problem: "Session created but Jules not working"

**Check Jules Dashboard:**
```bash
open https://jules.google.com
```

**Check Session Status:**
- Sessions können fehlschlagen wenn:
  - Repository nicht erreichbar
  - Prompt zu vage
  - Dependencies nicht installierbar

**Logs prüfen:**
- Im Jules Dashboard: Session öffnen → "View Logs"

#### Problem: "API returns 401 Unauthorized"

**Ursache:** API Key ungültig oder abgelaufen

**Lösung:**
1. Neuen API Key in Jules Web-UI generieren
2. Secret updaten:
   ```bash
   gh secret set JULES_API_KEY
   ```

---

## 📊 Monitoring & Best Practices

### Dashboard Commands

```bash
# Jules Activity überwachen
gh issue list --label "jules-task" --state open
gh pr list --label "jules-pr" --state open

# Workflow-Status
gh run list --workflow="Jules Session Trigger" --limit 5

# Session-Kommentare in Issues
for issue in $(gh issue list --label "jules-task" --limit 10 --json number -q '.[].number'); do
  echo "Issue #$issue:"
  gh issue view $issue --comments | grep -A 5 "Jules Session"
  echo "---"
done
```

### Best Practices

#### 1. Issue-Qualität

**Gute Issues für Jules:**
```markdown
# Implement Multi-Window Rendering

## Description
Implement window-per-output architecture for multi-projector setups.

## Acceptance Criteria
- [ ] Multiple output windows can be created
- [ ] Frame synchronization works across all outputs
- [ ] Handles display changes gracefully

## Technical Details
- Files: crates/mapmap-render/src/output.rs
- Use wgpu for multi-window support
- Implement VSync synchronization
```

**Schlechte Issues für Jules:**
```markdown
# Fix stuff
Something is broken, please fix.
```

#### 2. Labels konsistent verwenden

```bash
# Jules Issues
gh issue create --label "jules-task,priority: high,phase-1: core-engine" ...

# Nach PR-Erstellung
# Jules fügt automatisch 'jules-pr' Label hinzu
```

#### 3. Regelmäßig monitoren

```bash
# Weekly Check-Script
#!/bin/bash
echo "=== Jules Activity Report ==="
echo "Open Tasks: $(gh issue list --label jules-task --state open --json number -q 'length')"
echo "Open PRs: $(gh pr list --label jules-pr --state open --json number -q 'length')"
echo "Merged this week: $(gh pr list --label jules-pr --state closed --search 'merged:>$(date -d '7 days ago' +%Y-%m-%d)' --json number -q 'length')"
```

---

## 🎯 Zusammenfassung & Nächste Schritte

### Was ist jetzt konfiguriert?

- ✅ Workflow `CI-04_session-trigger.yml` ist implementiert
- ✅ Auto-Merge für Jules-PRs ist aktiv
- ✅ CI/CD Pipeline läuft automatisch
- ✅ Dokumentation ist aktualisiert

### Was musst du noch tun?

**Minimale Konfiguration (empfohlen):**
```bash
# 1. Jules GitHub App installieren
open https://github.com/apps/jules

# 2. Fertig! 🎉
```

**Erweiterte Konfiguration (optional):**
```bash
# 1. Jules Account erstellen
open https://jules.google.com

# 2. API Key generieren und als Secret hinzufügen
gh secret set JULES_API_KEY

# 3. Fertig! 🎉
```

### Workflow-Test

```bash
# 1. Test-Issue erstellen
gh issue create \
  --title "Test Jules Integration" \
  --body "Verify that Jules session creation works automatically." \
  --label "jules-task"

# 2. Workflow beobachten
gh run watch

# 3. Issue prüfen
gh issue view <issue-number> --comments

# 4. Auf PR warten (Jules braucht typisch 10-30 Minuten)
gh pr list --label "jules-pr"
```

---

## 🆘 Support

### Dokumentation
- [Jules API Docs](https://developers.google.com/jules/api)
- [Jules Integration Guide](.github/JULES_INTEGRATION.md)
- [CI/CD Overview](CI_CD_README.md)
- [Workflow Details](.github/workflows/README.md)

### Bei Problemen
1. **Check Workflow-Logs:** `gh run view --log`
2. **Check Jules Dashboard:** https://jules.google.com
3. **Issue öffnen:** Mit Label `automation` oder `workflows`

### Kontakt
- GitHub: @MrLongNight
- Repository: https://github.com/MrLongNight/VjMapper

---

**Erstellt:** 2024-12-04  
**Version:** 1.0  
**Status:** ✅ Production Ready
