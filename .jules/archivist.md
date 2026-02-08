# 🗂️ Archivist's Journal

Kritische Erkenntnisse aus Repository-Verwaltungsaktivitäten.

---

## Eintragsformat

```
## YYYY-MM-DD - [Titel]
**Erkenntnis:** [Was gelernt]
**Aktion:** [Wie beim nächsten Mal anwenden]
```

---

## 2026-01-02 - Root Directory Cleanup

**Erkenntnis:** Das Root-Verzeichnis enthielt mehrere temporäre Dateien (`check_*.txt`, `test_results.txt`, `core_error.txt`) sowie falsch platzierte Dokumentation (`SECURITY.md`, `knowledge.md`) und redundante Dateien (`VERSION.txt`).

**Aktion:**
- `SECURITY.md` nach `.github/` verschoben.
- `knowledge.md` nach `.jules/` verschoben.
- Temporäre Dateien nach `.temp-archive/2026-01-02-*` archiviert.
- `VjMapper.code-workspace` archiviert (Legacy-Name, nicht erlaubt im Root).

**Zusatz:** Merge-Konflikte in `module.rs`, `main.rs`, `module_eval.rs` behoben (HEAD priorisiert). Syntaxfehler in `module_canvas.rs` korrigiert.

## 2025-01-19 - WGSL Shader Cleanup

**Erkenntnis:** `crates/mapmap-render/shaders/` enthielt 10 `.wgsl` Dateien, die gegen die Projektstruktur verstoßen, da alle Shader in `shaders/` liegen sollten. Dies führte zu einer Inkonsistenz in der Shader-Verwaltung.

**Aktion:**
- Alle `.wgsl` Dateien aus `crates/mapmap-render/shaders/` nach `shaders/` verschoben.
- `crates/mapmap-render/src/effect_chain_renderer.rs` aktualisiert, um die Shader aus dem neuen Pfad (`../../../shaders/`) zu laden.
- `crates/mapmap-render/shaders/` Verzeichnis gelöscht.
- Build mit `cargo check` verifiziert.

## 2026-01-29 - Repository Cleanup

**Erkenntnis:** `CODE-OF-CONDUCT.md` befand sich fälschlicherweise im Root. Das `.github/` Verzeichnis enthielt allgemeine technische und Jules-spezifische Dokumentation, die dort nicht hingehört. `.gitignore` fehlten einige Standard-Ausschlüsse.

**Aktion:**
- `CODE-OF-CONDUCT.md` nach `.github/` verschoben.
- Technische Dokumentation (`GMAIL_API_SETUP.md`, `README_CICD.md`, etc.) aus `.github/` nach `docs/08-TECHNICAL/` verschoben.
- Jules-Dokumentation (`JULES_INTEGRATION.md`, etc.) aus `.github/` nach `.jules/` verschoben.
- `.gitignore` aktualisiert (`.idea/`, `*.swo`, `.vscode/settings.json`, `.env`, `*.tmp`).

## 2026-01-31 - Patch Cleanup & Doc Organization

**Erkenntnis:** Das Root-Verzeichnis enthielt getrackte Patch-Dateien (`pr397.patch`, `pr398.patch`), die dort nicht hingehören. Zudem existierte ein nicht-konformes `docu/` Verzeichnis mit Jules-spezifischen Notizen.

**Aktion:**
- `pr397.patch` und `pr398.patch` nach `.temp-archive/2026-01-31-*` archiviert und via `git rm` aus dem Repository entfernt.
- `docu/jules_gpu_ui.md` und `docu/jules_hw_accel.md` nach `.jules/` verschoben.
- `docu/` Verzeichnis entfernt.

## 2026-02-06 - Repository Cleanup & DLL Relocation

**Erkenntnis:** Das Root-Verzeichnis enthielt nicht erlaubte DLL-Dateien (FFmpeg Abhängigkeiten) und eine Patch-Datei (`pr398.patch`), was gegen die Repository-Richtlinien verstößt. Zudem lag `copy_ffmpeg_dlls.bat` im Root statt im `scripts/` Verzeichnis.

**Aktion:**
- DLLs (`avcodec-61.dll`, etc.) und `pr398.patch` nach `.temp-archive/2026-02-06-*` archiviert.
- `tmp/fix_player.ps1` nach `.temp-archive/` verschoben und `tmp/` entfernt.
- `copy_ffmpeg_dlls.bat` nach `scripts/` verschoben und Pfade korrigiert (`%~dp0..\`).

## 2026-02-08 - Tmp Directory Cleanup & Docs Review

**Erkenntnis:** Das `tmp/` Verzeichnis (mit `antigravity-jules-orchestration`) existierte erneut im Root, obwohl es nicht zur Standardstruktur gehört. Zudem ist die `docs/` Struktur inkonsistent: Doppelte Präfixe (`01-GENERAL`, `01-GETTING-STARTED` und `03-ARCHITECTURE`, `03-FEATURES`) sowie fehlende Standard-Ordner (`01-OVERVIEW`, `06-DEPLOYMENT`, `07-TECHNICAL`).

**Aktion:**
- `tmp/` Verzeichnis und Inhalt gelöscht.
- Dokumentations-Struktur als Diskussionsthema für zukünftige Restrukturierung notiert (keine sofortige Änderung, um größere Konflikte zu vermeiden).
