# Remote Control (MCP & OSC)

MapFlow provides comprehensive remote control capabilities through two primary interfaces: the Model Context Protocol (MCP) for AI agent integration, and Open Sound Control (OSC) for live performance, scripting, and media server interoperability. Both systems share a unified, expanded address space to ensure parity and headless operation.

## 1. Model Context Protocol (MCP) Tools

The MCP Server (`mapmap-mcp`) exposes a wide range of tools to fully operate MapFlow.

### Phase 1: Media in Layers
- **`layer_load_media`**: Load media file into a layer (requires `layer_id`, `media_path`).
- **`layer_set_media_time`**: Set media playback position (requires `layer_id`, `time_seconds`).
- **`layer_set_playback_speed`**: Set media playback speed (requires `layer_id`, `speed`).
- **`layer_set_loop_mode`**: Set layer loop mode (requires `layer_id`, `loop_mode`).
- **`media_library_list`**: List available media in library.
- **`media_import`**: Import media file into library.

### Phase 2: Audio Reactivity
- **`audio_bind_param`**: Bind audio frequency/beat to a parameter.
- **`audio_unbind_param`**: Remove an audio parameter binding.
- **`audio_bindings_list`**: List all active audio bindings.
- **`audio_set_sensitivity`**: Set sensitivity for a frequency band.
- **`audio_set_threshold`**: Set beat detection threshold.
- **`audio_analysis_config`**: Configure audio analysis parameters.

### Phase 3: Effects & Shaders
- **`effect_add`**: Add an effect to a layer.
- **`effect_remove`**: Remove an effect from a layer.
- **`effect_set_param`**: Set an effect parameter.
- **`effect_list`**: List all available effects.
- **`effect_chain_get`**: Get the effect chain for a layer.
- **`shader_load`**: Load a custom shader for a layer.
- **`shader_set_uniform`**: Set a shader uniform value.

### Phase 4: Timeline & Keyframes
- **`timeline_add_keyframe`**: Add a keyframe to the timeline.
- **`timeline_remove_keyframe`**: Remove a keyframe.
- **`timeline_get_keyframes`**: Get keyframes for a layer parameter.
- **`timeline_set_duration`**: Set the timeline duration.
- **`timeline_set_position`**: Set the timeline playback position.
- **`timeline_set_loop`**: Set timeline loop region.

### Phase 5: Mapping & Scenes
- **`surface_create`**: Create a new mapping surface.
- **`surface_delete`**: Delete a mapping surface.
- **`surface_set_corners`**: Update surface corner positions.
- **`surface_assign_layer`**: Assign a layer to a surface.
- **`mask_create`**: Create a mask for a layer.
- **`mask_edit`**: Edit mask points.
- **`scene_create`**: Create a new scene.
- **`scene_switch`**: Switch to a different scene.
- **`scene_list`**: List all scenes.
- **`preset_save`**: Save current state as a preset.
- **`preset_load`**: Load a saved preset.

---

## 2. Open Sound Control (OSC) Parity

OSC bindings provide real-time control with high precision. MapFlow maps OSC paths to internal `ControlTarget` actions, keeping parity with MCP.

### Address Space Mapping

**Layers & Media**
- `/mapmap/layer/{id}/opacity` -> Opacity control (0.0 to 1.0)
- `/mapmap/layer/{id}/position` -> Position coordinates
- `/mapmap/layer/{id}/rotation` -> Rotation angle
- `/mapmap/layer/{id}/scale` -> Scale multiplier
- `/mapmap/layer/{id}/visibility` -> Visibility toggle
- `/mapmap/layer/{id}/media/load` -> Load Media Path
- `/mapmap/layer/{id}/playback/time` -> Set media time
- `/mapmap/layer/{id}/playback/speed` -> Set playback speed
- `/mapmap/layer/{id}/loop_mode` -> Set loop mode

**Timeline & Playback**
- `/mapmap/playback/speed` -> Global playback speed
- `/mapmap/playback/position` -> Global playback position
- `/mapmap/timeline/position` -> Timeline specific position
- `/mapmap/timeline/duration` -> Timeline duration

**Parameters & Outputs**
- `/mapmap/paint/{id}/parameter/{name}` -> Paint module parameters
- `/mapmap/effect/{id}/parameter/{name}` -> Effect module parameters
- `/mapmap/output/{id}/brightness` -> Output display brightness
- `/mapmap/master/opacity` -> Global Master Opacity
- `/mapmap/master/blackout` -> Global Blackout toggle
