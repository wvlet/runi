mod format;

use format::UniFormatter;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub use tracing::Span;
pub use tracing::{Level, instrument, span};
pub use tracing::{debug, error, info, trace, warn};
pub use tracing_subscriber::util::TryInitError;

/// Initialize logging with sensible defaults.
///
/// - Log level from `RUNI_LOG` env var (default: `info`)
/// - Uni-style format: `timestamp LEVEL [target] message - (file:line)`
/// - Colored output for terminals, JSON for non-terminals
///
/// Panics if a global subscriber is already installed. For tests and other
/// repeated-init scenarios, use [`try_init`].
pub fn init() {
    init_with_env("RUNI_LOG");
}

/// Fallible counterpart of [`init`]. Returns an error instead of panicking
/// when a global subscriber is already installed — ideal for tests, where
/// multiple `#[test]` functions may race to install the subscriber.
///
/// ```no_run
/// fn test_logger() {
///     static ONCE: std::sync::Once = std::sync::Once::new();
///     ONCE.call_once(|| { let _ = runi_log::try_init(); });
/// }
/// ```
///
/// Combine with `RUNI_LOG` at the shell to bump a single component's level:
///
/// ```text
/// RUNI_LOG=my_crate::parser=debug cargo test parser -- --nocapture
/// ```
pub fn try_init() -> Result<(), TryInitError> {
    try_init_with_env("RUNI_LOG")
}

/// Initialize logging with a custom env var name for the filter.
pub fn init_with_env(env_var: &str) {
    try_init_with_env(env_var).expect("runi_log: a global subscriber is already installed");
}

/// Fallible counterpart of [`init_with_env`].
pub fn try_init_with_env(env_var: &str) -> Result<(), TryInitError> {
    let filter = EnvFilter::try_from_env(env_var).unwrap_or_else(|_| EnvFilter::new("info"));
    install(filter)
}

/// Initialize logging with a specific level string (e.g., "debug", "warn").
pub fn init_with_level(level: &str) {
    try_init_with_level(level).expect("runi_log: a global subscriber is already installed");
}

/// Fallible counterpart of [`init_with_level`].
pub fn try_init_with_level(level: &str) -> Result<(), TryInitError> {
    install(EnvFilter::new(level))
}

fn install(filter: EnvFilter) -> Result<(), TryInitError> {
    let is_terminal = std::io::IsTerminal::is_terminal(&std::io::stderr());

    if is_terminal {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .with_writer(std::io::stderr)
                    .event_format(UniFormatter::new(true)),
            )
            .try_init()
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .json()
                    .with_writer(std::io::stderr)
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .try_init()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_logger() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let _ = try_init();
        });
    }

    #[test]
    fn log_macros_compile() {
        test_logger();
        trace!("trace message");
        debug!("debug message");
        info!("info message");
        warn!("warn message");
        error!("error message");
    }

    #[test]
    fn log_with_fields() {
        test_logger();
        info!(host = "localhost", port = 8080, "server starting");
        debug!(elapsed_ms = 42, "request completed");
    }

    #[test]
    fn span_creation() {
        test_logger();
        let _span = span!(Level::INFO, "my_span", id = 42);
    }

    #[test]
    fn try_init_is_idempotent() {
        // First call may succeed or fail depending on test ordering; second
        // call must always return Err without panicking.
        let _ = try_init();
        assert!(try_init().is_err());
    }
}
