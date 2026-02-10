$branches = @(
    "feat/ws-auth-subprotocol-15045033110055451102",
    "cyber-dark-theme-refactor-797630662481252128",
    "optimize-mesh-renderer-writes-8498302422517446442",
    "feat/unified-hold-to-action-pattern-9802230962307445638",
    "jules-7436073049262925943-2f87a0e1",
    "guardian-test-enhancements-2148674246826468989"
)

foreach ($branch in $branches) {
    Write-Host "Fixing $branch"
    git checkout $branch
    git checkout main -- .jules/lina-styleui.md .jules/mary-styleux.md scripts/Final-Prepare-PreCommit.ps1 .agent/AGENTS.md .github/copilot-instructions.md .jules/JULES_INTEGRATION.md .jules/JULES_ISSUES_EXPLANATION.md .jules/SETUP_GUIDE.md docs/01-GENERAL/HOW_TO_CREATE_ISSUES.md docs/02-USER-GUIDE/MIDI_CONTROL.md docs/03-ARCHITECTURE/MULTI-PC-FEASIBILITY.md docs/05-DEVELOPMENT/Agents.md docs/06-ROADMAP/Planung_Multi-language.md docs/08-TECHNICAL/audits/2025-12-29-CODE_ANALYSIS_REPORT.md docs/10-CICD_PROZESS/WORKFLOW_CONTROL.md docs/10-CICD_PROZESS/WORKFLOW_QUICKREF.md
    git rm --cached -r .Jules
    python fix_formatting.py
    cargo fmt --all
    git add -A
    git commit -m "fix: restore journals and scripts"
    git push origin $branch --force
}
git checkout main
