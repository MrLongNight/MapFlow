# DOC-C1: Audio Subsystem

## 1. Engine-Anbindung
Vorce nutzt das `cpal` Crate für plattformübergreifenden Audio-Input.

## 2. Analyse-Pipeline
1.  **Sampling**: Abgreifen der Rohdaten vom System-Input.
2.  **FFT (Fast Fourier Transform)**: Umwandlung in den Frequenzbereich.
3.  **Band-Extraktion**: Aufteilung in 9 Bänder (SubBass bis Air).
4.  **Beat Detection**: Statistische Analyse von Pegelspitzen zur Tempo-Ermittlung.

## 3. Reaktivität
Die berechneten Werte (RMS, Peak, Band-Energie) werden dem `ModuleEvaluator` bereitgestellt und können via Node-Graph auf beliebige Parameter gemappt werden.
