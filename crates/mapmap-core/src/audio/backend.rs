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
    use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
    use std::time::Duration;

    enum Command {
        Pause,
        Play,
        Stop,
    }

    /// Status of the audio backend initialization
    #[derive(Debug, Clone)]
    pub enum InitStatus {
        Success,
        DeviceNotFound(String),
        ConfigError(String),
        StreamBuildError(String),
    }

    /// CPAL audio backend
    pub struct CpalBackend {
        sample_receiver: Receiver<Vec<f32>>,
        command_sender: Sender<Command>,
        thread_handle: Option<std::thread::JoinHandle<()>>,
        is_running: bool,
    }

    impl CpalBackend {
        /// Creates a new CPAL backend with the specified device.
        /// Uses a timeout to prevent indefinite blocking on problematic devices.
        pub fn new(device_name: Option<String>) -> Result<Self, AudioError> {
            let (sample_tx, sample_rx) = unbounded();
            let (command_tx, command_rx) = unbounded();
            // Channel for initialization status with timeout support
            let (init_tx, init_rx) = bounded::<InitStatus>(1);

            let device_name_clone = device_name.clone();
            let thread_handle = std::thread::Builder::new()
                .name("audio-backend".to_string())
                .spawn(move || {
                    Self::audio_thread(device_name_clone, sample_tx, command_rx, init_tx);
                })
                .map_err(|e| {
                    AudioError::StreamBuildError(format!("Failed to spawn audio thread: {}", e))
                })?;

            // Wait for initialization with a timeout (5 seconds)
            match init_rx.recv_timeout(Duration::from_secs(5)) {
                Ok(InitStatus::Success) => Ok(Self {
                    sample_receiver: sample_rx,
                    command_sender: command_tx,
                    thread_handle: Some(thread_handle),
                    is_running: false,
                }),
                Ok(InitStatus::DeviceNotFound(msg)) => {
                    // Clean up thread
                    let _ = command_tx.send(Command::Stop);
                    let _ = thread_handle.join();
                    Err(AudioError::NoDevicesFound(msg))
                }
                Ok(InitStatus::ConfigError(_msg)) => {
                    let _ = command_tx.send(Command::Stop);
                    let _ = thread_handle.join();
                    Err(AudioError::UnsupportedFormat)
                }
                Ok(InitStatus::StreamBuildError(msg)) => {
                    let _ = command_tx.send(Command::Stop);
                    let _ = thread_handle.join();
                    Err(AudioError::StreamBuildError(msg))
                }
                Err(_) => {
                    // Timeout - device is not responding
                    // Note: We cannot easily kill the thread, but we can detach it
                    // The thread will eventually exit or continue running in background
                    eprintln!(
                        "Audio device initialization timed out for device: {:?}",
                        device_name
                    );
                    Err(AudioError::StreamBuildError(
                        format!("Device initialization timed out. The device may be in use or not responding: {:?}", device_name)
                    ))
                }
            }
        }

        /// The audio processing thread
        fn audio_thread(
            device_name: Option<String>,
            sample_tx: Sender<Vec<f32>>,
            command_rx: Receiver<Command>,
            init_tx: Sender<InitStatus>,
        ) {
            let host = cpal::default_host();

            // Get device with proper error handling
            let device = if let Some(ref name) = device_name {
                match host.input_devices() {
                    Ok(mut devices) => {
                        match devices.find(|d| d.name().map(|n| n == *name).unwrap_or(false)) {
                            Some(dev) => dev,
                            None => {
                                let _ = init_tx.send(InitStatus::DeviceNotFound(format!(
                                    "Device '{}' not found",
                                    name
                                )));
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = init_tx.send(InitStatus::DeviceNotFound(e.to_string()));
                        return;
                    }
                }
            } else {
                match host.default_input_device() {
                    Some(dev) => dev,
                    None => {
                        let _ = init_tx.send(InitStatus::DeviceNotFound(
                            "No default input device available".to_string(),
                        ));
                        return;
                    }
                }
            };

            // Get device config with error handling
            let config = match device.default_input_config() {
                Ok(cfg) => cfg,
                Err(e) => {
                    let _ = init_tx.send(InitStatus::ConfigError(e.to_string()));
                    return;
                }
            };

            let err_fn = |err| eprintln!("Audio stream error: {}", err);

            // Build stream with proper error handling
            let stream_result = match config.sample_format() {
                cpal::SampleFormat::F32 => {
                    let tx = sample_tx.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
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
                format => {
                    let _ = init_tx.send(InitStatus::StreamBuildError(format!(
                        "Unsupported sample format: {:?}",
                        format
                    )));
                    return;
                }
            };

            let stream = match stream_result {
                Ok(s) => s,
                Err(e) => {
                    let _ = init_tx.send(InitStatus::StreamBuildError(e.to_string()));
                    return;
                }
            };

            // Signal successful initialization
            let _ = init_tx.send(InitStatus::Success);

            // Process commands
            loop {
                match command_rx.recv() {
                    Ok(Command::Play) => {
                        if let Err(e) = stream.play() {
                            eprintln!("Failed to play audio stream: {}", e);
                        }
                    }
                    Ok(Command::Pause) => {
                        if let Err(e) = stream.pause() {
                            eprintln!("Failed to pause audio stream: {}", e);
                        }
                    }
                    Ok(Command::Stop) | Err(_) => {
                        // Stop stream and exit thread
                        let _ = stream.pause();
                        break;
                    }
                }
            }
        }

        /// Lists available audio input devices.
        /// Uses a timeout to prevent hanging on problematic audio hosts.
        pub fn list_devices() -> Result<Option<Vec<String>>, AudioError> {
            // Use a thread with timeout to prevent hanging
            let (tx, rx) = bounded::<Result<Vec<String>, AudioError>>(1);

            std::thread::spawn(move || {
                let result = Self::list_devices_internal();
                let _ = tx.send(result);
            });

            match rx.recv_timeout(Duration::from_secs(3)) {
                Ok(result) => result.map(Some),
                Err(_) => {
                    eprintln!("Timeout while listing audio devices");
                    Ok(Some(vec![])) // Return empty list on timeout instead of error
                }
            }
        }

        fn list_devices_internal() -> Result<Vec<String>, AudioError> {
            let host = cpal::default_host();
            let devices = host
                .input_devices()
                .map_err(|e| AudioError::NoDevicesFound(e.to_string()))?;
            let device_names: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();
            Ok(device_names)
        }

        /// Check if the backend is currently running
        pub fn is_running(&self) -> bool {
            self.is_running
        }
    }

    impl AudioBackend for CpalBackend {
        fn start(&mut self) -> Result<(), AudioError> {
            if self.command_sender.send(Command::Play).is_err() {
                return Err(AudioError::StreamBuildError(
                    "Audio thread not responding".to_string(),
                ));
            }
            self.is_running = true;
            Ok(())
        }

        fn stop(&mut self) {
            let _ = self.command_sender.send(Command::Pause);
            self.is_running = false;
        }

        fn get_samples(&mut self) -> Vec<f32> {
            self.sample_receiver.try_iter().flatten().collect()
        }
    }

    impl Drop for CpalBackend {
        fn drop(&mut self) {
            // Send stop command and close channel
            let _ = self.command_sender.send(Command::Stop);

            // Wait for thread with timeout
            if let Some(handle) = self.thread_handle.take() {
                // Give thread time to clean up (don't block forever)
                std::thread::spawn(move || {
                    let _ = handle.join();
                });
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
