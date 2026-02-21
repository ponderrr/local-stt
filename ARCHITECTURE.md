# Audio Pipeline Architecture

The audio pipeline in WhisperType handles capturing microphone input securely and reliably, avoiding OS-level audio underflows or buffer dropouts, while continuously detecting user speech for the transcription engine.

## Threading Model

| Component | Thread Type | Lifecycle | Responsibility |
| :--- | :--- | :--- | :--- |
| **AudioActor** | Single-purpose (`cpal-actor`) | Spawned on dictation start. Terminated cleanly on stop via channel `Quit` command. | Captures raw PCM flow from the OS mic using `cpal`. Pushes samples directly to ring buffer. Zero locks or allocations. |
| **DSP Pipeline** | Single-purpose (`audio-dsp`) | Spawned on dictation start. Joins/terminates on `is_running = false`. | Drains the ring buffer. Converts audio to Mono. Resamples to 16kHz. Runs VAD. Dispatches speech chunks. |
| **Transcription** | Isolated Background | Spawned on dictation start. | Receives clean 16kHz Mono chunks. Feeds to `whisper-rs`. Dispatches STT results to Tauri UI. |

## Data Flow

Hardware (Microphone) -> `cpal-actor` -> lock-free `ringbuf` heap -> `audio-dsp` thread -> MPSC Channel -> Transcription Thread -> STT GPU Engine.

## Why the Old Architecture Was Replaced

The previous design relied on `unsafe impl Send/Sync` wrappers around `cpal::Stream` to juggle stream ownership across Tauri commands. Furthermore, heavy DSP workloads (multi-channel averaging and sample-rate decimation) were executed directly inside the real-time `cpal` audio callback loop. This caused tight loop violations, leading to XRUNs, input dropouts, and significant instability on Linux/PipeWire.

The new architecture decouples capture from processing:

1. The `cpal` callback now does highly-optimized `O(1)` memory copies into a lock-free `ringbuf`, eliminating OS audio dropouts.
2. A separate `audio-dsp` thread handles structural transformations safely.
3. Thread synchronization and stream ownership lifetimes are securely handled by cross-thread messaging (`mpsc`) rather than `unsafe` static coercion.
