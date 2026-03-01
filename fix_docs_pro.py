
import re
import os

def fix_docs(content, file_path):
    descriptions = {
        # General Fields
        "id": "Unique identifier for this entity.",
        "name": "Human-readable display name.",
        "version": "Version number for API or plugin compatibility.",
        "VERSION": "The current architectural version of the API or plugin.",
        "path": "File system path to the asset or resource.",
        "active": "Whether this component or feature is currently enabled and processing.",

        # UI & Graphics
        "color": "RGBA color value.",
        "position": "3D position coordinates [x, y, z].",
        "rotation": "Rotation angles in degrees.",
        "scale": "Scale factors for the object's dimensions.",
        "opacity": "Global opacity multiplier (0.0 to 1.0).",
        "blend_mode": "The blending algorithm used to composite this element.",
        "brightness": "Luminance offset applied to the image.",
        "contrast": "Difference between darkest and brightest areas.",
        "saturation": "Intensity of colors.",
        "hue_shift": "Color rotation in degrees.",

        # Media & Playback
        "speed": "Playback speed multiplier.",
        "loop_enabled": "Whether the media should automatically restart.",
        "start_time": "Timestamp where playback begins.",
        "end_time": "Timestamp where playback stops.",
        "target_fps": "Desired frame rate for playback or updates.",
        "reverse_playback": "When enabled, plays content backwards.",

        # MIDI & Control
        "channel": "The MIDI channel (0-15) associated with this message.",
        "note": "The MIDI note number (0-127).",
        "velocity": "Intensity value (0-127) of a MIDI note event.",
        "controller": "The MIDI continuous controller (CC) number.",
        "value": "The data value associated with the control or message.",
        "program": "The MIDI program/preset number.",
        "index": "Numerical index or position of the element.",
        "label": "User-friendly name for identifying the element.",
        "brand": "Manufacturer of the control hardware.",
        "model": "Specific model name of the hardware.",
        "elements": "List of interactive components (knobs, faders, buttons).",
        "mappings": "Set of links between control inputs and application targets.",
        "midi_input": "Handler for processing incoming MIDI messages.",
        "osc_server": "Server for receiving network control messages.",

        # Functions
        "new": "Creates a new, uninitialized instance with default settings.",
        "default": "Provides a default instance with standard configuration.",
        "update": "Processes pending events and updates internal state.",
        "apply_control": "Applies a control change after validation.",
        "execute_action": "Performs a high-level application action.",
        "handle_key_press": "Maps a physical key press to an action.",
        "from_bytes": "Parses a structure from a raw byte buffer.",
    }

    variants = {
        "NoteOn": "Triggered when a MIDI key or pad is pressed.",
        "NoteOff": "Triggered when a MIDI key or pad is released.",
        "ControlChange": "Sent when a continuous controller (fader/knob) is moved.",
        "ProgramChange": "Sent to switch device presets or programs.",
        "PitchBend": "Sent when the pitch bend wheel is adjusted.",
        "Clock": "MIDI timing clock message for synchronization.",
        "Note": "A standard MIDI Note event.",
        "CC": "A MIDI Continuous Controller event.",
        "Pressure": "MIDI Channel Pressure or Aftertouch event.",
        "Trigger": "Event-based trigger node.",
        "Source": "A node that provides visual or audio content.",
        "Modulizer": "A node that processes or modifies content.",
        "Layer": "A compositing layer within a scene.",
    }

    lines = content.splitlines()
    new_lines = []

    for i in range(len(lines)):
        line = lines[i]
        stripped = line.strip()

        garbage_phrases = ["Component property or field.", "Enumeration variant.", "Unique identifier.", "Display name.", "Unique ID"]
        is_garbage = any(f"/// {p}" in line for p in garbage_phrases)

        needs_doc = (stripped.startswith("pub ") or (i > 0 and lines[i-1].strip().startswith("#["))) and not stripped.startswith("pub use") and not stripped.startswith("pub mod")

        if not needs_doc and i > 0:
             prev_line = lines[i-1].strip()
             if (prev_line.startswith("///") or prev_line.startswith("#[")) and ":" in stripped and not stripped.startswith("impl"):
                 needs_doc = True

        if i > 0 and lines[i-1].strip().startswith("///") and not is_garbage:
            needs_doc = False

        if needs_doc or is_garbage:
            target_line = line if needs_doc else (lines[i+1] if i+1 < len(lines) else "")

            if i > 0 and "#[error(" in lines[i-1]:
                match = re.search(r'#\[error\("([^"]+)"', lines[i-1])
                if match:
                    err_text = match.group(1).split(":")[0].strip()
                    doc = f"/// Error: {err_text}."
                    if is_garbage: line = line.replace(line.strip(), doc)
                    else: new_lines.append(doc)
            else:
                name = None
                match = re.search(r'([a-zA-Z_0-9]+)\s*:', target_line)
                if match: name = match.group(1)
                else:
                    match = re.search(r'pub fn\s+([a-z_0-9]+)', target_line)
                    if match: name = match.group(1)
                    else:
                        match = re.search(r'^\s*([A-Z][a-zA-Z0-9]+)', target_line.strip())
                        if match: name = match.group(1)

                if name and name in descriptions:
                    doc = "/// " + descriptions[name]
                    if is_garbage: line = line.replace(line.strip(), doc)
                    else: new_lines.append(doc)
                elif name and name in variants:
                    doc = "/// " + variants[name]
                    if is_garbage: line = line.replace(line.strip(), doc)
                    else: new_lines.append(doc)

        new_lines.append(line)

    return "\n".join(new_lines)

paths = ['crates/mapmap-core/src', 'crates/mapmap-io/src', 'crates/mapmap-ui/src', 'crates/mapmap-control/src', 'crates/mapmap-ffi/src']
for path in paths:
    for root, dirs, files in os.walk(path):
        for file in files:
            if file.endswith('.rs'):
                file_path = os.path.join(root, file)
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                fixed_content = fix_docs(content, file_path)
                if fixed_content != content:
                    with open(file_path, 'w', encoding='utf-8', newline='\n') as f:
                        f.write(fixed_content)
                    print(f"Improved documentation in {file_path}")
