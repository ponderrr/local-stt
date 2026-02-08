use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, Stream, StreamConfig};
use std::sync::mpsc;

pub struct AudioCapture {
    stream: Option<Stream>,
    pub sample_rate: u32,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            stream: None,
            sample_rate: 16000,
        }
    }

    pub fn list_devices() -> Result<Vec<String>, String> {
        let host = cpal::default_host();
        let devices = host
            .input_devices()
            .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

        let names: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();

        Ok(names)
    }

    pub fn start(
        &mut self,
        device_name: Option<&str>,
        sender: mpsc::Sender<Vec<f32>>,
    ) -> Result<(), String> {
        let host = cpal::default_host();

        let device = match device_name {
            Some(name) => host
                .input_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| format!("Audio device not found: {}", name))?,
            None => host
                .default_input_device()
                .ok_or("No default input device found")?,
        };

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    sender.send(data.to_vec()).ok();
                },
                |err| eprintln!("[audio] Stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start audio stream: {}", e))?;

        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.stream = None; // Drop the stream, which stops capture
    }

    pub fn is_active(&self) -> bool {
        self.stream.is_some()
    }
}
