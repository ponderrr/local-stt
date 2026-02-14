pub mod buffer;
pub mod capture;
pub mod vad;

use buffer::AudioRingBuffer;
use capture::AudioCapture;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use vad::VoiceActivityDetector;

/// Convert interleaved multi-channel audio to mono by averaging channels.
fn to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels == 1 {
        return samples.to_vec();
    }
    let ch = channels as usize;
    samples
        .chunks_exact(ch)
        .map(|frame| frame.iter().sum::<f32>() / ch as f32)
        .collect()
}

/// Resample audio from src_rate to dst_rate using linear interpolation.
fn resample(input: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
    if src_rate == dst_rate {
        return input.to_vec();
    }
    let ratio = src_rate as f64 / dst_rate as f64;
    let output_len = (input.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);
    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = (src_pos - idx as f64) as f32;
        let a = input[idx];
        let b = input[(idx + 1).min(input.len() - 1)];
        output.push(a * (1.0 - frac) + b * frac);
    }
    output
}

pub struct AudioPipeline {
    is_running: Arc<AtomicBool>,
}

impl AudioPipeline {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the audio pipeline. Returns a receiver that yields speech audio chunks.
    pub fn start(
        &self,
        device_name: Option<String>,
        vad_threshold: f32,
        chunk_duration_ms: u32,
        overlap_ms: u32,
    ) -> Result<mpsc::Receiver<Vec<f32>>, String> {
        let is_running = self.is_running.clone();
        is_running.store(true, Ordering::SeqCst);

        let (chunk_tx, chunk_rx) = mpsc::channel::<Vec<f32>>();
        let (init_tx, init_rx) = mpsc::channel::<Result<(), String>>();

        // Spawn processing thread
        let running = is_running.clone();

        std::thread::spawn(move || {
            let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>();
            let mut capture = AudioCapture::new();

            // Try to start capture
            if let Err(e) = capture.start(device_name.as_deref(), audio_tx) {
                init_tx.send(Err(e)).ok();
                return;
            }
            // Signal success
            init_tx.send(Ok(())).ok();

            let device_rate = capture.device_sample_rate;
            let device_channels = capture.device_channels;

            let mut buffer = AudioRingBuffer::new(16000, chunk_duration_ms, overlap_ms, 30);
            let mut vad = VoiceActivityDetector::new(vad_threshold);

            while running.load(Ordering::SeqCst) {
                match audio_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(samples) => {
                        let mono = to_mono(&samples, device_channels);
                        let resampled = resample(&mono, device_rate, 16000);
                        buffer.write(&resampled);

                        if buffer.has_chunk() {
                            if let Some(chunk) = buffer.extract_chunk() {
                                if vad.contains_speech(&chunk) {
                                    if chunk_tx.send(chunk).is_err() {
                                        break; // Receiver dropped
                                    }
                                }
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        // Wait for initialization result from the thread
        match init_rx.recv() {
            Ok(result) => result.map(|_| chunk_rx),
            Err(_) => {
                Err("Failed to initialize audio thread (channel closed unexpectedly)".to_string())
            }
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}
