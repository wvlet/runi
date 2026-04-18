use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub use tracing::{debug, error, info, trace, warn};
pub use tracing::{instrument, span, Level};
pub use tracing::Span;

/// Initialize logging with sensible defaults.
///
/// - Log level from `RUNI_LOG` env var (default: `info`)
/// - Pretty output for terminals, compact otherwise
pub fn init() {
    init_with_env("RUNI_LOG");
}

/// Initialize logging with a custom env var name for the filter.
pub fn init_with_env(env_var: &str) {
    let filter = EnvFilter::try_from_env(env_var)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let is_terminal = std::io::IsTerminal::is_terminal(&std::io::stderr());

    if is_terminal {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(true).with_thread_ids(false))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json().with_target(true))
            .init();
    }
}

/// Initialize logging with a specific level string (e.g., "debug", "warn").
pub fn init_with_level(level: &str) {
    let filter = EnvFilter::new(level);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true))
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_macros_compile() {
        // Verify macros are usable without init (they're no-ops without a subscriber)
        trace!("trace message");
        debug!("debug message");
        info!("info message");
        warn!("warn message");
        error!("error message");
    }

    #[test]
    fn log_with_fields() {
        info!(host = "localhost", port = 8080, "server starting");
        debug!(elapsed_ms = 42, "request completed");
    }

    #[test]
    fn span_creation() {
        let _span = span!(Level::INFO, "my_span", id = 42);
    }
}
