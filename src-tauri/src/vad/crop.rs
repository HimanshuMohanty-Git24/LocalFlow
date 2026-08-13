//! Crop PCM to detected speech. Pure functions so tests need no model.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeechSpan {
    pub start_sample: usize,
    pub end_sample: usize,
}

/// whisper.cpp VAD timestamps are centiseconds (10 ms).
pub fn span_from_centiseconds(
    start_cs: f32,
    end_cs: f32,
    sample_rate: u32,
    len: usize,
) -> SpeechSpan {
    let to_sample = |cs: f32| {
        let s = ((cs as f64) / 100.0 * f64::from(sample_rate)).round() as i64;
        s.clamp(0, len as i64) as usize
    };
    let mut start = to_sample(start_cs);
    let mut end = to_sample(end_cs);
    if end < start {
        std::mem::swap(&mut start, &mut end);
    }
    if start == end && end < len {
        end += 1;
    }
    SpeechSpan {
        start_sample: start,
        end_sample: end.min(len),
    }
}

/// Keep audio from the first speech start through the last speech end.
pub fn crop_first_to_last(samples: &[f32], spans: &[SpeechSpan]) -> Vec<f32> {
    let Some(start) = spans.iter().map(|s| s.start_sample).min() else {
        return Vec::new();
    };
    let Some(end) = spans.iter().map(|s| s.end_sample).max() else {
        return Vec::new();
    };
    let end = end.min(samples.len());
    let start = start.min(end);
    samples[start..end].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centiseconds_at_16k() {
        let span = span_from_centiseconds(100.0, 200.0, 16_000, 100_000);
        assert_eq!(span.start_sample, 16_000);
        assert_eq!(span.end_sample, 32_000);
    }

    #[test]
    fn crop_uses_outer_bounds() {
        let samples: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let spans = [
            SpeechSpan {
                start_sample: 2,
                end_sample: 4,
            },
            SpeechSpan {
                start_sample: 6,
                end_sample: 8,
            },
        ];
        assert_eq!(crop_first_to_last(&samples, &spans), vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn empty_spans_yield_empty() {
        assert!(crop_first_to_last(&[1.0, 2.0], &[]).is_empty());
    }
}
