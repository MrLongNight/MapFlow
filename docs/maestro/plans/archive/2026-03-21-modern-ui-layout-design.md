---
design_depth: deep
task_complexity: complex
topic: modern-ui-layout
date: 2026-03-21
---

# Design-Dokument: Moderne & Modulare Mapflow-UI

## 1. Problemstellung (Problem Statement)
Die aktuelle Mapflow-UI basiert auf einem starren, statischen System (Sidebar links, Timeline unten, Inspector rechts), das modernen kreativen Workflows nicht gerecht wird.

**Kernprobleme:**
- **Mangelnde Dynamik**: Fehlende Animationen bei Layout-Änderungen erschweren die visuelle Orientierung.
- **Inflexible Inspector-Geometrie**: Der Inspector nutzt den Platz bei breiten Bildschirmen ineffizient ("zu breit").
- **Eingeschränktes Workspace-Konzept**: Keine Möglichkeit, Panels frei zwischen Grundbereichen (Slots) zu verschieben.
- **Node-Editor UX**: Standard-Skins und starre Interaktionen (Socket-Erkennung) bremsen den kreativen Flow.

## 2. Anforderungen (Requirements)

### 2.1 Layout-Flexibilität (Slot-System)
- **REQ-2.1.1**: Einführung von 5 Zonen (Top, Bottom, Left, Right, Center) als Container für UI-Module.
- **REQ-2.1.2**: Drag & Drop Funktionalität zum Verschieben von Panels zwischen Slots im "Extended UI Mode".
- **REQ-2.1.3**: Unterstützung von Layout-Templates (VJing, Mapping, Shader) inkl. Hotkey-Umschaltung.
- **REQ-2.1.4**: Animierter Auto-Collapse für leere Slots.

### 2.2 Inspector-Ergonomie (Adaptive Widgets)
- **REQ-2.2.1**: Multi-Column Grid-System für Parameter-Widgets (automatischer Umbruch ab >400px Breite).
- **REQ-2.2.2**: Widget-Density Skalierung (80% - 150%) zur individuellen Anpassung der Steuerelement-Größe.
- **REQ-2.2.3**: Persistente Speicherung der Layout-Präferenzen pro Panel-Typ.

### 2.3 Visuelle Qualität (Modern & Fluid UX)
- **REQ-2.3.1**: Zentrale Lerp-Transition-Engine für alle Geometrie-Änderungen (Slot Resize, Tab-Switch).
- **REQ-2.3.2**: Visuelles Feedback (Hover-Effekte, Drop-Zone-Highlights).
- **REQ-2.3.3**: Behebung der Socket-Erkennungs-Probleme im Node-Editor (Audit-Report Ref: 560).

### 2.4 Node-Ästhetik (Custom Node Design)
- **REQ-2.4.1**: Data-Driven Skinning-System für individuelle visuelle Designs pro Node-Typ (JSON/YAML).
- **REQ-2.4.2**: Interaktive Flow-Animationen zur Visualisierung des Datenflusses auf den Verbindungsleitungen.
- **REQ-2.4.3**: Zustands-Animationen (Pulsieren, Glow) für selektierte oder aktive Nodes.

## 3. Architektur-Ansatz (Approach)

### 3.1 Unified Slot & Node Engine
Wir implementieren ein **Molares Slot-System**, das Inhalte (Panels) von ihrer Darstellung (Slots) trennt. Eine universelle Schnittstelle (`AppModule`) erlaubt es jedem Panel, in jedem Slot zu existieren.

### 3.2 Entscheidungsmatrix (Deep Mode)

| Kriterium | Gewicht | Slot-based (Modular) | Begründung |
|-----------|--------|----------------------|------------|
| **Flexibilität** | 40% | 5/5 | Volle Kontrolle über Workspace-Belegung. |
| **UX-Modernität** | 30% | 5/5 | Animationen & Adaptive Widgets wirken professionell. |
| **Aufwand** | 20% | 2/5 | Hoher Refactor-Aufwand in `mapflow-ui`. |
| **Stabilität** | 10% | 4/5 | Bewährte UI-Muster aus DCC-Tools. |
| **Summe** | | **4.6** | |

### 3.3 Technische Entscheidungen
- **Format**: JSON für Layout-Profile & Node-Skins (Data-Driven Workflow).
- **Animationen**: Shader-basierte Glow-Effekte (WGSL) und Lerp-Transitionen in Rust.
- **Hot-Reloading**: Skin-Änderungen werden in Echtzeit ohne Neukompilierung geladen.

## 4. Komponenten-Design (Interfaces)
- **`SlotManager`**: Zentrale Geometrie-Verwaltung der 5 Zonen.
- **`AnimatedPanel`**: Wrapper für flüssige Positions- & Größenänderungen.
- **`WidgetFactory`**: Skalierbare UI-Elemente basierend auf `DensityScale`.
- **`SkinLoader`**: Schnittstelle zu den externen Node-Design-Dateien.

## 5. Agent Team & Phasen-Plan

### Agenten-Team
- `architect`: Slot-Core & Refactoring.
- `ux_designer`: Adaptive Grids & Widget-Skalierung.
- `design_system_engineer`: Skin-System & Node-Integration.
- `performance_engineer`: Animation-Engine & Shader-Optimierung.

### Phasen
1. **Phase 1: Foundations**: Slot-Manager & Layout-Tree (Lerp-Animationen).
2. **Phase 2: Ergonomics**: Adaptive Inspector-Grids & Widget-Density.
3. **Phase 3: Aesthetics**: SkinLoader & Custom Node-Skins (Flow-Animationen).
4. **Phase 4: Polish**: UI-Cleanup (Audit-Fixes) & Performance-Optimierung.

## 6. Risiko-Einschätzung (Risk Assessment)
- **egui Performance**: Zu viele gleichzeitige Animationen könnten FPS senken (Lösung: Shader-Overlays).
- **Skin-Komplexität**: Komplexe Vektor-Shapes schwer umsetzbar (Lösung: SVG-Parsing oder Pre-Rendering).
- **Abwärtskompatibilität**: Alte Layout-Configs brauchen Migrations-Script.

## 7. Erfolgs-Kriterien (Success Criteria)
- **SC-1**: Layout-Wechsel in <300ms mit flüssiger Animation.
- **SC-2**: Automatische Spalten-Adaption im Inspector (>400px = 2 Spalten).
- **SC-3**: Stufenlose Skalierung der Widgets (80% - 150%) ohne Layout-Fehler.
- **SC-4**: Individuelle Node-Skins und aktive Kabel-Animationen im Graph.

---
*Future Feature Reminder: Performance Checkup (Hardware-Benchmark).*
