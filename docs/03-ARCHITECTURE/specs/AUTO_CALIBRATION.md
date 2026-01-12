# Spezifikation: Kamera-basierte Beamer-Kalibrierung & Smart Mapping

## 1. Übersicht
Ziel ist die Implementierung einer automatischen Kalibrierung ("Auto-Mapping"), bei der eine Kamera die Projektionsfläche scannt und der Output von MapFlow automatisch an die Geometrie angepasst wird. Dies ermöglicht "Smart 3D Mapping", bei dem Inhalte präzise auf physische Objekte gemappt werden, ohne manuell Punkte ziehen zu müssen.

## 2. Technischer Ansatz: Strukturiertes Licht (Structured Light)
Wir verwenden das **Gray-Code** Verfahren (eine Form von Structured Light Scanning).

*   **Prinzip**: Sequentielle Projektion von s/w-Streifenmustern.
*   **Vorteil**: Gray-Codes ändern sich zwischen benachbarten Werten nur um 1 Bit. Das macht das Decoding robuster gegen leichte Fehlalignments oder Unschärfe als reine Binärcodes.
*   **Ablauf**:
    1.  Serie von Bildern mit horizontalen/vertikalen Balken projizieren.
    2.  Kamera nimmt jedes Bild auf.
    3.  Für jeden Kamerapixel wird aus der Helligkeitsfolge (hell/dunkel über die Zeit) die Zeile und Spalte des Beamers ermittelt, die diesen Punkt beleuchtet hat.

## 3. Architektur & Stack

### 3.1 Kamera-Zugriff (Video Capture)
Wir vermeiden schwere Abhängigkeiten wie OpenCV, um die Portabilität und Build-Zeiten von MapFlow zu schonen.

*   **Library**: `nokhwa` (https://crates.io/crates/nokhwa)
    *   *Features*: `input-native` (nutzt MediaFoundation auf Windows, V4L2 auf Linux, AVFoundation auf macOS).
*   **Integration**:
    *   Neues Crate (empfohlen): `crates/mapmap-vision`
    *   API-Wrapper um `nokhwa`, der Frames als `image::RgbaImage` oder Rohdaten liefert.

### 3.2 Bildverarbeitung (Computer Vision)
Die Verarbeitung erfolgt "Native Rust".

*   **Pattern Generation**:
    *   Ein neuer Modus im Renderer (`mapmap-render`), der statt User-Content mathematische Muster generiert.
*   **Decoding Engine**:
    *   Input: Serie von Kamerabildern.
    *   Prozess:
        *   **Diff-Bildung**: `(Bild_Pattern - Bild_Inverse)` für maximalen Kontrast und Eliminierung von Umgebungslicht.
        *   **Thresholding**: Entscheidung ob Bit 0 oder 1.
        *   **Reconstruction**: Zusammensetzen der Bits zu X/Y-Koordinaten (Projector-Space).
    *   Output: `Correspondence Map` (Sparse Map: Welcher Kamera-Pixel entspricht welchem Beamer-Pixel).
*   **Geometrie-Solver**:
    *   Für ebene Flächen (2D): Berechnung einer Homographie-Matrix (Projective Transformation) mittels RANSAC oder Least-Squares (SVD).
        *   Library: `nalgebra` oder `glam`.
    *   Für 3D-Objekte (Mesh warping):
        *   Erzeugung eines Gitters (Warp Mesh) basierend auf den gefundenen Korrespondenzen.

## 4. Workflow (User Experience)

### Phase 1: Setup
1.  **Hardware**: Beamer und Kamera aufstellen und verbinden.
2.  **View**: In MapFlow (neues Panel "Calibration") den Kamera-Input aktivieren.
3.  **Ausrichtung**: Sicherstellen, dass die Kamera das gesamte Projektionsbild sieht.

### Phase 2: Scan (Der "Wizard")
Der User startet den "Auto-Calibrate" Prozess.

1.  **Initialisierung**:
    *   Messen des "Schwarzbildes" (Umgebungslicht).
    *   Messen des "Weißbildes" (Maskierung: Wo ist überhaupt Projektion sichtbar?).
2.  **Pattern Loop**:
    *   Loop durch Gray-Codes (Horizontal & Vertikal).
    *   **Wichtig**: Synchronisation. Da wir keinen Hardware-Trigger haben, muss die Software konservativ warten (z.B. 500ms), bis der Beamer das Bild stabil zeigt und die Kamera-Autofokus/Belichtung sich beruhigt hat (oder besser: Auto-Exposure locken).
3.  **Feedback**: Progress Bar und Anzeige der aktuell erkannten Features.

### Phase 3: Resultat & Finetuning
1.  Das System berechnet das Warping.
2.  Preview des Ergebnisses (z.B. ein Gittermuster projizieren).
3.  User kann das Ergebnis akzeptieren (speichert als Preset) oder manuell nachjustieren.

## 5. Datenstrukturen (Entwurf)

```rust
// crates/mapmap-vision/src/lib.rs

/// Repräsentiert eine Kalibrierungs-Sitzung
pub struct CalibrationSession {
    pub camera_config: CameraConfig,
    pub pattern_sequence: Vec<PatternType>,
    pub captured_frames: Vec<CapturedFrame>,
}

/// Ergebnis der Analyse
pub struct CalibrationResult {
    /// Mapping von Kamera-Koordinaten zu Projektor-Koordinaten (Normalisiert 0.0 - 1.0)
    pub correspondences: Vec<(Vec2, Vec2)>,
    /// Berechnete Transformationsmatrix (für Quad-Warping)
    pub homography: Option<Mat3>,
    /// Generiertes Mesh (für Grid-Warping)
    pub warp_mesh: Option<MeshGrid>,
}
```

## 6. Implementierungs-Plan

### Schritt 1: `mapmap-vision` Crate Setup
- Erstellen des Crates.
- Einbinden von `nokhwa` und `image`.
- Implementieren von `CameraInput` (Open, Stream, Capture Frame).

### Schritt 2: Pattern Generator
- Erweitern von `mapmap-render` um `PatternRenderer`.
- Implementieren der Gray-Code Logik (rekursive Generierung der Streifenmuster).

### Schritt 3: UI Integration (Vorstufe)
- Ein "Camera View" Fenster in der GUI (ImGui Viewport), das den Live-Feed zeigt.

### Schritt 4: Die algorithmische Pipeline
- Offline-Entwicklung der Decoding-Logik (kann gut mit Test-Bildern entwickelt werden, ohne echten Beamer).
- Algorithmus zur Berechnung der Homographie (SVD solver).

### Schritt 5: Der "Wizard" State Machine
- Implementierung des asynchronen Prozesses:
  `Projizieren -> Warten -> Grabben -> Speichern -> Nächstes Pattern`.

## 7. Risiken & Mitigation
*   **Problem**: Beamer/Kamera Latenz ist unbekannt.
    *   *Lösung*: "Safe Mode" mit langen Wartezeiten (1s) vs. "Fast Mode".
*   **Problem**: Umgebungslicht stört.
    *   *Lösung*: Differenzbilder (Positiv - Negativ Pattern) nutzen, um konstantes Umgebungslicht mathematisch zu eliminieren.
*   **Problem**: Kamera-Verzerrung (Linseneffekt).
    *   *Lösung*: `nokhwa` oder `opencv` nutzen, um erst die Kamera selbst zu kalibrieren (Checkerboard), ist aber Step 2. Für den Anfang ignorieren wir Linsenverzerrung oder nehmen eine gute Webcam an.
