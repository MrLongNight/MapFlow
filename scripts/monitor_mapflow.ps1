$ScriptDir = Split-Path -Parent $PSCommandPath
& (Join-Path $ScriptDir 'gemini-cli\monitor-mapflow.ps1') @args
exit $LASTEXITCODE
