use super::api;
use super::models::{HueConfig, LightNode};
use super::stream::{
    dtls::HueStreamer,
    manager::{run_stream_loop, LightState},
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info};

/// Manages the connection to the Hue Bridge and the entertainment stream.
pub struct HueController {
    config: HueConfig,
    sender: Option<mpsc::Sender<Vec<LightState>>>,
    stream_handle: Option<JoinHandle<()>>,
    nodes: HashMap<String, LightNode>, // Map Light ID -> Node with Channel ID
    is_connected: bool,
}

impl HueController {
    pub fn new(config: HueConfig) -> Self {
        Self {
            config,
            sender: None,
            stream_handle: None,
            nodes: HashMap::new(),
            is_connected: false,
        }
    }

    /// Register a new user with the bridge.
    /// This requires the link button on the bridge to be pressed.
    /// This method polls the bridge for 60 seconds to give the user time.
    pub async fn register(&mut self, ip: &str) -> Result<HueConfig, String> {
        info!("Starting Bridge registration at {} (60s timeout)...", ip);

        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(60);
        let poll_interval = std::time::Duration::from_secs(2);

        while start_time.elapsed() < timeout {
            match api::client::HueClient::register_user(ip, "MapFlow").await {
                Ok(config) => {
                    info!("Successfully registered with Hue Bridge!");
                    return Ok(config);
                }
                Err(api::error::HueError::LinkButtonNotPressed) => {
                    let elapsed = start_time.elapsed().as_secs();
                    info!(
                        "Link button not pressed yet ({}s/60s). Retrying...",
                        elapsed
                    );
                }
                Err(e) => {
                    // Other errors (network, etc) should fail immediately
                    return Err(e.to_string());
                }
            }
            tokio::time::sleep(poll_interval).await;
        }

        Err("Timeout: Link button was not pressed within 60 seconds.".to_string())
    }

    /// Update the configuration (e.g. if settings change)
    pub fn update_config(&mut self, config: HueConfig) {
        self.config = config;
        // If connected, we might need to reconnect? For now just update struct.
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Connect to the bridge and start the entertainment stream.
    pub async fn connect(&mut self) -> Result<(), String> {
        if self.is_connected {
            return Ok(());
        }

        if self.config.bridge_ip.is_empty() {
            return Err("Bridge IP is missing".to_string());
        }

        // 1. Fetch application ID if missing (needed for DTLS Identity)
        if self.config.application_id.is_empty() {
            info!("Fetching application ID...");
            let app_id = api::client::HueClient::get_application_id(
                &self.config.bridge_ip,
                &self.config.username,
            )
            .await
            .map_err(|e| e.to_string())?;
            self.config.application_id = app_id;
        }

        // 2. Fetch entertainment groups to populate channel mapping
        if self.config.entertainment_group_id.is_empty() {
            return Err("No Entertainment Group selected".to_string());
        }

        info!("Fetching entertainment configuration...");
        let groups = api::groups::get_entertainment_groups(&self.config)
            .await
            .map_err(|e| e.to_string())?;

        // Find selected group
        let group = groups
            .iter()
            .find(|g| g.id == self.config.entertainment_group_id)
            .ok_or_else(|| {
                format!(
                    "Entertainment group '{}' not found",
                    self.config.entertainment_group_id
                )
            })?;

        // Populate nodes map
        self.nodes.clear();
        for light in &group.lights {
            self.nodes.insert(light.id.clone(), light.clone());
        }
        info!(
            "Loaded {} lights for entertainment group '{}'",
            self.nodes.len(),
            group.name
        );

        // 3. Activate stream on bridge via API
        info!("Activating stream mode...");
        api::groups::set_stream_active(&self.config, &self.config.entertainment_group_id, true)
            .await
            .map_err(|e| e.to_string())?;

        // 4. Start DTLS Stream (Synchronous)
        let streamer = match HueStreamer::connect(
            &self.config.bridge_ip,
            &self.config.application_id,
            &self.config.client_key,
        ) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to Hue Bridge DTLS: {}. Check if Windows Firewall blocks UDP port 2100.", e);
                // Continue without streaming? Or fail?
                // We should probably fail to connect.
                // But we successfully got nodes.
                // For now, fail.
                return Err(format!("Failed to establish DTLS connection: {}", e));
            }
        };

        // 5. Start stream loop
        let (tx, rx) = mpsc::channel(100);
        let group_id = self.config.entertainment_group_id.clone();

        // Spawn the loop
        self.stream_handle = Some(tokio::spawn(async move {
            run_stream_loop(streamer, rx, &group_id).await;
        }));

        self.sender = Some(tx);
        self.is_connected = true;
        info!("Hue Entertainment Stream connected.");

        Ok(())
    }

    /// Stop the stream and disconnect
    pub async fn disconnect(&mut self) {
        if !self.is_connected {
            return;
        }

        // Drop sender to close channel and stop loop
        self.sender = None;
        if let Some(handle) = self.stream_handle.take() {
            let _ = handle.await;
        }

        // Deactivate stream on bridge
        let _ = api::groups::set_stream_active(
            &self.config,
            &self.config.entertainment_group_id,
            false,
        )
        .await;

        self.is_connected = false;
        info!("Hue Controller disconnected.");
    }

    /// Process updates from the module system
    /// `updates` is a list of (LightID, R, G, B)
    pub async fn process_updates(&mut self, updates: Vec<(String, f32, f32, f32)>) {
        if !self.is_connected {
            return;
        }

        if let Some(tx) = &self.sender {
            let mut stream_updates = Vec::with_capacity(updates.len());

            for (light_id, r, g, b) in updates {
                // Find channel ID
                if let Some(node) = self.nodes.get(&light_id) {
                    stream_updates.push(LightState {
                        id: node.channel_id,
                        r: (r * 255.0).clamp(0.0, 255.0) as u8,
                        g: (g * 255.0).clamp(0.0, 255.0) as u8,
                        b: (b * 255.0).clamp(0.0, 255.0) as u8,
                    });
                }
            }

            if !stream_updates.is_empty() && (tx.send(stream_updates).await).is_err() {
                error!("Hue stream channel closed unexpectedly");
                self.is_connected = false;
            }
        }
    }

    /// Process a high-level command from the main loop (Sync wrapper)
    pub fn update_from_command(
        &mut self,
        ids: Option<&[String]>,
        brightness: f32,
        hue: Option<f32>,
        saturation: Option<f32>,
        _strobe: Option<f32>,
    ) {
        if !self.is_connected || self.sender.is_none() {
            return;
        }

        // 1. Calculate RGB from HSB
        use palette::{FromColor, Hsv, Srgb};

        let h = hue.unwrap_or(0.0) * 360.0;
        let s = saturation.unwrap_or(1.0);
        let v = brightness; // Use brightness as Value/Brightness

        let hsv = Hsv::new(h, s, v);
        let rgb: Srgb = Srgb::from_color(hsv);

        let r = rgb.red;
        let g = rgb.green;
        let b = rgb.blue;

        // 2. Identify targets
        let tx = self.sender.as_ref().unwrap();
        let mut stream_updates = Vec::new();

        if let Some(target_ids) = ids {
            // Update specific lights
            for id in target_ids {
                if let Some(node) = self.nodes.get(id) {
                    stream_updates.push(LightState {
                        id: node.channel_id,
                        r: (r * 255.0).clamp(0.0, 255.0) as u8,
                        g: (g * 255.0).clamp(0.0, 255.0) as u8,
                        b: (b * 255.0).clamp(0.0, 255.0) as u8,
                    });
                }
            }
        } else {
            // Update ALL lights (Broadcast/Group node)
            for node in self.nodes.values() {
                stream_updates.push(LightState {
                    id: node.channel_id,
                    r: (r * 255.0).clamp(0.0, 255.0) as u8,
                    g: (g * 255.0).clamp(0.0, 255.0) as u8,
                    b: (b * 255.0).clamp(0.0, 255.0) as u8,
                });
            }
        }

        if !stream_updates.is_empty() {
            // Use try_send for sync context. If channel is full, we drop the frame (better than blocking)
            if let Err(e) = tx.try_send(stream_updates) {
                use tokio::sync::mpsc::error::TrySendError;
                match e {
                    TrySendError::Full(_) => {
                        // warn!("Hue stream buffer full, dropping frame");
                    }
                    TrySendError::Closed(_) => {
                        error!("Hue stream channel closed");
                        self.is_connected = false;
                    }
                }
            }
        }
    }

    /// Handle update for a single lamp (convenience)
    pub fn get_lamp_position(&self, light_id: &str) -> Option<(f64, f64, f64)> {
        self.nodes.get(light_id).map(|n| (n.x, n.y, n.z))
    }
}




