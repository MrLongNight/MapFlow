# mapmap-mcp

**mapmap-mcp** implements a Model Context Protocol (MCP) server for MapFlow. It allows AI agents (like Claude or custom LLMs) to interact with the running MapFlow instance.

## Key Features

*   **MCP Server**: Implements the MCP specification (JSON-RPC 2.0).
*   **Transports**: Supports Stdio (Standard Input/Output) and SSE (Server-Sent Events) transports.
*   **Tools**: Exposes MapFlow functionality as tools for AI agents (e.g., "list_layers", "set_opacity").
*   **Resources**: Provides read access to application state as resources.
*   **Prompts**: Defines standard prompts for AI interactions.

## Usage

This crate is typically run as a sidecar process or integrated into the main application to enable AI-assisted control.
