//! Convert captured audio to Whisper's 16 kHz mono f32 layout.

pub const WHISPER_RATE: u32 = 16_000;

/// Downmix to mono, then resample to 16 kHz with linear interpolation.
pub fn to_whisper_mono16k(samples: &[f32], sample_rate: u32, channels: u16) -> Vec<f32> {
    if samples.is_empty() || sample_rate == 0 {
        return Vec::new();
    }
    let channels = channels.max(1) as usize;
    let mono = downmix_mono(samples, channels);
    if sample_rate == WHISPER_RATE {
        return mono;
    }
    resample_linear(&mono, sample_rate, WHISPER_RATE)
}

fn downmix_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    if channels == 1 {
        return samples.to_vec();
    }
    let frames = samples.len() / channels;
    let mut out = Vec::with_capacity(frames);
    for frame in 0..frames {
        let mut sum = 0.0;
        for ch in 0..channels {
            sum += samples[frame * channels + ch];
        }
        out.push(sum / channels as f32);
    }
    out
}

fn resample_linear(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if input.is_empty() || from == 0 || to == 0 {
        return Vec::new();
    }
    if from == to {
        return input.to_vec();
    }
    let out_len = ((input.len() as u64) * u64::from(to) / u64::from(from)).max(1) as usize;
    let mut out = vec![0.0; out_len];
    let ratio = f64::from(from) / f64::from(to);
    let last = (input.len() - 1) as f64;
    for (i, slot) in out.iter_mut().enumerate() {
        let src = (i as f64 * ratio).min(last);
        let i0 = src.floor() as usize;
        let i1 = (i0 + 1).min(input.len() - 1);
        let t = (src - i0 as f64) as f32;
        *slot = input[i0] * (1.0 - t) + input[i1] * t;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_downmix_averages_channels() {
        let samples = [1.0, -1.0, 0.5, 0.5];
        let mono = to_whisper_mono16k(&samples, WHISPER_RATE, 2);
        assert_eq!(mono, vec![0.0, 0.5]);
    }

    #[test]
    fn same_rate_mono_is_identity() {
        let samples = [0.1, 0.2, 0.3];
        assert_eq!(to_whisper_mono16k(&samples, WHISPER_RATE, 1), samples);
    }

    #[test]
    fn downsample_48k_to_16k_has_expected_length() {
        let input: Vec<f32> = (0..4800).map(|i| (i as f32) / 4800.0).collect();
        let out = to_whisper_mono16k(&input, 48_000, 1);
        assert!((out.len() as i32 - 1600).abs() <= 1);
    }
}
