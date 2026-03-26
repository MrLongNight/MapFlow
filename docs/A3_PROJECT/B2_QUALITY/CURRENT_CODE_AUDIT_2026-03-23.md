# Aktueller Code-Audit-Bericht (März 2026)

**Datum:** 23.03.2026
**Methode:** Isolierte Codebasis-Analyse durch autonome Subagents (Architektur-Investigator & Code-Qualitäts-Generalist). Bestehende Dokumentationen oder alte Berichte wurden für dieses Audit explizit ignoriert.

---

## 1. Architektonische Struktur & Crate-Ökosystem

### 1.1 Modulare Aufteilung
Das Projekt ist als Rust-Workspace organisiert und zeigt eine klare funktionale Trennung:
*   **`vorce-core`**: Fundamentale Geschäftslogik, Audio-Analyse und mathematische Modelle. Weist fast keine externen Abhängigkeiten auf.
*   **`vorce-render`**: Implementierung des WGPU-Renderings. Verwaltet die WGSL-Shader (`/shaders`).
*   **`vorce-ui`**: Egui-basiertes Interface. Stark modularisiert durch Panels und Editoren (z. B. Node-Editor).
*   **`vorce-bevy`**: ECS-Integration (Bevy 0.16) für komplexe 3D-Szenen und spezialisierte Simulationen.

### 1.2 Kritische Architektur-Risiken
*   **WGPU-Versionskonflikt (Blocker)**: `vorce-bevy` nutzt **WGPU 24.0**, während der restliche Workspace (insbesondere `vorce-render` und `vorce-ui`) auf **WGPU 27.0** ausgerichtet ist. Dies führt zu potenziellen Linker-Konflikten, inkompatiblen Grafik-Primitiven und massiven Problemen bei der Ressourcenübergabe.
*   **Engine-Isolation**: Bevy wird als isolierte Engine betrieben. Der Datenaustausch von Frame-Daten zwischen Bevy und dem Standard-Render-Pfad erfordert teure **GPU-CPU-GPU Kopien**. Dies stellt einen massiven Performance-Flaschenhals dar, insbesondere bei hohen Auflösungen (4K).

---

## 2. Code-Qualität & Sicherheit

### 2.1 Fehlerbehandlung (Error Handling)
*   **Inkonsistenz**: Es existiert eine Mischung aus `anyhow` (für die App-Ebene) und `thiserror` (für Bibliotheken).
*   **Anti-Patterns**: In `vorce-control` werden Fehler häufig nur geloggt (`error!`), anstatt sie strukturiert an den Aufrufer zurückzugeben. Dies kann in asynchronen Tasks zu "Silent Failures" führen (z.B. bei Verbindungsabbrüchen), bei denen die UI den Benutzer nicht informiert.

### 2.2 Sicherheit & Unsafe Code
*   **FFI-Risiken**: `unsafe`-Blöcke konzentrieren sich auf die FFI-Grenzen (FFmpeg, NDI, Spout).
*   **Fehlende Dokumentation & Validierung**: In `hap_player.rs` und `decoder.rs` fehlen die obligatorischen `// SAFETY:` Kommentare fast vollständig. Es wurden Codestellen identifiziert, an denen rohe Pointer dereferenziert werden, ohne dass eine explizite Null-Prüfung oder Validierung der Buffer-Größen stattfindet. Dies ist ein hohes Risiko für Speicherzugriffsfehler (Segfaults).

### 2.3 Ressourcen-Management
*   **Ineffizientes Caching**: Das WGPU-Management nutzt zwar einen `TexturePool`, leidet aber unter ineffizienten Readbacks.
*   **Memory-Leak Risiken**: In den Media-Decodern existieren unbegrenzte Kanäle (`crossbeam-channel` / `tokio::sync::mpsc`). Wenn die GPU die Frames langsamer verarbeitet, als der CPU-Decoder sie liefert, kann dies zu Out-of-Memory (OOM) Fehlern führen.

---

## 3. Thread-Sicherheit & Parallelität

*   **Deadlock-Risiko**: Die Kombination von `pollster::block_on` während der App-Initialisierung und synchronen Sperren (`RwLock`) im Render-Pfad ist riskant. Wenn Async-Tasks (wie z. B. der Audio-Update-Thread) gleichzeitig auf diese Ressourcen zugreifen, kann die gesamte Rendering-Loop blockieren.

---

## 4. Technische Schulden (High Priority)

1.  **Legacy imgui-Reste**: Obwohl die UI auf `egui` migriert wurde (Phase 6), ist `imgui` weiterhin in der `Cargo.toml` gelistet und als `vendor`-Crate vorhanden. Dies bläht die Kompilierzeiten unnötig auf und führt zu inkonsistentem Code.
2.  **Redundante Logik**: Die Audio-Reaktivität ist sowohl in `vorce-core` als auch in `vorce-bevy` implementiert. Diese Duplikation sollte zugunsten von `vorce-core` konsolidiert werden.
