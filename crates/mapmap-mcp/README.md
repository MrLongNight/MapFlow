# MapFlow MCP Server

The **Model Context Protocol (MCP)** server for MapFlow. This crate enables AI assistants (like Claude, Gemini, or custom agents) to interact with and control the MapFlow application.

## Overview

MapFlow MCP exposes the internal state and control surface of the application via the standard [Model Context Protocol](https://modelcontextprotocol.io/). This allows for:

- **Natural Language Control**: "Add a layer with the 'waves.mp4' file and set opacity to 50%."
- **Automated Workflows**: Scripts that can manipulate the project state.
- **Context-Aware Assistance**: AI agents can query the current project structure (layers, effects, mappings) to provide relevant help.

## Features

- **Project Management**: Save/Load projects.
- **Layer Control**: Create layers, set opacity, blend modes, and visibility.
- **Media Control**: Play, pause, seek, loop modes, and file management.
- **Effect Chain**: Add/remove effects and modify parameters.
- **Cue System**: Trigger cues and navigate the cue list.
- **Audio Reactivity**: Bind audio analysis parameters to visual properties.
- **Timeline Integration**: Manipulate keyframes and timeline playback.
- **Transport**: JSON-RPC 2.0 over Stdio or SSE (Server-Sent Events).

## Architecture

The MCP Server runs as a background service within the main MapFlow application (or as a standalone process for testing). It bridges external JSON-RPC requests to internal `McpAction` events, which are then processed by the main application loop.

### Tools

The server exposes a set of "Tools" that AI models can invoke. Examples include:

- `layer_add(name: String)`
- `media_play()`
- `effect_set_param(layer_id: u64, effect_id: u64, param: String, value: f32)`

## Usage

This crate is primarily used internally by `mapmap-control` and the main `mapmap` binary. To enable it, ensure the `mcp` feature is active (if applicable) or that the server is initialized in your configuration.

```rust,no_run
// Internal usage example
use mapmap_mcp::McpServer;

// The server is typically initialized by the App struct
// let server = McpServer::new(...);
```
