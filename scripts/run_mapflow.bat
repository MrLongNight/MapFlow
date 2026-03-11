@echo off
setlocal
call "%~dp0mapflow\\run-mapflow.bat" %*
exit /b %ERRORLEVEL%
