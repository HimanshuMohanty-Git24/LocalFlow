//! Qwen3-0.6B via llama.cpp. CPU only. Runs on the dictation worker.

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::{send_logs_to_tracing, LogOptions};

use crate::asr::paths;
use crate::errors::AppError;

use super::accept_rewrite;

const MAX_NEW_TOKENS: i32 = 512;
const N_CTX: u32 = 2048;

/// Lazy-loaded local Qwen. Missing model skips enhancement.
pub struct QwenBackend {
    backend: Option<LlamaBackend>,
    model: Option<LlamaModel>,
    tried: bool,
}

impl QwenBackend {
    pub fn new() -> Self {
        Self {
            backend: None,
            model: None,
            tried: false,
        }
    }

    fn load(&mut self) -> Result<bool, AppError> {
        if self.model.is_some() {
            return Ok(true);
        }
        if self.tried {
            return Ok(false);
        }
        self.tried = true;
        let path = match find_qwen() {
            Ok(path) => path,
            Err(AppError::ModelMissing) => {
                tracing::warn!("qwen gguf missing; using rule cleanup only");
                return Ok(false);
            }
            Err(err) => return Err(err),
        };
        send_logs_to_tracing(LogOptions::default().with_logs_enabled(false));
        tracing::info!(path = %path.display(), "loading qwen");
        let backend = LlamaBackend::init().map_err(|_| AppError::LlmInitFailed)?;
        let params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, &path, &params)
            .map_err(|_| AppError::LlmInitFailed)?;
        self.backend = Some(backend);
        self.model = Some(model);
        Ok(true)
    }

    /// Rewrite `text` locally. On any failure return `text` unchanged.
    pub fn enhance(&mut self, text: &str) -> String {
        if text.trim().is_empty() {
            return String::new();
        }
        match self.try_enhance(text) {
            Ok(Some(out)) => out,
            Ok(None) | Err(_) => {
                tracing::info!("llm skipped; using rules");
                text.to_string()
            }
        }
    }

    fn try_enhance(&mut self, text: &str) -> Result<Option<String>, AppError> {
        if !self.load()? {
            return Ok(None);
        }
        let started = Instant::now();
        let backend = self.backend.as_ref().ok_or(AppError::LlmInitFailed)?;
        let model = self.model.as_ref().ok_or(AppError::LlmInitFailed)?;
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(N_CTX))
            .with_n_threads(2)
            .with_n_threads_batch(2);
        let mut ctx = model
            .new_context(backend, ctx_params)
            .map_err(|_| AppError::LlmInitFailed)?;

        let prompt = chat_prompt(model, text)?;
        let tokens = model
            .str_to_token(&prompt, AddBos::Never)
            .map_err(|_| AppError::LlmInitFailed)?;
        if tokens.is_empty() || tokens.len() as i32 + MAX_NEW_TOKENS >= N_CTX as i32 {
            return Ok(None);
        }

        let mut batch = LlamaBatch::new(512, 1);
        let last = (tokens.len() - 1) as i32;
        for (i, token) in (0_i32..).zip(tokens.into_iter()) {
            batch
                .add(token, i, &[0], i == last)
                .map_err(|_| AppError::LlmInitFailed)?;
        }
        ctx.decode(&mut batch).map_err(|_| AppError::LlmInitFailed)?;

        let mut sampler = LlamaSampler::chain_simple([LlamaSampler::greedy()]);
        let mut n_cur = batch.n_tokens();
        let mut out = String::new();
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        for _ in 0..MAX_NEW_TOKENS {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);
            if model.is_eog_token(token) {
                break;
            }
            if let Ok(piece) = model.token_to_piece(token, &mut decoder, false, None) {
                out.push_str(&piece);
            }
            if out.contains("<|im_end|>") || out.contains("<|endoftext|>") {
                break;
            }
            if out.contains("</think>") && out.len() > 32 {
                let after = strip_after_think(&out);
                if after.chars().count() >= text.chars().count() / 2 {
                    break;
                }
            }
            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|_| AppError::LlmInitFailed)?;
            ctx.decode(&mut batch).map_err(|_| AppError::LlmInitFailed)?;
            n_cur += 1;
        }

        tracing::info!(llm_ms = started.elapsed().as_millis() as u64, "llm finished");
        Ok(accept_rewrite(text, &out))
    }
}

fn chat_prompt(model: &LlamaModel, text: &str) -> Result<String, AppError> {
    let tmpl = model
        .chat_template(None)
        .map_err(|_| AppError::LlmInitFailed)?;
    let user = format!(
        "Fix punctuation and capitalization only. Keep every word, including names and places (Himanshu, Odisha, Angul). Do not summarize, do not replace words, do not change meaning. Output the edited text and nothing else.\n\n{text}"
    );
    let chat = [LlamaChatMessage::new("user".into(), user).map_err(|_| AppError::LlmInitFailed)?];
    let mut prompt = model
        .apply_chat_template(&tmpl, &chat, true)
        .map_err(|_| AppError::LlmInitFailed)?;
    if !prompt.contains("</think>") {
        if prompt.contains("<think>") {
            prompt.push_str("</think>\n");
        } else {
            prompt.push_str("<think>\n</think>\n");
        }
    }
    Ok(prompt)
}

fn strip_after_think(s: &str) -> String {
    match s.rfind("</think>") {
        Some(i) => s[i + "</think>".len()..].trim().to_string(),
        None => s.to_string(),
    }
}

fn find_qwen() -> Result<PathBuf, AppError> {
    let mut found = Vec::new();
    for dir in paths::search_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if name.ends_with(".gguf") && name.contains("qwen") {
                found.push(path);
            }
        }
    }
    found.sort_by_key(|p| {
        let n = p
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if n.contains("0.6") || n.contains("0_6") {
            0
        } else {
            5
        }
    });
    found.into_iter().next().ok_or(AppError::ModelMissing)
}
