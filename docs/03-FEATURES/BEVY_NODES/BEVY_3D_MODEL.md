# Bevy 3D Model Node

The **Bevy 3D Model Node** allows you to load and display 3D models (GLTF/GLB) within the Bevy scene environment.

## Overview

This node acts as a source in the Bevy compositing pipeline. It spawns a 3D model entity into the shared Bevy scene, which is then rendered to the texture used by MapFlow.

## Parameters

*   **Path**: The file path to the GLTF or GLB model. This can be a relative path (from assets folder) or an absolute path.
*   **Position**: The [X, Y, Z] position of the model in the 3D world.
*   **Rotation**: The [X, Y, Z] rotation (Euler angles in degrees).
*   **Scale**: The [X, Y, Z] scale of the model. Default should be [1.0, 1.0, 1.0].

## Sockets

*   **Trigger In**: (Trigger) Standard visibility/activation trigger.
*   **Media Out**: (Media) The rendered output of the Bevy scene (note: currently Bevy nodes render to a shared framebuffer, so this output represents the "presence" of the model in that shared scene).

## Usage

1.  Add a `Bevy 3D Model` node to the Module Canvas.
2.  Enter the path to your `.glb` or `.gltf` file.
3.  Adjust Position, Rotation, and Scale to place the model in the scene.
4.  Connect the `Media Out` to a Layer or Output to visualize the Bevy scene.

## Notes

*   Ensure the path is accessible by the application.
*   Loading large models may take a moment.
*   The model is lit by the default scene lighting (currently a point light).
