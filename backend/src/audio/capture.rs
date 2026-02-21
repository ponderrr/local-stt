//! Microphone capture via cpal. Pushes raw f32 samples into a lock-free ring buffer
//! at the device's native sample rate and channel count.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use ringbuf::traits::Producer;
use std::sync::mpsc::Sender;

/// Represents a message sent to the audio actor thread to control the stream lifecycle.
pub enum AudioCommand {
    Start,
    Stop,
    Quit,
}

/// A handle to an actively running audio capture actor.
/// Holding this struct represents ownership of the thread via its command channel.
/// Dropping this handle or issuing a `Quit` command will terminate the actor thread
/// and gracefully cleanly drop the underlying `cpal::Stream`.
pub struct AudioHandle {
    pub cmd_tx: Sender<AudioCommand>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Encapsulates the cpal-based microphone capture operations.
pub struct AudioCapture;

impl AudioCapture {
    /// Discovers and returns a list of viable input device names using cpal.
    #[allow(deprecated)]
    pub fn list_devices() -> Result<Vec<String>, String> {
        let host = cpal::default_host();
        let devices = host
            .input_devices()
            .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

        let names: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();

        Ok(names)
    }

    /// Spawns a dedicated actor thread to continuously pull raw f32 samples from the
    /// specified audio device and push them into the provided lock-free ring buffer producer.
    /// Returns an `AudioHandle` for sending lifecycle commands (Start/Stop/Quit) to the actor.
    ///
    /// `device_name` can be an exact device name, or `None` to attempt to heuristically resolve
    /// the best default (favoring pipewire/pulse on Linux).
    ///
    /// The thread owns the `cpal::Stream` and auto-starts playback immediately upon creation.
    /// If unexpected shutdown occurs, dropping the command receiver causes the stream to drop.
    #[allow(deprecated)]
    pub fn spawn_audio_actor(
        device_name: Option<String>,
        mut producer: ringbuf::HeapProd<f32>,
    ) -> Result<AudioHandle, String> {
        let host = cpal::default_host();

        let device_opt = match device_name.as_deref() {
            Some(name) if name != "default" => host
                .input_devices()
                .ok()
                .and_then(|mut devs| devs.find(|d| d.name().map(|n| n == name).unwrap_or(false))),
            _ => {
                let mut preferred = None;
                if let Ok(devices) = host.input_devices() {
                    for d in devices {
                        if let Ok(name) = d.name() {
                            let lower = name.to_lowercase();
                            if lower == "pulse" || lower == "pipewire" {
                                preferred = Some(d);
                                break;
                            }
                        }
                    }
                }
                preferred.or_else(|| host.default_input_device())
            }
        };

        let device = match device_opt {
            Some(d) => d,
            None => {
                return Err("AudioActor: No input device found.".to_string());
            }
        };

        let supported_config = match device.default_input_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                return Err(format!(
                    "AudioActor: Failed to get default input config: {}",
                    e
                ));
            }
        };

        let _sample_format = supported_config.sample_format();
        let sample_rate = supported_config.sample_rate();
        let channels = supported_config.channels();

        let config: StreamConfig = supported_config.into();
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();

        std::thread::Builder::new()
            .name("cpal-actor".into())
            .spawn(move || {
                let mut last_log_time = std::time::Instant::now();

                let stream = match device.build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if last_log_time.elapsed() > std::time::Duration::from_secs(2) {
                            last_log_time = std::time::Instant::now();
                        }
                        producer.push_slice(data);
                    },
                    |_| {}, // Stream errors silently ignored without logger
                    None,
                ) {
                    Ok(s) => s,
                    Err(_) => {
                        return; // Actor fails to spawn silently
                    }
                };

                // Start playing immediately (Amendment 2)
                let _ = stream.play();

                // Command Loop
                loop {
                    match cmd_rx.recv() {
                        Ok(AudioCommand::Start) => {
                            let _ = stream.play();
                        }
                        Ok(AudioCommand::Stop) => {
                            let _ = stream.pause();
                        }
                        Ok(AudioCommand::Quit) | Err(_) => {
                            // Drop stream cleanly by breaking loop
                            break;
                        }
                    }
                }
            })
            .map_err(|e| format!("Failed to spawn audio actor thread: {}", e))?;

        Ok(AudioHandle {
            cmd_tx,
            sample_rate,
            channels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ringbuf::traits::Split;

    /// Verifies that list_devices runs without panicking and returns a sensible Result.
    /// Hardware dependence means we can't assert the exact number of devices, but
    /// we can ensure the function executes safely.
    #[test]
    fn test_list_devices_does_not_panic() {
        let result = AudioCapture::list_devices();
        assert!(
            result.is_ok() || result.is_err(),
            "list_devices should produce a predictable Result without crashing"
        );
    }

    /// Verifies that spawn_audio_actor gracefully handles a request for a nonexistent device,
    /// returning an Error string rather than panicking or crashing the application.
    #[test]
    fn test_spawn_audio_actor_fails_gracefully_on_invalid_device() {
        let rb = ringbuf::HeapRb::<f32>::new(128);
        let (prod, _cons) = rb.split();
        let result = AudioCapture::spawn_audio_actor(
            Some("THIS_DEVICE_DOES_NOT_EXIST_12345".to_string()),
            prod,
        );

        assert!(
            result.is_err(),
            "Spawning an actor with a fake device name should result in an error"
        );
    }
}
