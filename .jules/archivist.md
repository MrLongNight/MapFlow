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

## 2026-01-22 - Documentation Structure Divergence
**Erkenntnis:** Das Repository weicht von der in `MAPFLOW PROJEKTSTRUKTUR` definierten `docs/` Struktur ab.
- Standard: `docs/01-OVERVIEW`, `docs/02-USER-GUIDE`, `docs/05-DEVELOPMENT`.
- Ist-Zustand: `docs/01-GETTING-STARTED`, `docs/02-CONTRIBUTING`, `docs/05-ROADMAP`.
Zusätzlich wurde ein nicht-standardkonformes Verzeichnis `docs/agent_rules/` entdeckt.

**Aktion:**
- `docs/agent_rules/ci_cd_strategy.md` wurde nach `docs/07-TECHNICAL/CI_CD_STRATEGY.md` verschoben, um Konformität mit technischen Dokumentationsstandards herzustellen.
- Die Divergenz der `docs/` Struktur wurde dokumentiert, aber keine "Größere Ordnerumstrukturierung" ohne Rückfrage vorgenommen.
- `.jules/roles/archivist.md` wurde erstellt, um die Agenten-Rolle zu persistieren.
