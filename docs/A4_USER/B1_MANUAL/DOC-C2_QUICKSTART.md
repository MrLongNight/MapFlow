# Quick Start Guide

Welcome to Vorce! This guide will get you projecting in 5 minutes.

## 1. Launch Vorce

Run the application (`vorce` or `vorce.exe`). You will see the **Dashboard** interface.

## 2. Import Media

1. Locate the **Media Browser** panel (usually on the left).
2. Click **"Import"** or drag-and-drop video/image files into the browser area.
3. Supported formats: `.mp4`, `.mov` (ProRes; HAP currently experimental), `.png`, `.jpg`, `.gif`.

## 3. Create a Projection Surface

1. Switch to the **Module Canvas** (center view).
2. Right-click on the canvas and select **Add Node > Layer Assignment > Single Layer**.
3. A new **Layer Node** will appear.

## 4. Connect Media to Layer

1. Drag your imported media file from the Media Browser onto the Canvas. It creates a **Source Node**.
2. Click and drag from the **Video Out** socket of the Source Node.
3. Connect it to the **Media In** socket of the Layer Node.

## 5. Output to Projector

1. Create a **Projector Node** (Right-click > Add Node > Output > Projector).
2. Connect the **Layer Out** socket of your Layer Node to the **Layer In** socket of the Projector Node.
3. The video should now appear in the Preview Window!
4. Start with projector/display output for first tests. Virtual outputs such as NDI/Spout depend on enabled features and are not the baseline path.

## Next Steps

- **Warping**: Select the Projector Node and look for "Mesh Editing" properties to adjust corners (Keystoning).
- **Effects**: Add an **Effect Node** (e.g., Blur, Colorize) between the Source and the Layer.
- **Audio**: Import an audio file or select an input device in Settings to make effects react to sound.

[Read the Full User Guide](./DOC-C0_README.md)
