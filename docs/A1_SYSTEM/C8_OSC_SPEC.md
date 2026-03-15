# MapFlow OSC Control Specification

This document details the Open Sound Control (OSC) address space for MapFlow, which has 1:1 parity with the MCP (Model Context Protocol) API.

## Addresses

### Timeline
- `/mapmap/timeline/play` (no args or 1 for play, 0 for pause)
- `/mapmap/timeline/stop`
- `/mapmap/timeline/speed` [f32]
- `/mapmap/timeline/loop` [bool or 0/1]

### Effect Management
- `/mapmap/layer/{id}/effect/add` [string: effect_type]
- `/mapmap/layer/{id}/effect/{effect_id}/remove`
- `/mapmap/layer/{id}/effect/{effect_id}/parameter/{param_name}` [f32: value]

### Surface / Warping
- `/mapmap/surface/{id}/corner/{index}/position` [f32, f32: x, y]

### Scene / Cue
- `/mapmap/scene/switch/{id}`
- `/mapmap/cue/trigger/{id}`
