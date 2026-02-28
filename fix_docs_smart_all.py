import re
import os

def fix_docs(content):
    # Dictionary for field-based replacements
    field_descriptions = {
        "turbidity": "Atmospheric turbidity, affecting how clear or hazy the sky appears.",
        "rayleigh": "Rayleigh scattering coefficient, determining the sky color (blue during day, red at sunset).",
        "mie_coeff": "Mie scattering coefficient, affecting the size and intensity of the solar disk.",
        "mie_directional_g": "Mie scattering directionality (G-parameter), controlling how much light is scattered forward.",
        "sun_position": "Spherical coordinates (Azimuth, Elevation) of the sun in the sky.",
        "exposure": "HDR exposure level for the atmospheric rendering.",
        "radius": "Radius of the hexagonal grid or shape.",
        "rings": "Number of rings in the hexagonal grid layout.",
        "pointy_top": "Orientation of the hexagons: true for pointy top, false for flat top.",
        "spacing": "Distance between individual elements in a grid or particle system.",
        "rate": "Emission rate of particles per second.",
        "lifetime": "Maximum time a particle remains active before being destroyed.",
        "color_start": "Initial color of an element at the beginning of its lifecycle.",
        "color_end": "Final color of an element at the end of its lifecycle.",
        "unlit": "Disables lighting calculations, making the object appear with its full emission/color regardless of light sources.",
        "outline_width": "Thickness of the selection or decorative outline.",
        "outline_color": "The color used for the object's outline.",
        "text": "The literal string content to be rendered.",
        "font_size": "The size of the characters in pixels or points.",
        "alignment": "Alignment of the content (e.g., 'Center', 'Left', 'Right').",
        "fov": "Field of view in degrees for the camera's perspective projection.",
        "active": "Whether this component or feature is currently enabled and processing.",
        "loop_enabled": "Whether the media should automatically restart after reaching the end.",
        "start_time": "Timestamp (in seconds) where playback should begin within the media file.",
        "end_time": "Timestamp (in seconds) where playback should stop within the media file. 0.0 means end of file.",
        "brightness": "Luminance offset applied to the final image.",
        "contrast": "Difference between the darkest and brightest areas of the image.",
        "saturation": "Intensity of colors; 0.0 is grayscale, 1.0 is normal, >1.0 is vivid.",
        "hue_shift": "Rotates the colors around the hue circle in degrees.",
        "id": "Unique identifier for this entity.",
        "name": "Human-readable display name.",
        "color": "RGBA color value.",
        "position": "3D position coordinates [x, y, z].",
        "rotation": "Rotation angles in degrees.",
        "scale": "Scale factors for the object's dimensions.",
        "opacity": "Global opacity multiplier (0.0 to 1.0).",
        "blend_mode": "The blending algorithm used to composite this element with the background.",
        "scale_x": "Scale factor along the horizontal axis.",
        "scale_y": "Scale factor along the vertical axis.",
        "offset_x": "Horizontal translation offset.",
        "offset_y": "Vertical translation offset.",
        "target_width": "Optional width constraint for internal buffers or rendering targets.",
        "target_height": "Optional height constraint for internal buffers or rendering targets.",
        "target_fps": "Desired frame rate for playback or updates.",
        "reverse_playback": "When enabled, plays the content from end to beginning.",
        "params": "Dynamic parameters for the component, usually as (Name, Value) pairs.",
        "path": "File system path to the asset or resource.",
        "device_id": "System index of the hardware device.",
        "source_name": "Identification string for the network or system source.",
        "shared_id": "Unique key for accessing a shared resource or media pool.",
        "flip_horizontal": "Mirror the content horizontally.",
        "flip_vertical": "Mirror the content vertically.",
        "shape_type": "The geometric form of the 3D object (e.g., Cube, Sphere, Plane).",
        "modules": "The collection of all modules, indexed by their unique ID.",
        "next_module_id": "The ID to be assigned to the next created module.",
        "next_part_id": "The ID to be assigned to the next created part within a module.",
        "color_palette": "Predefined list of colors available for UI elements.",
        "next_color_index": "Index for cycling through the color palette for new items.",
        "shared_media": "Global registry of media assets shared across the project.",
        "graph_revision": "Incremental counter tracking changes to the graph structure.",
    }

    # Dictionary for variant-based replacements
    variant_descriptions = {
        "BevyAtmosphere": "Simulates realistic sky and atmospheric scattering.",
        "BevyHexGrid": "Renders a grid of hexagons in 3D space.",
        "BevyParticles": "GPU-accelerated 3D particle system.",
        "Bevy3DShape": "Standard 3D geometric primitive (Cube, Sphere, etc.).",
        "Bevy3DModel": "External 3D asset loaded from a file (GLTF, OBJ).",
        "Bevy3DText": "Three-dimensional text rendered in the scene.",
        "BevyCamera": "Virtual camera defining the viewpoint of the Bevy engine.",
        "SpoutInput": "Inter-process video sharing via Spout (Windows).",
        "VideoUni": "Single-instance video source with full transform controls.",
        "VideoMulti": "Media source sharing its state across multiple instances.",
        "Shader": "Custom WGSL shader source for creative effects.",
        "LiveInput": "Real-time capture from a webcam or capture card.",
        "NdiInput": "Network-based video stream via NDI protocol.",
        "SolidColor": "Fills the area with a single uniform color.",
        "SolidGradient": "Renders a color gradient across the surface.",
        "SyphonInput": "Inter-process video sharing via Syphon (macOS).",
        "Hue": "Integration with Philips Hue smart lighting.",
        "Layer": "A compositing layer within a scene.",
        "Trigger": "Event-based trigger node.",
        "Mesh": "Geometry definition for mapping.",
    }

    lines = content.split('\\n')
    new_lines = []
    for i in range(len(lines)):
        line = lines[i]
        
        # Replace garbage property comments
        if "/// Component property or field." in line:
            if i + 1 < len(lines):
                next_line = lines[i+1]
                match = re.search(r'([a-z_0-9]+)\\s*:', next_line)
                if match:
                    field_name = match.group(1)
                    if field_name in field_descriptions:
                        line = line.replace("/// Component property or field.", "/// " + field_descriptions[field_name])
        
        # Replace garbage variant comments
        if "/// Enumeration variant." in line:
            if i + 1 < len(lines):
                next_line = lines[i+1]
                match = re.search(r'^\\s*([A-Z][a-zA-Z0-9]+)', next_line)
                if match:
                    variant_name = match.group(1)
                    if variant_name in variant_descriptions:
                        line = line.replace("/// Enumeration variant.", "/// " + variant_descriptions[variant_name])

        # Replace other redundant comments
        redundant_phrases = ["Unique identifier.", "Display name.", "Unique ID", "Rotation angle.", "Playback speed multiplier.", "X axis offset.", "Y axis offset.", "Scale factor on X axis.", "Scale factor on Y axis."]
        for phrase in redundant_phrases:
            if f"/// {phrase}" in line:
                if i + 1 < len(lines):
                    next_line = lines[i+1]
                    match = re.search(r'([a-z_0-9]+)\\s*:', next_line)
                    if match:
                        field_name = match.group(1)
                        if field_name in field_descriptions:
                            line = line.replace(f"/// {phrase}", "/// " + field_descriptions[field_name])

        new_lines.append(line)

    return '\\n'.join(new_lines)

paths = ['crates/mapmap-core/src', 'crates/mapmap-io/src', 'crates/mapmap-ui/src']
for path in paths:
    for root, dirs, files in os.walk(path):
        for file in files:
            if file.endswith('.rs'):
                file_path = os.path.join(root, file)
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                if "/// Component property or field." in content or "/// Enumeration variant." in content or "/// Unique identifier." in content:
                    fixed_content = fix_docs(content)
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(fixed_content)
                    print(f"Fixed documentation in {file_path}")
