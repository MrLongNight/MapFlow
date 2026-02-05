# GitHub Actions & CI/CD - Dokumentationsübersicht

> **Zentrale Übersicht des vollautomatischen CI/CD Workflows (v3.0)**

## 🌐 Workflow-Struktur (Flow-basiert)

Wir setzen auf drei definierte Prozess-Abläufe (`CICD-Flow`), die für alle Nutzergruppen optimiert sind.

### 1. Developer Flow (`CICD-DevFlow`)
Für Antigravity, Jules und Entwickler. Fokus: Qualität & Automatisierung.

*   **`CICD-DevFlow_Job01_Validation.yml`**
    *   **Zweck:** Validiert *jeden* PR und Push auf Main.
    *   **Checks:** Rendering Tests, Unit Tests, Formatting, Security Audit.
    *   **OS:** Linux & Windows.
*   **`CICD-DevFlow_Job02_AutoMerge.yml`**
    *   **Zweck:** Merged PRs *automatisch*, sobald Validation (Job01) grün ist.
    *   **Regel:** Gilt für Jules-PRs und Entwickler-PRs (wenn fehlerfrei).

### 2. Issue Flow (`CICD-IssueFlow`)
Vollautomatische Bearbeitung von User-Issues durch Jules.

*   **`CICD-IssueFlow_Job01_SessionTrigger.yml`**
    *   **Trigger:** Label `jules-task` auf einem Issue.
    *   **Aktion:** Startet eine Jules Session mit dem Issue-Kontext.
*   **`CICD-IssueFlow_Job02_SessionMonitor.yml`**
    *   **Zweck:** Überwacht die aktive Session und meldet Fertigstellung (via PR).

### 3. Main Flow (`CICD-MainFlow`)
Maintenance und Release-Management auf dem `main` Branch.

*   **`CICD-MainFlow_Job01_Changelog.yml`**: Schließt Issues nach Merge und pflegt Changelog.
*   **`CICD-MainFlow_Job02_Backup.yml`**: Tägliche Backups.
*   **`CICD-MainFlow_Job03_Release.yml`**: Erstellt Releases.
*   **`CICD-MainFlow_Job04_SyncLabels.yml`**: Synchronisiert GitHub Labels.

---

## 🚀 Schnellstart für Entwickler

1.  **Lokal arbeiten:**
    *   Führe vor jedem Commit das Skript aus:
        *   `./scripts/Final-Prepare-PreCommit.ps1` (Windows)
        *   `./scripts/Final-Prepare-PreCommit.sh` (Linux/Mac)
    *   Dies fixt Formatierung und Linting-Fehler automatisch.

2.  **Pull Request:**
    *   Erstelle einen PR.
    *   `Job01_Validation` läuft los.
    *   Alles grün? -> `Job02_AutoMerge` merged automatisch.

## 🚀 Schnellstart für User (Issues)

1.  Erstelle ein Issue (Bug/Feature).
2.  Admins labeln es mit `jules-task`.
3.  Jules übernimmt: Session -> PR -> Merge -> Done.

---
**Status:** ✅ Produktionsbereit (v3.0 - Flow Architecture)
