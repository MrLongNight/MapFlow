$ScriptDir = Split-Path -Parent $PSCommandPath
& (Join-Path $ScriptDir 'build\setup-vcpkg.ps1') @args
exit $LASTEXITCODE
