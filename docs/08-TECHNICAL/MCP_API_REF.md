# MapFlow Control Protocol (MCP) API

This document describes the MapFlow Control Protocol (MCP) integration, which allows external agents (like Jules) to control the MapFlow application.

## Overview

The MCP server runs embedded within the MapFlow application and communicates via `stdio` using JSON-RPC 2.0. It adheres to the Model Context Protocol standard.

## Tools

The following tools are exposed by the MCP server:

### Project Management

* **`project_save`**
  * Description: Save the current project.
  * Parameters:
    * `path` (string, optional): Path to save the project file. If omitted, opens a save dialog.

* **`project_load`**
  * Description: Load a project.
  * Parameters:
    * `path` (string, optional): Path to the project file to load. If omitted, opens a load dialog.

### Layer Control

* **`layer_create`**
  * Description: Create a new layer.
  * Parameters:
    * `name` (string, optional): Name of the new layer.

* **`layer_delete`**
  * Description: Delete a layer.
  * Parameters:
    * `id` (number): The ID of the layer to delete.

* **`layer_set_opacity`**
  * Description: Set the opacity of a layer.
  * Parameters:
    * `layer_id` (number): The ID of the layer.
    * `opacity` (number): Opacity value (0.0 to 1.0).

* **`layer_set_visibility`**
  * Description: Set the visibility of a layer.
  * Parameters:
    * `layer_id` (number): The ID of the layer.
    * `visible` (boolean): Visibility state.

### Media Control

* **`media_play`**
  * Description: Start playback on a layer.
  * Parameters:
    * `layer_id` (number): The ID of the layer.

* **`media_pause`**
  * Description: Pause playback on a layer.
  * Parameters:
    * `layer_id` (number): The ID of the layer.

* **`media_stop`**
  * Description: Stop playback on a layer.
  * Parameters:
    * `layer_id` (number): The ID of the layer.

### Cue System

* **`cue_trigger`**
  * Description: Trigger a specific cue.
  * Parameters:
    * `id` (number): The ID of the cue to trigger.

* **`cue_next`**
  * Description: Go to the next cue.
  * Parameters: None.

* **`cue_previous`**
  * Description: Go to the previous cue.
  * Parameters: None.

### OSC

* **`send_osc`**
  * Description: Send an OSC message.
  * Parameters:
    * `address` (string): OSC address (e.g., "/layer/1/opacity").
    * `args` (array): List of arguments (float, int, string).

## Resources

The MCP server provides access to application state via resources:

* **`mapmap://resources/list`**: Lists all available resources.
* **`mapmap://app/state`**: Read-only access to the full application state (JSON).
* **`mapmap://layers`**: List of layers.
* **`mapmap://cues`**: List of cues.

## Prompts

* **`prompts/list`**: List available prompts.
* **`prompts/get`**: Get a specific prompt.

## Integration

To control MapFlow from an MCP client:

1. Start MapFlow.
2. Connect to the MCP server via `stdio`.
3. Send JSON-RPC requests to call tools or access resources.
