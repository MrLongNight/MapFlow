# Scribe Journal - 2026-01-09

## Discrepancy in Documentation Structure

I noticed a significant discrepancy between the documentation structure described in typical "Inventory" prompts/templates (listing `01-OVERVIEW`, `04-API`) and the actual filesystem structure (`01-GETTING-STARTED`, `04-USER-GUIDE`).

### Observations
- The prompt templates often cite `docs/01-OVERVIEW/` as the first item.
- The actual repository has `docs/01-GETTING-STARTED/`.
- `docs/INDEX.md` correctly reflects the filesystem.
- This mismatch can lead to confusion during automated reviews or when referencing documentation paths in new files.

### Decision
- I have chosen to link to the **actual filesystem paths** (`01-GETTING-STARTED`) in `crates/mapmap/README.md` to avoid broken links.
- I am documenting this here to clarify why the implementation might appear to diverge from the prompt's "inventory".

---

# Scribe Journal - 2026-01-08

## Missing Crate READMEs

I discovered that while the root documentation and some crates were well-documented, several key crates lacked `README.md` files. This makes navigation within the `crates/` directory difficult for new developers.

### Actions Taken
- Created `crates/mapmap-control/README.md`
- Created `crates/mapmap-io/README.md`
- Created `crates/mapmap-render/README.md`
- Created `crates/mapmap-media/README.md`
- Created `crates/mapmap-ui/README.md`
- Created `crates/mapmap/README.md`

### Observations
- The `lib.rs` documentation is generally good and serves as a solid basis for the READMEs.
- The project structure is clean, but documentation fragmentation was a minor issue.
- `mapmap-mcp` and `mapmap-ffi` already had some docs or were skipped/handled separately (I saw `mapmap-ffi` had one).

### Future Recommendations
- Ensure `cargo doc` output is checked in CI to prevent regression in API docs.
- Consider generating READMEs from `lib.rs` docs automatically using tools like `cargo-readme` to avoid duplication.
