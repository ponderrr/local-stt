# WhisperType Algorithms

This document explains the core algorithms used in WhisperType's audio processing
pipeline, from microphone capture through text output. Each section provides the
mathematical basis, implementation details, and relevant configuration parameters.

See also: [ARCHITECTURE.md](./ARCHITECTURE.md) for system-level context,
[API-REFERENCE.md](./API-REFERENCE.md) for the complete typed interface.

## Table of Contents

- [Audio Capture and Format Conversion](#audio-capture-and-format-conversion)
- [Ring Buffer and Chunk Extraction](#ring-buffer-and-chunk-extraction)
- [Voice Activity Detection](#voice-activity-detection)
- [Whisper Inference Pipeline](#whisper-inference-pipeline)
- [Text Output](#text-output)

---

## Audio Capture and Format Conversion

### Device Capture

Audio capture is handled by `cpal` 0.15 in `backend/src/audio/capture.rs`. The
`AudioCapture` struct opens an input stream on the default (or user-specified)
device, reading its native configuration:

```rust
let supported_config = device
    .default_input_config()
    .map_err(|e| format!("Failed to get default input config: {}", e))?;

let config: StreamConfig = supported_config.into();
self.device_sample_rate = config.sample_rate.0;  // e.g. 48000
self.device_channels = config.channels;           // e.g. 2
```

The input callback pushes raw `f32` samples into a lock-free ring buffer:

```rust
move |data: &[f32], _: &cpal::InputCallbackInfo| {
    producer.push_slice(data);
}
```

The ring buffer (`ringbuf::HeapRb<f32>`) has a capacity of `48000 * 2 * 3 = 288,000`
samples, which is approximately 3 seconds of 48kHz stereo audio. This provides a
generous buffer between the audio callback thread and the processing thread.

### Stereo-to-Mono Conversion

Multi-channel audio must be converted to mono before resampling and transcription.
The `to_mono()` function in `backend/src/audio/mod.rs` averages interleaved channel
samples:

```rust
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
```

**Algorithm**:

For interleaved audio with `C` channels, each frame contains `C` consecutive
samples `[s_0, s_1, ..., s_{C-1}]`. The mono output sample for each frame is:

```
mono_i = (1/C) * sum(s_j for j in 0..C)
```

- `chunks_exact(ch)` splits the flat sample array into non-overlapping frames
- Any remainder samples (when `len % channels != 0`) are silently dropped
- Single-channel input is returned as-is (zero-cost path)

**Example**: Stereo input `[L0, R0, L1, R1]` produces mono `[(L0+R0)/2, (L1+R1)/2]`.

### Sample Rate Conversion

Whisper requires 16kHz mono audio. Devices typically capture at 44.1kHz or 48kHz.
The `resample()` function in `backend/src/audio/mod.rs` performs linear interpolation:

```rust
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
```

**Algorithm**:

Given an input signal `x[n]` at sample rate `fs_src` and desired output `y[m]` at
sample rate `fs_dst`:

1. Compute the rate ratio: `R = fs_src / fs_dst`
2. Compute output length: `M = floor(N / R)` where `N` is the input length
3. For each output sample index `m`:
   - Compute the corresponding source position: `p = m * R`
   - Decompose into integer index `k = floor(p)` and fractional part `f = p - k`
   - Interpolate: `y[m] = x[k] * (1 - f) + x[k+1] * f`

**Properties**:
- Linear interpolation preserves constant (DC) signals exactly
- Signals well below the Nyquist frequency of the target rate are faithfully
  reproduced (verified by unit tests with 100Hz sine at 48kHz -> 16kHz)
- Output values remain bounded within the input range (no overshoot)
- Boundary handling: `x[k+1]` is clamped to the last sample when `k+1` exceeds
  the input length

**Common conversions**:

| Source Rate | Target Rate | Ratio | 100ms input | 100ms output |
|-------------|-------------|-------|-------------|--------------|
| 48,000 Hz   | 16,000 Hz  | 3.0   | 4,800       | 1,600        |
| 44,100 Hz   | 16,000 Hz  | 2.756 | 4,410       | 1,600        |
| 16,000 Hz   | 16,000 Hz  | 1.0   | passthrough | passthrough  |

### Processing Loop

The pipeline processing thread in `AudioPipeline::start()` runs a loop that
reads from the ring buffer, converts, buffers, and dispatches:

```rust
let mut read_buf = vec![0.0f32; 4800]; // 100ms at 48kHz
while running.load(Ordering::SeqCst) {
    let n = consumer.pop_slice(&mut read_buf);
    if n > 0 {
        let mono = to_mono(&read_buf[..n], device_channels);
        let resampled = resample(&mono, device_rate, 16000);
        buffer.write(&resampled);

        if buffer.has_chunk() {
            if let Some(chunk) = buffer.extract_chunk() {
                if vad.contains_speech(&chunk) {
                    if chunk_tx.send(chunk).is_err() {
                        break;
                    }
                }
            }
        }
    } else {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
```

When no data is available, the thread sleeps for 10ms to avoid busy-waiting.

---

## Ring Buffer and Chunk Extraction

### AudioRingBuffer Structure

The `AudioRingBuffer` in `backend/src/audio/buffer.rs` is a fixed-capacity circular
buffer that accumulates resampled 16kHz mono samples and extracts overlapping chunks:

```rust
pub struct AudioRingBuffer {
    data: Vec<f32>,
    write_pos: usize,
    capacity: usize,           // Total samples the buffer holds
    chunk_size: usize,         // Samples per transcription chunk
    overlap_size: usize,       // Overlap between consecutive chunks
    samples_since_last: usize, // Samples written since last chunk extraction
}
```

### Size Calculations

Given the default configuration parameters:

| Parameter            | Value   | Calculation                          |
|----------------------|---------|--------------------------------------|
| `sample_rate`        | 16,000  | Whisper's expected input rate        |
| `chunk_duration_ms`  | 3,000   | 3-second transcription windows       |
| `overlap_ms`         | 500     | 500ms overlap between chunks         |
| `buffer_duration_s`  | 30      | Total ring buffer capacity           |
| `chunk_size`         | 48,000  | `16000 * 3000 / 1000`               |
| `overlap_size`       | 8,000   | `16000 * 500 / 1000`                |
| `capacity`           | 480,000 | `16000 * 30`                         |

### Write Operation

Samples are written one at a time with modular indexing for wrap-around:

```rust
pub fn write(&mut self, samples: &[f32]) {
    for &sample in samples {
        self.data[self.write_pos % self.capacity] = sample;
        self.write_pos += 1;
        self.samples_since_last += 1;
    }
}
```

Note that `write_pos` increments monotonically (it does not wrap). Only the data
index wraps via `write_pos % capacity`. This allows the chunk extraction logic to
use simple subtraction to find the start of the latest chunk.

### Chunk Readiness

A new chunk is available when two conditions are met:

```rust
pub fn has_chunk(&self) -> bool {
    self.samples_since_last >= (self.chunk_size - self.overlap_size)
        && self.write_pos >= self.chunk_size
}
```

1. **Enough new data**: at least `chunk_size - overlap_size` new samples have been
   written since the last extraction (default: 48000 - 8000 = 40,000 samples = 2.5s)
2. **Enough total data**: at least `chunk_size` samples have been written in total
   (prevents extraction before the first full chunk)

### Chunk Extraction with Overlap

```rust
pub fn extract_chunk(&mut self) -> Option<Vec<f32>> {
    if !self.has_chunk() { return None; }

    let start = if self.write_pos >= self.chunk_size {
        self.write_pos - self.chunk_size
    } else {
        return None;
    };

    let mut chunk = Vec::with_capacity(self.chunk_size);
    for i in start..self.write_pos {
        chunk.push(self.data[i % self.capacity]);
    }

    self.samples_since_last = 0;
    Some(chunk)
}
```

The overlap works implicitly: after extracting a 3-second chunk, the next extraction
requires only 2.5 seconds of new data, but it reads the last 3 seconds of the
buffer. This means the first 500ms of the new chunk overlaps with the last 500ms
of the previous chunk.

```
Time ------>

Chunk 1:  [====================================]  (3.0s)
                                         |overlap|
Chunk 2:                  [====================================]  (3.0s)
                                                          |overlap|
Chunk 3:                                   [====================================]
          ^                ^              ^
          |                |              |
        t=0s             t=2.5s         t=5.0s
```

This overlap ensures that speech at chunk boundaries is not lost, since Whisper
has context from the end of the previous chunk.

---

## Voice Activity Detection

### RMS Energy Calculation

The `VoiceActivityDetector` in `backend/src/audio/vad.rs` uses Root Mean Square
(RMS) energy to detect speech:

```rust
fn rms_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}
```

**Formula**:

```
RMS = sqrt( (1/N) * sum(x_i^2 for i in 0..N) )
```

Where `N` is the number of samples in the frame and `x_i` are the sample values
(normalized `f32` in the range `[-1.0, 1.0]`).

**Properties**:
- Silence (all zeros) produces RMS = 0.0
- A constant signal of amplitude `A` produces RMS = `|A|`
- A sinusoidal signal of amplitude `A` produces RMS = `A / sqrt(2) ~= 0.707 * A`
- RMS is sign-invariant (positive and negative signals of equal magnitude produce
  the same energy)

### Frame-Level Detection

The VAD processes 30ms frames (480 samples at 16kHz) and uses a hysteresis
state machine:

```rust
pub struct VoiceActivityDetector {
    threshold: f32,           // RMS energy threshold (default: 0.01)
    min_speech_frames: usize, // 3 consecutive voiced frames to trigger
    min_silence_frames: usize,// 10 consecutive silent frames to end
    speech_frame_count: usize,
    silence_frame_count: usize,
    is_speech: bool,
}
```

**State transitions**:

```
                  energy > threshold
                  (3 consecutive frames)
    +--------+  ----------------------->  +---------+
    | SILENCE |                           | SPEECH  |
    +--------+  <-----------------------  +---------+
                  energy <= threshold
                  (10 consecutive frames)
```

```rust
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
```

**Key behaviors**:
- **Speech onset**: 3 consecutive frames with energy above threshold (3 * 30ms =
  90ms latency before detection)
- **Speech offset**: 10 consecutive frames with energy at or below threshold
  (10 * 30ms = 300ms hangover) -- this prevents premature cutoff during brief pauses
- **Counter reset**: a single contradicting frame resets the opposing counter
  (e.g., one speech frame during silence resets `silence_frame_count` to 0)

### Bulk Speech Check

The `contains_speech()` method processes an entire audio chunk in 480-sample frames:

```rust
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
```

This method returns `true` if any frame within the chunk triggered the speech state.
The VAD state persists across calls, so the hysteresis from previous chunks carries
forward.

### Threshold Tuning

The default VAD threshold of `0.01` is deliberately low to catch quiet speech. The
threshold is configurable via `Config.vad_threshold` and can be adjusted in the
settings.

| Threshold | Behavior                                               |
|-----------|--------------------------------------------------------|
| 0.0       | Detects any non-zero signal as speech                  |
| 0.01      | Default -- catches quiet speech, may include some noise|
| 0.05      | Moderate -- filters light background noise             |
| 0.5       | Very high -- only detects loud speech                  |

---

## Whisper Inference Pipeline

### Model Registry

WhisperType ships a registry of 5 official Whisper models in
`backend/src/transcription/models.rs`, stored as a `LazyLock<Vec<WhisperModel>>`:

| Model ID    | File                   | Size         | VRAM    |
|-------------|------------------------|--------------|---------|
| `tiny`      | `ggml-tiny.bin`        | ~75 MB       | ~1 GB   |
| `base`      | `ggml-base.bin`        | ~150 MB      | ~1 GB   |
| `small`     | `ggml-small.bin`       | ~500 MB      | ~1.5 GB |
| `medium`    | `ggml-medium.bin`      | ~1.5 GB      | ~3 GB   |
| `large-v3`  | `ggml-large-v3.bin`    | ~3 GB        | ~6 GB   |

Models are downloaded from HuggingFace (`huggingface.co/ggerganov/whisper.cpp`) and
stored in `~/.whispertype/models/`.

### Model Loading

The `TranscriptionEngine::load_model()` method in `backend/src/transcription/engine.rs`:

```rust
pub fn load_model(&self, model_path: &Path, model_id: &str) -> Result<(), String> {
    // Drop existing context first to free VRAM
    {
        let mut ctx = self.ctx.lock().map_err(|e| e.to_string())?;
        *ctx = None;
    }

    let mut params = WhisperContextParameters::default();
    params.use_gpu(true);

    let new_ctx = WhisperContext::new_with_params(
        model_path.to_str().ok_or("Invalid model path")?,
        params,
    )
    .map_err(|e| format!("Failed to load whisper model '{}': {}", model_id, e))?;

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
```

The existing context is explicitly dropped before the new one is created. This
ensures VRAM is freed before loading a new model, preventing out-of-memory errors
when switching between large models.

### Audio Preprocessing

Whisper expects 16kHz mono `f32` audio. By the time audio reaches the transcription
engine, it has already been:

1. Converted to mono (channel averaging)
2. Resampled to 16kHz (linear interpolation)
3. Chunked into ~3-second segments with 500ms overlap
4. Filtered by VAD (only speech chunks are sent)

No further preprocessing is applied -- the raw `f32` samples are passed directly
to `whisper_rs::WhisperState::full()`.

### Inference Parameters

```rust
let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

if language != "auto" {
    params.set_language(Some(language));
}

params.set_print_special(false);
params.set_print_progress(false);
params.set_print_realtime(false);
params.set_print_timestamps(false);
params.set_suppress_blank(true);
params.set_suppress_nst(true);
params.set_no_context(true);
```

| Parameter               | Value     | Purpose                                         |
|-------------------------|-----------|--------------------------------------------------|
| `SamplingStrategy`      | `Greedy`  | Greedy decoding, no beam search                  |
| `best_of`               | `1`       | Single candidate (fastest inference)             |
| `language`              | configurable | Auto-detect or fixed language code            |
| `suppress_blank`        | `true`    | Skip segments that decode to whitespace only     |
| `suppress_nst`          | `true`    | Suppress non-speech tokens (noise, music, etc.)  |
| `no_context`            | `true`    | Do not carry context between chunks              |
| `print_special/progress`| `false`   | Disable console logging from whisper.cpp         |

**Greedy decoding** was chosen over beam search for latency: with `best_of=1`, the
model produces a single token sequence without backtracking. This is significantly
faster for real-time dictation where low latency matters more than marginal accuracy
improvements.

**`no_context=true`** prevents Whisper from using the previous chunk's text as
context for the current chunk. While this slightly reduces coherence across chunk
boundaries, it prevents error accumulation (hallucinated text propagating forward)
and is appropriate for dictation use cases.

### Segment Extraction

After inference, segments are extracted from the Whisper state:

```rust
let num_segments = state.full_n_segments();

let mut segments = Vec::new();
for i in 0..num_segments {
    let segment = state
        .get_segment(i)
        .ok_or_else(|| format!("Segment {} out of bounds", i))?;

    let text = segment
        .to_str()
        .map_err(|e| format!("Failed to get segment text: {}", e))?;

    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        continue;
    }

    let start = segment.start_timestamp();
    let end = segment.end_timestamp();

    segments.push(TranscriptionSegment {
        text: trimmed,
        start,
        end,
    });
}
```

Empty segments are filtered out. Each `TranscriptionSegment` contains:
- `text`: the transcribed text (trimmed whitespace)
- `start`: start timestamp in Whisper time units
- `end`: end timestamp in Whisper time units

---

## Text Output

### Output Modes

The output dispatcher in `backend/src/output/mod.rs`:

```rust
pub fn output_text(text: &str, mode: &OutputMode) -> Result<(), String> {
    match mode {
        OutputMode::TypeIntoField => keyboard::type_text(text),
        OutputMode::Clipboard => clipboard::copy_to_clipboard(text),
        OutputMode::Both => {
            keyboard::type_text(text)?;
            clipboard::copy_to_clipboard(text)?;
            Ok(())
        }
    }
}
```

### Keyboard Simulation

`backend/src/output/keyboard.rs` uses `enigo` 0.6 with `x11rb` and `wayland`
feature flags:

```rust
pub fn type_text(text: &str) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to init keyboard simulator: {}", e))?;

    enigo
        .text(text)
        .map_err(|e| format!("Failed to type text: {}", e))?;

    Ok(())
}
```

`enigo::Enigo::text()` sends the text as a sequence of key events to the currently
focused window. On Linux, this works via either X11 (XTest extension) or the
Wayland virtual keyboard protocol, depending on the session type.

A new `Enigo` instance is created per call to avoid holding state between
transcription segments.

### Clipboard Copy

`backend/src/output/clipboard.rs` uses `arboard` 3:

```rust
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;

    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    Ok(())
}
```

This places the transcribed text on the system clipboard, available for the user
to paste with Ctrl+V.

### Error Handling

Output errors are non-fatal to the dictation pipeline. If `output_text()` fails,
the error is:
1. Printed to stderr
2. Emitted as an `output-error` event to the frontend
3. The transcription loop continues processing the next chunk

The frontend displays output errors in a red banner that auto-dismisses after 5
seconds (see `useDictation` hook).
