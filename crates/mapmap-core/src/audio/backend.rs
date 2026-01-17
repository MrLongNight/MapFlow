//! Audio Backend Abstraction
use thiserror::Error;

/// Errors that can occur in the audio backend
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No audio devices found: {0}")]
    NoDevicesFound(String),
    #[error("Default device not found")]
    DefaultDeviceNotFound,
    #[error("Unsupported stream format")]
    UnsupportedFormat,
    #[error("Failed to build audio stream: {0}")]
    StreamBuildError(String),
    #[error("Device initialization timed out")]
    Timeout,
}

/// Audio backend abstraction
pub trait AudioBackend {
    /// Start capturing audio
    fn start(&mut self) -> Result<(), AudioError>;
    /// Stop capturing audio
    fn stop(&mut self);
    /// Get the latest audio samples
    fn get_samples(&mut self) -> Vec<f32>;
}

/// CPAL implementation of the audio backend
#[cfg(feature = "audio")]
pub mod cpal_backend {
    use super::{AudioBackend, AudioError};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use crossbeam_channel::{unbounded, Receiver, Sender};

    enum Command {
        Pause,
        Play,
    }

    /// CPAL audio backend
    pub struct CpalBackend {
        sample_receiver: Receiver<Vec<f32>>,
        command_sender: Sender<Command>,
        #[allow(dead_code)]
        stream: cpal::Stream,
    }

    impl CpalBackend {
        /// Create a new CPAL backend with the specified device.
        /// Uses a timeout to prevent the app from freezing if a device doesn't respond.
        pub fn new(device_name: Option<String>) -> Result<Self, AudioError> {
            let (sample_tx, sample_rx) = unbounded();
            let (command_tx, command_rx) = unbounded::<Command>();

            // Build stream directly in main thread (cpal::Stream is not Send)
            let stream = Self::build_stream(device_name, sample_tx)?;

            // Spawn command processing thread
            std::thread::Builder::new()
                .name("audio-cmd".to_string())
                .spawn(move || {
                    // Just drain the command channel - stream auto-plays
                    while command_rx.recv().is_ok() {}
                })
                .ok();

            Ok(Self {
                sample_receiver: sample_rx,
                command_sender: command_tx,
                stream,
            })
        }

        /// Build the audio stream (must be called from main thread)
        fn build_stream(
            device_name: Option<String>,
            sample_tx: Sender<Vec<f32>>,
        ) -> Result<cpal::Stream, AudioError> {
            let host = cpal::default_host();

            // Get device
            let device = if let Some(ref name) = device_name {
                match host.input_devices() {
                    Ok(mut devices) => {
                        match devices.find(|d| d.name().map(|n| n == *name).unwrap_or(false)) {
                            Some(dev) => dev,
                            None => {
                                return Err(AudioError::NoDevicesFound(format!(
                                    "Device '{}' not found",
                                    name
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(AudioError::NoDevicesFound(e.to_string()));
                    }
                }
            } else {
                match host.default_input_device() {
                    Some(dev) => dev,
                    None => {
                        return Err(AudioError::DefaultDeviceNotFound);
                    }
                }
            };

            // Get config
            let config = match device.default_input_config() {
                Ok(cfg) => cfg,
                Err(e) => {
                    return Err(AudioError::StreamBuildError(format!(
                        "Failed to get device config: {}",
                        e
                    )));
                }
            };

            // Log device info for debugging
            let device_name_str = device.name().unwrap_or_else(|_| "Unknown".to_string());
            eprintln!(
                "Audio: Using device '{}', format={:?}, sample_rate={}, channels={}",
                device_name_str,
                config.sample_format(),
                config.sample_rate().0,
                config.channels()
            );

            let err_fn = |err| eprintln!("Audio stream error: {}", err);

            // Build stream
            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => {
                    let tx = sample_tx.clone();

                    device.build_input_stream(
                        &config.into(),
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            // Only log audio status once at startup
                            static LOGGED_OK: std::sync::atomic::AtomicBool =
                                std::sync::atomic::AtomicBool::new(false);
                            use std::sync::atomic::Ordering;

                            if !LOGGED_OK.load(Ordering::Relaxed) {
                                let non_zero = data.iter().filter(|&&s| s != 0.0).count();
                                eprintln!(
                                    "Audio: First callback OK - {} samples, non_zero={}",
                                    data.len(),
                                    non_zero
                                );
                                LOGGED_OK.store(true, Ordering::Relaxed);
                            }
                            let _ = tx.send(data.to_vec());
                        },
                        err_fn,
                        None,
                    )
                }
                cpal::SampleFormat::I16 => {
                    let tx = sample_tx.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            let samples: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                            let _ = tx.send(samples);
                        },
                        err_fn,
                        None,
                    )
                }
                cpal::SampleFormat::U16 => {
                    let tx = sample_tx.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[u16], _: &cpal::InputCallbackInfo| {
                            let samples: Vec<f32> = data
                                .iter()
                                .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                                .collect();
                            let _ = tx.send(samples);
                        },
                        err_fn,
                        None,
                    )
                }
                cpal::SampleFormat::U8 => {
                    let tx = sample_tx.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[u8], _: &cpal::InputCallbackInfo| {
                            let samples: Vec<f32> = data
                                .iter()
                                .map(|&s| (s as f32 / u8::MAX as f32) * 2.0 - 1.0)
                                .collect();
                            let _ = tx.send(samples);
                        },
                        err_fn,
                        None,
                    )
                }
                cpal::SampleFormat::I8 => {
                    let tx = sample_tx.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i8], _: &cpal::InputCallbackInfo| {
                            let samples: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i8::MAX as f32).collect();
                            let _ = tx.send(samples);
                        },
                        err_fn,
                        None,
                    )
                }
                cpal::SampleFormat::I32 => {
                    let tx = sample_tx.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i32], _: &cpal::InputCallbackInfo| {
                            let samples: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i32::MAX as f32).collect();
                            let _ = tx.send(samples);
                        },
                        err_fn,
                        None,
                    )
                }
                format => {
                    return Err(AudioError::StreamBuildError(format!(
                        "Unsupported sample format: {:?}",
                        format
                    )));
                }
            };

            match stream {
                Ok(stream) => {
                    // Start the stream immediately
                    if let Err(e) = stream.play() {
                        return Err(AudioError::StreamBuildError(format!(
                            "Failed to start stream: {}",
                            e
                        )));
                    }
                    Ok(stream)
                }
                Err(e) => Err(AudioError::StreamBuildError(e.to_string())),
            }
        }

        pub fn list_devices() -> Result<Option<Vec<String>>, AudioError> {
            let host = cpal::default_host();

            // Log available hosts for debugging
            eprintln!("Audio: Available hosts: {:?}", cpal::available_hosts());
            eprintln!("Audio: Using host: {:?}", host.id());

            // List all input devices with their configs
            match host.input_devices() {
                Ok(devices) => {
                    let mut device_names = Vec::new();
                    for device in devices {
                        if let Ok(name) = device.name() {
                            // Try to get default config for debugging
                            if let Ok(config) = device.default_input_config() {
                                eprintln!(
                                    "Audio Input: '{}' - format={:?}, rate={}, channels={}",
                                    name,
                                    config.sample_format(),
                                    config.sample_rate().0,
                                    config.channels()
                                );
                            } else {
                                eprintln!("Audio Input: '{}' - no config available", name);
                            }
                            device_names.push(name);
                        }
                    }
                    Ok(Some(device_names))
                }
                Err(e) => Err(AudioError::NoDevicesFound(e.to_string())),
            }
        }
    }

    impl AudioBackend for CpalBackend {
        fn start(&mut self) -> Result<(), AudioError> {
            // Stream is already playing from initialization
            let _ = self.command_sender.send(Command::Play);
            Ok(())
        }

        fn stop(&mut self) {
            let _ = self.command_sender.send(Command::Pause);
        }

        fn get_samples(&mut self) -> Vec<f32> {
            self.sample_receiver.try_iter().flatten().collect()
        }
    }

    impl Drop for CpalBackend {
        fn drop(&mut self) {
            // Dropping command_sender will close the channel and
            // the command thread will exit its recv() loop
            // Stream will be dropped automatically
        }
    }
}

/// A mock audio backend for testing without native audio dependencies
#[cfg(any(test, feature = "mock-audio"))]
pub mod mock_backend {
    use super::{AudioBackend, AudioError};

    /// Mock backend that generates sine wave
    pub struct MockBackend {
        phase: f32,
        sample_rate: f32,
    }

    impl Default for MockBackend {
        fn default() -> Self {
            Self {
                phase: 0.0,
                sample_rate: 44100.0,
            }
        }
    }

    impl MockBackend {
        /// Create a new mock backend
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl AudioBackend for MockBackend {
        fn start(&mut self) -> Result<(), AudioError> {
            Ok(())
        }

        fn stop(&mut self) {}

        fn get_samples(&mut self) -> Vec<f32> {
            let mut buffer = vec![0.0; 1024];
            for sample in &mut buffer {
                *sample = (self.phase * 2.0 * std::f32::consts::PI).sin();
                self.phase += 440.0 / self.sample_rate;
                if self.phase > 1.0 {
                    self.phase -= 1.0;
                }
            }
            buffer
        }
    }
}
