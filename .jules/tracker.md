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

## 2026-01-05 - CI-Fix fehlte im Changelog
**Erkenntnis:** Commit `9df760e` (CI fixes) war nicht im Changelog dokumentiert. Dies ist ein häufiges Muster bei reinen Wartungs-Commits.
**Aktion:** Tracker muss auch Commits mit `fix(ci)` oder `chore` prüfen und sicherstellen, dass sie im Changelog erscheinen (unter "Fixed" oder "Changed" oder "Unreleased"), um Transparenz über Pipeline-Änderungen zu gewährleisten.
