# 🧪 Guardian's Journal

Kritische Erkenntnisse aus Unit-Test-Aktivitäten.

---

## Eintragsformat

```
## YYYY-MM-DD - [Titel]
**Erkenntnis:** [Was gelernt]
**Aktion:** [Wie beim nächsten Mal anwenden]
```

---

## 2024-05-23 - Transform Matrix & Koordinatensystem
**Erkenntnis:** Das Transformationssystem (`Transform::to_matrix`) in `mapmap-core` geht davon aus, dass Input-Vertices **zentriert** sind (d.h. `(0,0)` ist die Mitte des Layers). `Position` definiert die Position des **Centers** (plus Offset), während `Anchor` nur den Pivot für Rotation/Skalierung verschiebt, aber nicht die absolute Position des Objekts ändert (Identitäts-Transformation wenn Rot/Scale unverändert).
**Aktion:** Tests für Transformationen müssen `Vec3::ZERO` als "Mitte" und relative Koordinaten (z.B. `-50` bis `+50`) verwenden, nicht `0` bis `100`.

## 2024-05-23 - WGPU Texturgrößenberechnung
**Erkenntnis:** Manuelle Berechnungen für `div_ceil` (z.B. `(width + 3) / 4`) werden von Clippy als Fehler markiert (`clippy::manual_div_ceil`).
**Aktion:** Immer `width.div_ceil(4)` verwenden, um sowohl Korrektheit als auch Linter-Konformität sicherzustellen.
