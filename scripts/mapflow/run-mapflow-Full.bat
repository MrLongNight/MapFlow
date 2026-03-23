@echo off
setlocal EnableExtensions EnableDelayedExpansion
REM MapFlow local startup script.

set "ROOT_DIR=%~dp0..\\.."
set "TARGET_DIR=%ROOT_DIR%\target\release"
set "VCPKG_BIN=%ROOT_DIR%\vcpkg_installed\x64-windows\bin"
set "VCPKG_LIB=%ROOT_DIR%\vcpkg_installed\x64-windows\lib"

if not exist "%VCPKG_BIN%" (
    set "VCPKG_BIN=%ROOT_DIR%\vcpkg\installed\x64-windows\bin"
)

if not exist "%VCPKG_LIB%" (
    set "VCPKG_LIB=%ROOT_DIR%\vcpkg\installed\x64-windows\lib"
)

set "EXTRA_FEATURES="
set "MPV_LIB_DIR="
set "MPV_BIN_DIR="

if exist "C:\Program Files\NDI\NDI 6 SDK\Lib\x64\Processing.NDI.Lib.x64.lib" (
    set "EXTRA_FEATURES=!EXTRA_FEATURES!,ndi"
)

if not defined MPV_LIB_DIR if defined MAPFLOW_MPV_DIR if exist "%MAPFLOW_MPV_DIR%\lib\mpv.lib" (
    set "MPV_LIB_DIR=%MAPFLOW_MPV_DIR%\lib"
    if exist "%MAPFLOW_MPV_DIR%\bin" set "MPV_BIN_DIR=%MAPFLOW_MPV_DIR%\bin"
)

if not defined MPV_LIB_DIR if defined MAPFLOW_MPV_DIR if exist "%MAPFLOW_MPV_DIR%\mpv.lib" (
    set "MPV_LIB_DIR=%MAPFLOW_MPV_DIR%"
    set "MPV_BIN_DIR=%MAPFLOW_MPV_DIR%"
)

if not defined MPV_LIB_DIR if defined LIBMPV_DIR if exist "%LIBMPV_DIR%\lib\mpv.lib" (
    set "MPV_LIB_DIR=%LIBMPV_DIR%\lib"
    if exist "%LIBMPV_DIR%\bin" set "MPV_BIN_DIR=%LIBMPV_DIR%\bin"
)

if not defined MPV_LIB_DIR if defined LIBMPV_DIR if exist "%LIBMPV_DIR%\mpv.lib" (
    set "MPV_LIB_DIR=%LIBMPV_DIR%"
    set "MPV_BIN_DIR=%LIBMPV_DIR%"
)

if not defined MPV_LIB_DIR if defined MPV_DIR if exist "%MPV_DIR%\lib\mpv.lib" (
    set "MPV_LIB_DIR=%MPV_DIR%\lib"
    if exist "%MPV_DIR%\bin" set "MPV_BIN_DIR=%MPV_DIR%\bin"
)

if not defined MPV_LIB_DIR if defined MPV_DIR if exist "%MPV_DIR%\mpv.lib" (
    set "MPV_LIB_DIR=%MPV_DIR%"
    set "MPV_BIN_DIR=%MPV_DIR%"
)

if not defined MPV_LIB_DIR if exist "%VCPKG_LIB%\mpv.lib" (
    set "MPV_LIB_DIR=%VCPKG_LIB%"
    if exist "%VCPKG_BIN%" set "MPV_BIN_DIR=%VCPKG_BIN%"
)

if defined MPV_LIB_DIR (
    set "EXTRA_FEATURES=!EXTRA_FEATURES!,libmpv"
)

if defined EXTRA_FEATURES (
    set "EXTRA_FEATURES=!EXTRA_FEATURES:~1!"
    echo Starting MapFlow in local release mode with default features + !EXTRA_FEATURES!...
) else (
    echo Starting MapFlow in local release mode with default features...
)

REM Copy FFmpeg DLLs into the release target when available.
if exist "%VCPKG_BIN%\*.dll" (
    echo Syncing vcpkg DLLs to %TARGET_DIR%...
    if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"
    xcopy /Y /D "%VCPKG_BIN%\av*.dll" "%TARGET_DIR%\" >nul 2>&1
    xcopy /Y /D "%VCPKG_BIN%\sw*.dll" "%TARGET_DIR%\" >nul 2>&1
    xcopy /Y /D "%VCPKG_BIN%\postproc*.dll" "%TARGET_DIR%\" >nul 2>&1
)

if defined MPV_BIN_DIR if exist "%MPV_BIN_DIR%\mpv*.dll" (
    echo Syncing libmpv DLLs to %TARGET_DIR%...
    if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"
    xcopy /Y /D "%MPV_BIN_DIR%\mpv*.dll" "%TARGET_DIR%\" >nul 2>&1
)

if defined MPV_LIB_DIR (
    set "LIB=%MPV_LIB_DIR%;%LIB%"
    if defined MPV_BIN_DIR set "PATH=%MPV_BIN_DIR%;%PATH%"
) else (
    echo libmpv not found. Skipping optional libmpv feature.
    echo Set MAPFLOW_MPV_DIR, LIBMPV_DIR, or MPV_DIR to a folder containing mpv.lib to enable it.
)

pushd "%ROOT_DIR%"
if defined EXTRA_FEATURES (
    cargo run --release -p mapmap --bin MapFlow --features "!EXTRA_FEATURES!"
) else (
    cargo run --release -p mapmap --bin MapFlow
)
set "EXIT_CODE=%ERRORLEVEL%"
popd

exit /b %EXIT_CODE%
