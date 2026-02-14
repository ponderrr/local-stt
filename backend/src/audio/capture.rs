use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use std::sync::mpsc;

pub struct AudioCapture {
    stream: Option<Stream>,
    pub device_sample_rate: u32,
    pub device_channels: u16,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            stream: None,
            device_sample_rate: 48000,
            device_channels: 1,
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

        let supported_config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        let sample_format = supported_config.sample_format();
        eprintln!("[audio] Device config: rate={}Hz, channels={}, format={:?}",
            supported_config.sample_rate().0, supported_config.channels(), sample_format);
        let config: StreamConfig = supported_config.into();
        self.device_sample_rate = config.sample_rate.0;
        self.device_channels = config.channels;

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

}
