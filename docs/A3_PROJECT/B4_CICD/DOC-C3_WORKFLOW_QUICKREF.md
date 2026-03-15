# CI/CD Workflow Quick Reference

> **Kompakte Übersicht der verfügbaren Steuerungsbefehle**

## 🚀 Workflows manuell starten

Befehle für die `gh` CLI (GitHub CLI):

| Ziel | Befehl |
|------|--------|
| **Vollständiger Check** | `gh workflow run "CI/CD"` |
| **Nur Linux Build** | `gh workflow run "CI/CD" -f skip_platforms=true` |
| **Build ohne Tests** | `gh workflow run "CI/CD" -f skip_tests=true` |
| **Schnellster Check** | `gh workflow run "CI/CD" -f skip_platforms=true -f skip_tests=true` |
| **Security Scan** | `gh workflow run "CodeQL"` |
| **Labels syncen** | `gh workflow run "Sync Labels"` |

## ⚙️ Einstellungen in Workflow-Dateien

Dateipfade relativ zum Repository-Root:

### Auto-Merge (Jules)
Datei: `.github/workflows/CI-05_pr-automation.yml`
```yaml
env:
  AUTO_MERGE_ENABLED: true # false zum Deaktivieren
```

### CodeQL PR-Scans
Datei: `.github/workflows/CI-02_security-scan.yml`
```yaml
env:
  SCAN_ON_PR_ENABLED: true # false zum Deaktivieren
```

## 🛠 Lokale Vorbereitung (vor Push)

Führe diese Befehle aus, um CI-Fehler zu vermeiden:

```bash
# 1. Code formatieren
cargo fmt

# 2. Linting
cargo clippy --all-targets --all-features -- -D warnings

# 3. Tests
cargo test

# 4. Alles zusammen (Empfohlen)
cargo fmt && cargo clippy && cargo test
```

## 📋 Status-Check Legende

In der GitHub PR Ansicht:

- 🟢 **Success:** Alle Checks bestanden. Merge bereit.
- 🟡 **Pending:** Checks laufen noch (Dauer ca. 5-15 Min).
- 🔴 **Failure:** Mindestens ein Check fehlgeschlagen. Fix erforderlich.
- ⚪ **Skipped:** Check wurde manuell oder durch Filter übersprungen.

## 🔍 Wichtige Log-Dateien

Wenn der Build in CI fehlschlägt:

1. Klicke auf **Details** neben dem fehlgeschlagenen Check.
2. Suche im Log nach:
   - `error:` (Compiler Fehler)
   - `FAILED` (Test Fehler)
   - `warning:` (Clippy/Lint Warnungen)

## 🔗 Links

- **CI/CD README:** [CI_CD_README.md](DOC-C1_README_CICD.md)
- **Workflow Details:** [workflows/README.md](../../../.github/workflows/README.md)
- **Roadmap:** [ROADMAP.md](../../project/roadmap/README.md)

## Self-hosted Post-Merge Quickref

- Workflow:
  - `.github/workflows/CICD-DevFlow_Job03_PostMergeSelfHosted.yml`
- Hauptschalter:
  - `MAPFLOW_ENABLE_SELF_HOSTED_POST_MERGE=true`
- optional spaeter:
  - `MAPFLOW_SELF_HOSTED_RUN_IGNORED_GPU_TESTS=true`
  - `MAPFLOW_SELF_HOSTED_RUN_VISUAL_AUTOMATION=true`
- schnell global pausieren:
  - `MAPFLOW_ENABLE_SELF_HOSTED_POST_MERGE=false`
- einzelnen PR ausnehmen:
  - Label `skip-self-hosted-post-merge`

---

**Stand:** 2024-12-04
**Version:** 1.0
