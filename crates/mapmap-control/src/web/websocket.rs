//! WebSocket handler for real-time updates

#[cfg(feature = "http-api")]
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::HeaderMap,
    response::Response,
};

#[cfg(feature = "http-api")]
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::{ControlTarget, ControlValue};

#[cfg(feature = "http-api")]
use super::server::AppState;

/// Maximum WebSocket message size (16 KB)
///
/// This limit prevents Denial of Service (DoS) attacks where a client sends massive messages
/// that consume excessive memory or CPU during parsing.
const MAX_MESSAGE_SIZE: usize = 16 * 1024;

/// WebSocket message from client to server
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    #[serde(rename = "set_parameter")]
    SetParameter {
        target: ControlTarget,
        value: ControlValue,
    },
    #[serde(rename = "subscribe")]
    Subscribe { targets: Vec<ControlTarget> },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { targets: Vec<ControlTarget> },
    #[serde(rename = "ping")]
    Ping,
}

/// WebSocket message from server to client
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsServerMessage {
    #[serde(rename = "parameter_changed")]
    ParameterChanged {
        target: ControlTarget,
        value: ControlValue,
    },
    #[serde(rename = "stats")]
    Stats { fps: f32, frame_time_ms: f32 },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "pong")]
    Pong,
}

/// WebSocket upgrade handler
#[cfg(feature = "http-api")]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    // Check if client requested a specific subprotocol (e.g. for auth)
    let protocol = extract_auth_protocol(&headers);

    let ws = if let Some(protocol) = protocol {
        ws.protocols([protocol])
    } else {
        ws
    };

    // Set max message size to prevent DoS attacks
    ws.max_message_size(MAX_MESSAGE_SIZE)
        .on_upgrade(|socket| handle_socket(socket, state))
}

/// Extract auth protocol from headers
#[cfg(feature = "http-api")]
fn extract_auth_protocol(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|value| value.to_str().ok())
        .and_then(|protocols| {
            protocols
                .split(',')
                .map(|p| p.trim())
                .find(|p| p.starts_with("mapmap.auth."))
                .map(|p| p.to_string())
        })
}

#[cfg(not(feature = "http-api"))]
pub async fn ws_handler() -> () {
    ()
}

/// Handle a WebSocket connection
#[cfg(feature = "http-api")]
async fn handle_socket(socket: WebSocket, _state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    tracing::info!("WebSocket client connected");

    // Spawn a task to send periodic stats updates
    let stats_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1000 / 60));

        loop {
            interval.tick().await;

            let stats = WsServerMessage::Stats {
                fps: 60.0,
                frame_time_ms: 16.6,
            };

            if let Ok(json) = serde_json::to_string(&stats) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            } else {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if text.len() > MAX_MESSAGE_SIZE {
                    tracing::warn!(
                        "WebSocket message too large: {} bytes (max {})",
                        text.len(),
                        MAX_MESSAGE_SIZE
                    );
                    // Close connection on violation
                    break;
                }

                if let Err(e) = handle_text_message(&text).await {
                    tracing::warn!("Error handling WebSocket message: {}", e);
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket client disconnected");
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    stats_task.abort();
}

/// Handle a text message from the client
#[cfg(feature = "http-api")]
async fn handle_text_message(text: &str) -> Result<(), String> {
    let message: WsClientMessage =
        serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {}", e))?;

    match message {
        WsClientMessage::SetParameter { target, value } => {
            // Security check: validate input
            target
                .validate()
                .map_err(|e| format!("Invalid target: {}", e))?;
            value
                .validate()
                .map_err(|e| format!("Invalid value: {}", e))?;

            tracing::debug!("WebSocket set parameter: {:?} = {:?}", target, value);
            // In a real implementation, this would update the project state
        }
        WsClientMessage::Subscribe { targets } => {
            // Security check: validate targets
            for target in &targets {
                target
                    .validate()
                    .map_err(|e| format!("Invalid subscription target: {}", e))?;
            }

            tracing::debug!("WebSocket subscribe: {:?}", targets);
            // In a real implementation, this would track subscriptions
        }
        WsClientMessage::Unsubscribe { targets } => {
            tracing::debug!("WebSocket unsubscribe: {:?}", targets);
        }
        WsClientMessage::Ping => {
            tracing::trace!("WebSocket ping");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_message_size_constant() {
        // Verify the constant is set to a reasonable value (16KB)
        assert_eq!(MAX_MESSAGE_SIZE, 16384);
    }

    #[test]
    fn test_ws_client_message_serialization() {
        let msg = WsClientMessage::SetParameter {
            target: ControlTarget::LayerOpacity(0),
            value: ControlValue::Float(0.5),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("set_parameter"));
        assert!(json.contains("LayerOpacity"));
    }

    #[test]
    fn test_ws_server_message_serialization() {
        let msg = WsServerMessage::Stats {
            fps: 60.0,
            frame_time_ms: 16.6,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("stats"));
        assert!(json.contains("60.0"));
    }

    #[test]
    fn test_ws_client_message_deserialization() {
        let json = r#"{"type":"ping"}"#;
        let msg: WsClientMessage = serde_json::from_str(json).unwrap();
        matches!(msg, WsClientMessage::Ping);
    }

    #[cfg(feature = "http-api")]
    #[tokio::test]
    async fn test_validation_rejects_long_string() {
        let long_str = "a".repeat(5000); // Limit is 4096
        let msg = WsClientMessage::SetParameter {
            target: ControlTarget::LayerOpacity(0),
            value: ControlValue::String(long_str),
        };
        let json = serde_json::to_string(&msg).unwrap();

        let result = handle_text_message(&json).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid value"));
    }

    #[cfg(feature = "http-api")]
    #[tokio::test]
    async fn test_validation_rejects_invalid_target() {
        let long_name = "a".repeat(300); // Limit is 256
        let msg = WsClientMessage::SetParameter {
            target: ControlTarget::Custom(long_name),
            value: ControlValue::Float(0.5),
        };
        let json = serde_json::to_string(&msg).unwrap();

        let result = handle_text_message(&json).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid target"));
    }

    #[cfg(feature = "http-api")]
    #[tokio::test]
    async fn test_validation_rejects_invalid_subscription() {
        let long_name = "a".repeat(300);
        let msg = WsClientMessage::Subscribe {
            targets: vec![ControlTarget::Custom(long_name)],
        };
        let json = serde_json::to_string(&msg).unwrap();

        let result = handle_text_message(&json).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid subscription target"));
    }

    #[cfg(feature = "http-api")]
    #[test]
    fn test_extract_auth_protocol() {
        use axum::http::HeaderMap;

        let mut headers = HeaderMap::new();
        headers.insert(
            "Sec-WebSocket-Protocol",
            "mapmap.auth.secret, json".parse().unwrap(),
        );

        let proto = extract_auth_protocol(&headers);
        assert_eq!(proto, Some("mapmap.auth.secret".to_string()));

        let headers_empty = HeaderMap::new();
        let proto_empty = extract_auth_protocol(&headers_empty);
        assert_eq!(proto_empty, None);

        let mut headers_other = HeaderMap::new();
        headers_other.insert("Sec-WebSocket-Protocol", "json, xml".parse().unwrap());
        let proto_other = extract_auth_protocol(&headers_other);
        assert_eq!(proto_other, None);
    }
}
