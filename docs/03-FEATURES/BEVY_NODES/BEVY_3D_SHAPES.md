# Bevy 3D Shapes Node

The **Bevy 3D Shapes Node** allows you to generate basic 3D geometric primitives directly within the MapFlow/MapMap environment using the Bevy engine integration.

## Overview

*   **Category:** Source > Bevy
*   **Icon:** 🧊
*   **Output:** Media (Texture of the rendered scene)

This node creates a single 3D object in the shared Bevy scene. It is useful for creating test patterns, background elements, or simple 3D compositions.

## Parameters

### Shape
Select the type of geometric primitive to render:
*   **Cube:** A standard box (1x1x1 units by default).
*   **Sphere:** A UV sphere.
*   **Capsule:** A capsule shape (cylinder with hemispherical ends).
*   **Torus:** A donut shape.
*   **Cylinder:** A cylinder.
*   **Plane:** A flat 2D plane (useful for floors or backgrounds).

### Appearance
*   **Color:** Sets the base color of the material (RGBA).
*   **Unlit Material:** If checked, the object ignores lighting and renders as a solid flat color (useful for masks or pure graphical elements).

### Transform (Local)
Controls the position, rotation, and scale of the object within the 3D scene.
*   **Pos (Position):** X, Y, Z coordinates.
*   **Rot (Rotation):** Euler rotation in degrees (X, Y, Z).
*   **Scl (Scale):** Scaling factors for X, Y, Z axes.

## Usage

1.  Add a **Source** node to the canvas.
2.  In the inspector, select **Bevy 3D Shape** from the "Bevy" dropdown category.
3.  Connect the **Media Out** socket to a Layer or Output node to see the result.
4.  Use **Trigger In** to toggle visibility or trigger animations (if supported in future updates).

## Performance Note
These shapes are rendered using the Bevy 3D engine. While efficient, having many complex 3D nodes active simultaneously may impact performance depending on your GPU.
