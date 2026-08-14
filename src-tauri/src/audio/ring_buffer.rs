//! Preallocated f32 ring. The audio callback must only push; it must not allocate.

/// Fixed-capacity ring buffer of `f32` samples.
#[derive(Debug)]
pub struct RingBuffer {
    buf: Vec<f32>,
    cap: usize,
    write: usize,
    len: usize,
}

impl RingBuffer {
    /// Creates a zeroed buffer. `cap` of 0 is treated as 1 so push never panics.
    pub fn with_capacity(cap: usize) -> Self {
        let cap = cap.max(1);
        Self {
            buf: vec![0.0; cap],
            cap,
            write: 0,
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Writes samples. If the buffer is full, oldest samples are overwritten.
    /// Returns how many input samples were stored (always `src.len()` when cap > 0).
    pub fn push_slice(&mut self, src: &[f32]) -> usize {
        for &sample in src {
            self.buf[self.write] = sample;
            self.write = (self.write + 1) % self.cap;
            if self.len < self.cap {
                self.len += 1;
            }
        }
        src.len()
    }

    /// Copies samples in chronological order into `out`. Returns the count written.
    pub fn snapshot(&self, out: &mut [f32]) -> usize {
        let n = self.len.min(out.len());
        if n == 0 {
            return 0;
        }
        let start = if self.len == self.cap { self.write } else { 0 };
        for (i, item) in out.iter_mut().enumerate().take(n) {
            *item = self.buf[(start + i) % self.cap];
        }
        n
    }

    /// Allocates a chronological copy. Call from a worker thread, never the audio callback.
    pub fn to_vec(&self) -> Vec<f32> {
        let mut out = vec![0.0; self.len];
        self.snapshot(&mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_then_snapshot_preserves_order() {
        let mut rb = RingBuffer::with_capacity(8);
        rb.push_slice(&[1.0, 2.0, 3.0]);
        assert_eq!(rb.to_vec(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn overwrite_keeps_newest() {
        let mut rb = RingBuffer::with_capacity(4);
        rb.push_slice(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(rb.len(), 4);
        assert_eq!(rb.to_vec(), vec![3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn wrap_then_partial_snapshot() {
        let mut rb = RingBuffer::with_capacity(3);
        rb.push_slice(&[1.0, 2.0, 3.0, 4.0]);
        let mut out = [0.0; 2];
        assert_eq!(rb.snapshot(&mut out), 2);
        assert_eq!(out, [2.0, 3.0]);
    }
}
