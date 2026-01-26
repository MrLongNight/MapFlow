#!/usr/bin/env bash
set -e

echo "Installiere FFmpeg-Entwicklerbibliotheken …"

sudo apt-get update

sudo apt-get install -y \
  libavutil-dev \
  libavcodec-dev \
  libavformat-dev \
  libswscale-dev \
  libavfilter-dev \
  libavfilter-dev \
  libavdevice-dev \
  libswresample-dev

echo "Alle relevanten FFmpeg-Dev-Abhängigkeiten installiert."
