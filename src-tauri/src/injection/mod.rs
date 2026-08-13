//! Insert text into the focused app. Never log the payload.

#[cfg(windows)]
mod windows;

/// UTF-16 with a trailing NUL, for CF_UNICODETEXT.
pub fn to_wide_null(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Paste `text` at the caret. If `preserve` is set, restore the previous
/// Unicode clipboard after paste. On total failure the text is left on the
/// clipboard and `InjectionFailed` is returned.
pub fn inject(text: &str, preserve: bool) -> Result<(), crate::errors::AppError> {
    if text.is_empty() {
        return Ok(());
    }
    #[cfg(windows)]
    {
        windows::inject(text, preserve)
    }
    #[cfg(not(windows))]
    {
        let _ = preserve;
        Err(crate::errors::AppError::InjectionFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_null_terminates() {
        let w = to_wide_null("ab");
        assert_eq!(w, vec![b'a' as u16, b'b' as u16, 0]);
    }
}
