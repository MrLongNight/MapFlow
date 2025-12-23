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
        thread_handle: Option<std::thread::JoinHandle<()>>,
    }

    impl CpalBackend {
        pub fn new(device_name: Option<String>) -> Result<Self, AudioError> {
            let (sample_tx, sample_rx) = unbounded();
            let (command_tx, command_rx) = unbounded();

            let thread_handle = std::thread::spawn(move || {
                let host = cpal::default_host();
                let device = if let Some(name) = device_name {
                    host.input_devices()
                        .unwrap()
                        .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                        .unwrap()
                } else {
                    host.default_input_device().unwrap()
                };

                let config = device.default_input_config().unwrap();

                let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

                let stream = match config.sample_format() {
                    cpal::SampleFormat::F32 => device.build_input_stream(
                        &config.into(),
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let _ = sample_tx.send(data.to_vec());
                        },
                        err_fn,
                        None,
                    ),
                    cpal::SampleFormat::I16 => device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            let samples: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                            let _ = sample_tx.send(samples);
                        },
                        err_fn,
                        None,
                    ),
                    cpal::SampleFormat::U16 => device.build_input_stream(
                        &config.into(),
                        move |data: &[u16], _: &cpal::InputCallbackInfo| {
                            let samples: Vec<f32> = data
                                .iter()
                                .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                                .collect();
                            let _ = sample_tx.send(samples);
                        },
                        err_fn,
                        None,
                    ),
                    _ => panic!("Unsupported sample format"),
                }
                .unwrap();

                loop {
                    match command_rx.recv() {
                        Ok(Command::Play) => stream.play().unwrap(),
                        Ok(Command::Pause) => stream.pause().unwrap(),
                        Err(_) => break, // Channel closed, exit thread
                    }
                }
            });

            Ok(Self {
                sample_receiver: sample_rx,
                command_sender: command_tx,
                thread_handle: Some(thread_handle),
            })
        }

        pub fn list_devices() -> Result<Option<Vec<String>>, AudioError> {
            let host = cpal::default_host();
            let devices = host
                .input_devices()
                .map_err(|e| AudioError::NoDevicesFound(e.to_string()))?;
            let device_names = devices
                .map(|d| d.name().unwrap_or_else(|_| "Unnamed Device".to_string()))
                .collect();
            Ok(Some(device_names))
        }
    }

    impl AudioBackend for CpalBackend {
        fn start(&mut self) -> Result<(), AudioError> {
            self.command_sender.send(Command::Play).unwrap();
            Ok(())
        }

        fn stop(&mut self) {
            self.command_sender.send(Command::Pause).unwrap();
        }

        fn get_samples(&mut self) -> Vec<f32> {
            self.sample_receiver.try_iter().flatten().collect()
        }
    }

    impl Drop for CpalBackend {
        fn drop(&mut self) {
            if let Some(handle) = self.thread_handle.take() {
                drop(self.command_sender.clone()); // Close channel to signal exit
                handle.join().unwrap();
            }
        }
    }
}

/// A mock audio backend for testing without native audio dependencies
#[cfg(any(test, feature = "mock-audio"))]
pub mod mock_backend {
    use super::{AudioBackend, AudioError};

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
