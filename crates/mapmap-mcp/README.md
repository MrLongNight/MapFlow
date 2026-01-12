# mapmap-mcp

**Model Context Protocol (MCP) Server for MapFlow.**

`mapmap-mcp` implements an MCP server that exposes MapFlow's capabilities to AI agents. It allows AI assistants to interact with the application programmatically, enabling features like automated project setup, live control, and intelligent assistance.

## Features

- **JSON-RPC 2.0:** Standardized communication protocol over stdio.
- **Project Management:** Save and load projects via AI commands.
- **Layer Control:** Create, delete, and modify layers (opacity, visibility, blend modes).
- **Media Control:** Playback control (play, pause, stop) and file management.
- **Cue System:** Trigger cues and navigate the cue list.
- **Audio Reactivity:** Bind audio analysis parameters to visual properties.
- **Timeline Integration:** Manipulate keyframes and timeline playback.

## Usage

This crate is integrated into the main `mapmap` application. When enabled (and if configured), the application acts as an MCP server.

## Architecture

The crate defines `McpAction` enums which translate RPC calls into internal application events. These events are then processed by the main application loop.

```rust
use mapmap_mcp::McpAction;

// Example action trigger
let action = McpAction::AddLayer("Background".to_string());
```
