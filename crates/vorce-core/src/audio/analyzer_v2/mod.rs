//! Audio Analyzer V2 - Simplified FFT-based audio analysis
//!
//! This module provides a working audio analyzer using rustfft directly,
//! with proper sample buffering and FFT processing.

mod analyzer;
mod beat;
mod fft;
mod types;

pub use analyzer::AudioAnalyzerV2;
pub use types::{AudioAnalysisV2, AudioAnalyzerV2Config};

#[cfg(test)]
mod tests;
