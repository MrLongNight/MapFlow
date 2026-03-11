$ScriptDir = Split-Path -Parent $PSCommandPath
& (Join-Path $ScriptDir 'dev-tools\pre-commit-check.ps1') @args
exit $LASTEXITCODE
