//! Deterministic transcript cleanup. Runs before injection; no LLM.

/// Apply filler removal, spoken punctuation, spacing, and sentence caps.
/// Never log `input` or the result.
pub fn clean(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < words.len() {
        if let Some((kind, n)) = match_command(&words[i..]) {
            match kind {
                Command::Drop => {}
                Command::Insert(s) => parts.push(s),
            }
            i += n;
            continue;
        }
        if is_filler(words[i]) {
            i += 1;
            continue;
        }
        parts.push(words[i].to_string());
        i += 1;
    }
    let stitched = stitch(&parts);
    let emailed = format_email(&stitched);
    let spaced = collapse_spaces(&emailed);
    let capped = capitalize_sentences(&spaced);
    let timed = format_clock(&capped);
    let deduped = collapse_the_the(&timed);
    ensure_period(&deduped)
}

enum Command {
    Drop,
    Insert(String),
}

fn eq(word: &str, want: &str) -> bool {
    let w = word.trim_matches(|c: char| !c.is_ascii_alphanumeric());
    w.eq_ignore_ascii_case(want)
}

fn match_command(words: &[&str]) -> Option<(Command, usize)> {
    let w0 = *words.first()?;
    if words.len() >= 2 {
        let w1 = words[1];
        if eq(w0, "new") && eq(w1, "paragraph") {
            return Some((Command::Insert("\n\n".into()), 2));
        }
        if eq(w0, "new") && eq(w1, "line") {
            return Some((Command::Insert("\n".into()), 2));
        }
        if eq(w0, "question") && eq(w1, "mark") {
            return Some((Command::Insert("?".into()), 2));
        }
        if eq(w0, "exclamation") && (eq(w1, "mark") || eq(w1, "point")) {
            return Some((Command::Insert("!".into()), 2));
        }
        if eq(w0, "full") && eq(w1, "stop") {
            return Some((Command::Insert(".".into()), 2));
        }
        if eq(w0, "start") && eq(w1, "up") {
            return Some((Command::Insert("startup".into()), 2));
        }
        if eq(w0, "you") && eq(w1, "know") {
            return Some((Command::Drop, 2));
        }
    }
    if words.len() >= 3 && eq(w0, "dot") && eq(words[1], "dot") && eq(words[2], "dot") {
        return Some((Command::Insert("...".into()), 3));
    }
    if eq(w0, "period") {
        return Some((Command::Insert(".".into()), 1));
    }
    if eq(w0, "comma") {
        return Some((Command::Insert(",".into()), 1));
    }
    if eq(w0, "colon") {
        return Some((Command::Insert(":".into()), 1));
    }
    if eq(w0, "semicolon") {
        return Some((Command::Insert(";".into()), 1));
    }
    if eq(w0, "newline") {
        return Some((Command::Insert("\n".into()), 1));
    }
    if eq(w0, "at") && words.len() >= 3 {
        let domain = words[1].trim_matches(|c: char| !c.is_ascii_alphanumeric());
        if words.len() >= 4 && (eq(words[2], "dot") || eq(words[2], "period")) && is_tld(words[3]) {
            let tld = words[3].trim_matches(|c: char| !c.is_ascii_alphanumeric());
            if domain.len() >= 2 {
                return Some((Command::Insert(format!("@{domain}.{tld}")), 4));
            }
        }
        if words[1].contains('.') && is_tld(words[2]) && domain.len() >= 2 {
            let tld = words[2].trim_matches(|c: char| !c.is_ascii_alphanumeric());
            return Some((Command::Insert(format!("@{domain}.{tld}")), 3));
        }
    }
    if eq(w0, "ellipsis") {
        return Some((Command::Insert("...".into()), 1));
    }
    None
}

fn is_tld(word: &str) -> bool {
    matches!(
        word.trim_matches(|c: char| !c.is_ascii_alphanumeric())
            .to_ascii_lowercase()
            .as_str(),
        "com" | "org" | "net" | "in" | "io" | "dev" | "ai" | "co"
    )
}

fn format_clock(s: &str) -> String {
    s.split('\n')
        .map(format_clock_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_clock_line(s: &str) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < words.len() {
        if let Some((clock, n)) = take_clock(&words[i..]) {
            out.push(clock);
            i += n;
            continue;
        }
        out.push(words[i].to_string());
        i += 1;
    }
    out.join(" ")
}

fn take_clock(words: &[&str]) -> Option<(String, usize)> {
    if let Some(clock) = parse_clock_token(words[0]) {
        return Some((clock, 1));
    }
    let hour = parse_hour(words[0])?;
    if words.len() >= 2 {
        if let Some((min, ap)) = parse_minutes_ampm(words[1]) {
            return Some((format!("{hour}:{min:02} {ap}"), 2));
        }
        if let Some(min) = parse_minutes(words[1]) {
            if words.len() >= 3 {
                if let Some(ap) = parse_ampm(words[2]) {
                    return Some((format!("{hour}:{min:02} {ap}"), 3));
                }
            }
        }
    }
    None
}

fn parse_clock_token(w: &str) -> Option<String> {
    let raw = w
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != ':' && c != '.')
        .to_ascii_lowercase();
    let (head, ap) = split_ampm(&raw)?;
    let (hour, min) = parse_hm(head)?;
    Some(format!("{hour}:{min:02} {ap}"))
}

fn split_ampm(s: &str) -> Option<(&str, &'static str)> {
    let s = s.trim_end_matches('.');
    for (suffix, ap) in [
        ("p.m", "p.m."),
        ("a.m", "a.m."),
        ("pm", "p.m."),
        ("am", "a.m."),
    ] {
        if let Some(head) = s.strip_suffix(suffix) {
            let head = head.trim_end_matches(['.', ':']);
            if !head.is_empty() {
                return Some((head, ap));
            }
        }
    }
    None
}

fn parse_hm(num: &str) -> Option<(u32, u32)> {
    let num = num.trim_end_matches('.');
    if let Some((h, m)) = num.split_once(':').or_else(|| num.split_once('.')) {
        let hour: u32 = h.trim_matches(|c: char| !c.is_ascii_digit()).parse().ok()?;
        let min: u32 = m.trim_matches(|c: char| !c.is_ascii_digit()).parse().ok()?;
        return ((1..=12).contains(&hour) && min < 60).then_some((hour, min));
    }
    let hour: u32 = num
        .trim_matches(|c: char| !c.is_ascii_digit())
        .parse()
        .ok()?;
    ((1..=12).contains(&hour)).then_some((hour, 0))
}

fn parse_minutes_ampm(w: &str) -> Option<(u32, &'static str)> {
    let lower = w.to_ascii_lowercase();
    let (head, ap) = split_ampm(&lower)?;
    let digits: String = head.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 2 {
        return None;
    }
    let min: u32 = digits.parse().ok()?;
    (min < 60).then_some((min, ap))
}

fn parse_hour(w: &str) -> Option<u32> {
    let core = w
        .trim_end_matches('.')
        .trim_matches(|c: char| !c.is_ascii_digit());
    let n: u32 = core.parse().ok()?;
    (1..=12).contains(&n).then_some(n)
}

fn parse_minutes(w: &str) -> Option<u32> {
    let core = w.trim_matches(|c: char| !c.is_ascii_digit());
    if core.len() != 2 {
        return None;
    }
    let n: u32 = core.parse().ok()?;
    (n < 60).then_some(n)
}

fn parse_ampm(w: &str) -> Option<&'static str> {
    let t = w
        .trim_matches(|c: char| !c.is_ascii_alphabetic())
        .to_ascii_lowercase()
        .replace('.', "");
    match t.as_str() {
        "am" | "a" => Some("a.m."),
        "pm" | "p" => Some("p.m."),
        _ => None,
    }
}

fn format_email(s: &str) -> String {
    s.replace(". Com", ".com")
        .replace(". com", ".com")
        .replace(" @", "@")
        .replace("@ ", "@")
}

fn collapse_the_the(s: &str) -> String {
    let mut out = s.to_string();
    for _ in 0..3 {
        out = out.replace(" the the ", " the ");
        out = out.replace(" The the ", " The ");
        out = out.replace(" The The ", " The ");
    }
    out
}

/// Drop a trailing sentence that already appeared earlier (Whisper 30s-window repeats).
pub fn drop_repeated_tail(text: &str) -> String {
    let mut sents: Vec<String> = text
        .split_inclusive(['.', '!', '?'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut i = sents.len();
    while i > 1 {
        i -= 1;
        let last = sents[i]
            .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != ' ')
            .to_ascii_lowercase();
        if last.split_whitespace().count() < 5 {
            continue;
        }
        let rest = sents[..i].join(" ").to_ascii_lowercase();
        if rest.contains(&last) || similar_enough(&rest, &last) {
            sents.remove(i);
        }
    }
    sents.join(" ")
}

fn similar_enough(hay: &str, last: &str) -> bool {
    let words: Vec<&str> = last.split_whitespace().take(8).collect();
    if words.len() < 5 {
        return false;
    }
    let probe = words.join(" ");
    hay.contains(&probe)
}

fn is_filler(word: &str) -> bool {
    let w = word.trim_matches(|c: char| c == ',' || c == '.' || c == '!' || c == '?');
    matches!(
        w.to_ascii_lowercase().as_str(),
        "um" | "uh" | "er" | "ah" | "hmm" | "hm" | "mm" | "uhh" | "umm"
    )
}

fn is_punct(s: &str) -> bool {
    matches!(s, "." | "," | "?" | "!" | ":" | ";" | "...")
}

fn stitch(parts: &[String]) -> String {
    let mut s = String::new();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        if part.starts_with('\n') || is_punct(part) {
            s.push_str(part);
        } else {
            if !s.is_empty() && !s.ends_with('\n') && !s.ends_with(' ') {
                s.push(' ');
            }
            s.push_str(part);
        }
    }
    s
}

fn collapse_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch == ' ' || ch == '\t' {
            if !prev_space && !out.ends_with('\n') {
                out.push(' ');
                prev_space = true;
            }
        } else {
            prev_space = false;
            out.push(ch);
        }
    }
    out.trim().to_string()
}

fn capitalize_sentences(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut cap = true;
    for ch in s.chars() {
        if cap && ch.is_alphabetic() {
            for up in ch.to_uppercase() {
                out.push(up);
            }
            cap = false;
        } else {
            out.push(ch);
            if ch == '.' || ch == '!' || ch == '?' || ch == '\n' {
                cap = true;
            }
        }
    }
    out
}

fn ensure_period(s: &str) -> String {
    let t = s.trim();
    if t.is_empty() {
        return String::new();
    }
    if t.chars().last().is_some_and(|c| c.is_alphanumeric()) {
        format!("{t}.")
    } else {
        t.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_fillers_and_caps() {
        assert_eq!(clean("um hello uh there"), "Hello there.");
    }

    #[test]
    fn spoken_punctuation() {
        assert_eq!(
            clean("hello period how are you question mark"),
            "Hello. How are you?"
        );
    }

    #[test]
    fn new_paragraph() {
        let out = clean("first new paragraph second");
        assert_eq!(out, "First\n\nSecond.");
    }

    #[test]
    fn you_know_is_dropped() {
        assert_eq!(clean("this is you know fine"), "This is fine.");
    }

    #[test]
    fn start_up_becomes_startup() {
        assert_eq!(clean("build this start up now"), "Build this startup now.");
    }

    #[test]
    fn empty_fillers_only() {
        assert_eq!(clean("um uh hmm"), "");
    }

    #[test]
    fn already_punctuated_not_doubled() {
        assert_eq!(clean("Hello there."), "Hello there.");
    }

    #[test]
    fn clock_uses_colon() {
        assert_eq!(
            clean("The meeting is at 3. 30 Pm on Monday"),
            "The meeting is at 3:30 p.m. on Monday."
        );
        assert_eq!(
            clean("I have a meeting at 3. 30Pm tomorrow and offline meeting at 5pm tomorrow"),
            "I have a meeting at 3:30 p.m. tomorrow and offline meeting at 5:00 p.m. tomorrow."
        );
    }

    #[test]
    fn spoken_email() {
        let out = clean("email is hello at localflow dot com");
        assert!(
            out.to_ascii_lowercase().contains("hello@localflow.com"),
            "{out}"
        );
    }

    #[test]
    fn question_mark_with_comma() {
        assert_eq!(
            clean("hello period this is a test question mark,"),
            "Hello. This is a test?"
        );
    }

    #[test]
    fn drops_repeated_ending() {
        let t = drop_repeated_tail(
            "It is written by Yuval Noah Harari and it is a pretty great book. Thank you. It is written by Yuval Noah Harari and it is a pretty great book. Thank you.",
        );
        assert_eq!(t.matches("written by").count(), 1, "{t}");
    }
}
