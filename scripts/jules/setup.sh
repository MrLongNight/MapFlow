#!/bin/bash
set -e

# Update package list
echo "Updating package list..."
sudo apt-get update

# Install Core Build Tools and Libraries
# - pkg-config: For finding libraries
# - libclang-dev: For bindgen (FFmpeg, etc.)
echo "Installing build tools..."
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libclang-dev

# Install Audio Dependencies (ALSA)
# Required by cpal
echo "Installing audio dependencies..."
sudo apt-get install -y \
    libasound2-dev \
    libudev-dev

# Install Windowing/Graphics Dependencies
# Required by winit and wgpu
echo "Installing windowing dependencies..."
sudo apt-get install -y \
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libgl1-mesa-dev \
    libvulkan-dev \
    libwayland-dev \
    libxkbcommon-dev

# Install FFmpeg Development Libraries
# Note: MapFlow uses ffmpeg-next = "7.0".
# If the system FFmpeg version is older (e.g. 6.x), this might cause link errors.
# In that case, we might need a specific PPA or static build.
# For now, we install the system libraries.
echo "Installing FFmpeg dependencies..."
sudo apt-get install -y \
    libavutil-dev \
    libavcodec-dev \
    libavformat-dev \
    libswscale-dev \
    libavdevice-dev \
    libavfilter-dev \
    ffmpeg

# Install NDI Dependencies (if feature is enabled)
# Ensure utilities are present
sudo apt-get install -y \
    curl \
    git

# Run a cargo check to verify environment
echo "Verifying environment with cargo check..."
cargo check --workspace

echo "Jules environment setup complete!"
