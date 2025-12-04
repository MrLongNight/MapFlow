use super::window_manager::WindowManager;
use mapmap_render::WgpuBackend;
use pollster;
use winit::event_loop::EventLoop;

#[test]
fn test_create_window_manager() {
    let wm = WindowManager::new();
    assert!(wm.main_window_id().is_none());
}

#[test]
fn test_create_main_window() {
    let event_loop = EventLoop::new();
    let backend = pollster::block_on(WgpuBackend::new()).unwrap();
    let mut wm = WindowManager::new();
    let main_window_id = wm.create_main_window(&event_loop, &backend).unwrap();
    assert_eq!(wm.main_window_id(), Some(main_window_id));
}
