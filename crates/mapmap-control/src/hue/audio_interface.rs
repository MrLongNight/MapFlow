#[derive(Debug, Clone, Copy, Default)]
pub struct AudioSpectrum {
    pub bass: f32,
    pub mids: f32,
    pub highs: f32,
    pub energy: f32,
}

pub trait AudioProcessor {
    fn process(&mut self, samples: &[f32]) -> AudioSpectrum;
}
