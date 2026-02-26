# JULES_SESSION_TEMPLATE
# Diese Vorlage wird von Clawdio für jede neue Session genutzt.

[TASK_DESCRIPTION]

## MANDATORY RULES FOR THIS SESSION:
1. **Validation**: Before finishing, you MUST run the local pre-commit script:
   `powershell.exe -ExecutionPolicy Bypass -File scripts\Slave-Local-PreCommit.ps1`
2. **Success Criterion**: If the script returns Exit Code 0, you are allowed to proceed. If it fails, fix the errors until it is green.
3. **Submission**: Upon successful validation, automatically submit a Pull Request to 'origin/main'.
4. **Context**: Use the existing MapFlow architecture and follow the 'Zero-Debt Policy' defined in SOUL.md.
