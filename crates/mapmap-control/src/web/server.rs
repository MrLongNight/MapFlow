//! Axum HTTP server

#[cfg(feature = "http-api")]
use axum::http::{header, HeaderValue, Method, StatusCode};

#[cfg(feature = "http-api")]
use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
};

#[cfg(feature = "http-api")]
use tower_http::cors::{Any, CorsLayer};

use std::net::SocketAddr;
use std::sync::Arc;

#[cfg(feature = "http-api")]
use tokio::sync::RwLock;

use crate::{error::ControlError, Result};

use super::auth::AuthConfig;
#[cfg(feature = "http-api")]
use super::routes::build_router;
#[cfg(feature = "http-api")]
use super::websocket::ws_handler;

/// Application state shared across all requests
#[derive(Clone)]
#[cfg(feature = "http-api")]
pub struct AppState {
    pub auth: Arc<RwLock<AuthConfig>>,
}

/// Web server configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    #[serde(default = "default_allowed_origins")]
    pub allowed_origins: Vec<String>,
    pub auth: AuthConfig,
}

fn default_allowed_origins() -> Vec<String> {
    vec!["*".to_string()]
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            // Secure by default: bind to localhost to prevent accidental network exposure
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_cors: true,
            allowed_origins: default_allowed_origins(),
            auth: AuthConfig::new(),
        }
    }
}

impl WebServerConfig {
    /// Create a new web server config
    pub fn new(port: u16) -> Self {
        Self {
            port,
            ..Default::default()
        }
    }

    /// Set the host address
    pub fn with_host(mut self, host: String) -> Self {
        self.host = host;
        self
    }

    /// Set CORS enabled/disabled
    pub fn with_cors(mut self, enable: bool) -> Self {
        self.enable_cors = enable;
        self
    }

    /// Set allowed origins for CORS
    pub fn with_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    /// Set authentication config
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }
}

/// Web server for REST API and WebSocket
pub struct WebServer {
    #[cfg(feature = "http-api")]
    config: WebServerConfig,
}

impl WebServer {
    /// Create a new web server
    #[cfg(feature = "http-api")]
    pub fn new(config: WebServerConfig) -> Self {
        Self { config }
    }

    #[cfg(not(feature = "http-api"))]
    pub fn new(_config: WebServerConfig) -> Self {
        Self {}
    }

    /// Run the web server (blocking)
    #[cfg(feature = "http-api")]
    pub async fn run(self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e| ControlError::HttpError(format!("Invalid address: {}", e)))?;

        let state = AppState {
            auth: Arc::new(RwLock::new(self.config.auth.clone())),
        };

        // Build router with state
        let app = build_router()
            .route("/ws", axum::routing::get(ws_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .layer(middleware::from_fn(security_headers)) // Apply security headers
            .with_state(state);

        // Add CORS if enabled
        let app = if self.config.enable_cors {
            let cors_layer = CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

            // If allowed_origins contains "*" or is empty (default permissive), allow Any
            if self.config.allowed_origins.contains(&"*".to_string())
                || self.config.allowed_origins.is_empty()
            {
                // Must be applied in separate branch to handle different concrete types
                app.layer(cors_layer.allow_origin(Any))
            } else {
                let origins: Result<Vec<HeaderValue>> = self
                    .config
                    .allowed_origins
                    .iter()
                    .map(|o| {
                        o.parse::<HeaderValue>().map_err(|e| {
                            ControlError::HttpError(format!("Invalid origin header: {}", e))
                        })
                    })
                    .collect();

                app.layer(cors_layer.allow_origin(origins?))
            }
        } else {
            app
        };

        tracing::info!("Web server listening on {}", addr);

        // Bind to the TCP listener
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| ControlError::HttpError(format!("Failed to bind: {}", e)))?;

        // Serve the application - use into_make_service()
        let make_service = app.into_make_service();
        axum::serve(listener, make_service)
            .await
            .map_err(|e| ControlError::HttpError(format!("Server error: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "http-api"))]
    pub async fn run(self) -> Result<()> {
        Err(ControlError::HttpError(
            "HTTP API feature not enabled".to_string(),
        ))
    }

    /// Spawn the server in a background task
    #[cfg(feature = "http-api")]
    pub fn spawn(self) -> tokio::task::JoinHandle<Result<()>> {
        tokio::spawn(async move { self.run().await })
    }

    #[cfg(not(feature = "http-api"))]
    pub fn spawn(self) -> Result<()> {
        Err(ControlError::HttpError(
            "HTTP API feature not enabled".to_string(),
        ))
    }
}

/// Authentication middleware
#[cfg(feature = "http-api")]
async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> core::result::Result<Response, StatusCode> {
    let auth_config = state.auth.read().await;

    if auth_config.is_enabled() {
        let headers = req.headers();
        let query = req.uri().query();

        // Extract and validate API key
        let api_key = super::auth::extract_api_key(headers, query);
        let is_valid = match api_key {
            Some(key) => auth_config.validate(&key),
            None => false,
        };

        if !is_valid {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    Ok(next.run(req).await)
}

/// Security headers middleware
#[cfg(feature = "http-api")]
async fn security_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Prevent MIME sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    // Legacy XSS protection (for defense in depth)
    headers.insert(
        header::X_XSS_PROTECTION,
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer Policy
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );

    response
}

#[cfg(all(test, feature = "http-api"))]
mod tests {
    use super::*;
    use axum::extract::Request;

    #[test]
    fn test_web_server_config() {
        let config = WebServerConfig::new(8080)
            .with_host("127.0.0.1".to_string())
            .with_cors(false)
            .with_allowed_origins(vec!["http://localhost:3000".to_string()]);

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(!config.enable_cors);
        assert_eq!(
            config.allowed_origins,
            vec!["http://localhost:3000".to_string()]
        );
    }

    #[test]
    fn test_web_server_default_origins() {
        let config = WebServerConfig::default();
        assert!(config.allowed_origins.contains(&"*".to_string()));
    }

    #[tokio::test]
    async fn test_web_server_creation() {
        let config = WebServerConfig::new(18080);
        let _server = WebServer::new(config);
        // Server created successfully
    }

    #[tokio::test]
    async fn test_security_headers() {
        use axum::body::Body;
        use tower::Service; // for call

        // Setup a simple app with the middleware
        let mut app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "Hello" }))
            .layer(middleware::from_fn(security_headers));

        let response = app
            .call(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let headers = response.headers();

        assert_eq!(
            headers
                .get("X-Content-Type-Options")
                .and_then(|h| h.to_str().ok()),
            Some("nosniff")
        );
        assert_eq!(
            headers.get("X-Frame-Options").and_then(|h| h.to_str().ok()),
            Some("DENY")
        );
        assert_eq!(
            headers
                .get("X-XSS-Protection")
                .and_then(|h| h.to_str().ok()),
            Some("1; mode=block")
        );
        assert_eq!(
            headers.get("Referrer-Policy").and_then(|h| h.to_str().ok()),
            Some("no-referrer")
        );
    }
}
