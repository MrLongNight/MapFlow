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

## 2026-01-26 - Configuration File Standards & Root Cleanup

**Erkenntnis:** Das Root-Verzeichnis enthielt `CODE-OF-CONDUCT.md`, das nicht in der Liste der erlaubten Root-Dateien stand. Zusätzlich wurden notwendige Konfigurationsdateien (`Makefile.toml`, `deny.toml`) und Dokumentation (`ROADMAP_2.0.md`, `AGENTS.md`) identifiziert, die im Root verbleiben müssen, aber nicht explizit gelistet waren.

**Aktion:**
- `CODE-OF-CONDUCT.md` nach `.github/` verschoben (Standard-Konvention).
- `Makefile.toml` (cargo-make), `deny.toml` (cargo-deny), `ROADMAP_2.0.md` und `AGENTS.md` als erlaubte Ausnahmen dokumentiert.
- Repository auf "Unauthorized Root Files" und "Misplaced Markdown Files" gescannt (Ergebnis: Sauber).

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
