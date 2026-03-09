# Spezifikation: GPU-based Fluid Simulation

## 1. Übersicht
Implementierung einer physikalisch basierten Flüssigkeits- und Rauchsimulation (Fluid Dynamics), die vollständig auf der GPU via `wgpu` Compute Shaders läuft. Diese Simulation dient als visueller Effekt-Layer, der auf Video-Bewegung (Optical Flow) oder Audio reagiert.

## 2. Technischer Ansatz: Grid-based Navier-Stokes
Wir verwenden den **Stable Fluids** Algorithmus von Jos Stam, aber portiert auf moderne Compute Shader Architektur.

### 2.1 Simulation Quantities (Fields)
Wir simulieren auf einem 2D-Gitter (z.B. 256x256 oder 512x512). Jede Zelle speichert:
*   **Velocity Field (vec2)**: Die Strömungsrichtung und Geschwindigkeit.
*   **Density Field (float/vec3)**: Die "Farbe" des Rauchs/Nebels (RGB).
*   **Divergence / Pressure (float)**: Temporäre Felder für die Physik-Berechnung.

### 2.2 Simulation Steps (Pro Frame)
Jeder Schritt ist ein eigener Compute-Pass:
1.  **Advection**: Bewegt Dichte und Geschwindigkeit entlang des Geschwindigkeitsfeldes ("Der Wind weht den Rauch").
2.  **Diffuse** (Optional): Viskosität/Unschärfe (oft weggelassen für Performance, da Advection schon leicht unscharf ist).
3.  **Add Forces**: Externe Kräfte hinzufügen (Mauszeiger, Video-Bewegung, Audio-Impuls).
4.  **Project (Pressure Solve)**: Der teuerste Schritt. Macht das Feld divergenzfrei (inkompressibel), damit es wie eine Flüssigkeit wirkt und wirbelt, statt sich nur aufzublähen.
    *   Benötigt einen *Jacobi-Iterator* (ca. 20 Iterationen pro Frame = 20 Shader Dispatches).

## 3. Architektur-Integration

### 3.1 `mapmap-render` Erweiterung
Neues Modul `crates/mapmap-render/src/fluids/`.
*   Verwaltet die Texturen (Ping-Pong Buffer für Read/Write).
*   Baut die Command-Buffer für die Compute-Passes.

### 3.2 Interaktion
Die Simulation soll nicht statisch sein, sondern "leben".
*   **Optical Flow**: Wir berechnen die Differenz zwischen Video-Frame N und N-1. Helle Pixel, die sich bewegen, addieren Velocity in die Simulation. -> *Das Video zieht Rauchschwaden hinter sich her.*
*   **Audio**: Bass-Schläge injizieren Dichte ("Explosionen") oder Turbulenz in die Mitte.

## 4. Performance-Optimierung
*   **Auflösung**: Die Simulation muss nicht in 4K laufen! Ein 512x512 Grid reicht völlig aus und wird dann bilinear hochskaliert (linear texture sampling). Das spart massiv Rechenleistung.
*   **Half-Float**: Nutzung von `R16Float` oder `Rg16Float` Texturen statt 32-Bit, um Bandbreite zu sparen (falls von wgpu supportet).

## 5. UI-Parameter
Der User kann live steuern:
*   **Dissipation**: Wie schnell löst sich der Rauch auf?
*   **Vorticity**: Wie sehr wirbelt es ("Locken"-Bildung)?
*   **Color**: Farbe des Rauchs (oder Übernahme der Videofarben).
*   **Force Scale**: Wie stark beeinflusst das Video den Rauch?

## 6. Risiken
*   **GPU-Last**: Viele Dispatch-Calls können schwache GPUs (Intel UHD) überfordern. -> Fallback-Option oder Deaktivierung auf Low-End Systems.
*   **wgpu Limits**: Compute Shader Support ist gut, aber Limits für Workgroup Sizes müssen beachtet werden (Safe default: 8x8).
