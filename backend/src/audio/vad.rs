pub struct VoiceActivityDetector {
    threshold: f32,
    min_speech_frames: usize,
    min_silence_frames: usize,
    speech_frame_count: usize,
    silence_frame_count: usize,
    is_speech: bool,
}

impl VoiceActivityDetector {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            min_speech_frames: 3, // Require 3 consecutive voiced frames to trigger
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_detected() {
        let mut vad = VoiceActivityDetector::new(0.01);
        let silence = vec![0.0f32; 480];
        assert!(!vad.process_frame(&silence));
    }

    #[test]
    fn test_speech_detected() {
        let mut vad = VoiceActivityDetector::new(0.01);
        // Simulate speech with higher energy
        let speech: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        // Need min_speech_frames consecutive
        vad.process_frame(&speech);
        vad.process_frame(&speech);
        assert!(vad.process_frame(&speech));
    }
}
