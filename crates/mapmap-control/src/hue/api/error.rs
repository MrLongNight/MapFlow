use thiserror::Error;

#[derive(Error, Debug)]
pub enum HueError {
    #[error("Bridge discovery failed")]
    /// Error: Bridge discovery failed.
    /// Error: Bridge discovery failed.
    /// Error: Bridge discovery failed.
    DiscoveryFailed,
    #[error("Link button not pressed. Please press the link button on the Hue Bridge.")]
    /// Error: Link button not pressed. Please press the link button on the Hue Bridge..
    /// Error: Link button not pressed. Please press the link button on the Hue Bridge..
    /// Error: Link button not pressed. Please press the link button on the Hue Bridge..
    LinkButtonNotPressed,
    #[error("Network error: {0}")]
    /// Error: Network error.
    /// Error: Network error.
    /// Error: Network error.
    Network(#[from] reqwest::Error),
    #[error("API error: {0}")]
    /// Error: API error.
    /// Error: API error.
    /// Error: API error.
    ApiError(String),
    #[error("Serialization error: {0}")]
    /// Error: Serialization error.
    /// Error: Serialization error.
    /// Error: Serialization error.
    Serde(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    /// Error: Other error.
    /// Error: Other error.
    /// Error: Other error.
    Other(String),
}
