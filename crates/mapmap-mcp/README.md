# MapFlow MCP Server

This crate implements a [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server for MapFlow, enabling AI agents (like Claude Desktop or custom tools) to interact with and control the MapFlow application.

## Features

- **JSON-RPC 2.0**: Standard communication protocol over stdio.
- **Project Management**: Save and load projects via AI commands.
- **Layer Control**: Create, delete, modify, and mix layers.
- **Media Control**: Playback control (Play, Pause, Stop, Seek) and library management.
- **Audio Reactivity**: Bind audio analysis parameters (bass, beat) to visual properties.
- **Timeline**: Keyframe animation control.
- **Scenes & Presets**: Manage scenes and recall presets.

## Usage

This crate typically runs as a sidecar process or integrated module within the main application. However, it can also be run standalone for testing or specific integrations.

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

*   **Project**: `SaveProject`, `LoadProject`
*   **Layers**: `AddLayer`, `SetLayerOpacity`, `SetLayerBlendMode`
*   **Media**: `MediaPlay`, `LayerLoadMedia`
*   **Audio**: `AudioBindParam`, `AudioSetSensitivity`
*   **Effects**: `EffectAdd`, `EffectSetParam`
