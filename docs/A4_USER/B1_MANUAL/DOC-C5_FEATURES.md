# Key Features

Vorce is packed with features designed for professional VJs and projection mapping artists. This guide provides an overview of the core capabilities.

## 1. Modular Node System

Vorce uses a powerful node-based architecture that gives you complete control over your signal flow.

* **Flexible Routing**: Connect any source to any effect or output.
* **Reusability**: Create complex effect chains and reuse them across different layers.
* **Visual Programming**: See exactly how your video data is flowing and being processed.

## 2. Advanced Projection Mapping

Vorce excels at mapping video onto complex physical surfaces.

* **Mesh Warping**: Use grid-based warping to fit video onto curved or irregular shapes.
* **Keystone Correction**: Quickly align projections with 4-point corner pinning.
* **Masking**: Draw custom masks (Bezier curves) to hide unwanted parts of the projection.
* **Edge Blending**: Seamlessly blend multiple projectors to create a single, large image.

## 3. Media Playback

A robust media engine ensures smooth playback of high-resolution content.

* **Wide Format Support**: Playback of H.264, H.265 (HEVC), VP8, VP9, and ProRes video.
* **HAP Codec**: Experimental support exists, but the HAP playback path is not yet the default production media pipeline on all setups.
* **Image Sequences**: Support for folders of images (PNG, JPG) played back as video.
* **GIF Animation**: Full support for animated GIFs with variable frame delays.

## 4. Real-time Effects

Enhance your visuals with a suite of GPU-accelerated effects.

* **Shader Graph**: Create custom effects using a visual node editor. No coding required!
* **Built-in Effects**: Includes Blur, Color Correction, Distortion, Pixelate, and more.
* **Post-Processing**: Apply global effects to the final output.

## 5. Audio Reactivity

Make your visuals dance to the music.

* **FFT Analysis**: Real-time frequency analysis breaks audio into bands (Bass, Mid, High).
* **Beat Detection**: Automatic BPM detection and beat synchronization.
* **Audio Triggers**: Use audio levels to drive any parameter (e.g., scale a layer based on bass volume).

## 6. Control Integration

Vorce integrates seamlessly with your hardware and software setup.

* **MIDI**: Full two-way MIDI support. Map knobs and faders to any parameter. Includes presets for popular controllers (e.g., Ecler NUO 4).
* **OSC**: Open Sound Control support for remote control from tablets or other software.
* **NDI**: Feature-gated network video support. Output/runtime coverage exists, but not every input/output path is production-ready on every build.
* **Spout / Syphon**: Platform-specific interop support. Availability and runtime coverage depend on OS/build features and should currently be treated as advanced/experimental.

## 7. Multi-Output Support

* **Multiple Displays**: Drive multiple projectors or screens from a single computer.
* **Virtual Outputs**: Physical projector/display outputs are the stable path. Virtual outputs such as NDI or Spout depend on enabled features and platform/runtime support.
* **Color Calibration**: Fine-tune color and gamma for each output independently.

## 8. Bevy Particles Node

Generate dynamic particle systems directly within Vorce.

* **Real-time Simulation**: GPU-accelerated particle simulation using Bevy engine.
* **Audio Reactive**: Control particle emission rate, speed, and color with audio triggers.
* **Customizable**: Adjust lifetime, speed, and color gradients to match your visual style.
* **Performance**: Lightweight implementation designed for high frame rates.
