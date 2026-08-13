//! Local structured logging. Transcripts, audio, clipboard, and window contents
//! must never be written here.

use tracing_subscriber::EnvFilter;

/// Environment variable that controls LocalFlow log verbosity (`error`, `warn`,
/// `info`, `debug`, `trace`, or a full `tracing` filter).
pub const LOG_ENV: &str = "LOCALFLOW_LOG";

const DEFAULT_FILTER: &str = "info,localflow_lib=info";

/// Builds an `EnvFilter` from `LOCALFLOW_LOG`, then `RUST_LOG`, then a default.
pub fn filter_from_env() -> EnvFilter {
    let spec = std::env::var(LOG_ENV)
        .or_else(|_| std::env::var("RUST_LOG"))
        .unwrap_or_else(|_| DEFAULT_FILTER.to_string());

    EnvFilter::try_new(&spec).unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER))
}

/// Installs the global tracing subscriber. Safe to call once at process start.
pub fn init() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter_from_env())
        .with_target(true)
        .with_thread_ids(false)
        .finish();

    // Ignore if tests or a host already installed a subscriber.
    let _ = tracing::subscriber::set_global_default(subscriber);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter_is_info() {
        let filter = filter_from_env();
        assert!(filter.to_string().contains("info"));
    }

    #[test]
    fn invalid_filter_falls_back() {
        let filter = EnvFilter::try_new("%%%not-a-filter%%%")
            .unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
        assert!(filter.to_string().contains("info"));
    }
}
