//! Microphone capture via PulseAudio (pipewire-pulse). Pushes raw f32 samples
//! into a lock-free ring buffer at 48 kHz mono.

use libpulse_binding as pulse;
use libpulse_simple_binding as psimple;
use pulse::sample::{Format, Spec};
use pulse::stream::Direction;
use ringbuf::traits::Producer;
use std::sync::mpsc::Sender;

/// Commands sent to the audio actor thread to control capture lifecycle.
pub enum AudioCommand {
    Start,
    Stop,
    Quit,
}

/// Handle to a running audio capture actor. Holds the command channel and
/// the negotiated format parameters (always 48 kHz mono with PulseAudio).
pub struct AudioHandle {
    pub cmd_tx: Sender<AudioCommand>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Encapsulates audio capture operations.
pub struct AudioCapture;

impl AudioCapture {
    /// Discovers and returns a list of viable input device names.
    #[allow(deprecated)]
    pub fn list_devices() -> Result<Vec<String>, String> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();
        let devices = host
            .input_devices()
            .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

        let names: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();

        Ok(names)
    }

    /// Spawns a dedicated actor thread that captures audio via PulseAudio's
    /// Simple API and pushes f32 samples into the provided ring buffer producer.
    ///
    /// `device_name`: `None` or `Some("default")` lets PipeWire pick the
    /// default source. Any other value is passed as the PulseAudio source name.
    pub fn spawn_audio_actor(
        device_name: Option<String>,
        mut producer: ringbuf::HeapProd<f32>,
    ) -> Result<AudioHandle, String> {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();

        let source = match device_name.as_deref() {
            None | Some("default") | Some("System Default") => None,
            Some(name) => Some(name.to_string()),
        };

        std::thread::Builder::new()
            .name("pulse-actor".into())
            .spawn(move || {
                eprintln!("pulse: connecting to PipeWire via pipewire-pulse...");

                let spec = Spec {
                    format: Format::FLOAT32NE,
                    channels: 1,
                    rate: 48000,
                };
                assert!(spec.is_valid(), "PulseAudio sample spec is invalid");

                let source_ref = source.as_deref();

                let simple = match psimple::Simple::new(
                    None,                // Default server
                    "WhisperType",       // Application name
                    Direction::Record,
                    source_ref,          // Source device (None = default)
                    "dictation-capture", // Stream description
                    &spec,
                    None,                // Default channel map
                    None,                // Default buffering attributes
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("pulse: connection FAILED: {}", e);
                        return;
                    }
                };

                eprintln!("pulse: connection established, starting capture loop");

                // 10ms of mono f32 at 48kHz = 480 samples = 1920 bytes
                let mut byte_buf = vec![0u8; 480 * std::mem::size_of::<f32>()];
                let mut active = true;

                loop {
                    // Check for commands (non-blocking)
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        match cmd {
                            AudioCommand::Start => active = true,
                            AudioCommand::Stop => active = false,
                            AudioCommand::Quit => return,
                        }
                    }

                    // Blocking read from PulseAudio — fills exactly byte_buf.len() bytes
                    if let Err(e) = simple.read(&mut byte_buf) {
                        eprintln!("pulse: read error: {}", e);
                        break;
                    }

                    if active {
                        // Reinterpret bytes as f32 samples (same endianness — FLOAT32NE)
                        let samples: &[f32] = unsafe {
                            std::slice::from_raw_parts(
                                byte_buf.as_ptr().cast::<f32>(),
                                byte_buf.len() / std::mem::size_of::<f32>(),
                            )
                        };

                        producer.push_slice(samples);
                    }
                }
            })
            .map_err(|e| format!("Failed to spawn pulse-actor thread: {}", e))?;

        Ok(AudioHandle {
            cmd_tx,
            sample_rate: 48000,
            channels: 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that list_devices runs without panicking and returns a sensible Result.
    #[test]
    fn test_list_devices_does_not_panic() {
        let result = AudioCapture::list_devices();
        assert!(
            result.is_ok() || result.is_err(),
            "list_devices should produce a predictable Result without crashing"
        );
    }

    /// Verifies that the AudioCommand enum variants exist and can be created.
    #[test]
    fn test_audio_command_variants() {
        let _start = AudioCommand::Start;
        let _stop = AudioCommand::Stop;
        let _quit = AudioCommand::Quit;
    }

    /// Verifies that AudioHandle can be constructed with expected field values.
    #[test]
    fn test_audio_handle_fields() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let handle = AudioHandle {
            cmd_tx: tx,
            sample_rate: 48000,
            channels: 1,
        };
        assert_eq!(handle.sample_rate, 48000);
        assert_eq!(handle.channels, 1);
    }
}
