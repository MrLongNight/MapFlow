# Anleitung: Build-Fehler durch "dead_code" beheben (`-D warnings`)

## Kontext

Das Projekt verwendet die Compiler-Einstellung `-D warnings`. Dadurch wird jede Warnung automatisch als Fehler behandelt. Besonders das Vorhandensein von "dead code" (nicht genutzte Felder, Funktionen oder Structs) sorgt so dafür, dass der Build fehlschlägt und kein CI-Workflow mehr durchläuft.

Viele dieser Stellen sind (temporär) mit `#[allow(dead_code)]` versehen, einige können aber unerwartet auftauchen oder es ist nicht klar, ob sie wirklich gebraucht werden.

---

## Ziel

- Der CI-Build MUSS fehlerfrei durchlaufen.
- **Kein** versehentliches Entfernen von potenziell nützlichem Code.
- Transparenz für alle Entwickler: _Diese Stellen müssen zu einem späteren Zeitpunkt nochmal geprüft werden!_

---

## Vorgehen

### 1. Warnungen nachvollziehen

Führe einen Build/Check-Vorgang durch, um alle "dead code"-Warnungen zu erhalten:

```sh
cargo check --all-targets
```

### 2. Lösung: `#[allow(dead_code)]` & Kommentar setzen

**So markierst du ungenutzte Items/Felder:**

```rust
#[allow(dead_code)] // TODO: Prüfen, ob dieses Feld dauerhaft benötigt wird!
field_name: FieldType,
```

oder für Funktionen:

```rust
#[allow(dead_code)] // TODO: Prüfen, ob diese Funktion benötigt wird!
fn ungenutzte_funktion() {}
```

**Immer:**

- Attribut direkt vor das Item setzen
- Klaren TODO-Kommentar ergänzen!

### 3. Dokumentation

Ergänze im `CHANGELOG.md` einen Eintrag zu dieser Aktion.
