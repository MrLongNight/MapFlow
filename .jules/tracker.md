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

## 2026-01-19 - Conflict Resolution & Missing Features Audit
**Erkenntnis:** Merge-Konflikt in `ROADMAP.md` entdeckt und behoben (Version 1.9.2). Zwei bedeutende Features (Hue Integration, Node Module System) fehlten im Changelog.
**Aktion:** ROADMAP-Status für FFmpeg/CI und Hue aktualisiert. Fehlende Changelog-Einträge für Commits #b8dd83b und #484c78e nachgetragen.
