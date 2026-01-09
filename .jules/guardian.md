## 2026-01-20 - [Transform/Mesh Coordinate Mismatch]

**Erkenntnis:** Eine Diskrepanz zwischen der Mesh-Definition (0..1 Top-Left origin) und der `Transform::to_matrix` Logik (die ein zentriertes Mesh annahm) führte dazu, dass Rotationen um den Ankerpunkt (default 0.5) fälschlicherweise um die Ecke (0,0) stattfanden.
Die Transform-Logik wurde korrigiert, um den Pivot basierend auf dem Anker relativ zur Top-Left-Ecke (0,0) zu berechnen: `pivot = size * anchor`.

**Aktion:** Bei zukünftigen Implementierungen von Rendering-Logik immer das Koordinatensystem (Normalized vs Pixel, Centered vs Top-Left) explizit validieren. Neue Tests sollten Transformationen mit nicht-trivialen Ankern prüfen.
