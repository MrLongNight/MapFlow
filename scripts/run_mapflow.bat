@echo off
REM MapFlow Startup Script - Sets up FFmpeg environment and runs the application

REM Set FFmpeg paths
set FFMPEG_DIR=C:\ffmpeg
set LIBCLANG_PATH=C:\Program Files\LLVM\bin
set PATH=C:\ffmpeg\bin;%PATH%

REM Run MapFlow
cargo run -p mapmap --release
