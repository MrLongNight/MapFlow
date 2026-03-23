# MapFlow Module Tree (Generated: 2026-03-08 23:39)

Diese Datei zeigt die physische und logische Struktur des Projekts.

## 1. Physische Crate-Struktur

``text
Auflistung der Ordnerpfade
Volumeseriennummer : 7AE1-A876
C:\USERS\VINYL\DESKTOP\VJMAPPER\VJMAPPER\CRATES
+---mapflow
|   |   build.rs
|   |   Cargo.toml
|   |   README.md
|   |
|   +---examples
|   |       hello_world_projection.rs
|   |       simple_render.rs
|   |
|   +---src
|   |   |   logging_setup.rs
|   |   |   main.rs
|   |   |   media_manager_ui.rs
|   |   |   window_manager.rs
|   |   |   window_manager_test.rs
|   |   |
|   |   +---app
|   |   |   |   actions.rs
|   |   |   |   events.rs
|   |   |   |   mod.rs
|   |   |   |   ui_layout.rs
|   |   |   |   update.rs
|   |   |   |
|   |   |   +---core
|   |   |   |       app_struct.rs
|   |   |   |       init.rs
|   |   |   |       mod.rs
|   |   |   |
|   |   |   \---loops
|   |   |           logic.rs
|   |   |           mod.rs
|   |   |           render.rs
|   |   |
|   |   +---orchestration
|   |   |       evaluation.rs
|   |   |       media.rs
|   |   |       mod.rs
|   |   |       node_logic.rs
|   |   |       outputs.rs
|   |   |
|   |   \---ui
|   |       |   mod.rs
|   |       |
|   |       +---dialogs
|   |       |       about.rs
|   |       |       icon_demo.rs
|   |       |       mod.rs
|   |       |       settings.rs
|   |       |
|   |       +---editors
|   |       |       mod.rs
|   |       |       module_canvas.rs
|   |       |       node_editor.rs
|   |       |       timeline.rs
|   |       |
|   |       +---panels
|   |       |       assignment.rs
|   |       |       edge_blend.rs
|   |       |       mapping.rs
|   |       |       mod.rs
|   |       |       output.rs
|   |       |       paint.rs
|   |       |
|   |       \---view
|   |               media_manager.rs
|   |               mod.rs
|   |
|   \---wix
|           License.rtf
|           main.wxs
|
+---mapflow-bevy
|   |   Cargo.toml
|   |   README.md
|   |
|   \---src
|           build_error.txt
|           build_error_2.txt
|           build_error_3.txt
|           build_error_4.txt
|           build_error_5.txt
|           build_error_6.txt
|           build_error_7.txt
|           build_error_8.txt
|           components.rs
|           lib.rs
|           resources.rs
|           systems.rs
|
+---mapflow-control
|   |   Cargo.toml
|   |   README.md
|   |
|   \---src
|       |   error.rs
|       |   lib.rs
|       |   link.rs
|       |   manager.rs
|       |   target.rs
|       |
|       +---cue
|       |       crossfade.rs
|       |       cue.rs
|       |       cue_list.rs
|       |       mod.rs
|       |       triggers.rs
|       |
|       +---dmx
|       |       artnet.rs
|       |       channels.rs
|       |       fixtures.rs
|       |       mod.rs
|       |       sacn.rs
|       |
|       +---hue
|       |   |   audio_interface.rs
|       |   |   controller.rs
|       |   |   engine.rs
|       |   |   mod.rs
|       |   |   models.rs
|       |   |
|       |   +---api
|       |   |       client.rs
|       |   |       discovery.rs
|       |   |       error.rs
|       |   |       groups.rs
|       |   |       mod.rs
|       |   |
|       |   +---effects
|       |   |       mod.rs
|       |   |
|       |   \---stream
|       |           dtls.rs
|       |           manager.rs
|       |           mod.rs
|       |           protocol.rs
|       |
|       +---midi
|       |       clock.rs
|       |       controller_element.rs
|       |       ecler_nuo4.rs
|       |       input.rs
|       |       mapping.rs
|       |       midi_learn.rs
|       |       mod.rs
|       |       output.rs
|       |       profiles.rs
|       |
|       +---osc
|       |       address.rs
|       |       client.rs
|       |       mapping.rs
|       |       mod.rs
|       |       server.rs
|       |       types.rs
|       |
|       +---shortcuts
|       |       bindings.rs
|       |       macros.rs
|       |       mod.rs
|       |       shortcuts.rs
|       |
|       \---web
|               auth.rs
|               handlers.rs
|               mod.rs
|               routes.rs
|               server.rs
|               websocket.rs
|
+---mapflow-core
|   |   Cargo.toml
|   |   README.md
|   |
|   +---src
|   |   |   animation.rs
|   |   |   assignment.rs
|   |   |   audio_media_pipeline.rs
|   |   |   audio_reactive.rs
|   |   |   codegen.rs
|   |   |   diagnostics.rs
|   |   |   effects.rs
|   |   |   effect_animation.rs
|   |   |   history.rs
|   |   |   lib.rs
|   |   |   logging.rs
|   |   |   lut.rs
|   |   |   macros.rs
|   |   |   mapping.rs
|   |   |   media_library.rs
|   |   |   mesh.rs
|   |   |   module_eval.rs
|   |   |   monitor.rs
|   |   |   oscillator.rs
|   |   |   output.rs
|   |   |   paint.rs
|   |   |   recent_effect_configs.rs
|   |   |   shader_graph.rs
|   |   |   state.rs
|   |   |   trigger_system.rs
|   |   |
|   |   +---audio
|   |   |       analyzer_v2.rs
|   |   |       backend.rs
|   |   |       mod.rs
|   |   |
|   |   +---layer
|   |   |       composition.rs
|   |   |       layer_struct.rs
|   |   |       manager.rs
|   |   |       mod.rs
|   |   |       transform.rs
|   |   |       types.rs
|   |   |
|   |   \---module
|   |       |   config.rs
|   |       |   manager.rs
|   |       |   manager_tests.rs
|   |       |   mod.rs
|   |       |
|   |       \---types
|   |               connection.rs
|   |               hue.rs
|   |               layer.rs
|   |               mask.rs
|   |               mesh.rs
|   |               mod.rs
|   |               module.rs
|   |               module_tests.rs
|   |               modulizer.rs
|   |               node_link.rs
|   |               output.rs
|   |               part.rs
|   |               shared_media.rs
|   |               socket.rs
|   |               source.rs
|   |               trigger.rs
|   |
|   \---tests
|           assignment_tests.rs
|           comprehensive_node_tests.rs
|           layer_tests.rs
|           module_coverage_tests.rs
|           module_playback_tests.rs
|           module_tests.rs
|           trigger_logic_tests.rs
|           trigger_system_tests.rs
|           trigger_tests.rs
|
+---mapflow-ffi
|   |   Cargo.toml
|   |   README.md
|   |
|   \---src
|           lib.rs
|
+---mapflow-io
|   |   Cargo.toml
|   |   README.md
|   |
|   +---src
|   |   |   converter.rs
|   |   |   error.rs
|   |   |   format.rs
|   |   |   lib.rs
|   |   |   project.rs
|   |   |   project_format.rs
|   |   |   sink.rs
|   |   |   source.rs
|   |   |
|   |   +---decklink
|   |   |       mod.rs
|   |   |
|   |   +---ndi
|   |   |       mod.rs
|   |   |
|   |   +---spout
|   |   |       mod.rs
|   |   |
|   |   +---stream
|   |   |       encoder.rs
|   |   |       mod.rs
|   |   |       rtmp.rs
|   |   |       srt.rs
|   |   |
|   |   +---syphon
|   |   |       mod.rs
|   |   |
|   |   \---virtual_camera
|   |           mod.rs
|   |
|   \---tests
|           project_tests.rs
|
+---mapflow-mcp
|   |   Cargo.toml
|   |   README.md
|   |
|   \---src
|           lib.rs
|           main.rs
|           protocol.rs
|           server.rs
|
+---mapflow-media
|   |   Cargo.toml
|   |   README.md
|   |
|   +---benches
|   |       video_decode.rs
|   |
|   \---src
|           decoder.rs
|           hap_decoder.rs
|           hap_player.rs
|           image_decoder.rs
|           lib.rs
|           mpv_decoder.rs
|           pipeline.rs
|           player.rs
|           sequence.rs
|
+---mapflow-render
|   |   Cargo.toml
|   |   README.md
|   |
|   +---benches
|   |       mesh_renderer_bench.rs
|   |
|   +---src
|   |       backend.rs
|   |       color_calibration_renderer.rs
|   |       compositor.rs
|   |       compressed_texture.rs
|   |       edge_blend_renderer.rs
|   |       effect_chain_renderer.rs
|   |       hot_reload.rs
|   |       lib.rs
|   |       mesh_buffer_cache.rs
|   |       mesh_renderer.rs
|   |       oscillator_renderer.rs
|   |       paint_texture_cache.rs
|   |       pipeline.rs
|   |       preset.rs
|   |       quad.rs
|   |       shader.rs
|   |       shader_graph_integration.rs
|   |       spout.rs
|   |       texture.rs
|   |       uploader.rs
|   |
|   \---tests
|           effect_chain_integration_tests.rs
|           effect_chain_tests.rs
|           multi_output_tests.rs
|
+---mapflow-ui
|   |   Cargo.toml
|   |   README.md
|   |
|   +---locales
|   |   +---de
|   |   |       main.ftl
|   |   |
|   |   \---en
|   |           main.ftl
|   |
|   +---src
|   |   |   lib.rs
|   |   |
|   |   +---core
|   |   |       asset_manager.rs
|   |   |       config.rs
|   |   |       i18n.rs
|   |   |       mod.rs
|   |   |       responsive.rs
|   |   |       theme.rs
|   |   |       toast.rs
|   |   |       undo_redo.rs
|   |   |
|   |   +---editors
|   |   |   |   mesh_editor.rs
|   |   |   |   mod.rs
|   |   |   |   node_editor.rs
|   |   |   |   node_editor.rs.tmp
|   |   |   |   shortcut_editor.rs
|   |   |   |   timeline_v2.rs
|   |   |   |
|   |   |   \---module_canvas
|   |   |       |   canvas_ui.rs
|   |   |       |   controller.rs
|   |   |       |   diagnostics.rs
|   |   |       |   draw.rs
|   |   |       |   geometry.rs
|   |   |       |   interaction_logic.rs
|   |   |       |   mesh.rs
|   |   |       |   mod.rs
|   |   |       |   node_rendering.rs
|   |   |       |   renderer.rs
|   |   |       |   state.rs
|   |   |       |   types.rs
|   |   |       |   utils.rs
|   |   |       |
|   |   |       \---inspector
|   |   |               common.rs
|   |   |               effect.rs
|   |   |               layer.rs
|   |   |               mod.rs
|   |   |               output.rs
|   |   |               source.rs
|   |   |               trigger.rs
|   |   |
|   |   +---panels
|   |   |   |   assignment_panel.rs
|   |   |   |   audio_panel.rs
|   |   |   |   controller_overlay_panel.rs
|   |   |   |   cue_panel.rs
|   |   |   |   edge_blend_panel.rs
|   |   |   |   effect_chain_panel.rs
|   |   |   |   inspector_panel.rs
|   |   |   |   layer_panel.rs
|   |   |   |   mapping_panel.rs
|   |   |   |   mod.rs
|   |   |   |   oscillator_panel.rs
|   |   |   |   osc_panel.rs
|   |   |   |   output_panel.rs
|   |   |   |   paint_panel.rs
|   |   |   |   preview_panel.rs
|   |   |   |   shortcuts_panel.rs
|   |   |   |   transform_panel.rs
|   |   |   |
|   |   |   \---inspector
|   |   |           layer.rs
|   |   |           mod.rs
|   |   |           module.rs
|   |   |           output.rs
|   |   |
|   |   +---view
|   |   |   |   dashboard.rs
|   |   |   |   media_browser.rs
|   |   |   |   media_manager_wrapper.rs
|   |   |   |   mod.rs
|   |   |   |   module_sidebar.rs
|   |   |   |
|   |   |   \---menu_bar
|   |   |           edit_menu.rs
|   |   |           file_menu.rs
|   |   |           help_menu.rs
|   |   |           mod.rs
|   |   |           toolbar.rs
|   |   |           view_menu.rs
|   |   |
|   |   \---widgets
|   |           audio_meter.rs
|   |           custom.rs
|   |           icons.rs
|   |           icon_demo_panel.rs
|   |           mod.rs
|   |           panel.rs
|   |
|   \---tests
|           timeline_automation_tests.rs
|
\---vendor
    +---egui_node_editor
    |       .cargo_vcs_info.json
    |
    \---imgui-wgpu
        |   Cargo.toml
        |
        \---src
                imgui.wgsl
                lib.rs

``

## 2. Workspace Crates

| Crate | Pfad | Beschreibung |
|-------|------|--------------|
| mapflow | crates/mapflow | MapFlow - Professional Projection Mapping Software |
| mapflow-bevy | crates/mapflow-bevy |  |
| mapflow-control | crates/mapflow-control |  |
| mapflow-core | crates/mapflow-core |  |
| mapflow-ffi | crates/mapflow-ffi | MapFlow C/C++ Foreign Function Interface bindings. |
| mapflow-io | crates/mapflow-io |  |
| mapflow-mcp | crates/mapflow-mcp |  |
| mapflow-media | crates/mapflow-media |  |
| mapflow-render | crates/mapflow-render |  |
| mapflow-ui | crates/mapflow-ui |  |

## 3. Logische Modul-Hierarchie

> Hinweis: Installiere cargo-modules (cargo install cargo-modules), um hier einen detaillierten logischen Modul-Graph zu sehen.
> Aktuell wird nur die Datei-Struktur oben angezeigt.
