# Project Knowledge Base

## Jules Task Creation Protocol

**Date Added:** 2025-12-26

When creating new tasks for Jules (`mcp_jules_create_coding_task`), STRICTLY follow these rules to avoid failure:

1. **`branch` Parameter:**
    * This specifies the **BASE** branch to branch *from* (e.g., `main`, `master`).
    * **NEVER** enter the name of the new feature branch here.
    * **ALWAYS** use an existing stable branch (usually `main`).

2. **`source` Parameter:**
    * Must match the exact resource string identifier.
    * **Action:** Run `read_resource(ServerName="jules", Uri="jules://sources")` to get the list.
    * **Format:** Typically `sources/github/{Owner}/{Repo}`.
    * **Example:** `sources/github/MrLongNight/MapFlow`.

3. **Procedure:**
    * Check `jules://sources`.
    * Call `create_coding_task(title="...", branch="main", source="...", prompt="...")`.
