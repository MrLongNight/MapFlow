//! OSC server for receiving messages

#[cfg(feature = "osc")]
use rosc::{decoder, OscPacket};
#[cfg(feature = "osc")]
use std::net::UdpSocket;
#[cfg(feature = "osc")]
use std::sync::mpsc::{channel, Receiver, Sender};
#[cfg(feature = "osc")]
use std::thread;

use crate::{error::ControlError, Result};

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

        let (sender, receiver) = channel();

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
    fn run_receiver(socket: UdpSocket, sender: Sender<OscPacket>) {
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
            .send_message("/mapmap/layer/0/opacity", vec![OscType::Float(0.5)])
            .unwrap();

        // Wait a bit for the message to arrive
        thread::sleep(Duration::from_millis(100));

        // Poll for packet
        if let Some(OscPacket::Message(msg)) = server.poll_packet() {
            assert_eq!(msg.addr, "/mapmap/layer/0/opacity");
            assert_eq!(msg.args, vec![OscType::Float(0.5)]);
        } else {
            panic!("Expected OSC packet");
        }
    }
}
