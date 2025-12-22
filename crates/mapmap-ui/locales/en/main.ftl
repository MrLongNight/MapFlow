#
# MapFlow - English Translations
#

#
# General / App-wide
#
app-title = MapFlow
project-filter-name = MapFlow Project
new-project-name = New Project
paint-new-color-name = New Color
new-output-name = New Output

#
# Main Menu & Toolbar (egui)
#
menu-file = File
menu-file-new-project = New Project...
menu-file-open-project = Open Project...
menu-file-open-recent = Open Recent
menu-file-save-project = Save Project
menu-file-save-as = Save As...
menu-file-export = Export...
menu-file-settings = Settings...
menu-file-exit = Exit

menu-edit = Edit
menu-edit-undo = Undo
menu-edit-redo = Redo
menu-edit-cut = Cut
menu-edit-copy = Copy
menu-edit-paste = Paste
menu-edit-delete = Delete
menu-edit-select-all = Select All

menu-view = View
view-egui-panels = Egui Panels
view-legacy-panels = Legacy Panels
view-reset-layout = Reset Layout
btn-fullscreen = Toggle Fullscreen

menu-help = Help
menu-help-docs = Documentation
menu-help-about = About
menu-help-license = License
menu-language = Language
language-en = English
language-de = Deutsch

toolbar-save = Save
toolbar-undo = Undo
toolbar-redo = Redo

#
# View Menu Checkboxes
#
check-show-osc = Show OSC
check-show-controls = Show Playback
check-show-layers = Show Layers
check-show-paints = Show Paints
check-show-mappings = Show Mappings
check-show-transforms = Show Transforms
check-show-master = Show Master
check-show-oscillator = Show Oscillator
check-show-audio = Show Audio
check-show-cues = Show Cues
check-show-stats = Show Performance
panel-dashboard = Dashboard
panel-media-browser = Media Browser
panel-asset-manager = Asset Manager
panel-mesh-editor = Mesh Editor
panel-node-editor = Node Editor
panel-timeline = Timeline
panel-effect-chain = Effect Chain

#
# Dashboard
#
panel-dashboard-title = Dashboard
dashboard-layout-grid = Grid
dashboard-layout-freeform = Freeform
dashboard-columns = Columns
dashboard-add-widget = ➕ Add Widget
dashboard-add-widget-tooltip = Add a new widget to the dashboard
dashboard-state = State
dashboard-speed = Speed
dashboard-loop = Loop
dashboard-audio-analysis = Audio Analysis
dashboard-device = Device
dashboard-no-device = No device selected
dashboard-volume = Volume
dashboard-rms = RMS
dashboard-peak = Peak
dashboard-spectrum = Frequency Spectrum
dashboard-no-audio-data = No audio analysis data available.
dashboard-remove-widget = Remove widget
dashboard-trigger = Trigger
btn-play-icon = ▶
btn-pause-icon = ⏸
btn-stop-icon = ⏹

#
# Legacy Playback Controls
#
panel-playback = Playback Controls
header-video-playback = Video Playback
btn-play = Play
btn-pause = Pause
btn-stop = Stop
label-speed = Speed
label-mode = Mode
mode-loop = Loop
mode-play-once = Play Once

#
# Performance Panel
#
panel-performance = Performance
label-fps = FPS
label-frame-time = Frame Time

#
# Layers Panel
#
panel-layers = Layers
label-total-layers = Total Layers: { $count }
check-bypass = Bypass (B)
check-solo = Solo (S)
label-master-opacity = Opacity (V)
btn-duplicate = Duplicate
btn-remove = Remove
btn-add-layer = Add Layer
btn-eject-all = Eject All (X)

#
# Paints Panel
#
panel-paints = Paints
label-total-paints = Total Paints: { $count }
check-playing = Playing
paints-color = Color
paint-label-name-type = { $name } ({ $type })
btn-add-paint = Add Paint
btn-remove-paint = Remove

#
# Mappings Panel
#
panel-mappings = Mappings
label-total-mappings = Total Mappings: { $count }
check-lock = Lock
label-depth = Depth
label-mesh = Mesh: { $type } ({ $count } vertices)
btn-add-quad = Add Quad Mapping
btn-remove-this = Remove

#
# Transform Panel
#
panel-transforms = Transform
header-transform-sys = Transform System
label-editing = Editing
transform-position = Position
drag-val-x = X:
drag-val-y = Y:
transform-rotation = Rotation (degrees)
slider-z = Z
btn-reset-rotation = Reset Rotation
transform-scale = Scale
drag-val-w = W:
drag-val-h = H:
btn-reset-scale = Reset Scale (1:1)
label-anchor = Anchor Point (0-1)
btn-center-anchor = Center Anchor (0.5, 0.5)
transform-presets = Resize Presets
transform-fill = Fill (Cover)
btn-resize-fit = Fit (Contain)
btn-resize-stretch = Stretch (Distort)
btn-resize-original = Original (1:1)
transform-no-layer = No layer selected.
transform-select-tip = Click a layer name in the Layers panel to select it.

#
# Master Controls Panel
#
panel-master = Master Controls
header-master = Master Controls
label-composition = Composition
label-master-speed = Master Speed (S)
label-size = Size
label-frame-rate = Frame Rate
label-effective-multipliers = Effective Multipliers:
text-mult-opacity = All layer opacity × Master Opacity
text-mult-speed = All playback speed × Master Speed

#
# Output Panel
#
panel-outputs = Outputs
header-outputs = Outputs
header-selected-output = Selected Output
label-canvas = Canvas
btn-projector-array = 2x2 Projector Array
btn-add-output = Add Output
label-name = Name
label-resolution = Resolution
label-canvas-region = Canvas Region
label-x = X
label-y = Y
label-width = Width
label-height = Height
label-none = None
output-tip = Tip
tip-panels-auto-open = Edge Blending and Color Calibration panels open automatically when an output is selected.
btn-remove-output = Remove Output
msg-multi-window-active = Multi-window rendering is ACTIVE.
msg-output-windows-tip = Output windows are automatically created and synchronized.

#
# Edge Blend Panel
#
panel-edge-blend = Edge Blending
label-output = Output: { $name }
check-left = Left Edge
check-right = Right Edge
check-top = Top Edge
check-bottom = Bottom Edge
label-offset = Offset
label-gamma = Blend Gamma
btn-reset-defaults = Reset to Defaults

#
# Color Calibration Panel
#
panel-color-cal = Color Calibration
label-brightness = Brightness
label-contrast = Contrast
label-saturation = Saturation
label-gamma-channels = Gamma (Per Channel)
label-gamma-red = Red Gamma
label-gamma-green = Green Gamma
label-gamma-blue = Blue Gamma
label-color-temp = Color Temperature

#
# Oscillator Panel
#
panel-oscillator = Oscillator Distortion
check-enable = Enable Effect
header-quick-presets = Quick Presets
btn-subtle = Subtle
btn-dramatic = Dramatic
btn-rings = Rings
btn-reset = Reset
header-distortion = Distortion Parameters
label-amount = Amount
label-dist-scale = Scale
label-dist-speed = Speed
header-visual-overlay = Visual Overlay
label-overlay-opacity = Overlay Opacity
label-color-mode = Color Mode
color-mode-off = Off
color-mode-rainbow = Rainbow
color-mode-black-white = Black & White
color-mode-complementary = Complementary
header-simulation = Simulation Parameters
sim-res-low = Low (128x128)
sim-res-medium = Medium (256x256)
sim-res-high = High (512x512)
label-kernel-radius = Kernel Radius
label-noise-amount = Noise Amount
label-freq-min = Frequency Min (Hz)
label-freq-max = Frequency Max (Hz)
label-coordinate-mode = Coordinate Mode
coord-mode-cartesian = Cartesian
coord-mode-log-polar = Log-Polar
label-phase-init = Phase Init
phase-init-random = Random
phase-init-uniform = Uniform
phase-init-plane-h = Plane H
phase-init-plane-v = Plane V
phase-init-diagonal = Diagonal
header-coupling = Coupling Rings (Advanced)
header-coupling-ring = Ring { $id }
label-diff-coupling = Coupling
btn-reset-ring = Reset Ring
btn-clear-ring = Clear Ring
tooltip-intensity = Intensity of the distortion effect
tooltip-dist-scale = Spatial scale of distortion
tooltip-dist-speed = Animation speed
tooltip-sim-res = Higher resolution = more detail but slower
tooltip-kernel-radius = Coupling interaction distance
tooltip-noise-amount = Random variation in oscillation
tooltip-coord-mode = Log-Polar creates radial/spiral patterns
tooltip-phase-init = Initial phase pattern for oscillators
tooltip-ring-dist = Distance from center (0-1)
tooltip-ring-width = Ring width (0-1)
tooltip-ring-coupling = Negative = anti-sync, Positive = sync

#
# Audio Panel
#
audio-panel-title = Audio Analysis
audio-panel-device = Input Device
audio-panel-no-device = No device
audio-panel-no-data = Waiting for audio data...
audio-panel-rms = RMS Volume
audio-panel-beat = Beat
audio-panel-bands = Frequency Bands

#
# OSC Panel
#
panel-osc-title = OSC Control
header-osc-server = OSC Server
label-status = Status
status-running = Running
status-stopped = Stopped
label-port = Port
btn-start-server = Start Server
header-feedback-clients = Feedback Clients
label-add-client = Add Client
btn-add = Add
header-address-mappings = Address Mappings
text-osc-edit-tip = (Edit osc_mappings.json to configure)

#
# Cue Panel
#
panel-cue = Cues

#
# Effect Chain Panel
#

blend-mode-normal = Normal
blend-mode-add = Add
blend-mode-subtract = Subtract
blend-mode-multiply = Multiply
blend-mode-screen = Screen
blend-mode-overlay = Overlay
blend-mode-soft-light = Soft Light
blend-mode-hard-light = Hard Light
blend-mode-lighten = Lighten
blend-mode-darken = Darken
blend-mode-color-dodge = Color Dodge
blend-mode-color-burn = Color Burn
blend-mode-difference = Difference
blend-mode-exclusion = Exclusion

effect-name-color-adjust = Color Adjust
effect-name-blur = Blur
effect-name-chromatic-aberration = Chromatic Aberration
effect-name-edge-detect = Edge Detect
effect-name-glow = Glow
effect-name-kaleidoscope = Kaleidoscope
effect-name-invert = Invert
effect-name-pixelate = Pixelate
effect-name-vignette = Vignette
effect-name-film-grain = Film Grain
effect-name-custom = Custom
effect-add = Add
effect-presets = Presets
effect-clear = Clear
effect-select-type = Select Effect Type
effect-no-effects = No effects in chain
effect-start-tip = Add an effect to get started
effect-intensity = Intensity
param-brightness = Brightness
param-contrast = Contrast
param-saturation = Saturation
param-radius = Radius
param-threshold = Threshold
param-segments = Segments
param-rotation = Rotation
param-pixel-size = Pixel Size
param-softness = Softness
no-parameters = No parameters
effect-save = Save
effect-presets-browser = Preset Browser
effect-search = Search...

#
# Generic Labels & Buttons
#
label-none = None
label-name = Name
label-mode = Mode
label-speed = Speed
label-width = Width
label-height = Height
label-master-opacity = Opacity
btn-remove = Remove
btn-reset-defaults = Reset to Defaults

#
# Logging Messages
#
error-list-audio-devices = Failed to list audio devices: { $error }
error-init-audio-backend = Failed to initialize audio backend: { $error }
error-start-audio-stream = Failed to start audio stream: { $error }
error-mcp-server = MCP Server error: { $error }
error-handle-event = Error handling event: { $error }
error-render = Render error on output { $output_id }: { $error }
error-autosave = Autosave failed: { $error }
info-autosave = Autosave successful
error-save-project = Failed to save project: { $error }
info-save-project = Project saved to { $path }
error-load-project = Failed to load project: { $error }
info-load-project = Project loaded from { $path }
info-language-switch = Language switched to: { $lang_code }
mcp-info-save = MCP: Saving project to { $path }
mcp-error-save = MCP: Failed to save project: { $error }
mcp-info-load = MCP: Loading project from { $path }
mcp-info-add-layer = MCP: Adding layer '{ $name }'
mcp-info-remove-layer = MCP: Removing layer { $id }
mcp-info-trigger-cue = MCP: Triggering cue { $id }
mcp-info-next-cue = MCP: Next cue
mcp-info-prev-cue = MCP: Prev cue
mcp-info-unimplemented = MCP: Unimplemented action received: { $action }
label-render-encoder = Render Encoder
label-egui-render-pass = Egui Render Pass
label-output-render-pass = Output Render Pass
label-imgui-render-pass = ImGui Render Pass
info-app-start = Starting MapFlow...
