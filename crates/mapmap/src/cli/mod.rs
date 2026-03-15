use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about = "MapFlow - Professional Projection Mapping Software", long_about = None)]
/// Command-line arguments for MapFlow.
pub struct CliArgs {
    /// Operating mode
    #[arg(short, long, value_enum, default_value_t = Mode::Editor)]
    pub mode: Mode,

    /// NDI Source name (for PlayerNdi mode)
    #[arg(long, default_value = "MAPFLOW-MASTER")]
    pub source: String,

    /// Fullscreen mode
    #[arg(short, long)]
    pub fullscreen: bool,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
/// Execution mode for MapFlow.
pub enum Mode {
    /// Full MapFlow Editor and Rendering
    Editor,
    /// Option A: NDI Receiver
    PlayerNdi,
    /// Option B: Distributed Rendering (Client)
    PlayerDist,
    /// Option C: RTSP/H.264 Legacy Player
    PlayerLegacy,
    /// Option D: Raspberry Pi Player
    PlayerPi,
}
