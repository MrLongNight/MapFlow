# mapmap-mcp

Model Context Protocol (MCP) Server for MapFlow.

## Overview
This crate implements an MCP server that allows AI assistants (like Claude or Gemini) to interact with MapFlow. It exposes the application's API as "Tools" that the AI can call.

## Capabilities

- **State Inspection**: AI can read the current project structure, layers, and settings.
- **Control**: AI can create layers, add nodes, and modify parameters.
- **Media Management**: AI can search for and assign media files.

## Integration
The server communicates via standard input/output (stdio), making it compatible with MCP-compliant clients.
