//! PCM WAV writer for microphone tests. Playback-friendly 16-bit little-endian.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::errors::AppError;

/// Writes interleaved `f32` samples (`-1.0..1.0`) as a 16-bit PCM WAV file.
pub fn write_pcm16_wav(
    path: &Path,
    sample_rate: u32,
    channels: u16,
    samples: &[f32],
) -> Result<(), AppError> {
    if sample_rate == 0 || channels == 0 {
        return Err(AppError::message("invalid wav format"));
    }

    let mut file = BufWriter::new(File::create(path).map_err(|e| AppError::message(e.to_string()))?);
    let frames = samples.len() / channels as usize;
    let data_bytes = (frames * channels as usize * 2) as u32;
    let byte_rate = sample_rate * u32::from(channels) * 2;
    let block_align = channels * 2;

    file.write_all(b"RIFF")
        .map_err(|e| AppError::message(e.to_string()))?;
    write_u32(&mut file, 36 + data_bytes)?;
    file.write_all(b"WAVEfmt ")
        .map_err(|e| AppError::message(e.to_string()))?;
    write_u32(&mut file, 16)?;
    write_u16(&mut file, 1)?;
    write_u16(&mut file, channels)?;
    write_u32(&mut file, sample_rate)?;
    write_u32(&mut file, byte_rate)?;
    write_u16(&mut file, block_align)?;
    write_u16(&mut file, 16)?;
    file.write_all(b"data")
        .map_err(|e| AppError::message(e.to_string()))?;
    write_u32(&mut file, data_bytes)?;

    for &sample in samples.iter().take(frames * channels as usize) {
        let clamped = sample.clamp(-1.0, 1.0);
        let pcm = (clamped * 32767.0) as i16;
        write_i16(&mut file, pcm)?;
    }

    file.flush().map_err(|e| AppError::message(e.to_string()))?;
    Ok(())
}

fn write_u16(w: &mut impl Write, value: u16) -> Result<(), AppError> {
    w.write_all(&value.to_le_bytes())
        .map_err(|e| AppError::message(e.to_string()))
}

fn write_u32(w: &mut impl Write, value: u32) -> Result<(), AppError> {
    w.write_all(&value.to_le_bytes())
        .map_err(|e| AppError::message(e.to_string()))
}

fn write_i16(w: &mut impl Write, value: i16) -> Result<(), AppError> {
    w.write_all(&value.to_le_bytes())
        .map_err(|e| AppError::message(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn wav_header_is_valid_pcm16() {
        let dir = std::env::temp_dir();
        let path = dir.join("localflow-wav-test.wav");
        write_pcm16_wav(&path, 16_000, 1, &[0.0, 0.5, -0.5, 1.0]).unwrap();
        let bytes = fs::read(&path).unwrap();
        let _ = fs::remove_file(&path);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[20..22], &[1, 0]); // PCM
        assert_eq!(&bytes[22..24], &[1, 0]); // mono
        assert_eq!(u32::from_le_bytes(bytes[24..28].try_into().unwrap()), 16_000);
        assert_eq!(bytes.len(), 44 + 8);
    }
}
