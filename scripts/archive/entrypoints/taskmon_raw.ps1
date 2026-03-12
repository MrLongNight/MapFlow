$ScriptDir = Split-Path -Parent $PSCommandPath
& (Join-Path $ScriptDir 'gemini-cli\taskmon-raw.ps1') @args
exit $LASTEXITCODE
