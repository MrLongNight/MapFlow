# Bevy Particles Node

The **Bevy Particles** node is a specialized **Source** node that generates a 3D particle system using the Bevy game engine. It is designed for high-performance visual effects.

## Parameters

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| **Rate** | Trigger/Float | Number of particles to spawn per second. | 100.0 |
| **Lifetime** | Float | How long each particle lives in seconds. | 2.0 |
| **Speed** | Float | Initial velocity magnitude. Particles move outward from the center. | 1.0 |
| **Color Start** | Color | RGBA color at birth. | Orange |
| **Color End** | Color | RGBA color at death. | Dark Red |
| **Position** | Vector3 | World space position offset. | [0, 0, 0] |
| **Rotation** | Vector3 | World space rotation. | [0, 0, 0] |

## Behavior

- Particles are spawned continuously based on the **Rate**.
- Each particle is emitted from the center (0,0,0 local space) with a random direction.
- Particles face the camera (Billboarding).
- Color interpolates linearly from **Start** to **End** over the **Lifetime**.
- The particle system is rendered as a single dynamic mesh for optimal performance (CPU Simulation, GPU Instancing via Batching).

## Usage

1. Add a **Source > Bevy Particles** node to your graph.
2. Connect its **Media Out** to a Layer or Output.
3. Adjust **Rate** and **Speed** to control density and flow.
4. Use **Color Start/End** to create fire, smoke, or magic effects.

## Technical Details

- **Implementation:** CPU-simulated particle system updating a dynamic `Mesh` every frame.
- **Rendering:** Uses Bevy's `StandardMaterial` with Vertex Colors.
- **Performance:** optimized for < 10,000 particles.
