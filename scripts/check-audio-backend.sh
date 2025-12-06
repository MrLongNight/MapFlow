#!/bin/bash
# check-audio-backend.sh - Verifies that the required audio backend libraries are installed.

echo "Checking for audio backend dependencies..."

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Platform: Linux"
    if pkg-config --exists alsa; then
        echo "✅ ALSA development libraries found."
    else
        echo "❌ ALSA development libraries (libasound2-dev) not found."
        echo "Please install them to build with audio support on Linux."
        echo "On Debian/Ubuntu: sudo apt-get install libasound2-dev"
        exit 1
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Platform: macOS"
    echo "✅ CoreAudio is part of the operating system. No extra setup needed."
elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    echo "Platform: Windows"
    echo "✅ WASAPI is part of the operating system. No extra setup needed."
else
    echo "Unsupported platform: $OSTYPE"
    exit 1
fi

echo "Audio environment check passed."
exit 0
