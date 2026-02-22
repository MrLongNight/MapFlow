## 2025-02-18 - [Critical Test Gaps]

**Erkenntnis:** Critical socket generation logic in `module.rs` was relying on untested `match` arms, particularly for `Bevy` source types and `Hue` integration. `Layer` transformation logic also lacked explicit verification of delegate calls to `Transform`.

**Aktion:** Implemented comprehensive socket verification tests (`test_bevy_source_sockets`, `test_hue_sockets`) and transform delegation tests (`test_layer_transform_delegation`). Future PRs should strictly enforce `socket_type` verification for any new `ModulePartType`.
