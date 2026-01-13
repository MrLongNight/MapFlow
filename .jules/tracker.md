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

## 2026-01-13 - Fehlender Eintrag für PR #228
**Erkenntnis:** PR #228 (Guardian Tests) war nicht im Changelog, obwohl bereits gemerged. Dies wurde beim Audit entdeckt.
**Aktion:** Tracker hat den Eintrag manuell hinzugefügt und den Roadmap-Stand aktualisiert. Das Muster zeigt, dass PRs oft ohne Changelog-Update gemerged werden.

## 2026-01-09 - Dokumentationslücke entdeckt
**Erkenntnis:** Zwischen dem 2026-01-02 und 2026-01-09 wurden ~8 wichtige Änderungen (PRs #205, #207, #210, #212, #213, #215 sowie direkte UI-Features) gemerged, aber nicht im CHANGELOG.md verzeichnet.
**Aktion:** Tracker hat einen Audit durchgeführt und die fehlenden Einträge basierend auf der Git-Historie rekonstruiert. Zukünftige PRs müssen strikter auf CHANGELOG-Updates geprüft werden.
