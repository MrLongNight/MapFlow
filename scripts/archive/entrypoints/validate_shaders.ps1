$ScriptDir = Split-Path -Parent $PSCommandPath
& (Join-Path $ScriptDir 'build\validate-shaders.ps1') @args
exit $LASTEXITCODE
