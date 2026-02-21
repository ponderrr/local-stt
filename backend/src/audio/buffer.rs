//! Fixed-size ring buffer for accumulating 16kHz mono audio. Chunks are extracted
//! with configurable overlap to provide context across transcription boundaries.

/// A ring buffer specifically designed for collecting continuous mono audio samples.
///
/// Accumulates incoming audio over time and extracts discrete overlapping "chunks"
/// formatted for the STT engine. The overlap ensures context across chunk boundaries
/// is preserved.
pub struct AudioRingBuffer {
    data: Vec<f32>,
    write_pos: usize,
    capacity: usize,           // Total samples the buffer holds
    chunk_size: usize,         // Samples per transcription chunk
    overlap_size: usize,       // Overlap between consecutive chunks
    samples_since_last: usize, // Samples written since last chunk extraction
}

impl AudioRingBuffer {
    /// Create a new ring buffer.
    /// - sample_rate: e.g., 16000
    /// - chunk_duration_ms: e.g., 3000 (3 seconds)
    /// - overlap_ms: e.g., 500
    /// - buffer_duration_s: total buffer capacity in seconds (e.g., 30)
    pub fn new(
        sample_rate: u32,
        chunk_duration_ms: u32,
        overlap_ms: u32,
        buffer_duration_s: u32,
    ) -> Self {
        let chunk_size = (sample_rate * chunk_duration_ms / 1000) as usize;
        let overlap_size = (sample_rate * overlap_ms / 1000) as usize;
        let capacity = (sample_rate * buffer_duration_s) as usize;

        Self {
            data: vec![0.0; capacity],
            write_pos: 0,
            capacity,
            chunk_size,
            overlap_size,
            samples_since_last: 0,
        }
    }

    /// Write audio samples into the buffer.
    pub fn write(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.data[self.write_pos % self.capacity] = sample;
            self.write_pos += 1;
            self.samples_since_last += 1;
        }
    }

    /// Check if enough samples have accumulated for a new chunk.
    pub fn has_chunk(&self) -> bool {
        self.samples_since_last >= (self.chunk_size - self.overlap_size)
            && self.write_pos >= self.chunk_size
    }

    /// Extract the latest chunk (with overlap from previous chunk).
    pub fn extract_chunk(&mut self) -> Option<Vec<f32>> {
        if !self.has_chunk() {
            return None;
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Construction and Initialization ---

    #[test]
    fn test_new_buffer_computes_sizes_correctly() {
        // 16kHz, 3s chunk, 500ms overlap, 30s capacity
        let buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        assert_eq!(
            buf.chunk_size, 48000,
            "chunk_size should be 16000 * 3000 / 1000"
        );
        assert_eq!(
            buf.overlap_size, 8000,
            "overlap_size should be 16000 * 500 / 1000"
        );
        assert_eq!(buf.capacity, 480000, "capacity should be 16000 * 30");
        assert_eq!(buf.write_pos, 0);
        assert_eq!(buf.samples_since_last, 0);
    }

    #[test]
    fn test_new_buffer_data_initialized_to_zero() {
        let buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        assert!(
            buf.data.iter().all(|&s| s == 0.0),
            "buffer data should be initialized to 0.0"
        );
    }

    // --- Write Tests ---

    #[test]
    fn test_write_advances_write_pos() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        let samples = vec![1.0f32; 100];
        buf.write(&samples);
        assert_eq!(buf.write_pos, 100);
        assert_eq!(buf.samples_since_last, 100);
    }

    #[test]
    fn test_write_stores_samples_correctly() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        let samples: Vec<f32> = (0..10).map(|i| i as f32 * 0.1).collect();
        buf.write(&samples);
        for i in 0..10 {
            assert!(
                (buf.data[i] - i as f32 * 0.1).abs() < 1e-6,
                "sample at index {} should match written value",
                i
            );
        }
    }

    #[test]
    fn test_write_empty_slice_is_noop() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        buf.write(&[]);
        assert_eq!(buf.write_pos, 0);
        assert_eq!(buf.samples_since_last, 0);
    }

    #[test]
    fn test_write_wraps_around_capacity() {
        // Small buffer: 1s capacity at 16kHz = 16000 samples
        let mut buf = AudioRingBuffer::new(16000, 500, 100, 1);
        assert_eq!(buf.capacity, 16000);

        // Write more than capacity to trigger wrap-around
        let samples: Vec<f32> = (0..20000).map(|i| (i % 1000) as f32).collect();
        buf.write(&samples);

        // write_pos should be 20000 (not wrapped, only data index wraps)
        assert_eq!(buf.write_pos, 20000);

        // The last 16000 samples should be in the buffer (wrapped)
        // Index 0 should hold sample at position 16000 (which is (16000 % 1000) = 0.0)
        // Index 3999 should hold sample at position 19999 (which is (19999 % 1000) = 999.0)
        let last_sample_idx = (20000 - 1) % 16000;
        assert!(
            (buf.data[last_sample_idx] - 999.0).abs() < 1e-6,
            "wrapped data should contain correct sample"
        );
    }

    #[test]
    fn test_write_multiple_small_batches() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        for _ in 0..100 {
            buf.write(&[0.5f32; 480]); // 30ms at 16kHz
        }
        assert_eq!(buf.write_pos, 48000);
        assert_eq!(buf.samples_since_last, 48000);
    }

    // --- has_chunk Tests ---

    #[test]
    fn test_has_chunk_false_on_empty_buffer() {
        let buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        assert!(!buf.has_chunk(), "empty buffer should not have a chunk");
    }

    #[test]
    fn test_has_chunk_false_below_threshold() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        // Write less than chunk_size - overlap_size = 40000 samples
        buf.write(&vec![0.0f32; 39999]);
        assert!(
            !buf.has_chunk(),
            "buffer with insufficient samples should not have a chunk"
        );
    }

    #[test]
    fn test_has_chunk_true_after_enough_samples() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        // chunk_size = 48000, need at least 48000 total and 40000 since last
        buf.write(&vec![0.0f32; 48000]);
        assert!(
            buf.has_chunk(),
            "buffer with chunk_size samples should have a chunk"
        );
    }

    #[test]
    fn test_has_chunk_false_after_extraction() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        buf.write(&vec![0.0f32; 48000]);
        buf.extract_chunk();
        // After extraction, samples_since_last is reset to 0
        assert!(
            !buf.has_chunk(),
            "should not have chunk immediately after extraction"
        );
    }

    #[test]
    fn test_has_chunk_true_after_overlap_amount_written() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        buf.write(&vec![0.0f32; 48000]);
        buf.extract_chunk();
        // Now write chunk_size - overlap_size = 40000 more samples
        buf.write(&vec![0.0f32; 40000]);
        assert!(
            buf.has_chunk(),
            "should have chunk after writing chunk_size - overlap_size samples"
        );
    }

    // --- extract_chunk Tests ---

    #[test]
    fn test_extract_chunk_returns_none_when_no_chunk() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        assert!(
            buf.extract_chunk().is_none(),
            "should return None with no data"
        );
    }

    #[test]
    fn test_extract_chunk_returns_correct_size() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        buf.write(&vec![0.0f32; 48000]);
        let chunk = buf.extract_chunk().unwrap();
        assert_eq!(
            chunk.len(),
            48000,
            "chunk should be exactly chunk_size samples"
        );
    }

    #[test]
    fn test_extract_chunk_returns_correct_data() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        // Write 48000 samples with known values
        let samples: Vec<f32> = (0..48000).map(|i| (i as f32) / 48000.0).collect();
        buf.write(&samples);
        let chunk = buf.extract_chunk().unwrap();
        for (i, &val) in chunk.iter().enumerate() {
            assert!(
                (val - (i as f32) / 48000.0).abs() < 1e-6,
                "chunk sample {} should match written data",
                i
            );
        }
    }

    #[test]
    fn test_extract_chunk_resets_samples_since_last() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        buf.write(&vec![0.0f32; 48000]);
        buf.extract_chunk();
        assert_eq!(
            buf.samples_since_last, 0,
            "samples_since_last should be reset after extraction"
        );
    }

    #[test]
    fn test_extract_chunk_preserves_write_pos() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        buf.write(&vec![0.0f32; 48000]);
        let pos_before = buf.write_pos;
        buf.extract_chunk();
        assert_eq!(
            buf.write_pos, pos_before,
            "write_pos should not change after extraction"
        );
    }

    // --- Basic (original) Tests ---

    #[test]
    fn test_buffer_basic() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        // Write 3 seconds of silence
        let samples = vec![0.0f32; 48000];
        buf.write(&samples);
        assert!(buf.has_chunk());
        let chunk = buf.extract_chunk().unwrap();
        assert_eq!(chunk.len(), 48000); // 3s * 16kHz
    }

    #[test]
    fn test_buffer_overlap() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        // Write 3 seconds
        buf.write(&vec![0.0f32; 48000]);
        buf.extract_chunk();
        // Write 2.5 more seconds (chunk_size - overlap)
        buf.write(&vec![0.0f32; 40000]);
        assert!(buf.has_chunk());
    }

    // --- Overlap Content Tests ---

    #[test]
    fn test_overlap_chunk_contains_previous_data() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        // First chunk: 48000 samples of value 1.0
        buf.write(&vec![1.0f32; 48000]);
        buf.extract_chunk();

        // Second write: 40000 samples of value 2.0
        buf.write(&vec![2.0f32; 40000]);
        let chunk = buf.extract_chunk().unwrap();

        // The first 8000 samples of the chunk should be overlap (value 1.0)
        let overlap_samples = &chunk[..8000];
        assert!(
            overlap_samples.iter().all(|&s| (s - 1.0).abs() < 1e-6),
            "overlap region should contain data from previous chunk"
        );

        // The remaining 40000 samples should be new data (value 2.0)
        let new_samples = &chunk[8000..];
        assert!(
            new_samples.iter().all(|&s| (s - 2.0).abs() < 1e-6),
            "new region should contain freshly written data"
        );
    }

    // --- Multiple Consecutive Extractions ---

    #[test]
    fn test_multiple_extractions_work_correctly() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);

        // First chunk
        buf.write(&vec![0.0f32; 48000]);
        assert!(buf.extract_chunk().is_some());

        // Second chunk
        buf.write(&vec![0.0f32; 40000]);
        assert!(buf.extract_chunk().is_some());

        // Third chunk
        buf.write(&vec![0.0f32; 40000]);
        let chunk = buf.extract_chunk();
        assert!(chunk.is_some());
        assert_eq!(chunk.unwrap().len(), 48000);
    }

    // --- Edge Case: Exact Boundary ---

    #[test]
    fn test_has_chunk_at_exact_boundary() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        // Write exactly chunk_size - overlap_size = 40000 ... but total < chunk_size
        buf.write(&vec![0.0f32; 40000]);
        // samples_since_last = 40000 >= 40000 but write_pos = 40000 < 48000
        assert!(
            !buf.has_chunk(),
            "should not have chunk if write_pos < chunk_size"
        );

        // Write the remaining 8000 to reach chunk_size
        buf.write(&vec![0.0f32; 8000]);
        assert!(buf.has_chunk(), "should have chunk at exactly chunk_size");
    }

    // --- Single Sample Writes ---

    #[test]
    fn test_single_sample_writes_accumulate() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        for i in 0..48000 {
            buf.write(&[i as f32]);
        }
        assert!(buf.has_chunk());
        let chunk = buf.extract_chunk().unwrap();
        assert_eq!(chunk.len(), 48000);
        // Verify first and last sample
        assert!((chunk[0] - 0.0).abs() < 1e-6);
        assert!((chunk[47999] - 47999.0).abs() < 1e-6);
    }
}
