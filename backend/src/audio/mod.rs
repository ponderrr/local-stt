//! Audio capture pipeline: microphone input via cpal, ring buffer staging,
//! format conversion (mono + resample to 16kHz), VAD filtering, and chunk dispatch.

pub mod buffer;
pub mod capture;
pub mod vad;

use buffer::AudioRingBuffer;
use capture::AudioCapture;
use ringbuf::traits::{Consumer, Split};
use ringbuf::HeapRb;
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- to_mono Tests ---

    #[test]
    fn test_to_mono_passthrough_single_channel() {
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let result = to_mono(&input, 1);
        assert_eq!(result, input, "mono input should pass through unchanged");
    }

    #[test]
    fn test_to_mono_stereo_averages_channels() {
        // Interleaved stereo: [L0, R0, L1, R1, ...]
        let input = vec![1.0, 3.0, 2.0, 4.0];
        let result = to_mono(&input, 2);
        assert_eq!(result.len(), 2, "stereo->mono should halve sample count");
        assert!(
            (result[0] - 2.0).abs() < 1e-6,
            "first sample should be avg of 1.0 and 3.0"
        );
        assert!(
            (result[1] - 3.0).abs() < 1e-6,
            "second sample should be avg of 2.0 and 4.0"
        );
    }

    #[test]
    fn test_to_mono_empty_input() {
        let result = to_mono(&[], 2);
        assert!(result.is_empty(), "empty input should produce empty output");
    }

    #[test]
    fn test_to_mono_multichannel() {
        // 4 channels: [C0, C1, C2, C3] per frame
        let input = vec![1.0, 2.0, 3.0, 4.0]; // one frame of 4 channels
        let result = to_mono(&input, 4);
        assert_eq!(result.len(), 1);
        assert!(
            (result[0] - 2.5).abs() < 1e-6,
            "should average all 4 channels: (1+2+3+4)/4 = 2.5"
        );
    }

    #[test]
    fn test_to_mono_stereo_preserves_signal_shape() {
        // Generate stereo sine wave (same signal on both channels)
        let mono_signal: Vec<f32> = (0..1000)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
            .collect();
        let stereo: Vec<f32> = mono_signal
            .iter()
            .flat_map(|&s| vec![s, s]) // duplicate to both channels
            .collect();

        let result = to_mono(&stereo, 2);
        assert_eq!(result.len(), 1000);
        for (i, (&expected, &actual)) in mono_signal.iter().zip(result.iter()).enumerate() {
            assert!(
                (expected - actual).abs() < 1e-6,
                "sample {} mismatch: expected {}, got {}",
                i,
                expected,
                actual
            );
        }
    }

    #[test]
    fn test_to_mono_stereo_remainder_ignored() {
        // chunks_exact drops remainder, so odd sample count is handled
        let input = vec![1.0, 3.0, 2.0]; // 1.5 frames of stereo -- last sample dropped
        let result = to_mono(&input, 2);
        assert_eq!(
            result.len(),
            1,
            "remainder samples should be dropped by chunks_exact"
        );
        assert!((result[0] - 2.0).abs() < 1e-6);
    }

    // --- resample Tests ---

    #[test]
    fn test_resample_same_rate_passthrough() {
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = resample(&input, 16000, 16000);
        assert_eq!(result, input, "same rate should pass through unchanged");
    }

    #[test]
    fn test_resample_empty_input() {
        let result = resample(&[], 48000, 16000);
        assert!(result.is_empty(), "empty input should produce empty output");
    }

    #[test]
    fn test_resample_48k_to_16k_ratio() {
        // 48kHz to 16kHz is a 3:1 ratio
        let input = vec![0.0f32; 4800]; // 100ms at 48kHz
        let result = resample(&input, 48000, 16000);
        assert_eq!(
            result.len(),
            1600,
            "48kHz->16kHz should produce 1/3 the samples"
        );
    }

    #[test]
    fn test_resample_44100_to_16000_ratio() {
        // 44.1kHz to 16kHz
        let input = vec![0.0f32; 4410]; // 100ms at 44.1kHz
        let result = resample(&input, 44100, 16000);
        let expected_len = (4410.0 / (44100.0 / 16000.0)) as usize;
        assert_eq!(
            result.len(),
            expected_len,
            "44.1kHz->16kHz should produce correct sample count"
        );
    }

    #[test]
    fn test_resample_preserves_dc_offset() {
        // A constant signal at any rate should remain constant after resampling
        let input = vec![0.75f32; 4800]; // 100ms at 48kHz
        let result = resample(&input, 48000, 16000);
        for (i, &val) in result.iter().enumerate() {
            assert!(
                (val - 0.75).abs() < 1e-5,
                "constant signal should be preserved after resampling, sample {} = {}",
                i,
                val
            );
        }
    }

    #[test]
    fn test_resample_low_frequency_signal_preserved() {
        // A 100Hz sine wave sampled at 48kHz should be faithfully reproduced at 16kHz
        // (well below Nyquist of 8kHz)
        let freq = 100.0f32;
        let input: Vec<f32> = (0..4800)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / 48000.0).sin())
            .collect();
        let result = resample(&input, 48000, 16000);

        // Verify the output is a 100Hz sine at 16kHz
        for (i, &val) in result.iter().enumerate() {
            let expected = (2.0 * std::f32::consts::PI * freq * i as f32 / 16000.0).sin();
            assert!(
                (val - expected).abs() < 0.05,
                "resampled sine wave sample {} deviates: expected {}, got {}",
                i,
                expected,
                val
            );
        }
    }

    #[test]
    fn test_resample_upsampling() {
        // Test upsampling from 16kHz to 48kHz
        let input = vec![0.0f32; 1600]; // 100ms at 16kHz
        let result = resample(&input, 16000, 48000);
        assert_eq!(
            result.len(),
            4800,
            "16kHz->48kHz should produce 3x the samples"
        );
    }

    #[test]
    fn test_resample_single_sample() {
        let input = vec![0.5f32];
        let result = resample(&input, 48000, 16000);
        // Output length should be 0 since (1.0 / 3.0) as usize = 0
        // This is an edge case -- very short input produces empty output
        assert!(
            result.is_empty() || result.len() == 1,
            "single sample resampling is an edge case"
        );
    }

    #[test]
    fn test_resample_output_in_valid_range() {
        // If input is in [-1.0, 1.0], output should also be in [-1.0, 1.0]
        // (linear interpolation preserves bounds)
        let input: Vec<f32> = (0..4800)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
            .collect();
        let result = resample(&input, 48000, 16000);
        for (i, &val) in result.iter().enumerate() {
            assert!(
                val >= -1.0 - 1e-6 && val <= 1.0 + 1e-6,
                "resampled sample {} = {} is out of [-1, 1] range",
                i,
                val
            );
        }
    }
}

pub struct AudioPipeline {
    is_running: Arc<AtomicBool>,
}

impl Default for AudioPipeline {
    fn default() -> Self {
        Self::new()
    }
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
            // 3 seconds of audio at 48kHz stereo (conservative capacity)
            let rb_capacity = 48000 * 2 * 3;
            let rb = HeapRb::<f32>::new(rb_capacity);
            let (producer, mut consumer) = rb.split();

            let mut capture = AudioCapture::new();

            // Try to start capture
            if let Err(e) = capture.start(device_name.as_deref(), producer) {
                init_tx.send(Err(e)).ok();
                return;
            }
            // Signal success
            init_tx.send(Ok(())).ok();

            let device_rate = capture.device_sample_rate;
            let device_channels = capture.device_channels;

            let mut buffer = AudioRingBuffer::new(16000, chunk_duration_ms, overlap_ms, 30);
            let mut vad = VoiceActivityDetector::new(vad_threshold);

            let mut read_buf = vec![0.0f32; 4800]; // 100ms at 48kHz
            while running.load(Ordering::SeqCst) {
                let n = consumer.pop_slice(&mut read_buf);
                if n > 0 {
                    let mono = to_mono(&read_buf[..n], device_channels);
                    let resampled = resample(&mono, device_rate, 16000);
                    buffer.write(&resampled);

                    if buffer.has_chunk() {
                        if let Some(chunk) = buffer.extract_chunk() {
                            if vad.contains_speech(&chunk) && chunk_tx.send(chunk).is_err() {
                                break; // Receiver dropped
                            }
                        }
                    }
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(10));
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
