# Bevy Camera Node

The Bevy Camera Node provides control over the main camera in the Bevy 3D scene. It allows you to switch between different camera behaviors like Orbit, Fly, and Static.

## Modes

### Orbit
Rotates the camera around a target point.
- **Target**: The center point of the orbit.
- **Distance**: The radius of the orbit.
- **Speed**: The speed of rotation (radians per second).
- **Position Y**: Used as a height offset relative to the target.

### Fly
Moves the camera continuously in a specific direction.
- **Position**: The starting position (at time t=0).
- **Target**: Defines the direction vector (from Position to Target).
- **Speed**: The speed of movement along the direction vector.
- **Note**: The movement is based on global time, so it is deterministic.

### Static
Fixed position and orientation.
- **Position**: The exact location of the camera.
- **Target**: The point the camera looks at.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| **Camera Type** | Enum | Orbit, Fly, Static |
| **Position** | Vec3 | Starting position or fixed location. |
| **Target** | Vec3 | Look-at target or direction reference. |
| **Distance** | Float | Radius for Orbit mode. |
| **Speed** | Float | Rotation speed (Orbit) or movement speed (Fly). |
| **FOV** | Float | Field of View in degrees. |

## Usage

1. Add a `Bevy Camera` node to your graph.
2. Connect it to the graph.
3. Adjust parameters to achieve the desired shot.
