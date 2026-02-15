# Jules Issues erstellen - Einfache Anleitung

## ⚠️ Wichtig: Warum Issues nicht automatisch erstellt wurden

GitHub Actions auf PR-Branches haben **keine Permission um Issues zu erstellen**. Dies ist eine Sicherheitsmaßnahme von GitHub.

**Auch ich (Copilot) kann keine Issues direkt erstellen**, weil:
1. Ich keine GitHub API Credentials in diesem Kontext habe
2. Die verfügbaren Tools nur lesenden Zugriff auf GitHub haben
3. Issue-Erstellung erfordert `issues: write` Permission

## ✅ Lösung: Lokales Script ausführen

Ich habe ein **fertiges Script erstellt** das alle 8 Jules Issues für dich erstellt!

### Option 1: Script lokal ausführen (5 Sekunden)

```bash
# 1. Repository clonen (falls noch nicht geschehen)
git clone https://github.com/MrLongNight/MapFlow.git
cd MapFlow

# 2. Zu diesem Branch wechseln
git checkout copilot/implement-ci-cd-workflow

# 3. Script ausführen
bash create-jules-issues.sh
```

**Das war's!** Alle 8 Issues werden erstellt. ✅

### Option 2: PR erst mergen (Workflow läuft dann)

```bash
# 1. PR mergen
gh pr merge 10 --squash

# 2. Workflow manuell ausführen
gh workflow run "Create Jules Development Issues"
```

### Option 3: Manuell über GitHub UI

Falls du die Issues lieber einzeln erstellen willst, sind alle Details im Script `create-jules-issues.sh` enthalten.

## 📋 Die 8 Jules Issues (fertig definiert)

Alle Issues sind **vollständig ausgearbeitet** mit:
- ✅ Detaillierte Beschreibungen
- ✅ Task-Listen
- ✅ Acceptance Criteria
- ✅ Technische Details (Dateipfade, APIs)
- ✅ Related Documentation
- ✅ Richtige Labels (jules-task, Priority, Phase)

### Issue 1: Implement Multi-Window Rendering
**Priority:** Critical | **Phase:** Phase 2
Multi-Fenster-Rendering mit synchronisierter Ausgabe für Multi-Projektor-Setups.

### Issue 2: Implement Frame Synchronization
**Priority:** Critical | **Phase:** Phase 2
Frame-perfekte Synchronisation über alle Ausgabefenster.

### Issue 3: Fix Build System - FreeType Linker Errors
**Priority:** High | **Phase:** Infrastructure
FreeType Linker-Fehler beheben, saubere Multi-Plattform-Builds.

### Issue 4: Complete Still Image Support (PNG, JPG, TIFF)
**Priority:** High | **Phase:** Phase 1
Umfassende Unterstützung für Standbildformate mit Caching.

### Issue 5: Add Animated Format Support (GIF, Image Sequences)
**Priority:** Medium | **Phase:** Phase 1
Unterstützung für animierte Formate und Bildsequenzen.

### Issue 6: Add ProRes Codec Support
**Priority:** Medium | **Phase:** Phase 1
Apple ProRes Codec-Varianten für professionelle Videos.

### Issue 7: Advanced Geometric Correction Tools
**Priority:** Medium | **Phase:** Phase 2
Keystone-Korrektur, Mesh-Warping, Bezier-Kurven für Projection Mapping.

### Issue 8: Implement Output Configuration Persistence
**Priority:** Medium | **Phase:** Phase 2
Projektdatei-Format zum Speichern/Laden von Konfigurationen.

## 🚀 Empfohlener Workflow

```bash
# 1. Script ausführen (erstellt alle Issues)
bash create-jules-issues.sh

# 2. Issues prüfen
gh issue list --label "jules-task"

# 3. Jules API konfigurieren
#    - Repository: MrLongNight/MapFlow
#    - Monitor Label: jules-task
#    - PR Label: jules-pr

# 4. Jules arbeitet an den Issues! 🤖
```

## 💡 Warum ein Script?

**Vorteile:**
- ✅ Funktioniert sofort (keine Permission-Probleme)
- ✅ Lokale Kontrolle (du führst es aus)
- ✅ Wiederholbar (falls Issues gelöscht werden)
- ✅ Transparent (du siehst was erstellt wird)

**Alternative Ansätze und warum sie nicht funktionieren:**
- ❌ Workflow auf PR: Keine `issues: write` Permission
- ❌ Copilot direkt: Keine GitHub API Credentials
- ❌ GitHub Actions auf Branch: Sicherheitseinschränkung

## 🔍 Script Inhalt

Das Script `create-jules-issues.sh` enthält:
- 8 `gh issue create` Befehle
- Vollständige Issue-Beschreibungen
- Alle Labels und Metadata
- Error Handling
- Status-Ausgabe

**Sicher und überprüfbar!** Du kannst den Script-Inhalt vor der Ausführung lesen.

## ✨ Nächste Schritte

1. **JETZT:** `bash create-jules-issues.sh` ausführen
2. **Dann:** Jules API konfigurieren
3. **Fertig:** Jules kann loslegen! 🎉

---

**Erstellt:** 2024-12-04
**Status:** Script fertig, ready to execute
**Datei:** `create-jules-issues.sh` (im Repository root)
