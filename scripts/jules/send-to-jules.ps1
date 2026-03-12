$ScriptDir = Split-Path -Parent $PSCommandPath
& (Join-Path $ScriptDir '..\archive\review\jules\send-to-jules.ps1') @args
exit $LASTEXITCODE
