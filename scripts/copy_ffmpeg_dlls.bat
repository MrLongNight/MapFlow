@echo off
REM Post-build script to copy FFmpeg DLLs to the executable directory

set VCPKG_BIN=%~dp0..\vcpkg_installed\x64-windows\bin
set TARGET_DIR=%~dp0..\target\debug

if not exist "%VCPKG_BIN%" (
    set VCPKG_BIN=%~dp0..\vcpkg\installed\x64-windows\bin
)

echo Copying FFmpeg DLLs from %VCPKG_BIN% to %TARGET_DIR%...

copy /Y "%VCPKG_BIN%\avcodec-61.dll" "%TARGET_DIR%\" >nul 2>&1
copy /Y "%VCPKG_BIN%\avdevice-61.dll" "%TARGET_DIR%\" >nul 2>&1
copy /Y "%VCPKG_BIN%\avfilter-10.dll" "%TARGET_DIR%\" >nul 2>&1
copy /Y "%VCPKG_BIN%\avformat-61.dll" "%TARGET_DIR%\" >nul 2>&1
copy /Y "%VCPKG_BIN%\avutil-59.dll" "%TARGET_DIR%\" >nul 2>&1
copy /Y "%VCPKG_BIN%\swresample-5.dll" "%TARGET_DIR%\" >nul 2>&1
copy /Y "%VCPKG_BIN%\swscale-8.dll" "%TARGET_DIR%\" >nul 2>&1
copy /Y "%VCPKG_BIN%\pkgconf-7.dll" "%TARGET_DIR%\" >nul 2>&1

echo Done! All FFmpeg DLLs copied.
