# Visual Reference Images

Diese PNG-Dateien sind die Referenzbilder fuer die lokalen visuellen Regressionstests in
`crates/mapmap/tests/visual_capture_tests.rs`.

## Erste Szenarien

- `checkerboard.png`
  - prueft Orientierung, UV-Flip, Farbkanalvertauschungen und harte Kanten auf einer echten Surface
- `alpha_overlay.png`
  - prueft transparentes Rendering und Alpha-Blending in einem sichtbaren Fenster
- `gradient.png`
  - prueft weiche Farbverlaeufe, Capture-Readback und zentrale Markierungen

Zusaetzlich wurden app-nahe Kernszenarien hinzugefuegt, um die wichtigsten sichtbaren Core-Features deterministisch abzubilden:

- `empty_project.png`
  - Main-Window Startzustand mit leerem Testprojekt. Prueft Fensteraufbau, Docking-Zustand, Initial-Rendering und fehlende Panels.
- `test_grid.png`
  - Output-Preview mit Test-Grid. Prueft den sichtbaren Renderpfad bis ins Preview-Fenster.
- `projector_warp.png`
  - Projektor-Output mit Keystone-, Warp- oder Maskenfixture. Findet Geometrie- und Sampling-Regressionsfehler.
- `media_playback.png`
  - Medienwiedergabe mit festem Referenzframe. Prueft Decoding, Texturupload, Present und sichtbare Farb-/Alpha-Ergebnisse zusammen.
- `timeline_step.png`
  - Timeline- oder Automationsschritt mit definierter Vorher/Nachher-Aufnahme. Prueft, dass der sichtbare Zustand wirklich umschaltet.

Diese Faelle liefern mehr Signal als reine Logik- oder offscreen-Tests, weil sie den echten
Fenster-, Swapchain-, Present- und Screenshot-Pfad verwenden. Sie testen den tatsaechlichen
visuellen Output, bei dem minimale Abweichungen durch Hardwareskalierung zulaessig sein koennen
(Toleranz: `CHANNEL_TOLERANCE` <= 2, max mismatch <= 0.1%).

## Referenzbilder bewusst neu erzeugen

Die Referenzbilder duerfen nur dann neu erzeugt werden, wenn eine **bewusste, gewollte visuelle Aenderung** im Render-Code stattgefunden hat.
Sie duerfen niemals einfach ueberschrieben werden, um einen fehlschlagenden Test "gruen" zu machen, ohne die Ursache des Fehlers zu verstehen!

Von der Repo-Wurzel aus:

```powershell
cargo run -p mapmap --bin mapflow_visual_harness --no-default-features -- reference --scenario checkerboard --output crates/mapmap/tests/reference_images/checkerboard.png
cargo run -p mapmap --bin mapflow_visual_harness --no-default-features -- reference --scenario alpha_overlay --output crates/mapmap/tests/reference_images/alpha_overlay.png
cargo run -p mapmap --bin mapflow_visual_harness --no-default-features -- reference --scenario gradient --output crates/mapmap/tests/reference_images/gradient.png
cargo run -p mapmap --bin mapflow_visual_harness --no-default-features -- reference --scenario empty_project --output crates/mapmap/tests/reference_images/empty_project.png
cargo run -p mapmap --bin mapflow_visual_harness --no-default-features -- reference --scenario test_grid --output crates/mapmap/tests/reference_images/test_grid.png
cargo run -p mapmap --bin mapflow_visual_harness --no-default-features -- reference --scenario projector_warp --output crates/mapmap/tests/reference_images/projector_warp.png
cargo run -p mapmap --bin mapflow_visual_harness --no-default-features -- reference --scenario media_playback --output crates/mapmap/tests/reference_images/media_playback.png
cargo run -p mapmap --bin mapflow_visual_harness --no-default-features -- reference --scenario timeline_step --output crates/mapmap/tests/reference_images/timeline_step.png
```

## Lokale visuelle Tests ausfuehren

```powershell
$env:MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR = "artifacts/visual-capture"
cargo test -p mapmap --no-default-features --test visual_capture_tests -- --ignored --nocapture
```

Wenn `MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR` nicht gesetzt ist, landen die Screenshots in einem
temporaeren Ordner unter `%TEMP%`.

Relative Pfade in `MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR` werden gegen die Repo-Wurzel aufgeloest.
