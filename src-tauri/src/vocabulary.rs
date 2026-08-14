//! Personal terms for Whisper priming and post-correction. Never log the transcript.

use std::path::PathBuf;

use crate::asr::paths;

#[derive(Debug, Clone)]
struct Term {
    canonical: String,
    aliases: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Vocabulary {
    terms: Vec<Term>,
}

impl Vocabulary {
    pub fn load() -> Self {
        let mut terms = builtin();
        for dir in paths::search_dirs() {
            let path = dir.join("vocabulary.txt");
            if let Ok(text) = std::fs::read_to_string(&path) {
                terms.extend(parse(&text));
                tracing::info!(path = %path.display(), "loaded vocabulary");
                break;
            }
        }
        Self {
            terms: dedupe(terms),
        }
    }

    pub fn whisper_prompt(&self) -> String {
        let mut names: Vec<&str> = self.terms.iter().map(|t| t.canonical.as_str()).collect();
        names.sort();
        names.dedup();
        let mut s = names.join(", ");
        if s.len() > 220 {
            s.truncate(220);
        }
        s
    }

    pub fn apply(&self, text: &str) -> String {
        if text.is_empty() || self.terms.is_empty() {
            return text.to_string();
        }
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let mut phrases: Vec<(&Term, Vec<String>)> = Vec::new();
        for term in &self.terms {
            for alias in &term.aliases {
                let parts: Vec<String> = alias
                    .split_whitespace()
                    .map(|s| s.to_ascii_lowercase())
                    .collect();
                if parts.len() >= 2 {
                    phrases.push((term, parts));
                }
            }
            let canon_parts: Vec<String> = term
                .canonical
                .split_whitespace()
                .map(|s| s.to_ascii_lowercase())
                .collect();
            if canon_parts.len() >= 2 {
                phrases.push((term, canon_parts));
            }
        }
        phrases.sort_by_key(|(_, p)| std::cmp::Reverse(p.len()));

        let mut out: Vec<String> = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            if let Some((term, n)) = match_phrase(&tokens[i..], &phrases) {
                out.push(term.canonical.clone());
                i += n;
                continue;
            }
            out.push(self.fix_token(tokens[i]));
            i += 1;
        }
        out.join(" ")
    }

    fn fix_token(&self, raw: &str) -> String {
        let (lead, core, trail) = split_punct(raw);
        if core.len() < 3 {
            return raw.to_string();
        }
        let heard = core.to_ascii_lowercase();
        for term in &self.terms {
            if matches_term(&heard, &term.canonical, &term.aliases) {
                return format!("{lead}{}{trail}", preserve_shape(&term.canonical, core));
            }
        }
        raw.to_string()
    }
}

fn builtin() -> Vec<Term> {
    vec![
        term(
            "Himanshu",
            &["Manchu", "Manshu", "Himansu", "Imansu", "Jiman"],
        ),
        term("Odisha", &["Orissa", "Orysa", "Odisa", "Odisha"]),
        term("Angul", &["Angul", "Angool"]),
        term("LocalFlow", &["localflow"]),
        term("startup", &["startup"]),
        term("ground up", &["gowns up", "grounds up"]),
        term("paying a penny", &["paying a fund", "paying for penny"]),
        term("Wispr Flow", &["whisper flow", "whisperflow"]),
        term(
            "defence tech",
            &["defend spec", "defense spec", "defend tech", "defense tech"],
        ),
        term(
            "B. R. Ambedkar",
            &["B. R. M. Betka", "BR Ambedkar", "Ambedkar"],
        ),
        term(
            "Yuval Noah Harari",
            &[
                "Juvall Nohar Harari",
                "Juval Noharari",
                "Juval Noah Harari",
                "Yuval Noah Harari",
            ],
        ),
        term("Bhagat Singh", &["Bhagat Singh"]),
    ]
}

fn term(canonical: &str, aliases: &[&str]) -> Term {
    Term {
        canonical: canonical.to_string(),
        aliases: aliases.iter().map(|s| s.to_string()).collect(),
    }
}

fn parse(text: &str) -> Vec<Term> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((left, right)) = line.split_once('=') {
            let canonical = left.trim();
            if canonical.is_empty() {
                continue;
            }
            let aliases: Vec<String> = right
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            out.push(Term {
                canonical: canonical.to_string(),
                aliases,
            });
        } else {
            out.push(Term {
                canonical: line.to_string(),
                aliases: Vec::new(),
            });
        }
    }
    out
}

fn dedupe(terms: Vec<Term>) -> Vec<Term> {
    let mut out: Vec<Term> = Vec::new();
    for term in terms {
        if let Some(existing) = out
            .iter_mut()
            .find(|t| t.canonical.eq_ignore_ascii_case(&term.canonical))
        {
            for a in term.aliases {
                if !existing.aliases.iter().any(|e| e.eq_ignore_ascii_case(&a)) {
                    existing.aliases.push(a);
                }
            }
        } else {
            out.push(term);
        }
    }
    out
}

fn match_phrase<'a>(
    tokens: &[&str],
    phrases: &[(&'a Term, Vec<String>)],
) -> Option<(&'a Term, usize)> {
    for (term, parts) in phrases {
        if tokens.len() < parts.len() {
            continue;
        }
        let ok = parts.iter().enumerate().all(|(j, want)| {
            let heard = split_punct(tokens[j]).1.to_ascii_lowercase();
            &heard == want
        });
        if ok {
            return Some((term, parts.len()));
        }
    }
    None
}

fn matches_term(heard: &str, canonical: &str, aliases: &[String]) -> bool {
    let want = canonical.to_ascii_lowercase();
    if heard == want {
        return true;
    }
    if aliases.iter().any(|a| a.eq_ignore_ascii_case(heard)) {
        return true;
    }
    close(heard, &want)
}

fn close(heard: &str, want: &str) -> bool {
    if want.len() < 5 || heard.len() < 4 {
        return false;
    }
    if levenshtein(heard, want) <= 2 || (want.len() >= 6 && levenshtein(heard, want) <= 3) {
        return true;
    }
    if want.len() > heard.len() {
        let skip = want.len() - heard.len();
        if skip <= 2 && levenshtein(heard, &want[skip..]) <= 1 {
            return true;
        }
    }
    false
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

fn split_punct(raw: &str) -> (&str, &str, &str) {
    let bytes = raw.as_bytes();
    let mut start = 0;
    let mut end = bytes.len();
    while start < end && !bytes[start].is_ascii_alphanumeric() {
        start += 1;
    }
    while end > start && !bytes[end - 1].is_ascii_alphanumeric() {
        end -= 1;
    }
    (&raw[..start], &raw[start..end], &raw[end..])
}

fn preserve_shape(canonical: &str, heard: &str) -> String {
    if heard.chars().all(|c| c.is_uppercase()) {
        return canonical.to_uppercase();
    }
    canonical.to_string()
}

#[allow(dead_code)]
pub fn search_path() -> Option<PathBuf> {
    paths::search_dirs()
        .into_iter()
        .find(|d| d.join("vocabulary.txt").exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixes_odisha_and_name() {
        let v = Vocabulary { terms: builtin() };
        let out = v.apply("Hi my name is Manchu from Orysa, India");
        assert!(out.contains("Himanshu"), "{out}");
        assert!(out.contains("Odisha"), "{out}");
    }

    #[test]
    fn parse_aliases() {
        let terms = parse("Odisha = Orissa, Orysa\n");
        assert_eq!(terms[0].canonical, "Odisha");
        assert!(terms[0].aliases.iter().any(|a| a == "Orysa"));
    }

    #[test]
    fn fixes_ground_up_and_penny() {
        let v = Vocabulary { terms: builtin() };
        let out = v.apply("a Defence Tech startup from Gowns Up without paying a fund");
        assert!(out.contains("ground up"), "{out}");
        assert!(!out.contains("the the"), "{out}");
        assert!(out.contains("paying a penny"), "{out}");
        assert!(!out.contains("fund"), "{out}");
    }
}
