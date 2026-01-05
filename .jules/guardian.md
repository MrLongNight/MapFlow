# 🧪 Guardian's Journal

Kritische Erkenntnisse aus Unit-Test-Aktivitäten.

---

## Eintragsformat

```
## YYYY-MM-DD - [Titel]
**Erkenntnis:** [Was gelernt]
**Aktion:** [Wie beim nächsten Mal anwenden]
```

---

## 2026-01-24 - ModulePartType Socket-Generation Komplexität
**Erkenntnis:** `ModulePartType::Trigger(TriggerType::AudioFFT)` hat eine komplexe Socket-Generierungslogik, die von der `AudioTriggerOutputConfig` abhängt. Mein neuer Test `test_audio_trigger_sockets` hat aufgedeckt, dass standardmäßig 10 Outputs (9 Bänder + 1 Beat) generiert werden, wenn Frequenzbänder aktiviert sind.
**Aktion:** Bei zukünftigen Änderungen an `AudioTriggerOutputConfig` müssen die Tests in `module_logic_tests.rs` angepasst werden, da sie stark an diese Implementierung gekoppelt sind.

## 2026-01-24 - LinkMode Socket-Dynamik
**Erkenntnis:** `ModulePartType::Source` hat standardmäßig 1 Input, aber der `LinkMode` (Master/Slave) fügt dynamisch weitere Sockets hinzu ("Link Out", "Trigger In (Vis)"). Dies war bisher ungetestet.
**Aktion:** Tests für Module müssen immer verschiedene `LinkMode`-Zustände prüfen, da sich die Socket-Topologie ändert.
