$ScriptDir = Split-Path -Parent $PSCommandPath
& (Join-Path $ScriptDir 'gemini-cli\taskmon-summary.ps1') @args
exit $LASTEXITCODE
