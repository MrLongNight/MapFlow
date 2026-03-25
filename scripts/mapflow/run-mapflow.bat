@echo off
setlocal
REM MapFlow local startup script.

set "ROOT_DIR=%~dp0..\\.."
set "TARGET_DIR=%ROOT_DIR%\target\release"
set "VCPKG_BIN=%ROOT_DIR%\vcpkg_installed\x64-windows\bin"

if not exist "%VCPKG_BIN%" (
    set "VCPKG_BIN=%ROOT_DIR%\vcpkg\installed\x64-windows\bin"
)

echo Starting MapFlow in local release mode...

REM Copy FFmpeg DLLs into the release target when available.
if exist "%VCPKG_BIN%\*.dll" (
    echo Syncing vcpkg DLLs to %TARGET_DIR%...
    if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"
    xcopy /Y /D "%VCPKG_BIN%\av*.dll" "%TARGET_DIR%\" >nul 2>&1
    xcopy /Y /D "%VCPKG_BIN%\sw*.dll" "%TARGET_DIR%\" >nul 2>&1
    xcopy /Y /D "%VCPKG_BIN%\postproc*.dll" "%TARGET_DIR%\" >nul 2>&1
)

pushd "%ROOT_DIR%"
cargo run --release -p vorce --bin Vorce --no-default-features --features "audio,ffmpeg"
set "EXIT_CODE=%ERRORLEVEL%"
popd

exit /b %EXIT_CODE%
