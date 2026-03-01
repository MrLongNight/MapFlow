#!/bin/bash
set -e

# Setup-Skript für MapFlow in der Codex-Entwicklungsumgebung (Ubuntu 24.04)
# Wird nach dem Erstellen neuer Container ausgeführt.

echo "🚀 Starte MapFlow Codex Setup (Ubuntu 24.04)..."

# 1. System-Abhängigkeiten aktualisieren und installieren
echo "📦 Installiere System-Abhängigkeiten..."
sudo apt-get update
sudo apt-get install -y
    build-essential pkg-config libclang-dev cmake
    libasound2-dev libudev-dev
    libx11-dev libxcursor-dev libxrandr-dev libxi-dev
    libgl1-mesa-dev libvulkan-dev libwayland-dev libxkbcommon-dev
    libavutil-dev libavcodec-dev libavformat-dev libswscale-dev
    libavdevice-dev libavfilter-dev libswresample-dev
    ffmpeg curl git

# 2. Rust Toolchain konfigurieren
echo "🦀 Konfiguriere Rust Toolchain..."
if command -v rustup &> /dev/null; then
    rustup toolchain install stable
    rustup default stable
    rustup component add rustfmt clippy
else
    echo "⚠️ rustup nicht gefunden. Überspringe Rust-Konfiguration."
fi

# 3. Hilfswerkzeuge installieren
echo "🛠️ Installiere Cargo-Helfer..."
cargo install cargo-sort --quiet || echo "cargo-sort Installation fehlgeschlagen (evtl. bereits vorhanden)."

# 4. Cargo-Cache vorwärmen
echo "📥 Lade Abhängigkeiten herunter (Warming up cache)..."
cargo fetch --locked --quiet

echo "✅ Codex Setup abgeschlossen!"
