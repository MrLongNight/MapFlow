@echo off
REM MapFlow Local Startup Script (ClawMaster Optimized)
REM Bypasses ffmpeg-sys-next build failures and LNK1140 PDB errors

echo 🦀 Starting MapFlow in Local Release Mode...

REM Run MapFlow with stable features in release mode for best performance
cargo run --release -p mapmap --bin MapFlow --no-default-features --features "audio"
