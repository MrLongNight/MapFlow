use thiserror::Error;

#[derive(Error, Debug)]
pub enum HueError {
    #[error("Bridge discovery failed")]
    DiscoveryFailed,
    #[error("Link button not pressed. Please press the link button on the Hue Bridge.")]
    LinkButtonNotPressed,
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}




