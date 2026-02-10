# Bevy 3D Text Node

The **Bevy 3D Text Node** allows you to render text within the 3D Bevy scene. It provides controls for content, styling, and 3D transformation.

## Parameters

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| **Text** | String | The actual text content to display. | "" |
| **Font Size** | Float | Size of the text font. | 32.0 |
| **Color** | Color (RGBA) | Color of the text. | White (1.0, 1.0, 1.0, 1.0) |
| **Position** | Vector3 | 3D position [X, Y, Z] in the scene. | [0.0, 0.0, 0.0] |
| **Rotation** | Vector3 | 3D rotation [X, Y, Z] in degrees (Euler angles). | [0.0, 0.0, 0.0] |
| **Alignment** | Enum | Text alignment (Left, Center, Right, Justify). | Center |

## Usage

1.  Add a **Bevy 3D Text** node to your MapFlow graph (under Sources -> Bevy).
2.  Connect its `Media Out` to a layer or effect chain.
3.  Adjust the `Text` field to change the message.
4.  Use `Position` and `Rotation` to place the text in the 3D world relative to the camera.
5.  Use `Font Size` and `Color` to style the text.

## Notes

*   This node uses the default Bevy font (`FiraMono-Medium.ttf` if available via `bevy_text` feature).
*   The text is rendered as a 2D plane (billboard) in the 3D world (`Text2d`), not as extruded 3D geometry.
*   Performance is generally high, but spawning thousands of text nodes may have an impact.
