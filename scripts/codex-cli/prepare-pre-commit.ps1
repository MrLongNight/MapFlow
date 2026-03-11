$ScriptDir = Split-Path -Parent $PSCommandPath
& (Join-Path $ScriptDir '..\dev-tools\prepare-pre-commit.ps1') -Profile 'codex-cli' @args
exit $LASTEXITCODE
