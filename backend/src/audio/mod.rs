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

            let mut buffer = AudioRingBuffer::new(16000, chunk_duration_ms, overlap_ms, 30);
            let mut vad = VoiceActivityDetector::new(vad_threshold);

            while running.load(Ordering::SeqCst) {
                match audio_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(samples) => {
                        buffer.write(&samples);

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
