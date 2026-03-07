use anyhow::{anyhow, Result};

pub struct HueStreamer {}

impl HueStreamer {
    /// Connects to the Hue Bridge via DTLS for entertainment streaming.
    ///
    /// STUBBED: OpenSSL is currently disabled due to build hangs on Windows.
    #[allow(unused_variables)]
    pub fn connect(_ip: &str, _application_id: &str, _client_key: &str) -> Result<Self> {
        Err(anyhow!("Hue Entertainment streaming is currently disabled because OpenSSL support is not compiled in (build hang avoidance)."))
    }

    #[allow(unused_variables)]
    pub fn write_all(&mut self, _buf: &[u8]) -> Result<()> {
        Err(anyhow!("Hue Entertainment streaming is disabled."))
    }
}
