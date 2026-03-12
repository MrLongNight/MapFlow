@echo off
setlocal
call "%~dp0build\\copy-ffmpeg-dlls.bat" %*
exit /b %ERRORLEVEL%
