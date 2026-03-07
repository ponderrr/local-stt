//! Voice activity detection backends: energy-based (RMS threshold with hysteresis)
//! and Silero (ONNX neural network with 512-sample frame accumulation).

use serde::{Deserialize, Serialize};

/// Selects which VAD backend to use for speech detection.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VadBackend {
    Energy,
    #[default]
    Silero,
}

/// Energy-based Voice Activity Detector (VAD).
///
/// Analyzes audio frames and determines the presence of speech based on RMS energy
/// exceeding a defined threshold. Implements hysteresis mapping: triggering requires
/// multiple consecutive high-energy frames (preventing false positives from clicks),
/// and releasing requires multiple consecutive low-energy frames (preventing stutter
/// within natural speech pauses).
pub struct EnergyVad {
    threshold: f32,
    min_speech_frames: usize,
    min_silence_frames: usize,
    speech_frame_count: usize,
    silence_frame_count: usize,
    is_speech: bool,
}

impl EnergyVad {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            min_speech_frames: 2, // Require 2 consecutive voiced frames to trigger
            min_silence_frames: 10, // Require 10 silent frames to end speech
            speech_frame_count: 0,
            silence_frame_count: 0,
            is_speech: false,
        }
    }

    /// Calculate RMS energy of an audio frame.
    fn rms_energy(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    /// Process a frame of audio and return whether speech is detected.
    /// Frame should be ~20-30ms of audio (320-480 samples at 16kHz).
    pub fn process_frame(&mut self, frame: &[f32]) -> bool {
        let energy = Self::rms_energy(frame);

        if energy > self.threshold {
            self.speech_frame_count += 1;
            self.silence_frame_count = 0;

            if self.speech_frame_count >= self.min_speech_frames {
                self.is_speech = true;
            }
        } else {
            self.silence_frame_count += 1;
            self.speech_frame_count = 0;

            if self.silence_frame_count >= self.min_silence_frames {
                self.is_speech = false;
            }
        }

        self.is_speech
    }

    /// Check if audio chunk contains speech (bulk check).
    pub fn contains_speech(&mut self, audio: &[f32]) -> bool {
        let frame_size = 480; // 30ms at 16kHz
        let mut any_speech = false;

        for frame in audio.chunks(frame_size) {
            if self.process_frame(frame) {
                any_speech = true;
            }
        }

        any_speech
    }
}

/// Silero VAD wrapper with 512-sample frame accumulation and hysteresis.
/// Runs Silero ONNX inference on each 512-sample frame (32ms at 16kHz)
/// and tracks speech state with configurable threshold and hold time.
pub struct SileroVad {
    model: silero_vad_rust::silero_vad::model::OnnxModel,
    frame_buffer: Vec<f32>,
    threshold: f32,
    is_speech: bool,
    /// Number of consecutive speech frames to trigger onset
    min_speech_frames: usize,
    /// Number of consecutive silence frames to trigger offset
    min_silence_frames: usize,
    speech_count: usize,
    silence_count: usize,
}

impl SileroVad {
    /// Create a new Silero VAD instance.
    /// `threshold`: speech probability threshold (0.0-1.0, default 0.5)
    pub fn new(threshold: f32) -> Result<Self, String> {
        let model = silero_vad_rust::load_silero_vad()
            .map_err(|e| format!("Failed to load Silero VAD: {}", e))?;
        Ok(Self {
            model,
            frame_buffer: Vec::with_capacity(1024),
            threshold,
            is_speech: false,
            min_speech_frames: 3,  // ~96ms at 32ms/frame
            min_silence_frames: 8, // ~256ms at 32ms/frame
            speech_count: 0,
            silence_count: 0,
        })
    }

    /// Reset internal states. Call at the start of each dictation session.
    pub fn reset(&mut self) {
        self.model.reset_states();
        self.frame_buffer.clear();
        self.is_speech = false;
        self.speech_count = 0;
        self.silence_count = 0;
    }

    /// Feed resampled 16kHz mono audio. Internally accumulates and processes
    /// 512-sample frames. Returns current speech state after processing all
    /// complete frames in the input.
    pub fn process_audio(&mut self, samples: &[f32]) -> bool {
        self.frame_buffer.extend_from_slice(samples);

        while self.frame_buffer.len() >= 512 {
            let frame: Vec<f32> = self.frame_buffer.drain(..512).collect();
            self.process_frame(&frame);
        }

        self.is_speech
    }

    /// Process a single 512-sample frame through Silero.
    fn process_frame(&mut self, frame: &[f32]) {
        let prob = match self.model.forward_chunk(frame, 16_000) {
            Ok(output) => output[[0, 0]],
            Err(e) => {
                eprintln!("silero-vad: inference error: {}", e);
                return; // Keep previous state on error
            }
        };

        if prob > self.threshold {
            self.speech_count += 1;
            self.silence_count = 0;
            if self.speech_count >= self.min_speech_frames {
                self.is_speech = true;
            }
        } else {
            self.silence_count += 1;
            self.speech_count = 0;
            if self.silence_count >= self.min_silence_frames {
                self.is_speech = false;
            }
        }
    }

    /// Check if the VAD currently detects speech.
    pub fn is_speech(&self) -> bool {
        self.is_speech
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- RMS Energy Tests ---

    #[test]
    fn test_rms_energy_of_silence_is_zero() {
        let silence = vec![0.0f32; 480];
        let energy = EnergyVad::rms_energy(&silence);
        assert!(
            (energy - 0.0).abs() < 1e-10,
            "RMS energy of silence should be 0.0"
        );
    }

    #[test]
    fn test_rms_energy_of_empty_slice_is_zero() {
        let energy = EnergyVad::rms_energy(&[]);
        assert!(
            (energy - 0.0).abs() < 1e-10,
            "RMS energy of empty slice should be 0.0"
        );
    }

    #[test]
    fn test_rms_energy_of_constant_signal() {
        // RMS of constant 0.5 should be 0.5
        let constant = vec![0.5f32; 480];
        let energy = EnergyVad::rms_energy(&constant);
        assert!(
            (energy - 0.5).abs() < 1e-5,
            "RMS energy of constant 0.5 should be 0.5, got {}",
            energy
        );
    }

    #[test]
    fn test_rms_energy_of_sine_wave() {
        // RMS of sin wave with amplitude A = A / sqrt(2) ~= 0.7071 for A=1.0
        let sine: Vec<f32> = (0..4800)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16000.0).sin())
            .collect();
        let energy = EnergyVad::rms_energy(&sine);
        let expected = 1.0 / (2.0f32).sqrt();
        assert!(
            (energy - expected).abs() < 0.01,
            "RMS of unit sine should be ~0.707, got {}",
            energy
        );
    }

    #[test]
    fn test_rms_energy_of_negative_values() {
        // RMS should be the same regardless of sign
        let positive = vec![0.3f32; 480];
        let negative = vec![-0.3f32; 480];
        let energy_pos = EnergyVad::rms_energy(&positive);
        let energy_neg = EnergyVad::rms_energy(&negative);
        assert!(
            (energy_pos - energy_neg).abs() < 1e-6,
            "RMS energy should be sign-invariant"
        );
    }

    #[test]
    fn test_rms_energy_single_sample() {
        let energy = EnergyVad::rms_energy(&[0.8]);
        assert!(
            (energy - 0.8).abs() < 1e-6,
            "RMS of single sample 0.8 should be 0.8, got {}",
            energy
        );
    }

    // --- process_frame: Silence Detection ---

    #[test]
    fn test_silence_detected() {
        let mut vad = EnergyVad::new(0.01);
        let silence = vec![0.0f32; 480];
        assert!(!vad.process_frame(&silence));
    }

    #[test]
    fn test_single_silent_frame_does_not_trigger_speech() {
        let mut vad = EnergyVad::new(0.01);
        let silence = vec![0.0f32; 480];
        let result = vad.process_frame(&silence);
        assert!(!result, "single silent frame should not indicate speech");
        assert!(!vad.is_speech, "vad.is_speech should remain false");
    }

    #[test]
    fn test_many_silent_frames_never_trigger_speech() {
        let mut vad = EnergyVad::new(0.01);
        let silence = vec![0.0f32; 480];
        for _ in 0..100 {
            assert!(
                !vad.process_frame(&silence),
                "silent frames should never trigger speech"
            );
        }
    }

    // --- process_frame: Speech Detection ---

    #[test]
    fn test_speech_detected() {
        let mut vad = EnergyVad::new(0.01);
        // Simulate speech with higher energy
        let speech: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        // Need min_speech_frames (2) consecutive
        vad.process_frame(&speech);
        assert!(vad.process_frame(&speech));
    }

    #[test]
    fn test_speech_requires_min_consecutive_frames() {
        let mut vad = EnergyVad::new(0.01);
        let speech: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();

        // Frame 1: not yet speech
        assert!(
            !vad.process_frame(&speech),
            "1 frame should not trigger speech"
        );
        // Frame 2: now speech is detected (min_speech_frames = 2)
        assert!(vad.process_frame(&speech), "2 frames should trigger speech");
    }

    #[test]
    fn test_speech_persists_after_detection() {
        let mut vad = EnergyVad::new(0.01);
        let speech: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();

        // Trigger speech
        for _ in 0..2 {
            vad.process_frame(&speech);
        }
        assert!(vad.is_speech);

        // More speech frames should keep it true
        assert!(vad.process_frame(&speech));
        assert!(vad.process_frame(&speech));
    }

    // --- process_frame: Speech-to-Silence Transition ---

    #[test]
    fn test_silence_requires_min_consecutive_frames_to_end_speech() {
        let mut vad = EnergyVad::new(0.01);
        let speech: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let silence = vec![0.0f32; 480];

        // Trigger speech
        for _ in 0..2 {
            vad.process_frame(&speech);
        }
        assert!(vad.is_speech);

        // 9 silent frames should NOT end speech (min_silence_frames = 10)
        for _ in 0..9 {
            let result = vad.process_frame(&silence);
            assert!(result, "speech should persist during silence countdown");
        }

        // 10th silent frame ends speech
        let result = vad.process_frame(&silence);
        assert!(
            !result,
            "speech should end after 10 consecutive silent frames"
        );
    }

    #[test]
    fn test_speech_frame_resets_silence_counter() {
        let mut vad = EnergyVad::new(0.01);
        let speech: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let silence = vec![0.0f32; 480];

        // Trigger speech
        for _ in 0..2 {
            vad.process_frame(&speech);
        }

        // 5 silent frames
        for _ in 0..5 {
            vad.process_frame(&silence);
        }
        assert!(
            vad.is_speech,
            "should still be in speech state after only 5 silent frames"
        );

        // 1 speech frame resets the silence counter
        vad.process_frame(&speech);
        assert_eq!(
            vad.silence_frame_count, 0,
            "silence count should be reset by speech"
        );

        // Another 5 silent frames (total 5, not 10)
        for _ in 0..5 {
            vad.process_frame(&silence);
        }
        assert!(
            vad.is_speech,
            "should still be in speech after reset + 5 silent frames"
        );
    }

    // --- process_frame: Threshold Boundary ---

    #[test]
    fn test_energy_below_threshold_is_silence() {
        // Energy below threshold is silence (the condition is energy > threshold)
        let threshold = 0.05f32;
        let mut vad = EnergyVad::new(threshold);

        // Create a frame where all samples are well below threshold
        let value = threshold * 0.5; // RMS will be half the threshold
        let frame = vec![value; 480];
        let energy = EnergyVad::rms_energy(&frame);
        assert!(
            energy < threshold,
            "energy {} should be below threshold {}",
            energy,
            threshold
        );

        // This should count as silence, speech should never trigger
        for _ in 0..10 {
            assert!(
                !vad.process_frame(&frame),
                "energy below threshold should be treated as silence"
            );
        }
    }

    #[test]
    fn test_energy_just_above_threshold_is_speech() {
        let threshold = 0.01f32;
        let mut vad = EnergyVad::new(threshold);
        let value = threshold + 0.001;
        let frame = vec![value; 480];

        // Should trigger speech after min_speech_frames (2)
        vad.process_frame(&frame);
        assert!(
            vad.process_frame(&frame),
            "energy above threshold should be detected as speech"
        );
    }

    // --- contains_speech Tests ---

    #[test]
    fn test_contains_speech_with_silence() {
        let mut vad = EnergyVad::new(0.01);
        let silence = vec![0.0f32; 4800]; // 300ms at 16kHz = 10 frames
        assert!(
            !vad.contains_speech(&silence),
            "silence should not be detected as speech"
        );
    }

    #[test]
    fn test_contains_speech_with_speech() {
        let mut vad = EnergyVad::new(0.01);
        // Generate a 480ms signal (16 frames) of loud audio
        let speech: Vec<f32> = (0..7680).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        assert!(
            vad.contains_speech(&speech),
            "loud audio should be detected as speech"
        );
    }

    #[test]
    fn test_contains_speech_partial_speech_detected() {
        let mut vad = EnergyVad::new(0.01);
        // Build audio: 5 frames of silence, then 5 frames of speech
        let mut audio = vec![0.0f32; 480 * 5]; // 5 silent frames
        let speech_frames: Vec<f32> = (0..(480 * 5))
            .map(|i| (i as f32 * 0.1).sin() * 0.5)
            .collect();
        audio.extend_from_slice(&speech_frames);

        // contains_speech returns true if any frame triggered speech
        assert!(
            vad.contains_speech(&audio),
            "audio with some speech should be detected"
        );
    }

    #[test]
    fn test_contains_speech_non_multiple_frame_size() {
        let mut vad = EnergyVad::new(0.01);
        // Audio that does not divide evenly into 480-sample frames
        let speech: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        // This should not panic. chunks() handles the remainder.
        let _ = vad.contains_speech(&speech);
    }

    // --- Threshold Variation ---

    #[test]
    fn test_high_threshold_ignores_quiet_speech() {
        let mut vad = EnergyVad::new(0.5); // Very high threshold
        let quiet: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.1).collect();
        for _ in 0..10 {
            assert!(
                !vad.process_frame(&quiet),
                "quiet signal should not exceed high threshold"
            );
        }
    }

    #[test]
    fn test_zero_threshold_detects_any_nonzero_signal() {
        let mut vad = EnergyVad::new(0.0);
        let tiny = vec![0.0001f32; 480];
        vad.process_frame(&tiny);
        vad.process_frame(&tiny);
        assert!(
            vad.process_frame(&tiny),
            "any nonzero signal should trigger speech with threshold 0"
        );
    }

    // --- VadBackend Tests ---

    #[test]
    fn test_vad_backend_default_is_silero() {
        assert_eq!(VadBackend::default(), VadBackend::Silero);
    }

    #[test]
    fn test_vad_backend_serialization() {
        let energy = serde_json::to_string(&VadBackend::Energy).unwrap();
        assert_eq!(energy, "\"energy\"");
        let silero = serde_json::to_string(&VadBackend::Silero).unwrap();
        assert_eq!(silero, "\"silero\"");
    }

    #[test]
    fn test_vad_backend_deserialization() {
        let energy: VadBackend = serde_json::from_str("\"energy\"").unwrap();
        assert_eq!(energy, VadBackend::Energy);
        let silero: VadBackend = serde_json::from_str("\"silero\"").unwrap();
        assert_eq!(silero, VadBackend::Silero);
    }

    #[test]
    fn test_vad_backend_invalid_deserialization_fails() {
        let result: Result<VadBackend, _> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_vad_backend_equality() {
        assert_eq!(VadBackend::Energy, VadBackend::Energy);
        assert_eq!(VadBackend::Silero, VadBackend::Silero);
        assert_ne!(VadBackend::Energy, VadBackend::Silero);
    }

    // --- SileroVad Tests ---

    #[test]
    fn test_silero_vad_loads_successfully() {
        let result = SileroVad::new(0.5);
        assert!(
            result.is_ok(),
            "Silero VAD model should load: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_silero_vad_silence_not_detected_as_speech() {
        let mut vad = SileroVad::new(0.5).unwrap();
        let silence = vec![0.0f32; 16000]; // 1 second of silence
        let result = vad.process_audio(&silence);
        assert!(!result, "silence should not be detected as speech");
    }

    #[test]
    fn test_silero_vad_loud_signal_pipeline_works() {
        // lower threshold for test reliability
        let mut vad = SileroVad::new(0.3).unwrap();
        // Generate 2 seconds of 440Hz sine wave (simulates speech-like energy)
        let speech: Vec<f32> = (0..32000)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16000.0).sin() * 0.5)
            .collect();
        let result = vad.process_audio(&speech);
        // Note: a pure sine wave may or may not trigger Silero (it's trained on speech, not tones).
        // This test validates the pipeline works without crashing. The actual speech detection
        // accuracy should be validated with real speech recordings.
        let _ = result;
    }

    #[test]
    fn test_silero_vad_reset_clears_state() {
        let mut vad = SileroVad::new(0.5).unwrap();
        let audio = vec![0.5f32; 4096]; // some audio
        vad.process_audio(&audio);
        vad.reset();
        assert!(!vad.is_speech(), "speech state should be false after reset");
        assert!(
            vad.frame_buffer.is_empty(),
            "frame buffer should be empty after reset"
        );
    }

    #[test]
    fn test_silero_vad_accumulates_partial_frames() {
        let mut vad = SileroVad::new(0.5).unwrap();
        // Feed 300 samples — less than one 512-sample frame
        let partial = vec![0.0f32; 300];
        vad.process_audio(&partial);
        assert_eq!(
            vad.frame_buffer.len(),
            300,
            "partial samples should accumulate"
        );

        // Feed 300 more — now 600 total, should process one frame (512) and leave 88
        vad.process_audio(&partial);
        assert_eq!(
            vad.frame_buffer.len(),
            88,
            "should have 600-512=88 remaining"
        );
    }

    #[test]
    fn test_silero_vad_processes_exact_frame() {
        let mut vad = SileroVad::new(0.5).unwrap();
        let frame = vec![0.0f32; 512];
        vad.process_audio(&frame);
        assert_eq!(
            vad.frame_buffer.len(),
            0,
            "exact frame should leave no remainder"
        );
    }

    #[test]
    fn test_silero_vad_processes_multiple_frames() {
        let mut vad = SileroVad::new(0.5).unwrap();
        // Feed 1600 samples (typical DSP cycle output) = 3 frames + 64 remainder
        let audio = vec![0.0f32; 1600];
        vad.process_audio(&audio);
        assert_eq!(
            vad.frame_buffer.len(),
            1600 - (512 * 3),
            "should process 3 frames, leave 64"
        );
    }

    #[test]
    fn test_silero_vad_hysteresis_prevents_flicker() {
        let mut vad = SileroVad::new(0.5).unwrap();
        // Feed silence — should not trigger
        let silence = vec![0.0f32; 8192]; // 16 frames of silence
        vad.process_audio(&silence);
        assert!(!vad.is_speech(), "silence should not trigger speech");
        assert_eq!(vad.speech_count, 0);
    }
}
