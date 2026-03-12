#!/bin/bash
set -euo pipefail

# Wartungsskript fuer MapFlow in der Codex-Entwicklungsumgebung.
# Wird in Containern ausgefuehrt, die aus dem Cache fortgesetzt wurden
# (z.B. nach einem Branch-Wechsel).

echo "Starte MapFlow Wartung (Maintenance)..."

# 1. Abhaengigkeiten fuer den aktuellen Branch aktualisieren
echo "Aktualisiere Abhaengigkeiten..."
fetch_log="$(mktemp)"
if cargo fetch --locked --quiet 2>"$fetch_log"; then
    rm -f "$fetch_log"
else
    if grep -q "cannot update the lock file" "$fetch_log"; then
        rm -f "$fetch_log"
        echo "Cargo.lock drift detected in this container; retrying dependency refresh without --locked..."
        cargo fetch --quiet
    else
        cat "$fetch_log" >&2
        rm -f "$fetch_log"
        exit 1
    fi
fi

# 2. Build-Umgebung verifizieren
echo "Validiere Build-Status..."
# Ein schneller Check stellt sicher, dass alle externen Libs korrekt gelinkt werden koennen.
cargo check --workspace --all-targets --quiet

# 3. Aufraeumen (optional)
# cargo clean -p <name> falls noetig, aber meistens kontraproduktiv im Cache.

echo "Wartung abgeschlossen!"
