# DOC-B2: Development Setup & Guidelines

Dieses Dokument enthält alle Informationen zum Einrichten der Entwicklungsumgebung sowie die verbindlichen Richtlinien für Code-Stil, Testing und Pull Requests.

## 1. System-Anforderungen & Setup

Vorce ist eine Rust-Anwendung. Stelle sicher, dass die aktuelle Rust-Toolchain (stable) installiert ist.

### Abhängigkeiten installieren
- **Windows**: Vcpkg für FFmpeg empfohlen.
- **Linux**: `libasound2-dev`, `pkg-config`, `build-essential`.
- **macOS**: `ffmpeg` via Homebrew.

### Build-Befehle
```bash
cargo build          # Entwicklungs-Build
cargo run            # Starten
cargo test           # Tests ausführen
cargo clippy         # Linting (Pflicht vor Commit)
```

---

## 2. Code-Konventionen

- **Formatierung**: Code **muss** per `cargo fmt` formatiert sein.
- **Sichtbarkeit**: Nutze `pub(crate)` für interne Modul-Kommunikation.
- **Sicherheit**: Jeder `unsafe` Block muss mit `// SAFETY:` dokumentiert werden.
- **Fehlerbehandlung**: Kein `.unwrap()` im Produktionscode. Nutze `?` oder `.expect("Grund")`.

### Größen-Limits
| Metrik | Maximum | Aktion bei Überschreitung |
|--------|---------|---------------------------|
| Datei (LOC) | 400 Zeilen | Refactoring in Submodule |
| Funktion (LOC) | 120 Zeilen | Extraktion in Hilfsfunktionen |
| Komplexität | 10 Branches | Refactoring / Pattern Matching |

---

## 3. Pull Request (PR) Prozess

1.  **Vorbereitung**: `fmt`, `clippy` und `test` müssen fehlerfrei durchlaufen.
2.  **Naming**: PR-Titel müssen die MF-ID enthalten (z.B. `feat(ui): [MF-023] add toasts`).
3.  **Template**: Nutze immer das PR-Template unter `.github/PULL_REQUEST_TEMPLATE.md`.
4.  **Changelog**: Jede Änderung muss im `CHANGELOG.md` eingetragen werden.

---

## 4. Spezielle Anweisungen für KI-Agenten (Jules/Gemini)

- **Sprache**: Kommuniziere ausschließlich auf Deutsch.
- **Rolle**: Du bist Senior Graphics Architect. Schreibe wartbaren Industrie-Code.
- **Performance**: Keine Allokationen im Hot-Path (Render-Loop). Pre-allokiere Buffer.
- **Dokumentation**: Nutze `///` für alle öffentlichen Items.
- **Commit Messages**: Format `type(scope): description` (z.B. `fix(core): resolve race condition`).

---

## 5. Testing-Strategie

- Jeder Bugfix und jedes Feature benötigt automatisierte Tests.
- **Unit-Tests**: Direkt im Modul (unten).
- **Integration-Tests**: In `tests/`.
- **Shader-Tests**: Validierung mit `naga` (falls verfügbar).
