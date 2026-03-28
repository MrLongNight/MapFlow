# FFT Analyzer

**Was es macht:**
Der FFT Analyzer liest das eingehende Audiosignal und zerlegt es in Echtzeit in verschiedene Frequenzbänder (Bass, Mid, High), um audio-reaktive visuelle Effekte zu steuern.

![Screenshot: Das FFT Analyzer Modul isoliert im Workspace](docs/assets/missing/modul-analyzer-ui.png)

**So benutzt du es:**

1. Ziehe das **FFT Analyzer** Modul in den Workspace.
2. Verbinde den gewünschten Frequenzausgang (z.B. **Bass Out**) mit einem Parameter-Eingang eines anderen Moduls (z.B. der **Scale** eines Shape-Moduls).
3. Stelle den Parameter **Gain** auf deinen gewünschten Wert ein, um die Empfindlichkeit anzupassen.

![Screenshot: FFT Analyzer verbunden mit einem Shape-Modul](docs/assets/missing/modul-analyzer-connection.png)

**Tipp 💡:**
> Nutze den **Smoothness** Parameter, um ruckartige Bewegungen bei sehr dynamischen Audio-Tracks abzufedern und weichere visuelle Übergänge zu erzeugen.

**Parameter-Übersicht:**

| Parameter | Beschreibung | Standardwert |
| --- | --- | --- |
| `Gain` | Steuert die Eingangsverstärkung des Audiosignals. | 1.0 |
| `Smoothness` | Dämpft die Reaktionsgeschwindigkeit für weichere Animationen. | 0.5 |
| `Threshold` | Setzt einen Mindestpegel, ab dem das Signal erst durchgelassen wird. | 0.1 |
