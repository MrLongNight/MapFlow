use anyhow::Result;
use std::sync::Arc;
use tokio::net::UdpSocket;
use webrtc_dtls::cipher_suite::CipherSuiteId;
use webrtc_dtls::config::Config;
use webrtc_dtls::conn::DTLSConn;

pub struct HueStreamer {
    conn: Arc<DTLSConn>,
}

impl HueStreamer {
    /// Connects to the Hue Bridge via DTLS for entertainment streaming.
<<<<<<< HEAD
    pub async fn connect(ip: &str, application_id: &str, client_key: &str) -> Result<Self> {
        let psk = hex::decode(client_key)?;
        let app_id = application_id.as_bytes().to_vec();

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(format!("{}:2100", ip)).await?;

        // Use UdpConn from webrtc_util
        let webrtc_conn = Arc::new(socket);

        let config = Config {
            psk: Some(Arc::new(move |_| Ok(psk.clone()))),
            psk_identity_hint: Some(app_id),
            insecure_skip_verify: true,
            cipher_suites: vec![CipherSuiteId::Tls_Psk_With_Aes_128_Gcm_Sha256],
            ..Default::default()
        };

        let conn = DTLSConn::new(
            webrtc_conn,
            config,
            true, // is_client
            None,
        )
        .await?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.conn.write(buf, None).await?;
        Ok(())
=======
    ///
    /// STUBBED: OpenSSL is currently disabled due to build hangs on Windows.
    #[allow(unused_variables)]
    pub fn connect(ip: &str, application_id: &str, client_key: &str) -> Result<Self> {
        Err(anyhow!("Hue Entertainment streaming is currently disabled because OpenSSL support is not compiled in (build hang avoidance)."))
    }

    #[allow(unused_variables)]
    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        Err(anyhow!("Hue Entertainment streaming is disabled."))
>>>>>>> origin/jules/ui-panel-consistency-3372896917882100483
    }
}
