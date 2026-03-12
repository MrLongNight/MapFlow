@echo off
setlocal
call "%~dp0build\\copy-ffmpeg-debug-dlls.bat" %*
exit /b %ERRORLEVEL%
