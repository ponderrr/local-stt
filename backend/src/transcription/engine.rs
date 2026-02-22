//! Whisper inference engine wrapping whisper-rs. Manages model loading/unloading
//! and runs greedy transcription on 16kHz f32 audio chunks.

use std::path::Path;
use std::sync::Mutex;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

pub struct TranscriptionEngine {
    ctx: Mutex<Option<WhisperContext>>,
    active_model: Mutex<Option<String>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start: i64,
    pub end: i64,
}

impl Default for TranscriptionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptionEngine {
    pub fn new() -> Self {
        Self {
            ctx: Mutex::new(None),
            active_model: Mutex::new(None),
        }
    }

    pub fn load_model(&self, model_path: &Path, model_id: &str) -> Result<(), String> {
        // Drop existing context first to free VRAM
        {
            let mut ctx = self.ctx.lock().map_err(|e| e.to_string())?;
            *ctx = None;
        }

        let path_str = model_path.to_str().ok_or("Invalid model path")?;

        // Try with flash attention enabled first, fall back if it fails
        let new_ctx = {
            let mut params = WhisperContextParameters::default();
            params.use_gpu(true);
            params.flash_attn(true);

            match WhisperContext::new_with_params(path_str, params) {
                Ok(ctx) => {
                    eprintln!("whisper: context created with flash_attn=true");
                    ctx
                }
                Err(e) => {
                    eprintln!(
                        "whisper: flash_attn failed ({}), falling back to non-flash",
                        e
                    );
                    let mut fallback_params = WhisperContextParameters::default();
                    fallback_params.use_gpu(true);

                    let ctx = WhisperContext::new_with_params(path_str, fallback_params)
                        .map_err(|e| {
                            format!("Failed to load whisper model '{}': {}", model_id, e)
                        })?;
                    eprintln!("whisper: context created with flash_attn=false (fallback)");
                    ctx
                }
            }
        };

        {
            let mut ctx = self.ctx.lock().map_err(|e| e.to_string())?;
            *ctx = Some(new_ctx);
        }
        {
            let mut active = self.active_model.lock().map_err(|e| e.to_string())?;
            *active = Some(model_id.to_string());
        }
        Ok(())
    }

    /// Unload the model and free VRAM. SAFETY: The transcription thread must be
    /// joined before calling this — any live WhisperState holds a C pointer into
    /// the context's model weights. See `join_transcription_thread()` in dictation.rs.
    pub fn unload_model(&self) -> Result<(), String> {
        let mut ctx = self.ctx.lock().map_err(|e| e.to_string())?;
        *ctx = None;
        let mut active = self.active_model.lock().map_err(|e| e.to_string())?;
        *active = None;
        Ok(())
    }

    pub fn get_active_model(&self) -> Option<String> {
        self.active_model.lock().ok().and_then(|m| m.clone())
    }

    pub fn is_loaded(&self) -> bool {
        self.ctx.lock().map(|c| c.is_some()).unwrap_or(false)
    }

    /// Create a WhisperState from the loaded context. Call once at the start of a
    /// dictation session, then reuse the state for every chunk via `transcribe()`.
    ///
    /// SAFETY INVARIANT: The returned WhisperState holds a C-level pointer into the
    /// WhisperContext's model weights. The state must be dropped before `unload_model()`
    /// is called. The current architecture enforces this: `join_transcription_thread()`
    /// joins the thread (dropping the state) before any model operation can proceed.
    pub fn create_inference_state(&self) -> Result<WhisperState, String> {
        let ctx_guard = self.ctx.lock().map_err(|e| e.to_string())?;
        let ctx = ctx_guard
            .as_ref()
            .ok_or("No model loaded. Load a model before creating state.")?;
        ctx.create_state()
            .map_err(|e| format!("Failed to create whisper state: {}", e))
    }

    pub fn transcribe(
        &self,
        state: &mut WhisperState,
        audio_data: &[f32],
        language: &str,
    ) -> Result<Vec<TranscriptionSegment>, String> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        if language != "auto" {
            params.set_language(Some(language));
        }

        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_token_timestamps(false);
        params.set_single_segment(true);
        params.set_no_timestamps(true);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);
        params.set_no_context(true);

        #[cfg(debug_assertions)]
        let start = std::time::Instant::now();

        state
            .full(params, audio_data)
            .map_err(|e| format!("Transcription failed: {}", e))?;

        #[cfg(debug_assertions)]
        eprintln!(
            "PERF: whisper transcribe took {}ms ({} samples, {:.1}s audio)",
            start.elapsed().as_millis(),
            audio_data.len(),
            audio_data.len() as f64 / 16000.0
        );

        let num_segments = state.full_n_segments();

        let mut segments = Vec::new();
        for i in 0..num_segments {
            let segment = state
                .get_segment(i)
                .ok_or_else(|| format!("Segment {} out of bounds", i))?;

            let text = segment
                .to_str()
                .map_err(|e| format!("Failed to get segment text: {}", e))?;

            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }

            let start = segment.start_timestamp();
            let end = segment.end_timestamp();

            segments.push(TranscriptionSegment {
                text: trimmed.to_string(),
                start,
                end,
            });
        }

        Ok(segments)
    }
}
