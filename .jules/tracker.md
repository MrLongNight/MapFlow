# 📋 Tracker's Journal

Kritische Erkenntnisse aus Projektmanagement-Aktivitäten.

---

## Eintragsformat

```
## YYYY-MM-DD - [Titel]
**Erkenntnis:** [Was gelernt]
**Aktion:** [Wie beim nächsten Mal anwenden]
```

---

## 2026-01-18 - Missing Documentation for PR #286
**Erkenntnis:** PR #286 (Archivist Cleanup) was merged and effective (audit reports moved), but missing from `CHANGELOG.md`.
**Aktion:** Added entry to `CHANGELOG.md` and updated `ROADMAP.md` timestamp.

## 2026-01-16 - Systematische Changelog-Lücke (PR #248-#270)
**Erkenntnis:** Trotz des Updates am 15.01. (#252) fehlten fast alle umgebenden PRs (#248-#251, #253-#270) im CHANGELOG. Das deutet auf manuelle Merges ohne Changelog-Pflege hin.
**Aktion:** Tracker hat 19 fehlende Einträge rekonstruiert und ROADMAP.md synchronisiert.

## 2026-01-15 - Massiver PR #252 ohne Dokumentation
**Erkenntnis:** PR #252 (feat/stateful-triggers) war mit 500+ Dateien (Icons, Scripts, Core-Features) ein massives Update, wurde aber komplett im Changelog vergessen.
**Aktion:** Tracker hat die Einträge rekonstruiert und in Gruppen aufgeteilt (feat, assets, chore). Solche großen PRs erfordern besondere Aufmerksamkeit beim Review.

## 2026-01-13 - Fehlender Eintrag für PR #228
**Erkenntnis:** PR #228 (Guardian Tests) war nicht im Changelog, obwohl bereits gemerged. Dies wurde beim Audit entdeckt.
**Aktion:** Tracker hat den Eintrag manuell hinzugefügt und den Roadmap-Stand aktualisiert. Das Muster zeigt, dass PRs oft ohne Changelog-Update gemerged werden.

## 2026-01-09 - Dokumentationslücke entdeckt
**Erkenntnis:** Zwischen dem 2026-01-02 und 2026-01-09 wurden ~8 wichtige Änderungen (PRs #205, #207, #210, #212, #213, #215 sowie direkte UI-Features) gemerged, aber nicht im CHANGELOG.md verzeichnet.
**Aktion:** Tracker hat einen Audit durchgeführt und die fehlenden Einträge basierend auf der Git-Historie rekonstruiert. Zukünftige PRs müssen strikter auf CHANGELOG-Updates geprüft werden.

## 2026-01-20 - Roadmap Synchronization
**Erkenntnis:** Discrepancies between implemented features (NDI, Hue) and Roadmap status (Planned/Empty).
**Aktion:** Updated `ROADMAP_2.0.md` to reflect active NDI and Hue features based on codebase analysis.

## 2026-01-19 - Conflict Resolution & Missing Features Audit
**Erkenntnis:** Merge-Konflikt in `ROADMAP.md` entdeckt und behoben (Version 1.9.2). Zwei bedeutende Features (Hue Integration, Node Module System) fehlten im Changelog.
**Aktion:** ROADMAP-Status für FFmpeg/CI und Hue aktualisiert. Fehlende Changelog-Einträge für Commits #b8dd83b und #484c78e nachgetragen.

## 2026-01-30 - Missing PR Link for #410
**Erkenntnis:** PR #410 (docs: Fix broken links) was in CHANGELOG but missing the PR number link. Roadmap "Stand" was outdated (20th vs 30th).
**Aktion:** Added (#410) to CHANGELOG entry and updated ROADMAP Stand date to 2026-01-30.

## 2026-01-26 - CI/CD & Packaging Fixes Documentation
**Erkenntnis:** Several CI/CD and Installer fixes were merged and documented in CHANGELOG.md but missing from ROADMAP_2.0.md.
**Aktion:** Updated ROADMAP_2.0.md to reflect the completion of FFmpeg integration in CI, pre-checks hardening, and WiX installer fixes.

## 2026-01-30 - ROADMAP Conflict Resolution
**Erkenntnis:** Merge-Konflikte in `ROADMAP_2.0.md` entdeckt (HEAD vs. Incoming Status für Windows Installer).
**Aktion:** Konflikte behoben, `Stand` aktualisiert, und Windows Installer Status konsolidiert (Completed + Detailed Checklist).

## 2026-02-07 - Undocumented PRs Discovery (Feb 6)
**Erkenntnis:** Discovered multiple merged PRs from 2026-02-06 (#589, #588, #584, #585) missing from CHANGELOG.md. Also corrected FUTURE DATE in ROADMAP.md (was 2026-02-15).
**Aktion:** Added missing entries to CHANGELOG.md and corrected ROADMAP.md Stand date to current date.

## 2026-02-10 - Discrepancy in PR Reference for Bevy Particles
**Erkenntnis:** CHANGELOG referenced PR #638 for "Bevy Particles", but git history shows merged commit 52bf7e7 is linked to PR #650.
**Aktion:** Corrected CHANGELOG entry to point to #650 and updated ROADMAP to reflect the new feature implementation details.

## 2026-02-16 - Squash Commit Discovery & Feature Verification
**Erkenntnis:** Discovered a massive squash commit (e1e8f37) adding 600+ files. Git history prior to this is unavailable in the current environment.
**Aktion:** Manually verified implementation of "Resize Drag", "Mesh Editor for Layers", "MIDI Selector", and "Audio Bands" in `module_canvas/mod.rs` and updated ROADMAP/CHANGELOG accordingly to reflect the current state.
