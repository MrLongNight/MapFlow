# 🎮 Bevy Advanced Integration & Extensions

Diese Dokumentation beschreibt die Vision und den aktuellen Stand der Bevy-Integration in MapFlow. Statt einer monolithischen "Bevy Scene" verfolgen wir einen modularen Ansatz, bei dem spezialisierte Bevy-Nodes nahtlos in den MapFlow-Graph integriert werden.

## 🚀 Die Vision: Modulares 3D-Compositing
Jede Bevy-Extension wird als spezialisierter Node-Typ exponiert. Dies ermöglicht es VJs, komplexe 3D-Szenen prozedural im Graph aufzubauen, ohne Bevy-Code schreiben zu müssen.

## 🛠 Geplante & Integrierte Nodes

| Node Typ | Extension | Status | Beschreibung | Funktionen |
| :--- | :--- | :--- | :--- | :--- |
| **🌌 Atmosphere** | `bevy_atmosphere` | ✅ Aktiv | Prozeduraler Himmel & Licht. | Sun-Pos, Turbidity, Sky-Color. |
| **⬡ Hex Grid** | `hexx` | ✅ Aktiv | Prozedurale hexagonale Strukturen. | Radius, Rings, Audio-Scale, Rotation. |
| **✨ Particles** | `bevy_enoki` | 🔄 Dev | GPU-beschleunigte Partikel. | Spawn-Rate, Lifetime, Velocity, Attractors. |
| **🧊 3D Shapes** | Native (Mesh3d) | ✅ Aktiv | Einfache geometrische Primitive. | Cube, Sphere, Capsule, Torus, Unlit. |
| **🎨 PostFX** | `bevy_mod_outline` | ✅ Aktiv | Mesh-Outlines & Border FX. | Width, Glow, Edge-Detection. |
| **👆 Interaction** | `bevy_picking` | ⬜ Planned | Klickbare 3D-Elemente im Canvas. | Trigger-Emission auf Click. |

## 📐 Architektur: Wie es funktioniert
1.  **Shared Engine**: MapFlow startet eine einzige Bevy-Instanz im Hintergrund.
2.  **Node-to-Entity Mapping**: Jeder Bevy-Node im MapFlow-Graph entspricht einer Entity oder einer Gruppe von Entities in der Bevy-World.
3.  **Parameter-Sync**: Änderungen an Node-Slidern werden in Echtzeit als Bevy-Resources oder Components an die Engine übertragen.
4.  **Audio-Link**: MapFlow's FFT-Daten werden direkt als `AudioInputResource` in Bevy eingespeist, wo sie von spezialisierten Systemen (z.B. für das Hex-Grid) verarbeitet werden.

## 📖 Node-Details

### Bevy Hex Grid
Das Hex-Grid ist ideal für techno-visuelle Hintergründe.
- **Audio-Modus**: Einzelne Kacheln pulsieren basierend auf den FFT-Bändern (Bass = Mitte, High = Rand).
- **Prozedural**: Anpassbare Grid-Formen (Hexagon, Kreis, Rechteck).

### Bevy Particles
Ein extrem performantes System für tausende Partikel.
- **Trigger**: Verknüpfe den "Spawn"-Eingang mit einem Beat-Trigger für synchrone Bursts.
- **Fields**: Definiere Gravitationsfelder oder Wind im 3D-Raum via Slider.

### Bevy 3D Shapes
Einfache geometrische Primitive für schnelles Prototyping oder minimalistische Designs.
- **Formen**: Würfel, Kugel, Kapsel, Torus, Zylinder, Ebene.
- **Material**: Wähle zwischen Standard-Shading (Licht-reaktiv) oder Unlit (reine Farbe).

## ⚠️ Inkompatible Extensions
- `bevy-vfx-bag`: Veraltet (Bevy 0.10). Wir implementieren eigene Shader-Nodes basierend auf WGPU.
- `bevy-ui-gradients`: Inkompatibel mit 0.14. Native MapFlow UI-Gradients werden bevorzugt.

## 🔜 Nächste Schritte
1.  **Node Expansion**: Implementierung der `SourceType` Varianten in `mapflow-core`.
2.  **UI-Integration**: Erstellen von spezialisierten Inspector-Panels für jeden Bevy-Node.
3.  **Renderer-Optimierung**: Verbessertes Readback der Bevy-Texture in den MapFlow-Main-Renderer.
