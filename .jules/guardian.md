## 2026-02-08 - Audio Data Sanitization

**Erkenntnis:** Rohe Audio-Buffer enthalten oft NaNs oder Infinities von Treibern oder leeren Streams. Das Propagieren dieser Werte in FFT- oder RMS-Berechnungen vergiftet die gesamte Analyse-Pipeline, was zu NaNs in Uniform Buffern führt, die GPU-Treiber zum Absturz bringen oder schwarze Bildschirme verursachen können.

**Aktion:** Audio-Input-Buffer immer am Eingangspunkt sanitizen (nicht-finite Werte durch 0.0 ersetzen). `test_sanitization_of_bad_input` wurde zu `AudioAnalyzerV2` hinzugefügt, um dies strikt durchzusetzen.
