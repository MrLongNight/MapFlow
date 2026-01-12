## 2026-01-20 - [Transform/Mesh Coordinate Mismatch]

**Erkenntnis:** Eine Diskrepanz zwischen der Mesh-Definition (0..1 Top-Left origin) und der `Transform::to_matrix` Logik (die ein zentriertes Mesh annahm) führte dazu, dass Rotationen um den Ankerpunkt (default 0.5) fälschlicherweise um die Ecke (0,0) stattfanden.
Die Transform-Logik wurde korrigiert, um den Pivot basierend auf dem Anker relativ zur Top-Left-Ecke (0,0) zu berechnen: `pivot = size * anchor`.

**Aktion:** Bei zukünftigen Implementierungen von Rendering-Logik immer das Koordinatensystem (Normalized vs Pixel, Centered vs Top-Left) explizit validieren. Neue Tests sollten Transformationen mit nicht-trivialen Ankern prüfen.

## 2026-01-24 - [AppState Serialization Guard]

**Erkenntnis:** `AppState` ist die Source-of-Truth und wächst schnell. Ein einfacher Roundtrip-Test (`serde_json`) fängt fehlende `Default`-Implementierungen oder invalide `serde`-Attribute sofort ab, bevor sie zur Laufzeit crashen.

**Aktion:** Bei jedem neuen komplexen Struct, das in `AppState` aufgenommen wird, sicherstellen, dass `Default` und `PartialEq` derived sind, damit der Roundtrip-Test automatisch greift.
