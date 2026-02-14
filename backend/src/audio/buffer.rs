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
}
