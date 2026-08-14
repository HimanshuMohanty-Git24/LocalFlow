//! Silero VAD via whisper.cpp. Runs on the dictation worker, never the audio callback.

use crate::asr::paths;
use crate::errors::AppError;
use whisper_rs::{WhisperVadContext, WhisperVadContextParams, WhisperVadParams};

/// Lazy Silero context. Missing model means VAD is skipped (full audio is used).
pub struct SileroVad {
    ctx: Option<WhisperVadContext>,
    tried: bool,
}

impl SileroVad {
    pub fn new() -> Self {
        Self {
            ctx: None,
            tried: false,
        }
    }

    fn load(&mut self) -> Result<bool, AppError> {
        if self.ctx.is_some() {
            return Ok(true);
        }
        if self.tried {
            return Ok(false);
        }
        self.tried = true;
        let path = match find_silero() {
            Ok(path) => path,
            Err(AppError::ModelMissing) => {
                tracing::warn!("silero vad model missing; transcribing full capture");
                return Ok(false);
            }
            Err(err) => return Err(err),
        };
        let path_str = path.to_str().ok_or(AppError::VadInitFailed)?;
        tracing::info!(path = %path.display(), "loading silero vad");
        let mut params = WhisperVadContextParams::default();
        params.set_use_gpu(false);
        params.set_n_threads(2);
        match WhisperVadContext::new(path_str, params) {
            Ok(ctx) => {
                self.ctx = Some(ctx);
                Ok(true)
            }
            Err(err) => {
                tracing::error!(error = %err, "silero vad init failed");
                Err(AppError::VadInitFailed)
            }
        }
    }

    /// Returns speech-only audio, or `None` if the utterance is silence.
    /// If the VAD model is missing, returns the original samples.
    pub fn extract_speech(
        &mut self,
        samples: &[f32],
        _sample_rate: u32,
    ) -> Result<Option<Vec<f32>>, AppError> {
        if samples.is_empty() {
            return Ok(None);
        }
        if !self.load()? {
            return Ok(Some(samples.to_vec()));
        }
        let ctx = self.ctx.as_mut().ok_or(AppError::VadInitFailed)?;
        let segments = ctx
            .segments_from_samples(WhisperVadParams::default(), samples)
            .map_err(|_| AppError::VadInitFailed)?;
        let n = segments.num_segments();
        tracing::info!(segments = n, "vad finished");
        if n <= 0 {
            return Ok(None);
        }
        tracing::info!(
            samples = samples.len(),
            "vad: speech present, keeping full capture"
        );
        Ok(Some(samples.to_vec()))
    }
}

fn find_silero() -> Result<std::path::PathBuf, AppError> {
    for dir in paths::search_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut found: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                name.contains("silero") && (name.ends_with(".bin") || name.ends_with(".gguf"))
            })
            .collect();
        found.sort();
        found.reverse();
        if let Some(path) = found.into_iter().next() {
            return Ok(path);
        }
    }
    Err(AppError::ModelMissing)
}
