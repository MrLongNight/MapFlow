use mapmap_core::module::ModuleId;

/// Actions triggered by timeline
#[derive(Debug, Clone, Copy)]
pub enum TimelineAction {
    Play,
    Pause,
    Stop,
    Seek(f32),
    SelectModule(ModuleId),
    AddMarker(f32),
    RemoveMarker(u64),
    ToggleMarkerPause(f32),
    JumpNextMarker,
    JumpPrevMarker,
}

/// Lightweight module descriptor for timeline arrangement UI.
#[derive(Debug, Clone)]
pub struct TimelineModule<'a> {
    /// Module ID
    pub id: ModuleId,
    /// Module display name
    // Optimization: Borrow name string to prevent allocation overhead in UI hot loop.
    pub name: &'a str,
}
