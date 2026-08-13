//! Microphone capture. The CPAL callback only converts samples and pushes a ring buffer.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat};
use serde::Serialize;

use super::device::resolve_input_device;
use super::ring_buffer::RingBuffer;
use super::wav::write_pcm16_wav;
use crate::errors::AppError;

const MAX_SECONDS: u32 = 180;

/// Result of a recording. `path` is a temp WAV; audio samples are not logged.
#[derive(Debug, Clone, Serialize)]
pub struct MicTestResult {
    pub path: String,
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub frames: usize,
}

/// Live capture session. Dropping without `stop` discards audio.
pub struct CaptureSession {
    stream: cpal::Stream,
    buffer: Arc<Mutex<RingBuffer>>,
    err_flag: Arc<Mutex<Option<String>>>,
    sample_rate: u32,
    channels: u16,
    started: Instant,
}

/// Interleaved f32 PCM from the microphone. Do not send this to the UI.
pub struct CapturedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: u64,
}

impl CapturedAudio {
    pub fn save_wav(&self) -> Result<MicTestResult, AppError> {
        let frames = self.samples.len() / self.channels.max(1) as usize;
        let path = recording_wav_path()?;
        write_pcm16_wav(&path, self.sample_rate, self.channels, &self.samples)?;
        tracing::info!(
            duration_ms = self.duration_ms,
            sample_rate = self.sample_rate,
            channels = self.channels,
            frames,
            "wav written"
        );
        Ok(MicTestResult {
            path: path.to_string_lossy().into_owned(),
            duration_ms: self.duration_ms,
            sample_rate: self.sample_rate,
            channels: self.channels,
            frames,
        })
    }
}

impl CaptureSession {
    /// Starts the microphone. Heavy work stays off the audio callback.
    pub fn start(preferred_id: Option<&str>, max_seconds: u32) -> Result<Self, AppError> {
        let (device, _info) = resolve_input_device(preferred_id)?;
        let config = device
            .default_input_config()
            .map_err(|_| AppError::MicrophoneAccess)?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let seconds = max_seconds.max(1) as usize;
        let capacity = (sample_rate as usize)
            .saturating_mul(channels as usize)
            .saturating_mul(seconds)
            .saturating_add(sample_rate as usize);

        let buffer = Arc::new(Mutex::new(RingBuffer::with_capacity(capacity)));
        let err_flag = Arc::new(Mutex::new(None::<String>));
        let stream = build_stream(
            &device,
            &config,
            Arc::clone(&buffer),
            Arc::clone(&err_flag),
        )?;

        tracing::info!(
            sample_rate,
            channels,
            host = ?cpal::default_host().id(),
            "microphone capture started"
        );

        stream.play().map_err(|_| AppError::MicrophoneAccess)?;
        Ok(Self {
            stream,
            buffer,
            err_flag,
            sample_rate,
            channels,
            started: Instant::now(),
        })
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    /// Stops capture immediately. Caller may write a WAV and/or run ASR.
    pub fn stop(self) -> Result<CapturedAudio, AppError> {
        let duration_ms = self.started.elapsed().as_millis() as u64;
        drop(self.stream);

        if let Ok(err) = self.err_flag.lock() {
            if let Some(msg) = err.as_ref() {
                tracing::error!(error = %msg, "capture stream error");
                return Err(AppError::MicrophoneAccess);
            }
        }

        let samples = {
            let ring = self.buffer.lock().map_err(|_| AppError::LockPoisoned)?;
            tracing::info!(
                stored = ring.len(),
                capacity = ring.capacity(),
                duration_ms,
                "capture buffer closed"
            );
            if ring.is_empty() {
                return Err(AppError::message("no audio captured"));
            }
            ring.to_vec()
        };
        let frames = samples.len() / self.channels.max(1) as usize;
        if frames == 0 {
            return Err(AppError::message("no audio captured"));
        }

        Ok(CapturedAudio {
            samples,
            sample_rate: self.sample_rate,
            channels: self.channels,
            duration_ms,
        })
    }
}

pub fn max_capture_seconds() -> u32 {
    MAX_SECONDS
}

fn recording_wav_path() -> Result<PathBuf, AppError> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(std::env::temp_dir().join(format!("localflow-mic-{stamp}.wav")))
}

fn build_stream(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    buffer: Arc<Mutex<RingBuffer>>,
    err_flag: Arc<Mutex<Option<String>>>,
) -> Result<cpal::Stream, AppError> {
    let stream_config = config.config();
    let on_err = move |err: cpal::StreamError| {
        tracing::error!(error = %err, "cpal stream error");
        if let Ok(mut slot) = err_flag.lock() {
            *slot = Some(err.to_string());
        }
    };

    let stream = match config.sample_format() {
        SampleFormat::F32 => input_stream::<f32>(device, &stream_config, buffer, on_err)?,
        SampleFormat::I16 => input_stream::<i16>(device, &stream_config, buffer, on_err)?,
        SampleFormat::U16 => input_stream::<u16>(device, &stream_config, buffer, on_err)?,
        SampleFormat::I32 => input_stream::<i32>(device, &stream_config, buffer, on_err)?,
        SampleFormat::F64 => input_stream::<f64>(device, &stream_config, buffer, on_err)?,
        _ => return Err(AppError::UnsupportedSampleFormat),
    };
    Ok(stream)
}

fn input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    buffer: Arc<Mutex<RingBuffer>>,
    on_err: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, AppError>
where
    T: Sample + cpal::SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let Ok(mut ring) = buffer.try_lock() else {
                    return;
                };
                let mut tmp = [0.0f32; 128];
                for chunk in data.chunks(tmp.len()) {
                    for (i, sample) in chunk.iter().enumerate() {
                        tmp[i] = (*sample).to_sample::<f32>();
                    }
                    ring.push_slice(&tmp[..chunk.len()]);
                }
            },
            on_err,
            None,
        )
        .map_err(|_| AppError::MicrophoneAccess)
}

/// Opens the containing folder in Explorer (Windows). Other platforms return the path only.
pub fn reveal_path(path: &str) -> Result<(), AppError> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(AppError::message("recording file is missing"));
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
            .map_err(|e| AppError::message(e.to_string()))?;
    }
    Ok(())
}
