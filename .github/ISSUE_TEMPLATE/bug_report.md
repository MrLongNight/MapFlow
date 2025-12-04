---
name: "Bug report"
about: "Report a bug in VjMapper — please include as much detail as possible to help us reproduce and fix it."
title: "[Bug] "
labels: "bug"
assignees: ""
---

## Short description
Summarize the bug briefly.

## Reproduction steps
1. VjMapper version (commit SHA or release):
2. Operating system (name + version):
3. GPU / driver (as precise as possible):
4. Steps to reproduce — please use a minimal, numbered list:
   - Step 1
   - Step 2
   - ...
5. Expected result
6. Actual result (attach screenshot/video if helpful)

## Minimal reproduction project
- If possible, attach a minimal repository, project files or a ZIP with the assets needed to reproduce.
- Alternatively, include a short code or config snippet below.

```text
# Example: brief config / command line / log excerpt
```

## Logs &amp; debug output
- Relevant log excerpts (full logs if available)
- Output with `RUST_LOG=debug` if relevant

## Pre-checked steps (please tick what you ran)
- [ ] `cargo fmt` executed
- [ ] `cargo clippy` executed
- [ ] `cargo test` executed successfully
- [ ] Latest GPU drivers installed
- [ ] Tested a release build

## Priority / frequency
- Reproducibility: one-off / always / under specific conditions: ...
- Impact: Blocker / High / Medium / Low

## Additional information
- Hardware specs, related issues/PRs, known workarounds
- If applicable: which crate/module is affected (e.g. `crates/render`, `crates/player`)
