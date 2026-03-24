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

    /// Project file to load automatically (Automation mode)
    #[arg(long)]
    pub fixture: Option<String>,

    /// Exit after N frames (Automation mode)
    #[arg(long)]
    pub exit_after_frames: Option<u64>,

    /// Output directory for visual capture screenshots
    #[arg(long, env = "MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR")]
    pub screenshot_dir: Option<String>,
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
    /// Option E: Automation Mode (Headless / Testing)
    Automation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_automation_mode_parsing() {
        let args = vec![
            "mapflow",
            "--mode",
            "automation",
            "--fixture",
            "test_fixture.mflow",
            "--exit-after-frames",
            "100",
            "--screenshot-dir",
            "/tmp/mapflow-screenshots",
        ];

        let cli = CliArgs::try_parse_from(args).expect("Failed to parse automation CLI args");

        assert_eq!(cli.mode, Mode::Automation);
        assert_eq!(cli.fixture.as_deref(), Some("test_fixture.mflow"));
        assert_eq!(cli.exit_after_frames, Some(100));
        assert_eq!(
            cli.screenshot_dir.as_deref(),
            Some("/tmp/mapflow-screenshots")
        );
    }

    #[test]
    fn test_cli_automation_mode_env_fallback() {
        std::env::set_var("MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR", "/tmp/env-dir");

        let args = vec!["mapflow", "--mode", "automation"];

        let cli = CliArgs::try_parse_from(args).expect("Failed to parse CLI args");

        assert_eq!(cli.mode, Mode::Automation);
        assert_eq!(cli.screenshot_dir.as_deref(), Some("/tmp/env-dir"));

        std::env::remove_var("MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR");
    }
}
