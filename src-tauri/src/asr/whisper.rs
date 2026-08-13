//! whisper.cpp backend via whisper-rs. CPU-only in Phase 3.

use std::path::PathBuf;
use std::time::Instant;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::engine::{SpeechRecognizer, Transcript, TranscriptionOptions};
use super::paths;
use crate::errors::AppError;

/// Local Whisper.cpp recognizer. Lazy-loads the ggml file on first use.
pub struct WhisperBackend {
    ctx: Option<WhisperContext>,
    label: String,
    model_path: Option<PathBuf>,
}

impl WhisperBackend {
    pub fn new() -> Self {
        Self {
            ctx: None,
            label: "Not loaded".to_string(),
            model_path: None,
        }
    }
}

impl Default for WhisperBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeechRecognizer for WhisperBackend {
    fn load(&mut self) -> Result<(), AppError> {
        if self.ctx.is_some() {
            return Ok(());
        }
        let path = paths::find_whisper_model()?;
        tracing::info!(path = %path.display(), "loading whisper model");
        let ctx = WhisperContext::new_with_params(&path, WhisperContextParameters::default())
            .map_err(|err| {
                tracing::error!(error = %err, "whisper init failed");
                AppError::AsrInitFailed
            })?;
        self.label = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("whisper")
            .to_string();
        self.model_path = Some(path);
        self.ctx = Some(ctx);
        Ok(())
    }

    fn unload(&mut self) {
        self.ctx = None;
        self.label = "Not loaded".to_string();
        self.model_path = None;
    }

    fn transcribe(
        &mut self,
        audio: &[f32],
        options: TranscriptionOptions,
    ) -> Result<Transcript, AppError> {
        self.load()?;
        if audio.is_empty() {
            return Ok(Transcript {
                text: String::new(),
                duration_ms: 0,
            });
        }

        let started = Instant::now();
        let mut text = decode(self, audio, &options)?;
        text = drop_hallucinations(&text);

        let rate = 16_000usize;
        if audio.len() > rate.saturating_mul(30) {
            let tail_n = rate.saturating_mul(10).min(audio.len());
            if let Ok(tail) = decode(self, &audio[audio.len() - tail_n..], &options) {
                let tail = drop_hallucinations(&tail);
                text = merge_tail(&text, &tail);
            }
        }
        text = crate::normalization::drop_repeated_tail(&text);

        let duration_ms = started.elapsed().as_millis() as u64;
        tracing::info!(duration_ms, "asr finished");
        Ok(Transcript {
            text: text.trim().to_string(),
            duration_ms,
        })
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn model_label(&self) -> &str {
        &self.label
    }
}

fn whisper_threads() -> i32 {
    std::thread::available_parallelism()
        .map(|n| (n.get() as i32).clamp(1, 4))
        .unwrap_or(2)
}

fn decode(
    backend: &WhisperBackend,
    audio: &[f32],
    options: &TranscriptionOptions,
) -> Result<String, AppError> {
    let ctx = backend.ctx.as_ref().ok_or(AppError::AsrInitFailed)?;
    let mut state = ctx.create_state().map_err(|_| AppError::AsrInitFailed)?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(options.language));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_no_context(true);
    params.set_single_segment(false);
    params.set_no_timestamps(true);
    params.set_token_timestamps(false);
    params.set_temperature(0.0);
    params.set_suppress_blank(true);
    params.set_suppress_nst(true);
    params.set_no_speech_thold(0.6);
    params.set_max_tokens(0);
    params.set_n_threads(whisper_threads());
    let prompt = crate::vocabulary::Vocabulary::load().whisper_prompt();
    if !prompt.is_empty() {
        params.set_initial_prompt(&prompt);
    }
    state
        .full(params, audio)
        .map_err(|_| AppError::TranscriptionFailed)?;
    let mut text = String::new();
    let n = state.full_n_segments();
    for i in 0..n {
        if let Some(segment) = state.get_segment(i) {
            if let Ok(piece) = segment.to_str_lossy() {
                let piece = piece.trim();
                if piece.is_empty() || is_hallucination(piece) {
                    continue;
                }
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(piece);
            }
        }
    }
    Ok(text)
}

fn is_hallucination(text: &str) -> bool {
    let t = text.trim().to_ascii_lowercase();
    t.contains("thank you for watching")
        || t.contains("thanks for watching")
        || t.contains("thank you for listening")
        || t.contains("thanks for listening")
        || t.contains("subscribe")
        || t.contains("please like")
        || (t.starts_with("this is ") && t.split_whitespace().count() <= 4)
}

fn drop_hallucinations(text: &str) -> String {
    text.split(['.', '!', '?'])
        .map(str::trim)
        .filter(|s| !s.is_empty() && !is_hallucination(s))
        .collect::<Vec<_>>()
        .join(". ")
}

fn merge_tail(main: &str, tail: &str) -> String {
    let tail = tail.trim();
    if tail.is_empty() {
        return main.trim().to_string();
    }
    let main_l = main.to_ascii_lowercase();
    let tail_l = tail.to_ascii_lowercase();
    if main_l.contains(&tail_l) {
        return main.trim().to_string();
    }
    let main_words: Vec<&str> = main.split_whitespace().collect();
    let tail_words: Vec<&str> = tail.split_whitespace().collect();
    let max = main_words.len().min(tail_words.len());
    for n in (1..=max).rev() {
        let a = main_words[main_words.len() - n..]
            .iter()
            .map(|w| w.trim_matches(|c: char| !c.is_ascii_alphanumeric()).to_ascii_lowercase())
            .collect::<Vec<_>>();
        let b = tail_words[..n]
            .iter()
            .map(|w| w.trim_matches(|c: char| !c.is_ascii_alphanumeric()).to_ascii_lowercase())
            .collect::<Vec<_>>();
        if a == b {
            let mut out: Vec<&str> = main_words;
            out.extend_from_slice(&tail_words[n..]);
            return out.join(" ");
        }
    }
    format!("{} {}", main.trim(), tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_thanks_for_listening() {
        let t = drop_hallucinations(
            "I know discipline. Thank you for listening to me. This is Marcia.",
        );
        assert!(t.to_ascii_lowercase().contains("discipline"));
        assert!(!t.to_ascii_lowercase().contains("listening"));
    }

    #[test]
    fn merge_appends_new_ending() {
        let merged = merge_tail(
            "I know discipline and consistency are",
            "consistency are one of the biggest thing",
        );
        assert!(merged.contains("biggest"));
    }
}
