# 🗂️ Archivist's Journal

Kritische Erkenntnisse aus Repository-Verwaltungsaktivitäten.

---

## Eintragsformat

```
## YYYY-MM-DD - [Titel]
**Erkenntnis:** [Was gelernt]
**Aktion:** [Wie beim nächsten Mal anwenden].
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

**Erkenntnis:** Das Root-Verzeichnis enthielt getrackte Patch-Dateien (`pr397.patch`, `pr398.patch`), die dort nicht hingehören. Zudem existierte ein nicht-konformes `docu/` Verzeichnis with Jules-spezifischen Notizen.

**Aktion:**
- `pr397.patch` and `pr398.patch` nach `.temp-archive/2026-01-31-*` archiviert and via `git rm` aus dem Repository entfernt.
- `docu/jules_gpu_ui.md` and `docu/jules_hw_accel.md` nach `.jules/` verschoben.
- `docu/` Verzeichnis entfernt.

## 2026-02-06 - Repository Cleanup & DLL Relocation

**Erkenntnis:** Das Root-Verzeichnis enthielt nicht erlaubt dem DLL-Dateien (FFmpeg Abhängigkeiten) und eine Patch-Datei (`pr398.patch`), was gegen die Repository-Richtlinien verstößt. Zudem lag `copy_ffmpeg_dlls.bat` im Root statt im `scripts/` Verzeichnis.

**Aktion:**
- DLLs (`avcodec-61.dll`, etc.) and `pr398.patch` nach `.temp-archive/2026-02-06-*` archiviert.
- `tmp/fix_player.ps1` nach `.temp-archive/` verschoben und `tmp/` entfernt.
- `copy_ffmpeg_dlls.bat` nach `scripts/` verschoben und Pfade korrigiert (`%~dp0..\`).

## 2026-02-09 - Archive Maintenance

**Erkenntnis:** Das Verzeichnis `.temp-archive/` enthielt mehrere getrackte Dateien vom 2026-01-02 (`.mapmap_autosave`, `check_*.txt`, `VERSION.txt`, `VjMapper.code-workspace`, `core_error.txt`, `test_results.txt`), die älter als 30 Tage waren und somit die Aufbewahrungsfrist überschritten hatten.

**Aktion:**
- Alle Dateien mit dem Präfix `2026-01-02-` aus `.temp-archive/` via `git rm` entfernt.
- `.temp-archive/` ist nun leer und wurde entfernt.

## 2026-02-10 - Root & Docs Cleanup

**Erkenntnis:** Das Root-Verzeichnis enthielt `apply_global_fix.ps1` (veralteter Patch-Script) und `fix_formatting.py` (Utility-Script), die dort nicht hingehören. Zudem befand sich ein Zip-Archiv `HueFlow-main.zip` in der Dokumentation.

**Aktion:**
- `apply_global_fix.ps1` nach `.temp-archive/2026-02-10-apply_global_fix.ps1` archiviert.
- `docs/03-ARCHITECTURE/specs/HueFlow-main.zip` nach `.temp-archive/2026-02-10-HueFlow-main.zip` archiviert.
- `fix_formatting.py` nach `scripts/fix_formatting.py` verschoben.

## 2026-02-18 - Documentation & Root Cleanup

**Erkenntnis:** Das Root-Verzeichnis enthielt Projekt-Dokumentation (`PR_MAINTENANCE_OVERVIEW.md`, `pr_tracking.md`) sowie Utility-Skripte (`check_links.py`) und temporäre Dateien (`GEMINI.md`), die dort nicht hingehören.

**Aktion:**
- `PR_MAINTENANCE_OVERVIEW.md` und `pr_tracking.md` nach `docs/project/` verschoben.
- `check_links.py` nach `scripts/` verschoben und Funktionalität verifiziert.
- `GEMINI.md` nach `.temp-archive/2026-02-18-GEMINI.md` archiviert.

## 2026-02-18 - Cleanup Verification & Completion

**Erkenntnis:** Trotz des vorangegangenen Eintrags vom selben Tag befanden sich `PR_MAINTENANCE_OVERVIEW.md`, `pr_tracking.md` und `GEMINI.md` weiterhin im Root-Verzeichnis.

**Aktion:**
- Dateien erneut verschoben bzw. archiviert.
- Verifikation der Dateipfade erfolgreich durchgeführt.

## 2026-03-02 - Root Directory Cleanup

**Erkenntnis:** Das Root-Verzeichnis enthielt mehrere Skripte (`fix_docs_*.py`, `fix_ui_layout_*.sh`, `fix_wgpu*.py`) sowie eine leere temporäre Datei `GEMINI.md`, die dort nicht hingehören. Da die weitere Verwendung oder der eigentliche Speicherort der Skripte unklar war, war es am sichersten, sie zunächst zu archivieren.

**Aktion:**
- Alle identifizierten Skripte (`fix_docs_pro.py`, `fix_docs_smart.py`, `fix_docs_smart_all.py`, `fix_ui_layout_10.sh`, `fix_ui_layout_3.sh`, `fix_ui_layout_4.sh`, `fix_ui_layout_5.sh`, `fix_ui_layout_6.sh`, `fix_ui_layout_7.sh`, `fix_ui_layout_8.sh`, `fix_ui_layout_9.sh`, `fix_wgpu.py`, `fix_wgpu_2.py`, `fix_wgpu_clean.py`) nach `.temp-archive/` mit aktuellem Datums-Präfix archiviert.
- `GEMINI.md` nach `.temp-archive/` mit aktuellem Datums-Präfix archiviert.

## 2026-03-09 - Root Directory Cleanup

**Erkenntnis:** Das Root-Verzeichnis enthielt `GEMINI.md` und das Skript `update_docs.sh`, die beide gegen die Projektstruktur-Richtlinien verstoßen.

**Aktion:**
- `GEMINI.md` nach `.jules/GEMINI.md` verschoben.
- `update_docs.sh` nach `scripts/update_docs.sh` verschoben.

## 2026-03-12 - Root Directory Cleanup

**Erkenntnis:** Es wurden MapFlow-Log-Dateien (`mapflow.log.*`) im Verzeichnis `scripts/archive/logs/` gefunden, welche fälschlicherweise in Git verfolgt wurden, da die aktuelle `.gitignore`-Regel (`/logs/` und `*.log`) das Datums-Suffix nicht erfasste.
**Aktion:** Log-Dateien aus Git mit `git rm` entfernt und `.gitignore` aktualisiert (`scripts/archive/logs/`), um zukünftige Verfolgung von diesen Dateien zu verhindern.

## 2026-03-02 - Temporäre Dateien im Root verschoben
**Erkenntnis:** Im Root-Verzeichnis befanden sich temporäre Entwicklungsskripte und Patches (`fix_bevy_test.py`, `fix_script.py`, `patch.diff`, `test_script.py`), die nicht den Projektstandards für Root-Dateien entsprechen und unnötig mit Git getrackt wurden.
**Aktion:** Dateien via `git rm --cached` aus Git entfernt und mit Datum-Präfix ins `.temp-archive/` verschoben.

## 2026-03-19 - Patch Cleanup
**Erkenntnis:** Das Root-Verzeichnis enthielt eine getrackte Patch-Datei (`patch.diff`), die dort nicht hingehört.
**Aktion:** `patch.diff` nach `.temp-archive/2026-03-19-patch.diff` archiviert und via `git rm` aus dem Repository entfernt.

## 2026-03-19 - Vcpkg JSON Cleanup
**Erkenntnis:** Das Root-Verzeichnis enthielt eine `vcpkg.json`, die typischerweise im Projekt-Root toleriert wird (C++ Dependency-Management), jedoch laut Regeln zur Prüfung gemeldet wurde. Da es im Standard-Kontext erlaubt sein könnte, wurde entschieden, die Datei dort zu belassen, aber im Journal als geprüft zu vermerken.
**Aktion:** `vcpkg.json` verifiziert. Keine Aktion erforderlich.

## 2026-03-19 - CI Failure Analysis
**Erkenntnis:** Ein Test in mapmap-bevy schlug in der CI fehl (`headless_runner_disables_embedded_host_plugins`), da ihm das `#[ignore]` Tag für GPU-Tests fehlte. Des Weiteren gab es diverse `cargo fmt` Fehlschläge im Code.
**Aktion:** Der Test wurde gemäß den Repository-Regeln (AGENTS.md) mit `#[ignore]` markiert, da Render/GPU-Tests ohne interaktive GUI-Umgebung auf CI nicht laufen. Außerdem wurde `cargo fmt` global ausgeführt, um Formatierungswarnungen zu beheben.

## 2026-03-22 - Temporäre Dateien im Root verschoben
**Erkenntnis:** Im Root-Verzeichnis befanden sich temporäre Entwicklungsskripte und Patches (`fix.sh`, `fix_bevy_test.py`, `fix_script.py`, `patch_diag.py`, `patch_source3.py`, `patch_tests.py`, `test_script.py`) sowie die Testdatei `test.ron`, die nicht den Projektstandards für Root-Dateien entsprechen und unnötig mit Git getrackt wurden.
**Aktion:** Dateien via `git mv` in das `.temp-archive/` verschoben und mit dem Datum-Präfix `2026-03-22-` versehen, um das Root-Verzeichnis sauber zu halten.

## 2026-03-23 - Temporäre Dateien im Root entfernt
**Erkenntnis:** Im Root-Verzeichnis befanden sich ein ausführbares Programm `test_macos` und die dazugehörige Quelldatei `test_macos.rs`, die nicht den Projektstandards für Root-Dateien entsprechen und unnötig mit Git getrackt wurden.
**Aktion:** Dateien via `git rm` aus dem Repository entfernt.

## 2026-03-26 - Temporäre Dateien im Root verschoben
**Erkenntnis:** Im Root-Verzeichnis befanden sich `update_layer_inspector.patch` und `issue1203_body.md`, die nicht den Projektstandards für Root-Dateien entsprechen.
**Aktion:** Dateien ins `.temp-archive/` verschoben und mit dem aktuellen Datum versehen.
