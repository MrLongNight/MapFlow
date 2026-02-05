# Jules PR Check Fix Guide

Dieses Dokument erklärt, wie Jules die häufigsten PR Check Fehler beheben kann.

## Vor jedem PR: Pre-PR Script ausführen

**WICHTIG:** Führe vor jedem PR dieses Skript aus:

```powershell
# Windows PowerShell
cargo fmt
cargo clippy --fix --allow-dirty
cargo test --workspace
```

Oder nutze das Pre-PR Skript (falls vorhanden):

```powershell
.\scripts\pre-pr.ps1
```

---

## Häufige Fehler und Lösungen

### 1. Trailing Whitespace (Formatierung)

**Fehler:**

```
error[internal]: left behind trailing whitespace
--> crates/mapmap/src/main.rs:1099
```

**Lösung:**

```bash
cargo fmt
```

Dies entfernt automatisch alle trailing whitespaces.

---

### 2. Unresolved Import

**Fehler:**

```
error[E0432]: unresolved import `mapmap_render::MeshBufferCache`
--> crates/mapmap/src/main.rs:26:38
```

**Lösung:**

1. Prüfe, ob das Item in `lib.rs` des Ziel-Crates exportiert wird:

   ```rust
   // crates/mapmap-render/src/lib.rs
   pub use mesh_buffer_cache::MeshBufferCache;  // Hinzufügen!
   ```

2. Oder ändere den Import-Pfad:

   ```rust
   // Falsch:
   use mapmap_render::MeshBufferCache;

   // Richtig (wenn nicht re-exported):
   use mapmap_render::mesh_buffer_cache::MeshBufferCache;
   ```

---

### 3. Unused Imports

**Fehler:**

```
warning: unused import: `debug`
```

**Lösung:**
Entferne den Import oder nutze `#[allow(unused_imports)]` wenn beabsichtigt.

---

### 4. Merge Conflicts

**Fehler:**

```
This branch has conflicts that must be resolved
Conflicting files:
- CHANGELOG.md
- crates/mapmap-core/src/audio/analyzer_v2.rs
```

**Lösung:**

```bash
git fetch origin main
git merge origin/main
# Konflikte manuell lösen
git add .
git commit -m "Merge main and resolve conflicts"
git push
```

Bei Konflikten in `CHANGELOG.md`:

- Behalte BEIDE Änderungen (chronologisch sortiert)
- Neueste Einträge oben

---

### 5. Test Failures

**Fehler:**

```
test xxx::test_name ... FAILED
```

**Lösung:**

1. Führe den Test lokal aus:

   ```bash
   cargo test test_name -- --nocapture
   ```

2. Analysiere die Ausgabe
3. Fixe den Code oder den Test

Für GPU-abhängige Tests die auf CI fehlschlagen:

```rust
#[test]
#[ignore]  // GPU tests fail on CI without GPU
fn test_gpu_feature() {
    // ...
}
```

---

### 6. Borrow Checker Errors

**Fehler:**

```
error[E0499]: cannot borrow `x` as mutable more than once at a time
```

**Lösung:**

- Nutze lokale Variablen für Zwischenwerte
- Trenne Borrows in separate Scopes
- Nutze `Clone` wenn nötig

Beispiel:

```rust
// Falsch:
let a = &mut self.x;
let b = &mut self.x;  // Error!

// Richtig:
let value = self.x.clone();
// Oder:
{
    let a = &mut self.x;
    // use a
}
{
    let b = &mut self.x;
    // use b
}
```

---

## CI Workflow Übersicht

Die CI führt folgende Checks aus:

1. **Build (Ubuntu)** - `cargo build --release`
2. **Build (Windows)** - `cargo build --release`
3. **Code Quality** - `cargo fmt --check` und `cargo clippy`
4. **Tests** - `cargo test --workspace`

Alle müssen grün sein bevor ein Merge möglich ist.

---

## Checkliste vor PR

- [ ] `cargo fmt` ausgeführt
- [ ] `cargo clippy` ohne Warnungen
- [ ] `cargo test --workspace` alle Tests grün
- [ ] `cargo build --release` kompiliert ohne Fehler
- [ ] Keine Merge-Konflikte mit `main`
- [ ] CHANGELOG.md aktualisiert (falls relevant)
