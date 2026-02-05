# OSC Command Reference

This document provides a comprehensive reference for the Open Sound Control (OSC)
address space used in MapFlow.

## Overview

- **Namespace:** All MapFlow commands are prefixed with `/mapmap`.
- **Data Types:** OSC arguments are specified in brackets, e.g., `[f32]`. Common types include:
  - `[bang]`: An OSC message with no arguments.
  - `[bool]`: Boolean value (`true` or `false`).
  - `[i32]`: 32-bit integer.
  - `[f32]`: 32-bit float, typically normalized between `0.0` and `1.0` unless otherwise specified.
  - `[string]`: A string of characters.
  - `[f32, f32]`: Multiple float arguments for vectors (e.g., X and Y coordinates).

---

## 1. Layer Controls

Address layers by their unique ID (e.g., `/mapmap/layer/1/...`).

| Address                       | Arguments                            | Description                                                 |
| ----------------------------- | ------------------------------------ | ----------------------------------------------------------- |
| `/mapmap/layer/{id}/opacity`  | `[f32: 0.0-1.0]`                     | Sets the opacity for the specified layer.                   |
| `/mapmap/layer/{id}/visibility`| `[bool]`                             | Toggles the visibility of the layer.                        |
| `/mapmap/layer/{id}/position` | `[f32, f32]`                         | Sets the X and Y position of the layer.                     |
| `/mapmap/layer/{id}/rotation` | `[f32]`                              | Sets the rotation in degrees.                               |
| `/mapmap/layer/{id}/scale`    | `[f32]`                              | Sets the uniform scale.                                     |

---

## 2. Paint Controls

Control paint parameters by their ID.

| Address                                  | Arguments                      | Description                                                  |
| ---------------------------------------- | ------------------------------ | ------------------------------------------------------------ |
| `/mapmap/paint/{id}/parameter/{name}`    | `[varies]`                     | Controls a specific parameter of the paint (e.g., "color").  |

---

## 3. Effect Controls

Control effect parameters by their ID.

| Address                                   | Arguments        | Description                                                  |
| ----------------------------------------- | ---------------- | ------------------------------------------------------------ |
| `/mapmap/effect/{id}/parameter/{name}`    | `[varies]`       | Controls a specific parameter of the effect (e.g., "amount").|

---

## 4. Playback Controls

Global playback commands.

| Address                   | Arguments          | Description                                                     |
| ------------------------- | ------------------ | --------------------------------------------------------------- |
| `/mapmap/playback/speed`  | `[f32]`            | Sets the playback speed (1.0 is normal speed).                  |
| `/mapmap/playback/position`| `[f32: 0.0-1.0]`   | Jumps to a specific position in the media.                      |

---

## 5. Output Controls

Manage individual outputs and projectors.

| Address                          | Arguments        | Description                                       |
| -------------------------------- | ---------------- | ------------------------------------------------- |
| `/mapmap/output/{id}/brightness` | `[f32: 0.0-1.0]` | Sets the brightness for the specified output.     |
