@echo off
REM Script to copy FFmpeg DEBUG DLLs to the debug target directory

set VCPKG_DEBUG_BIN=%~dp0..\vcpkg\installed\x64-windows\debug\bin
set TARGET_DIR=%~dp0..	arget\debug

echo Copying DEBUG FFmpeg DLLs from %VCPKG_DEBUG_BIN% to %TARGET_DIR%...

if not exist "%VCPKG_DEBUG_BIN%" (
    echo Error: Debug bin directory not found at %VCPKG_DEBUG_BIN%
    exit /b 1
)

if not exist "%TARGET_DIR%" (
    mkdir "%TARGET_DIR%"
)

copy /Y "%VCPKG_DEBUG_BIN%\avcodec-61.dll" "%TARGET_DIR%" >nul 2>&1
copy /Y "%VCPKG_DEBUG_BIN%\avdevice-61.dll" "%TARGET_DIR%" >nul 2>&1
copy /Y "%VCPKG_DEBUG_BIN%\avfilter-10.dll" "%TARGET_DIR%" >nul 2>&1
copy /Y "%VCPKG_DEBUG_BIN%\avformat-61.dll" "%TARGET_DIR%" >nul 2>&1
copy /Y "%VCPKG_DEBUG_BIN%\avutil-59.dll" "%TARGET_DIR%" >nul 2>&1
copy /Y "%VCPKG_DEBUG_BIN%\swresample-5.dll" "%TARGET_DIR%" >nul 2>&1
copy /Y "%VCPKG_DEBUG_BIN%\swscale-8.dll" "%TARGET_DIR%" >nul 2>&1
copy /Y "%VCPKG_DEBUG_BIN%\pkgconf-7.dll" "%TARGET_DIR%" >nul 2>&1

echo Done! Debug DLLs copied.
