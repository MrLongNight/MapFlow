# Output Window Rendering Implementation Plan

## 🎯 Ziel
Implementiere tatsächliche Window-Ausgabe für Projector Output Nodes aus dem Module Canvas.

## 📋 Implementierungsschritte

### 1. Window-Lifecycle-Management
- [x] WindowManager existiert bereits
- [x] Sync Windows mit Output Nodes aus Active Module
- [x] Window Creation/Destruction basierend auf Module Changes
- [x] Fullscreen/Monitor-Selection Support

### 2. Rendering Pipeline
- [x] Für jedes Output Assignment:
  - [x] Get Source Texture aus Texture Pool
  - [x] Apply Effects (Pre-computed in module graph)
  - [x] Render zu Output Window Surface (Direct Quad Render)
- [x] Preview Panel Integration
- [ ] Separate Preview Window Support

### 3. Event Loop Integration
- [x] Render alle aktiven Output Windows pro Frame
- [x] Handle Window Events (Close, Resize)
- [x] Cursor hiding auf Output Windows

### 4. Module Canvas UI Sync
- [x] Bei Output Node Creation → Window erstellen
- [x] Bei Output Node Deletion → Window schließen
- [x] Bei Output Settings Change → Window update

## 🔧 Code-Änderungen

### `main.rs`
1. **Window Sync in Event Loop**:
   ```rust
   // Nach Module Evaluation
   if let Some(active_module_id) = self.ui_state.module_canvas.active_module_id {
       // Sync windows mit output assignments
       self.sync_output_windows(elwt, &result)?;
   }
   ```

2. **Render Loop für alle Outputs**:
   ```rust
   // Render Main Window (ID 0)
   self.render(0)?;
   
   // Render alle Output Windows
   for &output_id in self.window_manager.window_ids() {
       if output_id != 0 {
           self.render_output(output_id, &result)?;
       }
   }
   ```

3. **Neue Funktion**: `sync_output_windows`
   - Create/Remove windows based on output_assignments
   - Apply settings (fullscreen, monitor, etc.)

4. **Neue Funktion**: `render_output`
   - Get source texture from assignment
   - Apply effects if needed
   - Blit to window surface

### `window_manager.rs`
- [x] Add method: `create_projector_window` for Module Outputs
- [x] Monitor/Screen selection logic
- [x] Cursor hiding implementation

## 📊 Datenfluss

```
Module Canvas → Output Node → Module Evaluator
                                    ↓
                            output_assignments
                                    ↓
                            sync_output_windows()
                                    ↓
                            WindowManager.create
                                    ↓
                            render_output()
                                    ↓
                            Window Surface
```

## ✅ Testing
- [ ] Create Projector Output Node
- [ ] Connect Source → Output
- [ ] Verify Window opens
- [ ] Verify content renders
- [ ] Test fullscreen toggle
- [ ] Test monitor selection
- [ ] Test window close
