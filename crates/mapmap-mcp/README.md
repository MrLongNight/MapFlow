# MapFlow MCP

Model Context Protocol (MCP) server integration for MapFlow.

## Purpose

This crate exposes MapFlow's internal state and control surface to AI agents via the Model Context Protocol. It allows LLMs to:
- Inspect application state.
- Control playback and parameters.
- Query documentation and resources.

## Features

- **JSON-RPC 2.0**: Transport layer over stdio or SSE.
- **Tools**: Exposed functions like `play`, `pause`, `set_opacity`.
- **Resources**: Access to logs, config, and state.
- **Prompts**: Pre-defined prompts for AI assistance.
