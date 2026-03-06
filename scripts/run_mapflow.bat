@echo off
REM MapFlow Local Startup Script (ClawMaster Optimized)
REM Bypasses ffmpeg-sys-next build failures and LNK1140 PDB errors

echo 🦀 Starting MapFlow in Local Release Mode...

REM Copy DLLs if they exist in vcpkg (Required for vcpkg dynamic linking)
if exist "vcpkg_installed\x64-windows\bin\*.dll" (
    echo 📦 Syncing vcpkg DLLs to target\release...
    if not exist "target\release" mkdir "target\release"
    xcopy /Y /D "vcpkg_installed\x64-windows\bin\av*.dll" "target\release\" >nul 2>&1
    xcopy /Y /D "vcpkg_installed\x64-windows\bin\sw*.dll" "target\release\" >nul 2>&1
    xcopy /Y /D "vcpkg_installed\x64-windows\bin\postproc*.dll" "target\release\" >nul 2>&1
)

REM Run MapFlow with stable features in release mode for best performance
cargo run --release -p mapmap --bin MapFlow --no-default-features --features "audio,ffmpeg"
