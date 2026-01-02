# MapFlow MCP (`mapmap-mcp`)

Model Context Protocol (MCP) Server for MapFlow. This allows AI agents to interact with and control the MapFlow application.

## Features

- **Tools**: Exposes internal functions (e.g., `load_project`, `set_opacity`) to AI agents.
- **Resources**: Provides access to application state and logs.
- **Prompts**: Defines standard AI interaction patterns.

## Usage

This crate is typically run as a sidecar process or integrated into the main application to enable AI-driven control.
