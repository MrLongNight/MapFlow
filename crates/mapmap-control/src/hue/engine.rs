use super::audio_interface::AudioSpectrum;
use super::effects::LightEffect;
use super::models::LightNode;
use super::stream::manager::LightState;
use tokio::sync::mpsc;

pub struct EntertainmentEngine {
    audio_rx: tokio::sync::broadcast::Receiver<AudioSpectrum>,
    dtls_tx: mpsc::Sender<Vec<LightState>>,
    nodes: Vec<LightNode>,
    effect: Box<dyn LightEffect>,
}

impl EntertainmentEngine {
    pub fn new(
        audio_rx: tokio::sync::broadcast::Receiver<AudioSpectrum>,
        dtls_tx: mpsc::Sender<Vec<LightState>>,
        nodes: Vec<LightNode>,
        effect: Box<dyn LightEffect>,
    ) -> Self {
        Self {
            audio_rx,
            dtls_tx,
            nodes,
            effect,
        }
    }

    pub fn set_effect(&mut self, effect: Box<dyn LightEffect>) {
        self.effect = effect;
    }

    pub fn set_nodes(&mut self, nodes: Vec<LightNode>) {
        self.nodes = nodes;
    }

    pub async fn run(&mut self) {
        loop {
            match self.audio_rx.recv().await {
                Ok(audio) => {
                    let updates_map = self.effect.update(&audio, &self.nodes);
                    let mut updates_vec = Vec::new();
                    for (id, (r, g, b)) in updates_map {
                        updates_vec.push(LightState { id, r, g, b });
                    }
                    if (self.dtls_tx.send(updates_vec).await).is_err() {
                        break; // Receiver closed
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    }
}




