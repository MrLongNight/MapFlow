use super::dtls::HueStreamer;
use super::protocol;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct LightState {
    pub id: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Runs the entertainment streaming loop.
///
/// # Arguments
/// * `streamer` - The DTLS connection to the Hue Bridge
/// * `receiver` - Channel receiving light state updates
/// * `area_id` - The Entertainment Area ID (UUID string, 36 characters)
pub async fn run_stream_loop(
    mut streamer: HueStreamer,
    mut receiver: mpsc::Receiver<Vec<LightState>>,
    area_id: &str,
) {
    let target_frame_time = Duration::from_millis(20); // 50 FPS
    let mut last_frame_time = Instant::now();

    let mut current_lights: HashMap<u8, (u8, u8, u8)> = HashMap::new();

    loop {
        let deadline = last_frame_time + target_frame_time;

        // Wait for new data or timeout (keep-alive)
        let timeout = tokio::time::sleep_until(deadline);
        tokio::select! {
            res = receiver.recv() => {
                match res {
                    Some(updates) => {
                        // Update current state
                        for light in updates {
                            current_lights.insert(light.id, (light.r, light.g, light.b));
                        }
                    }
                    None => {
                        // Channel closed
                        break;
                    }
                }
            }
            _ = timeout => {
                // Time to send a frame (or keep-alive)
            }
        }

        // Check if we need to send
        let now = Instant::now();
        if now >= last_frame_time + target_frame_time {
            // Create message with the correct Entertainment Area ID
            if !current_lights.is_empty() {
                let msg = protocol::create_message(area_id, &current_lights);

                match streamer.write_all(&msg) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error sending Hue stream frame: {}", e);
                    }
                }
            }
            last_frame_time = now;
        }
    }
}




