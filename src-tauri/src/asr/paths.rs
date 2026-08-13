//! Locate a local Whisper ggml file. Never downloads.

use std::path::{Path, PathBuf};

use crate::errors::AppError;

/// Directories searched for `ggml-*.bin` model files.
pub fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(custom) = std::env::var("LOCALFLOW_MODELS_DIR") {
        dirs.push(PathBuf::from(custom));
    }
    dirs.push(PathBuf::from("models"));
    dirs.push(PathBuf::from("../models"));
    dirs.push(PathBuf::from("../../models"));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("models"));
            dirs.push(parent.join("resources").join("models"));
            dirs.push(parent.to_path_buf());
            dirs.push(parent.join("../../../models"));
        }
    }
    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        dirs.push(PathBuf::from(appdata).join("LocalFlow").join("models"));
    }
    dirs
}

/// Prefer small.en, then base.en, then tiny.en. Any other ggml is last resort.
pub fn find_whisper_model() -> Result<PathBuf, AppError> {
    let mut found: Vec<PathBuf> = Vec::new();
    for dir in search_dirs() {
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
            if name.starts_with("ggml-") && (name.ends_with(".bin") || name.ends_with(".gguf")) {
                if name.contains("silero") || name.contains("vad") {
                    continue;
                }
                found.push(path);
            }
        }
    }
    found.sort_by_key(|p| rank(p));
    found.into_iter().next().ok_or(AppError::ModelMissing)
}

fn rank(path: &Path) -> i32 {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name.contains("small.en") {
        0
    } else if name.contains("base.en") {
        1
    } else if name.contains("tiny.en") {
        2
    } else if name.contains("small") {
        3
    } else if name.contains("base") {
        4
    } else {
        9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_en_ranks_ahead_of_tiny() {
        assert!(rank(Path::new("ggml-small.en.bin")) < rank(Path::new("ggml-tiny.en.bin")));
        assert!(rank(Path::new("ggml-base.en-q5_1.bin")) < rank(Path::new("ggml-tiny.en.bin")));
    }
}
