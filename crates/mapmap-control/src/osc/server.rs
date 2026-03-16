//! OSC server for receiving messages

#[cfg(feature = "osc")]
use rosc::{decoder, OscPacket};
#[cfg(feature = "osc")]
use std::net::UdpSocket;
#[cfg(feature = "osc")]
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
#[cfg(feature = "osc")]
use std::thread;

use crate::{error::ControlError, Result};

/// Maximum number of pending OSC packets in the channel.
/// This limit prevents memory exhaustion attacks (DoS) by applying backpressure
/// to the UDP socket when the application cannot keep up with incoming packets.
#[cfg(feature = "osc")]
const MAX_PENDING_PACKETS: usize = 1024;

/// OSC server for receiving control messages
pub struct OscServer {
    #[cfg(feature = "osc")]
    receiver: Receiver<OscPacket>,
    #[cfg(feature = "osc")]
    _handle: Option<thread::JoinHandle<()>>,
}

impl OscServer {
    /// Create a new OSC server listening on the specified port (bound to 127.0.0.1)
    ///
    /// # Arguments
    /// * `port` - UDP port to listen on (default: 8000)
    #[cfg(feature = "osc")]
    pub fn new(port: u16) -> Result<Self> {
        Self::new_with_host("127.0.0.1", port)
    }

    /// Create a new OSC server listening on the specified host and port
    ///
    /// # Arguments
    /// * `host` - Host address to bind to (e.g. "127.0.0.1" or "0.0.0.0")
    /// * `port` - UDP port to listen on
    #[cfg(feature = "osc")]
    pub fn new_with_host(host: &str, port: u16) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        let socket = UdpSocket::bind(&addr)
            .map_err(|e| ControlError::OscError(format!("Failed to bind to {}: {}", addr, e)))?;

        tracing::info!("OSC server listening on {}", addr);

        // Use a bounded channel to prevent memory exhaustion DoS
        let (sender, receiver) = sync_channel(MAX_PENDING_PACKETS);

        // Spawn receiver thread
        let handle = thread::spawn(move || {
            Self::run_receiver(socket, sender);
        });

        Ok(Self {
            receiver,
            _handle: Some(handle),
        })
    }

    #[cfg(not(feature = "osc"))]
    pub fn new(_port: u16) -> Result<Self> {
        Err(ControlError::OscError(
            "OSC feature not enabled".to_string(),
        ))
    }

    #[cfg(not(feature = "osc"))]
    pub fn new_with_host(_host: &str, _port: u16) -> Result<Self> {
        Err(ControlError::OscError(
            "OSC feature not enabled".to_string(),
        ))
    }

    /// Run the receiver loop (blocking)
    #[cfg(feature = "osc")]
    fn run_receiver(socket: UdpSocket, sender: SyncSender<OscPacket>) {
        let mut buf = [0u8; 65536]; // Max UDP packet size

        loop {
            match socket.recv_from(&mut buf) {
                Ok((size, addr)) => match decoder::decode_udp(&buf[..size]) {
                    Ok((_, packet)) => {
                        if sender.send(packet).is_err() {
                            // Stop the thread if the receiver has disconnected
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to decode OSC packet from {}: {}", addr, e);
                    }
                },
                Err(e) => {
                    tracing::error!("OSC socket error: {}", e);
                    break;
                }
            }
        }
    }

    /// Poll for new OSC packets (non-blocking)
    ///
    /// Returns `None` if no packets are available
    #[cfg(feature = "osc")]
    pub fn poll_packet(&self) -> Option<OscPacket> {
        self.receiver.try_recv().ok()
    }

    #[cfg(not(feature = "osc"))]
    pub fn poll_packet(&self) -> Option<OscPacket> {
        None
    }
}

#[cfg(all(test, feature = "osc"))]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_osc_server_creation() {
        // Use a high port to avoid permission issues
        let server = OscServer::new(18000);
        assert!(server.is_ok());
    }

    #[test]
    fn test_osc_server_client_communication() {
        use crate::osc::client::OscClient;
        use rosc::OscType;

        // Create server on a high port
        let server = OscServer::new(18001).unwrap();

        // Give server time to start
        thread::sleep(Duration::from_millis(100));

        // Create client
        let client = OscClient::new("127.0.0.1:18001").unwrap();

        // Send a message
        client
            .send_message("/stagegraph/layer/0/opacity", vec![OscType::Float(0.5)])
            .unwrap();

        // Wait a bit for the message to arrive
        thread::sleep(Duration::from_millis(100));

        // Poll for packet
        if let Some(OscPacket::Message(msg)) = server.poll_packet() {
            assert_eq!(msg.addr, "/stagegraph/layer/0/opacity");
            assert_eq!(msg.args, vec![OscType::Float(0.5)]);
        } else {
            panic!("Expected OSC packet");
        }
    }

    #[test]
    fn test_osc_backpressure() {
        use crate::osc::client::OscClient;
        use rosc::OscType;

        // Use a different port to avoid conflict
        let port = 18002;
        let server = OscServer::new(port).expect("Failed to bind server");

        // Give server time to start
        thread::sleep(Duration::from_millis(100));

        let client =
            OscClient::new(&format!("127.0.0.1:{}", port)).expect("Failed to create client");

        // Send more packets than the buffer size (MAX_PENDING_PACKETS = 1024)
        // We send 2000 packets quickly to trigger backpressure.
        for i in 0..2000 {
            // Errors here (packet drop) are expected if kernel buffer fills up
            let _ = client.send_message("/test", vec![OscType::Int(i)]);
        }

        // Wait for processing
        thread::sleep(Duration::from_millis(500));

        // Drain the channel
        let mut count = 0;
        let mut empty_streak = 0;
        loop {
            if server.poll_packet().is_some() {
                count += 1;
                empty_streak = 0;
            } else {
                empty_streak += 1;
                thread::sleep(Duration::from_millis(10));
            }

            if empty_streak > 10 {
                // 100ms of silence
                break;
            }
            if count > 5000 {
                break;
            }
        }

        // We expect to have received some packets (at least buffer size + kernel buffer)
        assert!(count > 0, "Should have received packets");

        // Verify server is still responsive
        client.send_message("/test/alive", vec![]).unwrap();
        thread::sleep(Duration::from_millis(100));

        let mut found_alive = false;
        // Check next few packets for alive message
        for _ in 0..100 {
            if let Some(rosc::OscPacket::Message(msg)) = server.poll_packet() {
                if msg.addr == "/test/alive" {
                    found_alive = true;
                    break;
                }
            } else {
                break;
            }
        }
        assert!(found_alive, "Server should be responsive after flood");
    }
}
