# 📚 Scribe's Journal

## 2026-01-10: Initial Audit & Cleanup

### 🔍 Discovery
- **Docs Structure:** The `docs/` folder structure mostly aligns with the standard, but contains some legacy artifacts.
- **Changelog Duplication:** Found a legacy `docs/08-CHANGELOG/CHANGELOG.md` (last update 2018) conflicting with the active root `CHANGELOG.md` (active 2026).
- **Index Outdated:** `docs/INDEX.md` hasn't been updated since 2025-12-04.
- **Crate Docs:** `mapmap-core` has decent module docs, but top-level structs like `Project` need more detail.

### 🛠️ Actions Taken
1. **Legacy Cleanup:** Removed the outdated 2018 changelog from `docs/` to prevent confusion. Replaced with a pointer to the root changelog.
2. **Index Update:** Updated `docs/INDEX.md` to reflect the current date and correct file locations.
3. **Core API Docs:** Enhanced the `Project` struct documentation in `mapmap-core` with detailed field descriptions and examples.

### 📝 Notes for Future Scribes
- The root `ROADMAP.md` and `CHANGELOG.md` are the sources of truth. `docs/` should reference them rather than duplicate them.
- `cargo doc` warnings in CI should be monitored to catch missing docs in new features.
