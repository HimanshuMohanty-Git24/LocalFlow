//! Optional local LLM cleanup. Never log prompts or completions.

mod qwen;

pub use qwen::QwenBackend;

/// Drop Qwen3 thinking blocks and leftover chat markers.
pub fn strip_think(text: &str) -> String {
    let mut t = text.trim().to_string();
    if let Some(idx) = t.rfind("</think>") {
        t = t[idx + "</think>".len()..].trim().to_string();
    }
    for marker in ["<|im_end|>", "<|im_start|>", "<|endoftext|>", "<think>", "</think>"] {
        t = t.replace(marker, "");
    }
    t.trim().to_string()
}

fn content_words(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() > 1)
        .map(|w| w.to_ascii_lowercase())
        .collect()
}

/// Reject unfinished thinking, control tokens, and rewrites that drifted.
pub fn accept_rewrite(original: &str, candidate: &str) -> Option<String> {
    let raw = candidate.trim();
    if raw.contains("<think>") && !raw.contains("</think>") {
        return None;
    }
    let out = strip_think(candidate);
    if out.is_empty() {
        return None;
    }
    if out.contains("<|") {
        return None;
    }
    let orig_len = original.chars().count();
    let out_len = out.chars().count();
    if out_len > orig_len.saturating_mul(3).max(120) {
        return None;
    }
    let orig_words = content_words(original);
    let out_words = content_words(&out);
    if orig_words.len() >= 8 && out_words.len() * 5 < orig_words.len() * 4 {
        return None;
    }
    if orig_words.len() >= 4 {
        let hay = out.to_ascii_lowercase();
        let hits = orig_words.iter().filter(|w| hay.contains(w.as_str())).count();
        if hits * 2 < orig_words.len() {
            return None;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_think_block() {
        assert_eq!(
            strip_think("<think>plan</think>\nHello there."),
            "Hello there."
        );
    }

    #[test]
    fn rejects_unfinished_think() {
        assert!(accept_rewrite("Hello there friend", "<think>I should rewrite this by").is_none());
    }

    #[test]
    fn rejects_runaway_rewrite() {
        assert!(accept_rewrite("Hi.", &"word ".repeat(200)).is_none());
        assert_eq!(
            accept_rewrite("hello there", "Hello there."),
            Some("Hello there.".into())
        );
    }

    #[test]
    fn rejects_unrelated_gibberish() {
        assert!(accept_rewrite(
            "Hello my name is Jiman and I am from India",
            "fn main() { println!(\"ok\"); }"
        )
        .is_none());
    }

    #[test]
    fn keeps_light_edit() {
        let orig = "Hello my name is Jiman Su and I am from India";
        let edit = "Hello, my name is Jiman Su and I am from India.";
        assert_eq!(accept_rewrite(orig, edit).as_deref(), Some(edit));
    }

    #[test]
    fn rejects_truncated_rewrite() {
        let orig = "Hello my name is Manshu and I am from a small city in India I want to build the biggest different startup out there in India and I know discipline and consistency are important";
        let short = "Hello, my name is Manshu and I am from a small city in India.";
        assert!(accept_rewrite(orig, short).is_none());
    }
}
