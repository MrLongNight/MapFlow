# MapFlow MCP Server

The **Model Context Protocol (MCP)** server for MapFlow. This crate enables AI assistants (like Claude, Gemini, or custom agents) to interact with and control the MapFlow application.

## Overview

MapFlow MCP exposes the internal state and control surface of the application via the standard [Model Context Protocol](https://modelcontextprotocol.io/). This allows for:

- **Natural Language Control**: "Add a layer with the 'waves.mp4' file and set opacity to 50%."
- **Automated Workflows**: Scripts that can manipulate the project state.
- **Context-Aware Assistance**: AI agents can query the current project structure (layers, effects, mappings) to provide relevant help.

## Architecture

The MCP Server runs as a background service within the main MapFlow application (or as a standalone process for testing). It bridges external JSON-RPC requests to internal `McpAction` events, which are then processed by the main application loop.

## Features

- **JSON-RPC 2.0**: Standard communication protocol over stdio or SSE.
- **Project Management**: Save and load projects via AI commands.
- **Layer Control**: Create, delete, modify, and mix layers.
- **Media Control**: Playback control (Play, Pause, Stop, Seek) and library management.
- **Audio Reactivity**: Bind audio analysis parameters (bass, beat) to visual properties.
- **Timeline**: Keyframe animation control.
- **Scenes & Presets**: Manage scenes and recall presets.

## Usage

This crate is primarily used internally by `mapmap-control` and the main `mapmap` binary. To enable it, ensure the `mcp` feature is active (if applicable) or that the server is initialized in your configuration.

It can also be run standalone for testing:

```bash
# Run the MCP server (stdio mode)
cargo run -p mapmap-mcp
```

## Integration

To integrate with an MCP client (e.g., Claude Desktop), add the following to your MCP settings file:

```json
{
  "mcpServers": {
    "mapflow": {
      "command": "cargo",
      "args": [
        "run",
        "-p",
        "mapmap-mcp",
        "--quiet"
      ]
    }
  }
}
```

## Supported Actions

The server supports a wide range of actions defined in `McpAction`, including:

- **Project**: `SaveProject`, `LoadProject`
- **Layers**: `AddLayer`, `SetLayerOpacity`, `SetLayerBlendMode`
- **Media**: `MediaPlay`, `LayerLoadMedia`
- **Audio**: `AudioBindParam`, `AudioSetSensitivity`
- **Effects**: `EffectAdd`, `EffectSetParam`
