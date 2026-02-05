# Oscillator Distortion Effect Control Panel

## Overview

A comprehensive ImGui-based control panel for the Kuramoto-based oscillator distortion effect, providing real-time adjustment of all simulation parameters.

## Location

**File:** `/home/user/mapmap/crates/mapmap-ui/src/lib.rs`
**Method:** `AppUI::render_oscillator_panel()`
**Lines:** 1099-1314

## Features

### Quick Access

- **View Menu Toggle:** "Show Oscillator" checkbox in View menu
- **Window Controls:** Collapsible, resizable, movable panel
- **Default Position:** (870, 100) with size 450×750

### Control Categories

#### 1. Master Controls

- **Enable Effect** - Master on/off toggle
- **Preset Buttons:**
  - Subtle - Gentle organic wobble
  - Dramatic - Intense swirling distortion
  - Rings - Concentric ring/wave patterns
  - Reset - Return to defaults

#### 2. Distortion Parameters

- **Amount** (0-1) - Intensity of distortion effect
- **Scale** (0-0.1) - Spatial scale of distortion
- **Speed** (0-5) - Animation speed multiplier

#### 3. Visual Overlay

- **Overlay Opacity** (0-1) - Visibility of phase visualization
- **Color Mode** - Dropdown with options:
  - Off - No color overlay
  - Rainbow - Full spectrum visualization
  - Black & White - Monochrome phase display
  - Complementary - Two-tone phase display

#### 4. Simulation Parameters

- **Resolution** - Quality preset dropdown:
  - Low (128×128) - Fast, lower detail
  - Medium (256×256) - Balanced (default)
  - High (512×512) - High detail, slower
- **Kernel Radius** (1-64) - Coupling interaction distance
- **Noise Amount** (0-1) - Random variation in oscillation
- **Frequency Min/Max** (0-10 Hz) - Oscillation frequency range

#### 5. Advanced Parameters

- **Coordinate Mode** - Dropdown:
  - Cartesian - Standard X/Y coordinates
  - Log-Polar - Radial/spiral patterns
- **Phase Init** - Initial phase pattern:
  - Random - Random phase distribution
  - Uniform - All same phase
  - Plane H/V - Horizontal/Vertical gradient
  - Diagonal - Diagonal gradient

#### 6. Coupling Rings (Advanced)

Collapsible section with 4 configurable rings:

- **Distance** (0-1) - Position from center
- **Width** (0-1) - Ring thickness
- **Coupling** (-5 to +5) - Synchronization strength
  - Negative = Anti-synchronization
  - Positive = Synchronization
- **Per-Ring Actions:**
  - Reset Ring - Return to defaults
  - Clear Ring - Disable ring

## Tooltips

All controls include helpful tooltips that appear on hover:

- Parameter explanations
- Value ranges
- Performance implications
- Visual effects descriptions

## Integration

### UI State

Added to `AppUI` struct:

```rust
pub show_oscillator: bool,  // Toggle visibility
```

### Default State

```rust
show_oscillator: true,  // Visible by default
```

### Menu Integration

Added to View menu in `render_menu_bar()`:

```rust
ui.checkbox("Show Oscillator", &mut self.show_oscillator);
```

### Usage Example

```rust
// In main render loop
ui_state.render_oscillator_panel(ui, &mut oscillator_config);
```

## Design Patterns

### ImGui Patterns Used

1. **Window Management** - Movable, resizable windows
2. **Sliders** - Continuous value adjustment
3. **Combo Boxes** - Enum selection
4. **Checkboxes** - Boolean toggles
5. **Tree Nodes** - Hierarchical organization
6. **Tooltips** - Contextual help
7. **ID Pushing** - Multiple similar controls
8. **Same Line** - Horizontal layout

### State Management

- Direct mutation of `OscillatorConfig` struct
- No intermediate UI state required
- Changes applied immediately
- Presets replace entire config

## Parameters Summary

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| enabled | bool | - | true | Master enable/disable |
| distortion_amount | f32 | 0-1 | 0.5 | Effect intensity |
| distortion_scale | f32 | 0-0.1 | 0.02 | Spatial scale |
| distortion_speed | f32 | 0-5 | 1.0 | Animation speed |
| overlay_opacity | f32 | 0-1 | 0.0 | Overlay visibility |
| color_mode | enum | - | Off | Color visualization |
| simulation_resolution | enum | - | Medium | Sim quality |
| kernel_radius | f32 | 1-64 | 16.0 | Coupling distance |
| noise_amount | f32 | 0-1 | 0.1 | Phase noise |
| frequency_min | f32 | 0-10 | 0.5 | Min frequency (Hz) |
| frequency_max | f32 | 0-10 | 2.0 | Max frequency (Hz) |
| coordinate_mode | enum | - | Cartesian | Coord system |
| phase_init_mode | enum | - | Random | Initial pattern |
| rings[0-3] | struct | - | varies | Coupling rings |

## Future Enhancements

Potential improvements:

1. **Parameter Animation** - Keyframe support for parameters
2. **Preset Management** - Save/load custom presets
3. **Visual Preview** - Small thumbnail preview
4. **Performance Metrics** - FPS impact display
5. **Advanced Ring Editor** - Visual ring configuration
6. **Parameter Linking** - Link multiple parameters
7. **Random Seed Control** - Reproducible random patterns
8. **Audio Reactivity** - Link parameters to audio
9. **MIDI Mapping** - Hardware control support
10. **Undo/Redo** - Parameter change history

## Testing

To test the control panel:

1. Launch MapMap application
2. Open View menu
3. Enable "Show Oscillator"
4. Adjust parameters and observe effects
5. Try different presets
6. Test advanced ring configuration
7. Verify tooltips on hover
8. Test window resize/reposition

## Related Files

- **Config Definition:** `crates/mapmap-core/src/oscillator.rs`
- **Renderer:** `crates/mapmap-render/src/oscillator_renderer.rs`
- **Shaders:**
  - `shaders/oscillator_simulation.wgsl`
  - `shaders/oscillator_distortion.wgsl`
- **UI Implementation:** `crates/mapmap-ui/src/lib.rs`

## Implementation Notes

- Uses ImGui 0.11 (legacy framework)
- Compatible with Phase 0-5 architecture
- No external dependencies beyond imgui crate
- Direct config mutation (no command pattern needed)
- All enums properly converted to/from UI indices
- Ring controls use ID pushing for uniqueness
- Collapsible headers save screen space
- Tooltips provide user guidance
